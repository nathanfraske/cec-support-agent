//! The engine's API face: `cec-support-agent serve`.
//!
//! A loopback-bound HTTP service consumed by embedding apps (AllMyStuff /
//! MyOwnMesh) over a process boundary — the app talks versioned JSON, never
//! links the engine, so the MIT/AGPL firewall holds and AGPL §13 covers the
//! network-service case by design. Supersedes the spawn-per-diagnosis sidecar
//! (RFC D1, superseded 2026-07-02).
//!
//! Endpoints:
//! - `GET  /v1/health`   — liveness + advertised schema versions.
//! - `POST /v1/diagnose` — the diagnose pipeline (headless: no interview
//!   questions), returning the `cec-diagnose/v1` envelope plus an additive
//!   `session_id` for the execute phase.
//! - `POST /v1/execute`  — two-phase consent preserved: the caller presents
//!   the session, an explicit consent assertion, and a sign-off level; the
//!   engine re-checks the judge's required escalation, executes through the
//!   same consent-gated signed-plan path as the CLI, and returns a
//!   post-execution `cec-execute/v1` envelope (outcome label + verification
//!   verdict). A session is one-shot: consumed on execute, expired after
//!   [`SESSION_TTL`].
//!
//! De-identification discipline: responses are built ONLY from the same
//! de-identified envelope constructions the CLI's `--json` mode uses — the
//! API is a new egress sink, exactly the class of surface the leak-prevention
//! methodology warns about, so its responses carry vocabulary tokens and
//! pinned enum strings, never candidate/step free text or tool output prose.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::Deserialize;

use agent_core::{verify_outcome, Consent, Dispatcher, Verdict};
use common::{
    Candidate, CandidateSource, CoarseHostInventory, ConfigClass, ExternalInventory,
    FaultSignature, InventoryProvider, Risk,
};
use corpus_client::{CorpusStore, FileCorpus, LocalCorpus, OutcomeLabel, RowProvenance, SignOff};
use intake::{Interview, Reproducibility};
use panel::{best_of_n, required_escalation, route_for, Escalation, HeuristicJudge, Route};
use provenance::SigningKey;
use swarm::{Generator, Swarm};
use tools_windows::windows_tools;

use crate::{
    collect_diagnostics, diagnose_envelope, is_executable, is_simple_request, label_for,
    parse_env_authority, parse_env_pubkey, record_outcome, run_id, sandbox_validated_for,
    verification_class_for, wire_escalation, HeuristicGenerator, ModelGenerator, HYPOTHESES,
};

/// A diagnose session awaiting its execute phase. One-shot and TTL-bound.
struct Session {
    signature: FaultSignature,
    config_class: ConfigClass,
    route: Route,
    candidates: Vec<Candidate>,
    selected: usize,
    escalation: Escalation,
    reproducibility: Reproducibility,
    retrieval_first: bool,
    primed_from: Vec<String>,
    created: Instant,
}

/// How long a diagnose session stays executable. Long enough for a human to
/// read a rendered plan and consent; short enough that a stale consent cannot
/// authorize execution far from the diagnosis it was given for.
const SESSION_TTL: Duration = Duration::from_secs(15 * 60);

/// Cap on concurrently pending sessions — a bound, not a throughput knob: a
/// client that diagnoses without ever executing cannot grow memory unboundedly.
const MAX_SESSIONS: usize = 256;

struct AppState {
    args: crate::Args,
    corpus: Box<dyn CorpusStore>,
    authority: Option<corpus_client::SignOffAuthority>,
    sessions: Mutex<HashMap<String, Session>>,
}

/// A structured API refusal: HTTP status + a machine-readable reason. The
/// reason is a fixed vocabulary string, never request-derived text.
#[derive(Debug)]
struct ApiError {
    status: StatusCode,
    reason: &'static str,
}

impl ApiError {
    fn new(status: StatusCode, reason: &'static str) -> Self {
        Self { status, reason }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let body = serde_json::json!({ "error": self.reason });
        (self.status, Json(body)).into_response()
    }
}

#[derive(Deserialize)]
struct DiagnoseRequest {
    describe: String,
    /// Identity-free config keys from the caller's inventory (the app-side
    /// de-id allowlist's output). Hashed into the config class, never stored.
    #[serde(default)]
    inventory_keys: Vec<String>,
}

#[derive(Deserialize)]
struct ExecuteRequest {
    session_id: String,
    /// The caller's assertion that a human consented to the rendered plan.
    /// `false` records the ticket as withdrawn — same semantics as declining
    /// the CLI's consent prompt.
    consented: bool,
    /// `"verifier"` or `"human"` — must meet the judge's required escalation.
    sign_off: String,
    /// Execute a specific candidate from the session's slate instead of the
    /// judge's winner (the app-side retry loop). Must be executable.
    #[serde(default)]
    plan_id: Option<String>,
}

/// Pinned `cec-execute/v1` wire value for an outcome label (mirrors the
/// corpus row tag style; `part_class` is taxonomy vocabulary, never prose).
fn wire_label(label: &OutcomeLabel) -> String {
    match label {
        OutcomeLabel::ResolvedConfirmed => "resolved_confirmed".into(),
        OutcomeLabel::ResolvedProvisional => "resolved_provisional".into(),
        OutcomeLabel::Reopened => "reopened".into(),
        OutcomeLabel::EscalatedHardware { part_class } => {
            format!("escalated_hardware:{part_class}")
        }
        OutcomeLabel::EscalatedHumanUnresolved => "escalated_human_unresolved".into(),
        OutcomeLabel::Withdrawn => "withdrawn".into(),
    }
}

/// Pinned `cec-execute/v1` wire value for a verification verdict.
fn wire_verdict(verdict: &Verdict) -> &'static str {
    match verdict {
        Verdict::Pass => "pass",
        Verdict::ProvisionalPass => "provisional_pass",
        Verdict::Fail { .. } => "fail",
        Verdict::Unverified => "unverified",
        Verdict::OffMachine => "off_machine",
    }
}

/// Refuse a bind address that is not loopback unless the operator explicitly
/// opted in: the engine's API is local-first (leak-C2 posture — raw request
/// prose crosses this surface, so exposure is a deliberate decision, never a
/// default).
fn validate_bind(addr: &std::net::SocketAddr, allow_remote: bool) -> anyhow::Result<()> {
    if !addr.ip().is_loopback() && !allow_remote {
        anyhow::bail!(
            "refusing to bind {addr}: not a loopback address. The API carries raw request \
             prose; pass --allow-remote to expose it deliberately."
        );
    }
    Ok(())
}

pub(crate) async fn serve(args: crate::Args) -> anyhow::Result<()> {
    let addr: std::net::SocketAddr = args
        .bind
        .parse()
        .map_err(|e| anyhow::anyhow!("--bind {:?} is not a socket address: {e}", args.bind))?;
    validate_bind(&addr, args.allow_remote)?;
    // Trusted calls only: the same loopback discipline the CLI applies to the
    // model-inference egress (leak class C2) — a non-loopback endpoint carries
    // raw request prose off the box only under an explicit, audited opt-in.
    crate::validate_inference_endpoints(&args)?;

    // Same attestation posture as the CLI: enforce when a pubkey is present,
    // self-attest when the seed is held, derive enforcement from the seed so
    // single-operator mode never attests without enforcing.
    let authority = parse_env_authority()?;
    let mut pubkey = parse_env_pubkey()?;
    if pubkey.is_none() {
        pubkey = authority.as_ref().map(|a| a.public_key());
    }

    let corpus: Box<dyn CorpusStore> = match &args.corpus {
        Some(path) => {
            let mut file = FileCorpus::open(path)?;
            if let Some(pubkey) = &pubkey {
                file = file.with_authority(pubkey.clone())?;
            }
            eprintln!(
                "serve: corpus file-backed at {path} ({} row(s))",
                file.len()
            );
            Box::new(file)
        }
        None => {
            let mut local = LocalCorpus::new();
            if let Some(pubkey) = &pubkey {
                local = local.with_authority(pubkey.clone());
            }
            Box::new(local)
        }
    };

    let state = Arc::new(AppState {
        args,
        corpus,
        authority,
        sessions: Mutex::new(HashMap::new()),
    });

    let router = Router::new()
        .route("/v1/health", get(health))
        .route("/v1/diagnose", post(diagnose))
        .route("/v1/execute", post(execute))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    eprintln!("serve: listening on http://{addr} (cec-diagnose/v1, cec-execute/v1)");
    axum::serve(listener, router).await?;
    Ok(())
}

async fn health() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "ok",
        "engine": env!("CARGO_PKG_VERSION"),
        "schema_versions": ["cec-diagnose/v1", "cec-execute/v1"],
    }))
}

async fn diagnose(
    State(state): State<Arc<AppState>>,
    Json(request): Json<DiagnoseRequest>,
) -> Result<Json<serde_json::Value>, ApiError> {
    handle_diagnose(&state, request).await.map(Json)
}

async fn execute(
    State(state): State<Arc<AppState>>,
    Json(request): Json<ExecuteRequest>,
) -> Result<Json<serde_json::Value>, ApiError> {
    handle_execute(&state, request).await.map(Json)
}

/// The diagnose pipeline, headless — the same stages the CLI runs with
/// `--no-questions --json`, composed from the same functions, ending in the
/// same de-identified envelope.
async fn handle_diagnose(
    state: &AppState,
    request: DiagnoseRequest,
) -> Result<serde_json::Value, ApiError> {
    if request.describe.trim().is_empty() {
        return Err(ApiError::new(StatusCode::BAD_REQUEST, "describe_empty"));
    }

    // Intake without questions: infer everything the description answers.
    let case = Interview::new(&request.describe).into_case();
    let signature = case.signature();

    // Config class from the caller's identity-free keys (hashed, never
    // stored), or the coarse host default — the same seam as --inventory-keys.
    let external = ExternalInventory::new(request.inventory_keys);
    let config_class = if external.is_empty() {
        CoarseHostInventory.config_class()
    } else {
        external.config_class()
    };

    let route = route_for(&signature);

    let mut dispatcher = Dispatcher::new();
    for tool in windows_tools() {
        dispatcher.register(tool);
    }

    let known = state
        .corpus
        .query(&signature, &config_class)
        .await
        .map_err(|error| {
            eprintln!("serve: corpus query failed: {error}");
            ApiError::new(StatusCode::BAD_GATEWAY, "corpus_unavailable")
        })?;

    // Retrieval-first: confirmed precedents join the slate and de novo
    // generation is skipped — identical to the CLI path.
    let mut candidates: Vec<Candidate> = known
        .iter()
        .map(|mapping| {
            // Rehydrate the served (de-identified) StoredPlan into an in-flight
            // plan for the judge/consent/execute pipeline.
            Candidate::new(
                mapping.plan.to_plan(),
                format!(
                    "Corpus precedent: resolved this signature at this config class \
                     ({} confirmation(s))",
                    mapping.confirmations
                ),
                CandidateSource::CorpusPrimed,
            )
        })
        .collect();
    let retrieval_first = !candidates.is_empty();

    let events = collect_diagnostics(&request.describe);
    let swarm = Swarm::new();
    let mut generators: Vec<Box<dyn Generator>> = vec![Box::new(HeuristicGenerator)];
    if !state.args.offline && !retrieval_first {
        if let Some(base_url) = &state.args.endpoint {
            let (generation_url, generation_model) = if is_simple_request(&route, &case) {
                let url = state
                    .args
                    .fast_endpoint
                    .clone()
                    .unwrap_or_else(|| base_url.clone());
                let model = state
                    .args
                    .fast_model
                    .clone()
                    .unwrap_or_else(|| state.args.model.clone());
                (url, model)
            } else {
                (base_url.clone(), state.args.model.clone())
            };
            for hypothesis in HYPOTHESES {
                generators.push(Box::new(ModelGenerator {
                    base_url: generation_url.clone(),
                    model: generation_model.clone(),
                    hypothesis,
                    case_brief: case.brief(),
                }));
            }
        }
    }
    let gathered = swarm.gather(&generators, &events).await.map_err(|error| {
        eprintln!("serve: generation failed: {error}");
        ApiError::new(StatusCode::INTERNAL_SERVER_ERROR, "generation_failed")
    })?;
    for failure in &gathered.failures {
        eprintln!("serve: note: a generator failed ({failure}); continuing with the rest");
    }
    candidates.extend(gathered.candidates);

    // Risk reconciliation before judging/consent, as in the CLI.
    for candidate in &mut candidates {
        let (reconciled, _corrections) = dispatcher.reconcile_risk(&candidate.plan);
        candidate.plan = reconciled;
    }

    let judge = HeuristicJudge;
    let (selected, best, score) =
        best_of_n(&judge, &candidates).expect("the heuristic candidate is always present");
    let sandbox_validated = sandbox_validated_for(None, best, true).await;
    let escalation = required_escalation(&route, sandbox_validated, best, &score);
    let consent_needed = match best.plan.risk() {
        Risk::ReadOnly => Consent::ReadOnlyOnly,
        Risk::Reversible => Consent::AllowReversible,
        Risk::Destructive => Consent::AllowDestructive,
    };

    let mut envelope = diagnose_envelope(
        &signature,
        &config_class,
        &route,
        &candidates,
        selected,
        &consent_needed,
        &escalation,
    );

    // Register the session for the execute phase; the id is additive within
    // cec-diagnose/v1.
    let session_id = run_id();
    {
        let mut sessions = state.sessions.lock().expect("sessions lock");
        sessions.retain(|_, s| s.created.elapsed() < SESSION_TTL);
        if sessions.len() >= MAX_SESSIONS {
            return Err(ApiError::new(
                StatusCode::TOO_MANY_REQUESTS,
                "too_many_pending_sessions",
            ));
        }
        sessions.insert(
            session_id.clone(),
            Session {
                signature,
                config_class,
                route,
                candidates,
                selected,
                escalation,
                reproducibility: case.reproducibility,
                retrieval_first,
                primed_from: known.iter().map(|m| m.plan.id().to_string()).collect(),
                created: Instant::now(),
            },
        );
    }
    envelope["session_id"] = serde_json::json!(session_id);
    Ok(envelope)
}

/// The execute phase: consent + sign-off over a pending session's plan,
/// through the same signed-plan, consent-gated, verify-label-record path as
/// the CLI's `--sign-off` mode. Returns the post-execution `cec-execute/v1`
/// envelope.
async fn handle_execute(
    state: &AppState,
    request: ExecuteRequest,
) -> Result<serde_json::Value, ApiError> {
    let sign_off = match request.sign_off.as_str() {
        "verifier" => SignOff::VerifierConfirmed,
        "human" => SignOff::HumanConfirmed,
        _ => return Err(ApiError::new(StatusCode::BAD_REQUEST, "sign_off_invalid")),
    };

    // One-shot: the session is consumed here — a consent cannot be replayed.
    let session = {
        let mut sessions = state.sessions.lock().expect("sessions lock");
        sessions.retain(|_, s| s.created.elapsed() < SESSION_TTL);
        sessions.remove(&request.session_id)
    };
    let Some(session) = session else {
        return Err(ApiError::new(
            StatusCode::NOT_FOUND,
            "session_unknown_or_expired",
        ));
    };

    let row_provenance = RowProvenance {
        run_id: run_id(),
        retrieval_first: session.retrieval_first,
        primed_from: session.primed_from.clone(),
    };

    let mut dispatcher = Dispatcher::new();
    for tool in windows_tools() {
        dispatcher.register(tool);
    }

    // The caller picks a candidate (app-side retry) or defaults to the winner.
    let candidate = match &request.plan_id {
        Some(plan_id) => session
            .candidates
            .iter()
            .find(|c| &c.plan.id == plan_id)
            .ok_or(ApiError::new(StatusCode::NOT_FOUND, "plan_unknown"))?,
        None => &session.candidates[session.selected],
    };

    // Declined consent is a real outcome: the ticket is withdrawn and
    // recorded, exactly as at the CLI prompt.
    if !request.consented {
        let label = OutcomeLabel::Withdrawn;
        record_outcome(
            &*state.corpus,
            &session.signature,
            &candidate.plan,
            label.clone(),
            &session.config_class,
            sign_off,
            None,
            Some(row_provenance),
            state.authority.as_ref(),
            true,
        )
        .await;
        return Ok(serde_json::json!({
            "schema_version": "cec-execute/v1",
            "executed": false,
            "label": wire_label(&label),
        }));
    }

    // The sign-off must meet the judge's required escalation — a verifier
    // cannot authorize a run the panel routed to a human.
    if session.escalation == Escalation::HumanConfirm && sign_off != SignOff::HumanConfirmed {
        return Err(ApiError::new(
            StatusCode::CONFLICT,
            "sign_off_below_required_escalation",
        ));
    }

    // A plan outside the agent's operation vocabulary is advisory-only:
    // never executed, and the refusal is recorded as an escalation.
    if !is_executable(&candidate.plan, &dispatcher) {
        let label = OutcomeLabel::EscalatedHumanUnresolved;
        record_outcome(
            &*state.corpus,
            &session.signature,
            &candidate.plan,
            label.clone(),
            &session.config_class,
            sign_off,
            None,
            Some(row_provenance),
            state.authority.as_ref(),
            true,
        )
        .await;
        return Ok(serde_json::json!({
            "schema_version": "cec-execute/v1",
            "executed": false,
            "label": wire_label(&label),
            "reason": "plan_advisory_only",
        }));
    }

    let granted = match sign_off {
        SignOff::HumanConfirmed => Consent::AllowDestructive,
        SignOff::VerifierConfirmed => Consent::AllowReversible,
        SignOff::Unconfirmed => Consent::ReadOnlyOnly,
    };

    // Judge-signed plan, re-verified at the executor; the consent gate still
    // refuses any step exceeding the granted level.
    let signer = SigningKey::generate();
    let signed = signer.sign(&candidate.plan);
    let execution = agent_core::execute_signed_plan(&dispatcher, &signed, &signer, granted)
        .await
        .map_err(|error| {
            eprintln!("serve: execution refused: {error}");
            ApiError::new(StatusCode::INTERNAL_SERVER_ERROR, "execution_refused")
        })?;

    let class = verification_class_for(&session.route, session.reproducibility);
    let post = crate::recollect_post_signature();
    let verdict = verify_outcome(&session.signature, post.as_ref(), class);
    let label = label_for(&session.route, &execution, &verdict);

    record_outcome(
        &*state.corpus,
        &session.signature,
        &candidate.plan,
        label.clone(),
        &session.config_class,
        sign_off,
        Some(verdict.to_verification(class)),
        Some(row_provenance),
        state.authority.as_ref(),
        true,
    )
    .await;

    // The post-execution envelope: pinned vocabulary only — action names and
    // ok flags, never step summaries (tool output can carry machine identity).
    Ok(serde_json::json!({
        "schema_version": "cec-execute/v1",
        "executed": true,
        "completed": execution.completed,
        "steps": execution
            .steps
            .iter()
            .map(|s| serde_json::json!({ "action": s.action, "ok": s.ok }))
            .collect::<Vec<_>>(),
        "verification": wire_verdict(&verdict),
        "label": wire_label(&label),
        "escalation_required": wire_escalation(&session.escalation),
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn offline_state() -> Arc<AppState> {
        Arc::new(AppState {
            args: crate::Args {
                describe: String::new(),
                endpoint: None,
                model: "local-model".into(),
                fast_endpoint: None,
                fast_model: None,
                corpus: None,
                offline: true,
                no_questions: true,
                sign_off: None,
                inventory_keys: None,
                json: true,
                serve: true,
                bind: "127.0.0.1:0".into(),
                allow_remote: false,
                allow_remote_inference: false,
            },
            corpus: Box::new(LocalCorpus::new()),
            authority: None,
            sessions: Mutex::new(HashMap::new()),
        })
    }

    const POISONED_DESCRIBE: &str = "DESKTOP-NATHAN01 (user nathan, MAC 00:1A:2B:3C:4D:5E, \
         serial SN12345678): explorer.exe crashes on login, WER bucket 0x1234; logs under \
         C:\\Users\\nathan; contact nathan@example.com or 192.168.1.20";

    #[tokio::test]
    async fn diagnose_returns_the_envelope_with_a_session_and_no_request_prose() {
        let state = offline_state();
        let envelope = handle_diagnose(
            &state,
            DiagnoseRequest {
                describe: POISONED_DESCRIBE.into(),
                inventory_keys: vec!["hostname:DESKTOP-NATHAN01".into(), "os:windows 11".into()],
            },
        )
        .await
        .expect("diagnose succeeds offline");

        assert_eq!(envelope["schema_version"], "cec-diagnose/v1");
        assert!(envelope["session_id"].is_string());
        let serialized = serde_json::to_string(&envelope).expect("serializes");
        // The API response is a new egress sink: planted identity in the
        // request and the inventory keys must not survive into it.
        for token in [
            "DESKTOP-NATHAN01",
            "nathan",
            "00:1A:2B:3C:4D:5E",
            "SN12345678",
            "192.168.1.20",
        ] {
            assert!(
                !serialized.to_lowercase().contains(&token.to_lowercase()),
                "planted identity {token:?} leaked into the API diagnose response"
            );
        }
    }

    #[tokio::test]
    async fn execute_without_consent_withdraws_and_consumes_the_session() {
        let state = offline_state();
        let envelope = handle_diagnose(
            &state,
            DiagnoseRequest {
                describe: "explorer.exe crashes on login 0x1234".into(),
                inventory_keys: vec![],
            },
        )
        .await
        .expect("diagnose succeeds");
        let session_id = envelope["session_id"]
            .as_str()
            .expect("session id")
            .to_string();

        let result = handle_execute(
            &state,
            ExecuteRequest {
                session_id: session_id.clone(),
                consented: false,
                sign_off: "human".into(),
                plan_id: None,
            },
        )
        .await
        .expect("withdrawal is a recorded outcome, not an error");
        assert_eq!(result["schema_version"], "cec-execute/v1");
        assert_eq!(result["executed"], false);
        assert_eq!(result["label"], "withdrawn");

        // One-shot: the session was consumed.
        let replay = handle_execute(
            &state,
            ExecuteRequest {
                session_id,
                consented: true,
                sign_off: "human".into(),
                plan_id: None,
            },
        )
        .await;
        assert!(replay.is_err(), "a consumed session must not be replayable");
    }

    #[tokio::test]
    async fn execute_refuses_a_sign_off_below_the_required_escalation() {
        let state = offline_state();
        let envelope = handle_diagnose(
            &state,
            DiagnoseRequest {
                describe: "explorer.exe crashes on login 0x1234".into(),
                inventory_keys: vec![],
            },
        )
        .await
        .expect("diagnose succeeds");
        // The offline winner is a state-changing plan with no sandbox
        // validation, so the judge requires HumanConfirm.
        assert_eq!(envelope["escalation"], "human_confirm");
        let session_id = envelope["session_id"]
            .as_str()
            .expect("session id")
            .to_string();

        let refused = handle_execute(
            &state,
            ExecuteRequest {
                session_id,
                consented: true,
                sign_off: "verifier".into(),
                plan_id: None,
            },
        )
        .await;
        assert!(
            refused.is_err(),
            "a verifier sign-off must not authorize a HumanConfirm run"
        );
    }

    #[tokio::test]
    async fn execute_with_human_sign_off_returns_the_post_execution_envelope() {
        let state = offline_state();
        let envelope = handle_diagnose(
            &state,
            DiagnoseRequest {
                describe: POISONED_DESCRIBE.into(),
                inventory_keys: vec![],
            },
        )
        .await
        .expect("diagnose succeeds");
        let session_id = envelope["session_id"]
            .as_str()
            .expect("session id")
            .to_string();

        let result = handle_execute(
            &state,
            ExecuteRequest {
                session_id,
                consented: true,
                sign_off: "human".into(),
                plan_id: None,
            },
        )
        .await
        .expect("execute succeeds (halts honestly off-Windows)");
        assert_eq!(result["schema_version"], "cec-execute/v1");
        assert_eq!(result["executed"], true);
        // Off-Windows the tools report unsupported, so the run cannot be
        // resolved — the label must be an honest escalation, never a
        // self-minted success.
        let label = result["label"].as_str().expect("label");
        assert!(
            label.starts_with("escalated_") || label == "reopened",
            "off-Windows execution must not claim resolution, got {label:?}"
        );
        // The post-execution envelope is de-identified: no request prose.
        let serialized = serde_json::to_string(&result).expect("serializes");
        assert!(
            !serialized.to_lowercase().contains("desktop-nathan01")
                && !serialized.contains("SN12345678"),
            "planted identity leaked into the API execute response"
        );
    }

    #[test]
    fn non_loopback_bind_is_refused_without_the_explicit_flag() {
        let remote: std::net::SocketAddr = "0.0.0.0:8127".parse().expect("addr");
        let local: std::net::SocketAddr = "127.0.0.1:8127".parse().expect("addr");
        assert!(validate_bind(&remote, false).is_err());
        assert!(validate_bind(&remote, true).is_ok());
        assert!(validate_bind(&local, false).is_ok());
    }

    #[test]
    fn execute_wire_values_are_pinned_for_cec_execute_v1() {
        assert_eq!(
            wire_label(&OutcomeLabel::ResolvedConfirmed),
            "resolved_confirmed"
        );
        assert_eq!(
            wire_label(&OutcomeLabel::ResolvedProvisional),
            "resolved_provisional"
        );
        assert_eq!(wire_label(&OutcomeLabel::Reopened), "reopened");
        assert_eq!(
            wire_label(&OutcomeLabel::EscalatedHardware {
                part_class: "psu".into()
            }),
            "escalated_hardware:psu"
        );
        assert_eq!(
            wire_label(&OutcomeLabel::EscalatedHumanUnresolved),
            "escalated_human_unresolved"
        );
        assert_eq!(wire_label(&OutcomeLabel::Withdrawn), "withdrawn");
        assert_eq!(wire_verdict(&Verdict::Pass), "pass");
        assert_eq!(wire_verdict(&Verdict::ProvisionalPass), "provisional_pass");
        assert_eq!(wire_verdict(&Verdict::Fail { recurring: vec![] }), "fail");
        assert_eq!(wire_verdict(&Verdict::Unverified), "unverified");
        assert_eq!(wire_verdict(&Verdict::OffMachine), "off_machine");
    }
}
