// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 The cec-support-agent authors
//! cec-support-agent: the headless CLI face of the engine.
//!
//! It assembles the pipeline from the diagram — run the intake interview to
//! map the request to a structured case (asking the standard helpdesk
//! follow-ups for whatever the description leaves open), collect diagnostics,
//! derive a de-identified fault signature by structured extraction, route the
//! case (software-state, hardware-evidenced, or ambiguous), generate candidate
//! plans through the swarm with distinct causal hypotheses, run the judge
//! panel, and report the winner with its required escalation. Passing
//! `--sign-off` closes the loop: execute under the consent gate, verify the
//! outcome against the original signature, emit an outcome label, and record
//! the de-identified triple through the corpus sign-off gate.
//!
//! Cold start (a bootstrap invariant): with no `--endpoint` (or with
//! `--offline`) the agent runs the whole pipeline using a model-free heuristic
//! candidate and an empty in-memory corpus. No CEC-hosted service is required.

use std::io::{IsTerminal, Write};
use std::process::ExitCode;

use agent_core::{verify_outcome, Consent, Dispatcher, Verdict, VerificationClass};
use async_trait::async_trait;
use common::{
    extract_symptoms, Candidate, CandidateSource, CoarseHostInventory, ConfigClass,
    DiagnosticEvent, EventKind, ExecutionResult, ExternalInventory, FaultSignature,
    InventoryProvider, Plan, PlanStep, Risk, Severity,
};
use corpus_client::{
    Contribution, CorpusStore, FileCorpus, LocalCorpus, Outcome, OutcomeLabel, RowProvenance,
    SignOff,
};
use inference::{ChatCompletionRequest, ChatMessage, Completer, Endpoint, OpenAiClient};
use intake::{
    Interview, Interviewer, ModelInterviewer, RecentChange, Reproducibility, ScriptedInterviewer,
};
use panel::{best_of_n, required_escalation, route_for, Escalation, HeuristicJudge, Judge, Route};
use provenance::SigningKey;
use swarm::{Generator, SandboxValidator, Swarm, SwarmError};
use tools_windows::{firmware_advisory, windows_tools, BoardIdentity};

/// Parsed command-line arguments for the `diagnose` flow.
struct Args {
    describe: String,
    endpoint: Option<String>,
    model: String,
    fast_endpoint: Option<String>,
    fast_model: Option<String>,
    corpus: Option<String>,
    offline: bool,
    no_questions: bool,
    sign_off: Option<SignOff>,
    /// `--inventory-keys <file|->`: identity-free config keys from an external
    /// inventory source (e.g. a device-inventory tool driving the engine over a
    /// process boundary). Replaces the coarse os/arch/family config class.
    inventory_keys: Option<String>,
    /// `--json`: emit a machine-readable `cec-diagnose/v1` result envelope on
    /// stdout (for an embedder driving the engine as a sidecar).
    json: bool,
}

#[tokio::main]
async fn main() -> ExitCode {
    match parse_args() {
        Ok(None) => ExitCode::SUCCESS,
        Ok(Some(args)) => match run(args).await {
            Ok(()) => ExitCode::SUCCESS,
            Err(error) => {
                eprintln!("error: {error:#}");
                ExitCode::FAILURE
            }
        },
        Err(message) => {
            eprintln!("error: {message}");
            eprintln!();
            print_help();
            ExitCode::FAILURE
        }
    }
}

fn parse_args() -> Result<Option<Args>, String> {
    let mut describe = String::new();
    let mut endpoint = None;
    let mut model = "local-model".to_string();
    let mut fast_endpoint = None;
    let mut fast_model = None;
    let mut corpus = None;
    let mut offline = false;
    let mut no_questions = false;
    let mut sign_off = None;
    let mut inventory_keys = None;
    let mut json = false;

    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "-h" | "--help" => {
                print_help();
                return Ok(None);
            }
            "-V" | "--version" => {
                println!("cec-support-agent {}", env!("CARGO_PKG_VERSION"));
                return Ok(None);
            }
            "--offline" => offline = true,
            "--no-questions" => no_questions = true,
            "--endpoint" => {
                endpoint = Some(args.next().ok_or("--endpoint requires a value")?);
            }
            "--model" => {
                model = args.next().ok_or("--model requires a value")?;
            }
            "--fast-endpoint" => {
                fast_endpoint = Some(args.next().ok_or("--fast-endpoint requires a value")?);
            }
            "--fast-model" => {
                fast_model = Some(args.next().ok_or("--fast-model requires a value")?);
            }
            "--corpus" => {
                corpus = Some(args.next().ok_or("--corpus requires a value")?);
            }
            "--sign-off" => {
                let value = args.next().ok_or("--sign-off requires a value")?;
                sign_off = Some(match value.as_str() {
                    "verifier" => SignOff::VerifierConfirmed,
                    "human" => SignOff::HumanConfirmed,
                    other => {
                        return Err(format!(
                            "--sign-off must be 'verifier' or 'human', got {other:?}"
                        ))
                    }
                });
            }
            "--describe" => {
                describe = args.next().ok_or("--describe requires a value")?;
            }
            "--inventory-keys" => {
                inventory_keys = Some(args.next().ok_or("--inventory-keys requires a value")?);
            }
            "--json" => json = true,
            // The single verb; accepted for readability of the command line.
            "diagnose" => {}
            // Generate a sign-off authority key pair. The PUBLIC key goes on the
            // engine (CEC_SIGNOFF_PUBKEY) to ENFORCE attestation; the SECRET seed
            // (CEC_SIGNOFF_SEED) is held only where sign-off is performed — never
            // on the engine in a split deployment, or alongside it for a single
            // operator who is themselves the authority.
            "gen-signoff-key" => {
                let authority = corpus_client::SignOffAuthority::generate();
                println!("# cec-support-agent sign-off authority key — store the seed securely");
                println!("# PUBLIC KEY (engine, enforces attestation):");
                println!(
                    "export CEC_SIGNOFF_PUBKEY={}",
                    authority.public_key().to_hex()
                );
                println!("# SECRET SEED (sign-off side only; keep off the engine):");
                println!("export CEC_SIGNOFF_SEED={}", authority.seed_hex());
                return Ok(None);
            }
            other => return Err(format!("unknown argument: {other}")),
        }
    }

    if describe.is_empty() {
        describe =
            "unspecified fault: collect diagnostics and propose a safe first step".to_string();
    }

    Ok(Some(Args {
        describe,
        endpoint,
        model,
        fast_endpoint,
        fast_model,
        corpus,
        offline,
        no_questions,
        sign_off,
        inventory_keys,
        json,
    }))
}

async fn run(args: Args) -> anyhow::Result<()> {
    // Under --json, stdout carries ONLY the cec-diagnose/v1 envelope; the human-
    // readable trace goes to stderr so an embedder gets clean machine output on stdout.
    macro_rules! human {
        () => { if args.json { eprintln!() } else { println!() } };
        ($($a:tt)*) => {{ if args.json { eprintln!($($a)*) } else { println!($($a)*) } }};
    }
    macro_rules! hprint {
        ($($a:tt)*) => {{ if args.json { eprint!($($a)*) } else { print!($($a)*) } }};
    }
    human!("cec-support-agent: diagnose");
    human!("  request: {}", args.describe);

    // 0. Intake: map the person's input to an actual case (the "identify the
    //    problem" step of the standard troubleshooting methodology). The
    //    intake judge infers what the description already answers and, when a
    //    terminal is attached, asks the helpdesk follow-ups for the rest.
    //    Which field is asked and when the funnel stops is always the
    //    deterministic Interview; with an endpoint configured the model only
    //    sharpens the wording, and any failure falls back to the script.
    //    Headless runs proceed with what was inferred.
    let mut interview = Interview::new(&args.describe);
    if !args.no_questions && std::io::stdin().is_terminal() {
        // Phrasing one question is the lightest request in the pipeline, so
        // it always goes to the fast tier when one is configured. A short
        // timeout keeps it honest: past it, the interviewer falls back to the
        // scripted prompt and the user is never left at a dead prompt.
        let intake_base = args.fast_endpoint.as_ref().or(args.endpoint.as_ref());
        let intake_client = match (intake_base, args.offline) {
            (Some(base_url), false) => Some(OpenAiClient::new(
                Endpoint::new(base_url).with_timeout(std::time::Duration::from_secs(30)),
            )),
            _ => None,
        };
        let intake_model = args
            .fast_model
            .clone()
            .unwrap_or_else(|| args.model.clone());
        let interviewer: Box<dyn Interviewer + '_> = match &intake_client {
            Some(client) => Box::new(ModelInterviewer::new(client, intake_model)),
            None => Box::new(ScriptedInterviewer),
        };
        while let Some(question) = interview.next_question() {
            let prompt = interviewer.ask(&interview, question.kind).await;
            human!("  ? {prompt}");
            hprint!("  > ");
            std::io::stdout().flush()?;
            let mut line = String::new();
            if std::io::stdin().read_line(&mut line)? == 0 {
                break; // EOF: proceed with what we have.
            }
            interview.answer(question.kind, line.trim());
        }
    }
    let case = interview.into_case();
    human!("  case: {}", case.brief());
    // The register check: how reasoned the person's explanation was decides
    // how measured the response is. It calibrates teaching (definitions,
    // examples, walkthroughs), never safety or what gets checked.
    human!(
        "  register: {:?} ({})",
        case.fluency,
        match case.fluency {
            common::Fluency::Guided => "terms will be explained, with examples",
            common::Fluency::Technical => "the explanation was precise; responses will be concise",
        }
    );

    // 1. Collect diagnostics (stubbed here from the request text). The fault
    //    signature comes from the case: structured extraction over the
    //    statement plus every interview answer — only vocabulary terms,
    //    structured codes, and module names survive, so it is de-identified
    //    by construction.
    let events = collect_diagnostics(&args.describe);
    let signature = case.signature();
    human!(
        "  fault signature: {} ({} structured symptom(s))",
        signature.fingerprint,
        signature.symptoms.len()
    );

    // 2. The config class scopes every corpus row and query to like configs:
    //    the BOM revision on a CEC build, a derived inventory hash otherwise.
    let config_class = host_config_class(&args)?;
    human!("  config class: {}", config_class.key());

    // 3. Routing precedes scoring: the routing verdict determines which gates
    //    are load-bearing. A hardware-evidenced case's deliverable is a
    //    diagnosis plus a parts action; an ambiguous case escalates.
    let route = route_for(&signature);
    human!("  route: {route:?}");
    if case.fluency == common::Fluency::Guided {
        human!("  what this means: {}", route.explanation());
    }

    // The agent's operation vocabulary, behind the consent gate. Built here
    // because routing may immediately use the read-only tools for enrichment.
    let mut dispatcher = Dispatcher::new();
    for tool in windows_tools() {
        dispatcher.register(tool);
    }

    // Board enrichment: when the evidence implicates the platform or a
    // driver, read the board identity (read-only, auto-consented;
    // configuration fields only, never serial numbers) and emit the firmware
    // advisory — at minimum the user leaves with their exact board, their
    // installed BIOS version, and precise download instructions. Flashing is
    // advisory-only by design: the agent never executes firmware changes.
    if matches!(route, Route::HardwareEvidenced { .. })
        || case.recent_change == RecentChange::DriverUpdate
    {
        match dispatcher
            .dispatch("board_info", serde_json::Value::Null, Consent::ReadOnlyOnly)
            .await
        {
            Ok(outcome) if outcome.ok => match BoardIdentity::from_tool_data(&outcome.data) {
                Some(board) => {
                    human!(
                        "  board: {} {} — BIOS {} ({})",
                        board.manufacturer,
                        board.product,
                        board.bios_version,
                        board.bios_date
                    );
                    let advisory = firmware_advisory(&board, case.fluency);
                    human!("  firmware advisory ({}):", advisory.vendor);
                    for step in &advisory.steps {
                        human!("    {step}");
                    }
                }
                None => human!("  board: identity payload unrecognized"),
            },
            Ok(outcome) => human!("  board: unavailable ({})", outcome.summary),
            Err(error) => human!("  board: unavailable ({error})"),
        }
    }

    // Sign-off attestation (MH-1): if CEC_SIGNOFF_PUBKEY is set, the corpus
    // store ENFORCES that every confirmed row carries a valid ed25519
    // attestation by that authority (a self-asserted HumanConfirmed is refused).
    // If CEC_SIGNOFF_SEED is set, this run holds the authority and attests its
    // own outcomes (single-operator mode). A set-but-invalid key is a hard error
    // — never silently run unprotected. `signoff_authority` is threaded into
    // record_outcome so each contribution is attested before submit.
    let signoff_authority = parse_env_authority()?;
    let mut signoff_pubkey = parse_env_pubkey()?;
    // A seed set without an explicit pubkey would otherwise self-attest into a
    // store that does not enforce — attesting but not protected, silently. Derive
    // the enforcing key from the seed so single-operator mode actually enforces:
    // never attest without enforcing.
    let derived_enforcement = signoff_pubkey.is_none() && signoff_authority.is_some();
    if derived_enforcement {
        signoff_pubkey = signoff_authority.as_ref().map(|a| a.public_key());
    }
    if let Some(pubkey) = &signoff_pubkey {
        human!(
            "  sign-off: attestation ENFORCED (authority {}…){}",
            &pubkey.id(),
            if derived_enforcement {
                "; enforcing key derived from CEC_SIGNOFF_SEED \
                 (single-operator mode: this run self-attests and enforces)"
            } else if signoff_authority.is_some() {
                "; this run holds the seed and self-attests (single-operator mode)"
            } else {
                "; this run must receive attestations (none produced here)"
            }
        );
    }

    // 4. The corpus: file-backed when `--corpus` names a path — the
    //    self-hosted flywheel, where the next run facing a known signature
    //    starts from this run's outcome — and in-memory otherwise. Cold start
    //    either way: no CEC service is required.
    let corpus: Box<dyn CorpusStore> = match &args.corpus {
        Some(path) => {
            let mut file = FileCorpus::open(path)?;
            if let Some(pubkey) = &signoff_pubkey {
                // Re-admits every at-rest row under the authority; a corpus whose
                // on-disk history is unattested (or forged) is refused here.
                file = file.with_authority(pubkey.clone())?;
            }
            human!("  corpus: file-backed at {path} ({} row(s))", file.len());
            Box::new(file)
        }
        None => {
            let mut local = LocalCorpus::new();
            if let Some(pubkey) = &signoff_pubkey {
                local = local.with_authority(pubkey.clone());
            }
            Box::new(local)
        }
    };
    let known = corpus.query(&signature, &config_class).await?;
    human!(
        "  corpus: {} known mapping(s) for this signature at this config class",
        known.len()
    );

    // 5. Generate candidate plans. Retrieval-first: when the corpus holds
    //    confirmed precedent for this signature at this config class, the
    //    precedents join the slate as CorpusPrimed candidates and de novo
    //    model generation is skipped — adaptation is cheaper and more
    //    reliable than synthesis. Otherwise the swarm fans out, each model
    //    generator seeded with a distinct causal hypothesis — not
    //    temperature-jittered variants of one guess — and the model-free
    //    heuristic generator always contributes, so the slate is never empty.
    //    A failed generator degrades the gather instead of emptying it.
    let mut candidates: Vec<Candidate> = known
        .iter()
        .map(|mapping| {
            Candidate::new(
                mapping.plan.clone(),
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
    if retrieval_first {
        human!(
            "  retrieval-first: adapting {} precedent plan(s); skipping de novo generation",
            candidates.len()
        );
    }

    let swarm = Swarm::new();
    human!(
        "  swarm: {} trusted node(s) registered",
        swarm.nodes().len()
    );
    let mut generators: Vec<Box<dyn Generator>> = vec![Box::new(HeuristicGenerator)];
    if !args.offline && !retrieval_first {
        if let Some(base_url) = &args.endpoint {
            // Model tiering: a routine ticket — every intake field established
            // and the route software-state — samples the fast tier when one is
            // configured. Vague, ambiguous, or hardware-evidenced cases keep
            // the heavyweight model: novelty escalates, in model choice as in
            // sign-off.
            let (generation_url, generation_model) = if is_simple_request(&route, &case) {
                let url = args
                    .fast_endpoint
                    .clone()
                    .unwrap_or_else(|| base_url.clone());
                let model = args
                    .fast_model
                    .clone()
                    .unwrap_or_else(|| args.model.clone());
                (url, model)
            } else {
                (base_url.clone(), args.model.clone())
            };
            human!("  generation model: {generation_model}");
            for hypothesis in HYPOTHESES {
                generators.push(Box::new(ModelGenerator {
                    base_url: generation_url.clone(),
                    model: generation_model.clone(),
                    hypothesis,
                    // The interview findings prime every hypothesis: "began
                    // after a driver update" is exactly what a generator
                    // should weigh. The brief is built only from enum fields,
                    // so it is prompt-safe.
                    case_brief: case.brief(),
                }));
            }
        }
    }
    let gathered = swarm.gather(&generators, &events).await?;
    for failure in &gathered.failures {
        eprintln!("  note: a generator failed ({failure}); continuing with the rest");
    }
    candidates.extend(gathered.candidates);

    // 5b. Reconcile model-claimed risk against each registered tool's real risk.
    //     A generator (or a corpus row) cannot mislabel a state-changing action
    //     as ReadOnly to understate the rendered consent or slip past the
    //     consent gate: an under-stated step is RAISED to the tool's true risk
    //     before the plan is judged, consented to, or executed.
    for candidate in &mut candidates {
        let (reconciled, corrections) = dispatcher.reconcile_risk(&candidate.plan);
        for correction in &corrections {
            human!(
                "  risk reconciled: '{}' claimed {:?} but is {:?} — raised before consent",
                correction.action,
                correction.claimed,
                correction.actual
            );
        }
        candidate.plan = reconciled;
    }

    // 6/7. Judge panel: score the slate and pick best-of-N. Then sandbox-validate
    //      the WINNER and decide the escalation from the route, the REAL
    //      validation state, and the risk/score ladder. The CLI configures no VM
    //      backend, so the winner is unvalidated — and unvalidated equals
    //      escalate: a state-changing plan still requires human sign-off. A
    //      deployment that wires a disposable-VM `SandboxValidator` gets positive
    //      validation evidence, and a clean apply can lower the bar.
    let judge = HeuristicJudge;
    let (index, best, score) =
        best_of_n(&judge, &candidates).expect("the heuristic candidate is always present");
    let sandbox: Option<Box<dyn SandboxValidator>> = None; // deployment wires a disposable-VM backend
    if sandbox.is_none() {
        human!(
            "  sandbox: no validator configured; the winner is unvalidated (unvalidated = escalate)"
        );
    }
    let sandbox_validated = sandbox_validated_for(sandbox.as_deref(), best, args.json).await;
    let escalation = required_escalation(&route, sandbox_validated, best, &score);

    let consent_needed = match best.plan.risk() {
        Risk::ReadOnly => Consent::ReadOnlyOnly,
        Risk::Reversible => Consent::AllowReversible,
        Risk::Destructive => Consent::AllowDestructive,
    };

    human!();
    human!(
        "selected candidate #{index} of {} (source: {:?})",
        candidates.len(),
        best.source
    );
    human!("  title:      {}", best.plan.title);
    human!("  rationale:  {}", best.rationale);
    human!("  risk:       {:?}", best.plan.risk());
    human!("  score:      {:.3}", score.total());
    human!("  escalation: {escalation:?}");
    human!("  steps:");
    for (i, step) in best.plan.steps.iter().enumerate() {
        human!(
            "    {}. [{:?}] {} -> {}",
            i + 1,
            step.risk,
            step.description,
            step.action
        );
    }
    human!("  tools available: {:?}", dispatcher.tool_names());
    human!("  consent needed:  {consent_needed:?}");

    // Machine-readable result envelope for an embedder driving the engine as a
    // sidecar (cec-diagnose/v1). De-identified by construction (see
    // `diagnose_envelope`). The ONLY line written to stdout under --json — the
    // human trace went to stderr via `human!`/`tprintln!`.
    if args.json {
        let envelope = diagnose_envelope(
            &signature,
            &config_class,
            &route,
            &candidates,
            index,
            &consent_needed,
            &escalation,
        );
        println!(
            "{}",
            serde_json::to_string(&envelope).expect("diagnose envelope serializes")
        );
    }

    human!();
    match args.sign_off {
        None => {
            if !args.json {
                human!(
                    "Execution and corpus write-back are gated on {escalation:?} sign-off and are NOT \
                     performed by this run. Re-run with --sign-off <verifier|human> to execute the \
                     winning plan, verify the outcome, and record the labeled result."
                );
            }
        }
        Some(sign_off) => {
            // Run-provenance for every row this run records: a fresh run id, the
            // retrieval-first lane, and which precedents primed the slate — so the
            // corpus counts only independent confirmations (EI-03/A5).
            let row_provenance = RowProvenance {
                run_id: run_id(),
                retrieval_first,
                primed_from: known.iter().map(|m| m.plan.id.clone()).collect(),
            };

            // The sign-off must meet the judge's required escalation: a
            // verifier cannot authorize a run the panel routed to a human.
            if escalation == Escalation::HumanConfirm && sign_off != SignOff::HumanConfirmed {
                human!(
                    "sign-off refused: the judge requires HumanConfirm for this run \
                     (route: {route:?}, sandbox: unvalidated). Re-run with --sign-off human."
                );
                return Ok(());
            }

            // Sign-off authorizes execution at a matching consent level: a
            // verifier may authorize reversible changes; a human may authorize
            // destructive ones. The consent gate still refuses any step that
            // exceeds the granted level.
            let granted = match sign_off {
                SignOff::HumanConfirmed => Consent::AllowDestructive,
                SignOff::VerifierConfirmed => Consent::AllowReversible,
                SignOff::Unconfirmed => Consent::ReadOnlyOnly,
            };
            human!("sign-off: {sign_off:?} -> executing under consent {granted:?}");

            // 9. The agent executes only plans drawn from its operation
            //    vocabulary (the registered tools); anything else is
            //    advisory-only. Executable candidates are ranked by judge
            //    score so the retry loop has a next-best to fall back to.
            let mut ranked: Vec<(f32, &Candidate)> = candidates
                .iter()
                .filter(|candidate| is_executable(&candidate.plan, &dispatcher))
                .map(|candidate| (judge.score(candidate).total(), candidate))
                .collect();
            ranked.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));

            if ranked.is_empty() {
                human!(
                    "no executable plan: every candidate contains operations outside the \
                     agent's vocabulary (advisory-only). Escalating to a human."
                );
                let label = OutcomeLabel::EscalatedHumanUnresolved;
                record_outcome(
                    &*corpus,
                    &signature,
                    &best.plan,
                    label,
                    &config_class,
                    sign_off,
                    None,
                    Some(row_provenance.clone()),
                    signoff_authority.as_ref(),
                    args.json,
                )
                .await;
                return Ok(());
            }
            if !is_executable(&best.plan, &dispatcher) {
                human!(
                    "  note: the judge's winner is advisory-only; executing the best plan \
                     from the agent's vocabulary instead"
                );
            }

            // 10. The judge signs what may cross into the on-machine zone.
            //     execute_signed_plan re-verifies the signature at the
            //     executor before any step runs.
            let signer = SigningKey::generate();

            // 11. Execute → verify → label, with a bounded retry: a failed
            //     attempt is recorded as a hard negative and the next-best
            //     plan gets one chance, then the ticket escalates instead of
            //     thrashing.
            let mut final_label: Option<OutcomeLabel> = None;
            for (attempt, (_, candidate)) in ranked.iter().take(MAX_ATTEMPTS).enumerate() {
                if attempt > 0 {
                    human!();
                    human!("retry {attempt}: next-best plan '{}'", candidate.plan.title);
                }

                // Consent is to a rendered plan, never an opaque script:
                // plain-language steps, risk class, and the restore-point
                // coverage boundary.
                human!("{}", render_consent(&candidate.plan));
                if std::io::stdin().is_terminal() {
                    hprint!("  Type 'yes' to consent, anything else to decline: ");
                    std::io::stdout().flush()?;
                    let mut line = String::new();
                    std::io::stdin().read_line(&mut line)?;
                    if !line.trim().eq_ignore_ascii_case("yes") {
                        human!("  consent declined; ticket withdrawn");
                        let label = OutcomeLabel::Withdrawn;
                        record_outcome(
                            &*corpus,
                            &signature,
                            &candidate.plan,
                            label.clone(),
                            &config_class,
                            sign_off,
                            None,
                            Some(row_provenance.clone()),
                            signoff_authority.as_ref(),
                            args.json,
                        )
                        .await;
                        final_label = Some(label);
                        break;
                    }
                } else {
                    human!("  (headless run: --sign-off {sign_off:?} is the recorded consent)");
                }

                let signed = signer.sign(&candidate.plan);
                human!("  plan signature: {}…", &signed.signature[..16]);
                let execution =
                    agent_core::execute_signed_plan(&dispatcher, &signed, &signer, granted)
                        .await
                        .map_err(|error| anyhow::anyhow!(error))?;
                for step in &execution.steps {
                    human!(
                        "  step {} [{}] {} -> {}",
                        step.step,
                        if step.ok { "ok" } else { "fail" },
                        step.action,
                        step.summary
                    );
                }
                human!(
                    "  execution: {} ({}/{} step(s) ok)",
                    if execution.completed {
                        "completed"
                    } else {
                        "halted"
                    },
                    execution.steps.iter().filter(|s| s.ok).count(),
                    execution.steps.len()
                );

                // Verify the outcome: re-collect from the live machine and diff
                // against the original failure signature. The claim "fixed" is
                // only valid against the same instrument that established
                // "broken", so the post-state must be a REAL re-collection, not
                // a re-read of the request text. `recollect_post_signature`
                // returns None for the bootstrap (no live tools) — and a None
                // post yields `Verdict::Unverified`, so a run that never
                // observed the machine afterwards escalates instead of being
                // recorded as resolved (NR-1).
                let class = verification_class_for(&route, case.reproducibility);
                let post = recollect_post_signature();
                if post.is_none() {
                    human!(
                        "  verification: no live re-collection available — the outcome cannot be \
                         confirmed and will escalate for human verification (NR-1)"
                    );
                }
                let verdict = verify_outcome(&signature, post.as_ref(), class);
                human!("  verification ({class:?}): {verdict:?}");
                if let Verdict::Fail { recurring } = &verdict {
                    human!(
                        "  hard negative: {} original symptom(s) recurred; the failed plan \
                         and this diff enter the retry context",
                        recurring.len()
                    );
                }

                // Sign-off is the labeling event: every attempt emits a
                // label — a failure enters the corpus as a hard negative, not
                // a discard — because an unlabeled ticket is corpus poison.
                let label = label_for(&route, &execution, &verdict);
                human!("  outcome label: {label:?}");
                // Bind the verifier's verdict to the row: a resolved label is
                // gated on a matching passing verdict, and stays auditable.
                record_outcome(
                    &*corpus,
                    &signature,
                    &candidate.plan,
                    label.clone(),
                    &config_class,
                    sign_off,
                    Some(verdict.to_verification(class)),
                    Some(row_provenance.clone()),
                    signoff_authority.as_ref(),
                    args.json,
                )
                .await;

                // Hardware verdicts and resolved outcomes end the loop; only
                // a software-state failure earns a retry.
                let retry = matches!(route, Route::SoftwareState | Route::Ambiguous)
                    && !label.is_resolved();
                final_label = Some(label);
                if !retry {
                    break;
                }
            }

            if let Some(label) = final_label {
                human!();
                human!("ticket label: {label:?}");
                if case.fluency == common::Fluency::Guided {
                    human!("  what this means: {}", explain_label(&label));
                }
            }
        }
    }
    Ok(())
}

/// Retry cap (bounded, time-boxed remediation): after this many failed
/// attempts the ticket escalates to a human instead of thrashing the machine.
const MAX_ATTEMPTS: usize = 2;

/// Whether every step's action is in the agent's operation vocabulary (the
/// registered tools). A plan with out-of-vocabulary actions — e.g. a model's
/// free-text "review" step — is advisory-only and never agent-executed; an
/// empty plan has nothing to execute.
fn is_executable(plan: &Plan, dispatcher: &Dispatcher) -> bool {
    !plan.steps.is_empty()
        && plan
            .steps
            .iter()
            .all(|step| dispatcher.contains(&step.action))
}

/// Render the plain-language consent for a plan: what each step does and
/// whether it can be undone, in words a non-technical user can act on.
/// Consent to an opaque script is liability theater — and so is consent to
/// jargon the user cannot evaluate.
fn render_consent(plan: &Plan) -> String {
    let mut text = format!(
        "  Permission needed. The support agent would like to run these steps on this \
         computer (plan: '{}'):\n",
        plan.title
    );
    for (i, step) in plan.steps.iter().enumerate() {
        let risk = match step.risk {
            Risk::ReadOnly => "this step only looks at information — it changes nothing",
            Risk::Reversible => "this step makes a change that can be undone",
            Risk::Destructive => "CAUTION: this step makes a change that can NOT be easily undone",
        };
        text.push_str(&format!("    {}. {} ({risk})\n", i + 1, step.description));
    }
    text.push_str(
        "    Before anything is changed, a 'restore point' is saved. That is a snapshot \
         of Windows' own files and settings, so the system can be put back the way it \
         was if a change causes trouble.\n    Good to know: a restore point protects \
         Windows files, settings, and drivers. It does NOT include your personal files \
         (documents, photos — these steps do not touch them), and it can NOT undo \
         updates to the BIOS (the main board's built-in software).",
    );
    text
}

/// A plain-language explanation of the final ticket label: what happened,
/// what it means for the user, and what happens next. Shown on the user's
/// screen next to the label itself.
fn explain_label(label: &OutcomeLabel) -> &'static str {
    match label {
        OutcomeLabel::ResolvedConfirmed => {
            "The fix was applied and the computer was checked again afterwards: the \
             original problem is no longer present."
        }
        OutcomeLabel::ResolvedProvisional => {
            "The fix was applied and things look good so far. Because this problem came \
             and went, one clean check is not enough to call it gone — it will be \
             watched, and the case reopens automatically if it comes back."
        }
        OutcomeLabel::Reopened => {
            "The problem came back after a fix had looked good, so the case is open \
             again and will be looked at further."
        }
        OutcomeLabel::EscalatedHardware { .. } => {
            "The evidence points at a physical part, which software cannot repair. A \
             person will take it from here (inspection, and possibly a replacement \
             part). Anything run on the computer today was only a temporary measure."
        }
        OutcomeLabel::EscalatedHumanUnresolved => {
            "The automatic steps could not safely fix this, so the case is being handed \
             to a person. Nothing was left half-done: any step that could not finish \
             was stopped."
        }
        OutcomeLabel::Withdrawn => {
            "You declined, so nothing was changed on the computer and the case is \
             closed."
        }
    }
}

/// Record one labeled outcome through the corpus evidence-integrity gate.
/// `Contribution::new` strips the plan to its action vocabulary; the gate
/// refuses — in code — an unconfirmed outcome, a resolved label with no matching
/// passing verdict, or a resolved destructive plan without human sign-off. The
/// `verification` is the verdict bound to the row so a resolved outcome is
/// auditable against its evidence; `None` for outcomes that never executed
/// (a withdrawn ticket, or no executable plan).
/// Route a human-trace line to stderr under `--json` (keeping stdout pure for the
/// single `cec-diagnose/v1` envelope) or to stdout otherwise. Module-scoped so the
/// free helpers reached from `run()` honor the stdout contract too — the `human!`
/// macro is local to `run()` and does not cover functions it calls.
macro_rules! tprintln {
    ($json:expr, $($a:tt)*) => {{ if $json { eprintln!($($a)*) } else { println!($($a)*) } }};
}

#[allow(clippy::too_many_arguments)]
async fn record_outcome(
    corpus: &dyn CorpusStore,
    signature: &FaultSignature,
    plan: &Plan,
    label: OutcomeLabel,
    config_class: &ConfigClass,
    sign_off: SignOff,
    verification: Option<common::Verification>,
    provenance: Option<RowProvenance>,
    authority: Option<&corpus_client::SignOffAuthority>,
    json: bool,
) {
    // The validating de-id mints run inside `Contribution::new`: a plan whose
    // action/id is not admissible vocabulary is REFUSED here (the row never
    // forms), rather than de-identified in name only. A refusal is a guard hit,
    // not a normal outcome — surface it and do not record.
    let mut contribution = match Contribution::new(
        Outcome {
            signature: signature.clone(),
            plan: plan.clone(),
            label: label.clone(),
            verification,
        },
        config_class.clone(),
        sign_off,
    ) {
        Ok(contribution) => contribution,
        Err(reject) => {
            tprintln!(
                json,
                "  corpus: row REFUSED by the leak guard ({reject}); not recorded"
            );
            return;
        }
    };
    if let Some(provenance) = provenance {
        contribution = contribution.with_provenance(provenance);
    }
    // Attest AFTER provenance so the run-provenance pin is bound into the
    // signature (a valid attestation cannot be replayed onto a fabricated run).
    if let Some(authority) = authority {
        contribution = contribution.attested_by(authority);
    }
    match corpus.submit(&contribution).await {
        Ok(()) => tprintln!(
            json,
            "  corpus: outcome recorded (label={label:?}, sign-off={sign_off:?})"
        ),
        Err(error) => tprintln!(json, "  corpus: submit refused: {error}"),
    }
}

fn collect_diagnostics(describe: &str) -> Vec<DiagnosticEvent> {
    // A real run gathers logs, WER, WHEA, and CIM state via the Windows tools;
    // the bootstrap seeds a single event from the request text.
    vec![DiagnosticEvent::new(
        EventKind::Log,
        "support-request",
        describe,
        Severity::Error,
        0,
    )]
}

/// Whether the chosen plan was POSITIVELY validated in a disposable sandbox.
/// With no validator configured (the CLI default) this is `false` — and
/// unvalidated equals escalate, so a state-changing plan still requires human
/// sign-off. A deployment that wires a `SandboxValidator` (a disposable VM with
/// no user data) gets a real per-plan report: a clean apply is positive
/// evidence that lowers the escalation bar; a dirty apply or a validation error
/// does not (it stays conservative — escalate).
async fn sandbox_validated_for(
    sandbox: Option<&dyn SandboxValidator>,
    best: &Candidate,
    json: bool,
) -> bool {
    let Some(validator) = sandbox else {
        return false;
    };
    match validator.validate(best).await {
        Ok(report) => {
            tprintln!(
                json,
                "  sandbox: {} — {}",
                if report.applied_cleanly {
                    "validated cleanly"
                } else {
                    "did NOT apply cleanly (stays unvalidated)"
                },
                report.notes
            );
            report.applied_cleanly
        }
        Err(error) => {
            tprintln!(
                json,
                "  sandbox: validation failed ({error}); treating as unvalidated"
            );
            false
        }
    }
}

/// Re-collect the post-execution signature from the LIVE machine, for the
/// verification diff. A genuine re-collection re-runs the diagnostic tools
/// (event log, WER/WHEA, CIM) against the host and builds a fresh signature with
/// the same instrument that established the fault. The bootstrap has no live
/// collector, so this returns `None` — and a `None` post is treated as
/// [`Verdict::Unverified`], not a pass: re-reading the request text is not an
/// observation of the post-fix state, so a run that never re-observed the
/// machine escalates instead of being recorded as resolved (NR-1). A Windows
/// build wires the real re-collection (via `tools-windows`) here.
fn recollect_post_signature() -> Option<FaultSignature> {
    None
}

/// Build the fault signature from the structured symptoms of every event.
/// Free text never reaches the signature: extraction keeps only vocabulary
/// terms, hex codes, prefixed ids, and module names. This is the builder a live
/// re-collection (`recollect_post_signature` on a Windows build) uses to turn
/// re-collected events into the post-fix signature; it is exercised by the
/// tests today.
#[allow(dead_code)] // wired in by the real (Windows) re-collection; used by tests
fn signature_of(events: &[DiagnosticEvent]) -> FaultSignature {
    let mut symptoms: Vec<common::Symptom> = events
        .iter()
        .flat_map(|event| extract_symptoms(&event.message))
        .collect();
    symptoms.sort_unstable_by(|a, b| a.0.cmp(&b.0));
    symptoms.dedup();
    FaultSignature::from_symptoms(symptoms)
}

/// Build the `cec-diagnose/v1` machine-readable result envelope for an embedder
/// driving the engine as a sidecar. De-identified BY CONSTRUCTION: it carries
/// only the hashed `fault.fingerprint`, the vocabulary `symptoms` (never the
/// request text), the hashed `config_class`, the route/consent/escalation enums,
/// and per candidate the `plan_id`, `source`, `max_risk`, and the **action
/// vocabulary** (tool names). It deliberately OMITS a candidate's free-text
/// `title`/`rationale` and a step's `description`, which can carry raw request
/// prose (hostname/user/IP/serial) — the app maps the action vocabulary to its
/// own human-readable labels. Returns the JSON value; the caller emits the
/// single stdout line so the function stays unit-testable.
/// Pinned `cec-diagnose/v1` wire values for the envelope's enum fields.
///
/// The envelope is a versioned contract (additive-only within a major), so its
/// enum fields must not ride on `Debug` formatting — a Rust-side variant rename
/// would silently change the wire with no schema bump. Each mapping below IS
/// the v1 grammar (mechanical snake_case of the variant name); the exhaustive
/// matches force every new variant to receive an explicit wire value, and the
/// pinning test freezes each value so a change fails loudly and demands a
/// version decision.
fn wire_source(source: &CandidateSource) -> &'static str {
    match source {
        CandidateSource::ColdModel => "cold_model",
        CandidateSource::CorpusPrimed => "corpus_primed",
        CandidateSource::Human => "human",
    }
}

fn wire_risk(risk: Risk) -> &'static str {
    match risk {
        Risk::ReadOnly => "read_only",
        Risk::Reversible => "reversible",
        Risk::Destructive => "destructive",
    }
}

fn wire_route(route: &Route) -> &'static str {
    match route {
        Route::SoftwareState => "software_state",
        Route::HardwareEvidenced { .. } => "hardware_evidenced",
        Route::Ambiguous => "ambiguous",
    }
}

fn wire_consent(consent: &Consent) -> &'static str {
    match consent {
        Consent::ReadOnlyOnly => "read_only_only",
        Consent::AllowReversible => "allow_reversible",
        Consent::AllowDestructive => "allow_destructive",
    }
}

fn wire_escalation(escalation: &Escalation) -> &'static str {
    match escalation {
        Escalation::Auto => "auto",
        Escalation::VerifierConfirm => "verifier_confirm",
        Escalation::HumanConfirm => "human_confirm",
    }
}

#[allow(clippy::too_many_arguments)]
fn diagnose_envelope(
    signature: &FaultSignature,
    config_class: &ConfigClass,
    route: &Route,
    candidates: &[Candidate],
    selected: usize,
    consent: &Consent,
    escalation: &Escalation,
) -> serde_json::Value {
    let cands: Vec<_> = candidates
        .iter()
        .map(|c| {
            serde_json::json!({
                "plan_id": c.plan.id,
                "source": wire_source(&c.source),
                "max_risk": wire_risk(c.plan.risk()),
                "actions": c.plan.steps.iter().map(|s| s.action.clone()).collect::<Vec<_>>(),
            })
        })
        .collect();
    let mut envelope = serde_json::json!({
        "schema_version": "cec-diagnose/v1",
        "fault": {
            "fingerprint": signature.fingerprint,
            "symptoms": signature.symptoms.iter().map(|s| s.0.clone()).collect::<Vec<_>>(),
        },
        "config_class": config_class.key(),
        "route": wire_route(route),
        "candidates": cands,
        "selected": selected,
        "consent_required": wire_consent(consent),
        "escalation": wire_escalation(escalation),
        "executed": false,
    });
    // The implicated part class rides as a sibling field only on a
    // hardware-evidenced route (additive-optional under the v1 policy). It is
    // a fixed taxonomy token ("psu", "storage", …), never request prose — and
    // it was already on the wire embedded in the old Debug-formatted route.
    if let Route::HardwareEvidenced { part_class } = route {
        envelope["part_class"] = serde_json::json!(part_class);
    }
    envelope
}

fn host_config_class(args: &Args) -> anyhow::Result<ConfigClass> {
    // An external inventory source (e.g. a device-inventory tool driving the engine
    // over a process boundary) supplies identity-free keys via --inventory-keys;
    // otherwise the coarse os/arch/family default keeps cold-start behavior intact.
    if let Some(src) = &args.inventory_keys {
        let external = ExternalInventory::new(read_inventory_keys(src)?);
        if !external.is_empty() {
            return Ok(external.config_class());
        }
    }
    Ok(CoarseHostInventory.config_class())
}

/// Read identity-free inventory keys, one per line, from a file path or stdin
/// (`-`). The supplying tool is responsible for keys being de-identified; the
/// engine re-derives the class and never stores the keys (a de-id regression test
/// guards this path).
fn read_inventory_keys(src: &str) -> anyhow::Result<Vec<String>> {
    let text = if src == "-" {
        use std::io::Read as _;
        let mut buf = String::new();
        std::io::stdin().read_to_string(&mut buf)?;
        buf
    } else {
        std::fs::read_to_string(src)
            .map_err(|e| anyhow::anyhow!("reading --inventory-keys {src}: {e}"))?
    };
    Ok(text.lines().map(str::to_string).collect())
}

// The coarse os/arch/family inventory default lives in `common::CoarseHostInventory`
// (the `InventoryProvider` seam). A real CEC build keys on the BOM revision; a richer
// host derives the class from real hardware/driver inventory, supplied identity-free
// via `--inventory-keys` (e.g. by a device-inventory tool over a process boundary) —
// see `docs/integration-myown-family.md`. The config class is bound into a row's
// sign-off attestation, so whatever it is derived from is tamper-evident.

/// Read the sign-off authority PUBLIC key from `CEC_SIGNOFF_PUBKEY` (hex). Absent
/// → `None` (attestation not enforced — cold start). Present but invalid → a hard
/// error: a typo must never silently drop the engine into an unprotected mode.
fn parse_env_pubkey() -> anyhow::Result<Option<corpus_client::SignOffPublicKey>> {
    match std::env::var("CEC_SIGNOFF_PUBKEY") {
        Err(_) => Ok(None),
        Ok(hex) => corpus_client::SignOffPublicKey::from_hex(hex.trim())
            .map(Some)
            .ok_or_else(|| {
                anyhow::anyhow!("CEC_SIGNOFF_PUBKEY is not a valid ed25519 public key (hex)")
            }),
    }
}

/// Read the sign-off authority SECRET seed from `CEC_SIGNOFF_SEED` (hex), for a
/// single operator who is themselves the authority. Absent → `None`. Present but
/// invalid → a hard error.
fn parse_env_authority() -> anyhow::Result<Option<corpus_client::SignOffAuthority>> {
    match std::env::var("CEC_SIGNOFF_SEED") {
        Err(_) => Ok(None),
        Ok(hex) => corpus_client::SignOffAuthority::from_seed_hex(hex.trim())
            .map(Some)
            .ok_or_else(|| anyhow::anyhow!("CEC_SIGNOFF_SEED is not a valid 32-byte seed (hex)")),
    }
}

/// A fresh, opaque id for this run, so the corpus can tell independent
/// confirmations apart from a re-submission of the same run (EI-03/A5). It
/// carries no identity — it is random bytes.
fn run_id() -> String {
    let mut bytes = [0u8; 16];
    getrandom::getrandom(&mut bytes).expect("OS entropy source");
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

/// Whether this run is a routine ticket that may sample a lighter model tier:
/// the route is plain software-state and the intake interview established
/// every case field. Anything vague (unestablished fields), novel (ambiguous
/// route), or physical (hardware-evidenced) stays on the heavyweight model.
fn is_simple_request(route: &Route, case: &intake::Case) -> bool {
    matches!(route, Route::SoftwareState) && case.is_established()
}

/// The verification class for a run: the route decides hardware; otherwise
/// the case's reproducibility decides. Only a fault the user reports as
/// reproducing *every time* is verified deterministically — anything else
/// (intermittent, observed once, or never established) gets the conservative
/// class, where a clean re-collection paroles the ticket rather than
/// confirming the fix.
fn verification_class_for(route: &Route, reproducibility: Reproducibility) -> VerificationClass {
    match route {
        Route::HardwareEvidenced { .. } => VerificationClass::Hardware,
        _ => match reproducibility {
            Reproducibility::Always => VerificationClass::Deterministic,
            _ => VerificationClass::Intermittent,
        },
    }
}

/// Map the routing verdict, the execution record, and the verification
/// verdict to the outcome label sign-off emits.
fn label_for(route: &Route, execution: &ExecutionResult, verdict: &Verdict) -> OutcomeLabel {
    // A hardware-evidenced case is labeled as such regardless of how any
    // consented mitigation ran: the deliverable is the diagnosis and the
    // parts action, and machine-side verification is moot.
    if let Route::HardwareEvidenced { part_class } = route {
        return OutcomeLabel::EscalatedHardware {
            part_class: part_class.clone(),
        };
    }
    if !execution.completed {
        return OutcomeLabel::EscalatedHumanUnresolved;
    }
    match verdict {
        Verdict::Pass => OutcomeLabel::ResolvedConfirmed,
        Verdict::ProvisionalPass => OutcomeLabel::ResolvedProvisional,
        // Fail (the fix did not hold), OffMachine (hardware belongs to the
        // bench), and Unverified (no live re-collection observed the outcome)
        // all escalate to a human rather than claim a resolution.
        Verdict::Fail { .. } | Verdict::OffMachine | Verdict::Unverified => {
            OutcomeLabel::EscalatedHumanUnresolved
        }
    }
}

/// A distinct causal hypothesis used to seed one swarm generator. Parallel
/// generation across different hypotheses prevents the anchoring that
/// sequential attempts produce.
struct Hypothesis {
    /// Stable slug used in plan ids and failure messages.
    slug: &'static str,
    /// The causal seed handed to the model.
    seed: &'static str,
}

const HYPOTHESES: &[Hypothesis] = &[
    Hypothesis {
        slug: "driver-regression",
        seed: "a recently updated or faulty device driver caused the fault",
    },
    Hypothesis {
        slug: "state-corruption",
        seed: "OS component-store or system-file corruption caused the fault",
    },
    Hypothesis {
        slug: "config-interaction",
        seed: "a configuration interaction (power plan, startup items, services) \
               caused the fault",
    },
];

/// The model-free generator: always available, so the slate is never empty
/// and the pipeline runs at cold start with no endpoint.
struct HeuristicGenerator;

#[async_trait]
impl Generator for HeuristicGenerator {
    async fn generate(&self, events: &[DiagnosticEvent]) -> Result<Vec<Candidate>, SwarmError> {
        let describe = events
            .first()
            .map(|event| event.message.clone())
            .unwrap_or_default();
        Ok(vec![heuristic_candidate(&describe)])
    }
}

/// One hypothesis-seeded model generator. Each instance proposes from its own
/// causal seed against the configured OpenAI-compatible endpoint, primed with
/// the intake interview's structured findings.
struct ModelGenerator {
    base_url: String,
    model: String,
    hypothesis: &'static Hypothesis,
    case_brief: String,
}

#[async_trait]
impl Generator for ModelGenerator {
    async fn generate(&self, events: &[DiagnosticEvent]) -> Result<Vec<Candidate>, SwarmError> {
        let describe = events
            .iter()
            .map(|event| event.message.as_str())
            .collect::<Vec<_>>()
            .join("\n");
        let system = format!(
            "You are a Windows support diagnostician. Working hypothesis: {}. \
             Intake interview findings: {}. Propose a single safe, reversible \
             first step that tests or addresses this hypothesis.",
            self.hypothesis.seed, self.case_brief
        );
        let client = OpenAiClient::new(Endpoint::new(&self.base_url));
        let request = ChatCompletionRequest::new(
            self.model.clone(),
            vec![ChatMessage::system(system), ChatMessage::user(describe)],
        );
        let response = client
            .complete(request)
            .await
            .map_err(|error| SwarmError::Rejected(format!("{}: {error}", self.hypothesis.slug)))?;
        let content = response
            .choices
            .into_iter()
            .next()
            .map(|choice| choice.message.content)
            .filter(|content| !content.trim().is_empty())
            .ok_or_else(|| {
                SwarmError::Rejected(format!("{}: empty completion", self.hypothesis.slug))
            })?;

        let mut plan = Plan::new(
            format!("model-{}", self.hypothesis.slug),
            format!("Model-proposed first step ({})", self.hypothesis.slug),
        );
        plan.steps.push(PlanStep {
            description: content,
            action: "review".to_string(),
            risk: Risk::ReadOnly,
        });
        Ok(vec![Candidate::new(
            plan,
            format!("Hypothesis: {}", self.hypothesis.seed),
            CandidateSource::ColdModel,
        )])
    }
}

fn heuristic_candidate(describe: &str) -> Candidate {
    let mut plan = Plan::new(
        "heuristic-1",
        "Collect diagnostics and capture a restore point",
    );
    plan.steps.push(PlanStep {
        description: "Gather logs, WER, WHEA, and CIM state".to_string(),
        action: "cim_query".to_string(),
        risk: Risk::ReadOnly,
    });
    plan.steps.push(PlanStep {
        description: "Create a system restore point before any change".to_string(),
        action: "create_restore_point".to_string(),
        risk: Risk::Reversible,
    });
    Candidate::new(
        plan,
        format!("Safe, reversible first response to: {describe}"),
        CandidateSource::ColdModel,
    )
}

fn print_help() {
    println!(
        "cec-support-agent {version}

USAGE:
    cec-support-agent diagnose [OPTIONS]
    cec-support-agent gen-signoff-key

COMMANDS:
    diagnose             Run the diagnostic pipeline (the default).
    gen-signoff-key      Generate a sign-off authority ed25519 key pair and print
                         the CEC_SIGNOFF_PUBKEY / CEC_SIGNOFF_SEED exports.

OPTIONS:
    --describe <TEXT>    Describe the user's problem
    --endpoint <URL>     OpenAI-compatible base URL (e.g. http://localhost:8080/v1)
    --model <NAME>       Model name to request (default: local-model)
    --fast-model <NAME>  Lighter model for simple requests: intake question
                         phrasing always, and plan generation when the case is
                         routine (software-state route, every intake field
                         established). Defaults to --model.
    --fast-endpoint <URL> Endpoint for the fast model (defaults to --endpoint)
    --corpus <PATH>      File-backed corpus (one JSON row per line). Outcomes
                         persist across runs, and a known signature is served
                         retrieval-first from precedent. Default: in-memory.
    --offline            Skip the model calls; use only the model-free heuristic
    --no-questions       Skip the intake follow-up questions (asked only when a
                         terminal is attached; headless runs never prompt)
    --sign-off <LEVEL>   Execute the winning plan, verify the outcome, and record
                         the labeled result, where LEVEL is 'verifier' (authorizes
                         reversible steps) or 'human' (authorizes destructive
                         steps). The judge may require 'human'. Omit it to stop
                         before execution (the sign-off-gated default).
    -h, --help           Print help
    -V, --version        Print version

ENVIRONMENT:
    CEC_SIGNOFF_PUBKEY   Sign-off authority public key (hex). When set, the corpus
                         ENFORCES that every confirmed row carries a valid ed25519
                         attestation — a self-asserted sign-off is refused, and a
                         file-backed corpus whose on-disk rows are unattested (or
                         forged) is refused at open, not served.
    CEC_SIGNOFF_SEED     Sign-off authority secret seed (hex). When set, this run
                         holds the authority and attests its own outcomes
                         (single-operator mode). Set without CEC_SIGNOFF_PUBKEY,
                         the enforcing pubkey is DERIVED from the seed so the run
                         both self-attests and enforces — it never attests into an
                         unprotected store. In a split deployment, set only the
                         pubkey on the engine and produce attestations elsewhere.
                         A set-but-invalid value is a hard error.

Cold start: with no --endpoint (or with --offline), the agent runs the full
diagnostic -> route -> candidate -> judge pipeline using the model-free
heuristic and an empty in-memory corpus. No CEC-hosted service is required.",
        version = env!("CARGO_PKG_VERSION")
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- Leak-prevention (Phase 0): the action-vocabulary drift guard.

    #[test]
    fn every_registered_tool_is_in_the_frozen_action_vocabulary() {
        // The de-id mint admits only `deid::ACTION_VOCABULARY` into a corpus row.
        // If a new dispatcher tool is registered without being added to that frozen
        // list, its de-identified plans would be REFUSED at write time. This drift
        // guard keeps the live registry and the frozen vocabulary in lockstep, so
        // the two cannot silently diverge.
        let mut dispatcher = Dispatcher::new();
        for tool in windows_tools() {
            dispatcher.register(tool);
        }
        for name in dispatcher.tool_names() {
            assert!(
                deid::ACTION_VOCABULARY.binary_search(&name).is_ok(),
                "registered tool {name:?} is not in deid::ACTION_VOCABULARY — add it (keep the list sorted)"
            );
        }
    }

    #[test]
    fn diagnose_envelope_rejects_every_leakguard_poison_token() {
        // The envelope de-id guard, driven by the canonical poison set (single
        // source of truth) instead of a local token list — so it widens as the
        // poison set does.
        let mut cand = heuristic_candidate("explorer.exe crashes on login 0x1234");
        for tok in leakguard::POISON {
            cand.rationale = format!("addresses {tok}");
            cand.plan.title = format!("fix {tok}");
            for step in &mut cand.plan.steps {
                step.description = format!("run {tok}");
            }
            let sig = FaultSignature::from_symptoms(extract_symptoms("explorer.exe 0x1234"));
            let env = diagnose_envelope(
                &sig,
                &CoarseHostInventory.config_class(),
                &Route::SoftwareState,
                std::slice::from_ref(&cand),
                0,
                &Consent::AllowReversible,
                &Escalation::HumanConfirm,
            );
            leakguard::assert_no_poison(
                &serde_json::to_string(&env).unwrap(),
                "cec-diagnose/v1 envelope",
            );
        }
    }

    // --- Inventory seam (P0): external keys are de-identified into the config class.

    #[test]
    fn external_inventory_keys_are_de_identified_into_the_config_class() {
        // The engine re-derives the config class as a one-way hash of the keys, so
        // an identity-bearing key is CONSUMED into the hash (it scopes retrieval)
        // but never stored or echoed verbatim. These assertions bite — the earlier
        // version checked a 16-hex hash for identity substrings, which is true by
        // construction and could never catch a regression.
        let keys = vec![
            "os:windows 11".to_string(),
            "host:DESKTOP-NATHAN01".to_string(),
        ];
        let class = ExternalInventory::new(keys.clone()).config_class();
        // (a) it is the derived hash: 16 lowercase-hex chars, == from_inventory(keys).
        let key = class.key();
        assert_eq!(key.len(), 16, "expected a 16-hex DerivedHash, got {key:?}");
        assert!(
            key.chars().all(|c| c.is_ascii_hexdigit()) && key == key.to_lowercase(),
            "config class is not lowercase hex: {key:?}"
        );
        assert_eq!(class, ConfigClass::from_inventory(keys.clone()));
        // (b) the identity key is CONSUMED, not silently dropped: changing only the
        // hostname changes the class (so a vacuous "ignored it" impl would fail here).
        let other = ExternalInventory::new(vec![
            "os:windows 11".to_string(),
            "host:DESKTOP-OTHER99".to_string(),
        ])
        .config_class();
        assert_ne!(class, other, "an identity key must affect the class");
        // ...but the SAME facts in any order collide (order-independent hash).
        let reordered = ExternalInventory::new(vec![
            "host:DESKTOP-NATHAN01".to_string(),
            "os:windows 11".to_string(),
        ])
        .config_class();
        assert_eq!(class, reordered);
    }

    #[test]
    fn diagnose_envelope_emits_no_candidate_free_text_so_request_prose_cannot_leak() {
        // The envelope is a SEPARATE serialization path from the corpus de-id. A
        // candidate's free-text rationale/title and a step description can carry the
        // raw request prose (hostname/user/IP/serial); the envelope must ship only
        // the de-identified action vocabulary. Plant identity in every free-text
        // field and assert none survives serialization (the D1 regression guard).
        let plant = "DESKTOP-NATHAN01 nathan 192.168.1.20 SN12345678";
        let mut cand = heuristic_candidate("explorer.exe crashes on login 0x1234");
        cand.rationale = format!("addresses {plant}");
        cand.plan.title = format!("fix for {plant}");
        for step in &mut cand.plan.steps {
            step.description = format!("run against {plant}");
        }
        let sig =
            FaultSignature::from_symptoms(extract_symptoms("explorer.exe crashes on login 0x1234"));
        let cfg = CoarseHostInventory.config_class();
        let env = diagnose_envelope(
            &sig,
            &cfg,
            &Route::SoftwareState,
            std::slice::from_ref(&cand),
            0,
            &Consent::AllowReversible,
            &Escalation::HumanConfirm,
        );
        let blob = serde_json::to_string(&env).unwrap().to_lowercase();
        for tok in ["desktop-nathan01", "nathan", "192.168.1.20", "sn12345678"] {
            assert!(
                !blob.contains(tok),
                "identity {tok:?} leaked into the envelope: {blob}"
            );
        }
        // The de-identified action vocabulary IS present (the plan shape the app renders).
        let actions = env["candidates"][0]["actions"]
            .as_array()
            .expect("actions array");
        assert!(!actions.is_empty());
    }

    #[test]
    fn diagnose_envelope_has_the_cec_diagnose_v1_shape() {
        let cand = heuristic_candidate("explorer.exe crashes on login 0x1234");
        let sig =
            FaultSignature::from_symptoms(extract_symptoms("explorer.exe crashes on login 0x1234"));
        let cfg = CoarseHostInventory.config_class();
        let env = diagnose_envelope(
            &sig,
            &cfg,
            &Route::SoftwareState,
            std::slice::from_ref(&cand),
            0,
            &Consent::AllowReversible,
            &Escalation::HumanConfirm,
        );
        assert_eq!(env["schema_version"], "cec-diagnose/v1");
        assert_eq!(env["executed"], false);
        assert_eq!(env["config_class"], cfg.key());
        let syms: Vec<String> = sig.symptoms.iter().map(|s| s.0.clone()).collect();
        assert_eq!(env["fault"]["symptoms"], serde_json::json!(syms));
        // selected index is in range
        let selected = env["selected"].as_u64().unwrap() as usize;
        let cands = env["candidates"].as_array().unwrap();
        assert!(selected < cands.len());
        // each candidate carries exactly the de-identified fields — no title/rationale
        let c0 = &cands[0];
        for f in ["plan_id", "source", "max_risk", "actions"] {
            assert!(c0.get(f).is_some(), "envelope candidate missing {f}");
        }
        assert!(
            c0.get("title").is_none() && c0.get("rationale").is_none(),
            "envelope candidate must not carry free-text fields"
        );
    }

    #[test]
    fn envelope_enum_wire_values_are_pinned_for_cec_diagnose_v1() {
        // v1 freezes these exact strings. A Rust-side variant rename must fail
        // HERE, not silently change the wire: within a major the values are
        // immutable, and changing one demands a schema-major decision.
        assert_eq!(wire_source(&CandidateSource::ColdModel), "cold_model");
        assert_eq!(wire_source(&CandidateSource::CorpusPrimed), "corpus_primed");
        assert_eq!(wire_source(&CandidateSource::Human), "human");
        assert_eq!(wire_risk(Risk::ReadOnly), "read_only");
        assert_eq!(wire_risk(Risk::Reversible), "reversible");
        assert_eq!(wire_risk(Risk::Destructive), "destructive");
        assert_eq!(wire_route(&Route::SoftwareState), "software_state");
        assert_eq!(
            wire_route(&Route::HardwareEvidenced {
                part_class: "psu".into()
            }),
            "hardware_evidenced"
        );
        assert_eq!(wire_route(&Route::Ambiguous), "ambiguous");
        assert_eq!(wire_consent(&Consent::ReadOnlyOnly), "read_only_only");
        assert_eq!(wire_consent(&Consent::AllowReversible), "allow_reversible");
        assert_eq!(
            wire_consent(&Consent::AllowDestructive),
            "allow_destructive"
        );
        assert_eq!(wire_escalation(&Escalation::Auto), "auto");
        assert_eq!(
            wire_escalation(&Escalation::VerifierConfirm),
            "verifier_confirm"
        );
        assert_eq!(wire_escalation(&Escalation::HumanConfirm), "human_confirm");

        // The envelope carries the pinned tokens, and a hardware-evidenced
        // route hoists its part class into the additive sibling field.
        let cand = heuristic_candidate("explorer.exe crashes on login 0x1234");
        let sig =
            FaultSignature::from_symptoms(extract_symptoms("explorer.exe crashes on login 0x1234"));
        let env = diagnose_envelope(
            &sig,
            &CoarseHostInventory.config_class(),
            &Route::HardwareEvidenced {
                part_class: "psu".into(),
            },
            std::slice::from_ref(&cand),
            0,
            &Consent::AllowReversible,
            &Escalation::HumanConfirm,
        );
        assert_eq!(env["route"], "hardware_evidenced");
        assert_eq!(env["part_class"], "psu");
        assert_eq!(env["consent_required"], "allow_reversible");
        assert_eq!(env["escalation"], "human_confirm");
        assert_eq!(env["candidates"][0]["source"], "cold_model");
        let software = diagnose_envelope(
            &sig,
            &CoarseHostInventory.config_class(),
            &Route::SoftwareState,
            std::slice::from_ref(&cand),
            0,
            &Consent::AllowReversible,
            &Escalation::Auto,
        );
        assert!(
            software.get("part_class").is_none(),
            "part_class rides only on a hardware-evidenced route"
        );
    }

    #[test]
    fn read_inventory_keys_then_external_trims_and_drops_blanks() {
        let path = std::env::temp_dir().join(format!("cec-invkeys-{}.txt", std::process::id()));
        std::fs::write(&path, "os:windows 11\n\n  gpu:rtx-4070  \n").expect("write");
        let raw = read_inventory_keys(path.to_str().unwrap()).expect("read");
        let _ = std::fs::remove_file(&path);
        assert_eq!(
            ExternalInventory::new(raw).inventory_keys(),
            vec!["os:windows 11", "gpu:rtx-4070"]
        );
    }

    struct FakeSandbox {
        clean: bool,
    }

    #[async_trait]
    impl SandboxValidator for FakeSandbox {
        async fn validate(
            &self,
            candidate: &Candidate,
        ) -> Result<swarm::ValidationReport, SwarmError> {
            Ok(swarm::ValidationReport {
                candidate_id: candidate.plan.id.clone(),
                applied_cleanly: self.clean,
                notes: "fake sandbox".into(),
            })
        }
    }

    #[tokio::test]
    async fn sandbox_validation_drives_the_validated_flag() {
        let best = heuristic_candidate("x");
        assert!(
            !sandbox_validated_for(None, &best, false).await,
            "no validator -> unvalidated (escalate)"
        );
        assert!(
            sandbox_validated_for(Some(&FakeSandbox { clean: true }), &best, false).await,
            "a clean sandbox apply is positive validation evidence"
        );
        assert!(
            !sandbox_validated_for(Some(&FakeSandbox { clean: false }), &best, false).await,
            "a dirty apply stays unvalidated"
        );
    }

    #[test]
    fn heuristic_plan_is_reversible_and_needs_consent() {
        let candidate = heuristic_candidate("disk is full");
        assert_eq!(candidate.plan.risk(), Risk::Reversible);
        assert!(candidate.plan.requires_consent());
    }

    #[test]
    fn pipeline_selects_a_candidate_with_no_endpoint() {
        let judge = HeuristicJudge;
        let candidates = vec![heuristic_candidate("anything")];
        assert!(best_of_n(&judge, &candidates).is_some());
    }

    #[test]
    fn executability_is_the_operation_vocabulary_check() {
        let mut dispatcher = Dispatcher::new();
        for tool in windows_tools() {
            dispatcher.register(tool);
        }
        // The heuristic plan uses only registered tools.
        assert!(is_executable(&heuristic_candidate("x").plan, &dispatcher));
        // A model plan's free-text "review" step is advisory-only.
        let mut advisory = Plan::new("model-1", "advice");
        advisory.steps.push(PlanStep {
            description: "think about it".into(),
            action: "review".into(),
            risk: Risk::ReadOnly,
        });
        assert!(!is_executable(&advisory, &dispatcher));
        // An empty plan has nothing to execute.
        assert!(!is_executable(&Plan::new("p", "t"), &dispatcher));
    }

    #[test]
    fn consent_rendering_is_plain_language_with_the_coverage_boundary() {
        let rendered = render_consent(&heuristic_candidate("disk is full").plan);
        // Each step says in words whether it can be undone.
        assert!(rendered.contains("only looks at information"));
        assert!(rendered.contains("can be undone"));
        // The restore point is explained, not just named.
        assert!(rendered.contains("snapshot"), "{rendered}");
        // The coverage boundary is stated in user terms.
        assert!(rendered.contains("does NOT include your personal files"));
        assert!(rendered.contains("can NOT undo"), "{rendered}");
        assert!(rendered.contains("BIOS (the main board's built-in software)"));
        // No raw enum jargon reaches the user's screen.
        assert!(
            !rendered.contains("ReadOnly") && !rendered.contains("[Reversible]"),
            "raw risk enums leaked into consent: {rendered}"
        );
    }

    #[test]
    fn every_outcome_label_has_a_plain_language_explanation() {
        let labels = [
            OutcomeLabel::ResolvedConfirmed,
            OutcomeLabel::ResolvedProvisional,
            OutcomeLabel::Reopened,
            OutcomeLabel::EscalatedHardware {
                part_class: "psu".into(),
            },
            OutcomeLabel::EscalatedHumanUnresolved,
            OutcomeLabel::Withdrawn,
        ];
        for label in &labels {
            let text = explain_label(label);
            assert!(text.len() > 40, "too terse for {label:?}: {text}");
            // What happens next must be part of every explanation.
            assert!(
                text.contains("case")
                    || text.contains("watched")
                    || text.contains("person")
                    || text.contains("no longer present"),
                "no next-step in {label:?}: {text}"
            );
        }
    }

    #[tokio::test]
    async fn the_flywheel_turns_a_resolved_outcome_into_a_precedent() {
        // Run 1 records a resolved outcome...
        let corpus = LocalCorpus::new();
        let signature =
            FaultSignature::from_symptoms(extract_symptoms("explorer.exe crashes on login 0x1234"));
        let config_class = CoarseHostInventory.config_class();
        record_outcome(
            &corpus,
            &signature,
            &heuristic_candidate("x").plan,
            OutcomeLabel::ResolvedConfirmed,
            &config_class,
            SignOff::HumanConfirmed,
            Some(common::Verification::pass()),
            None,
            None,
            false,
        )
        .await;
        // ...and run 2 facing the same signature retrieves it as precedent.
        let known = corpus
            .query(&signature, &config_class)
            .await
            .expect("query");
        assert_eq!(known.len(), 1);
        let primed = Candidate::new(
            known[0].plan.clone(),
            "Corpus precedent",
            CandidateSource::CorpusPrimed,
        );
        // The judge prefers the precedent over a cold candidate of the same
        // shape, so retrieval-first actually changes the selection.
        let judge = HeuristicJudge;
        let candidates = vec![heuristic_candidate("x"), primed];
        let (index, best, _) = best_of_n(&judge, &candidates).expect("non-empty");
        assert_eq!(index, 1);
        assert_eq!(best.source, CandidateSource::CorpusPrimed);
    }

    #[test]
    fn only_established_software_state_cases_are_simple_requests() {
        // Fully established + software-state: may sample the fast tier.
        let routine = Interview::new(
            "explorer.exe crashes every time I log in, started yesterday \
             right after a driver update; just one program",
        );
        let case = routine.case();
        assert_eq!(route_for(&case.signature()), Route::SoftwareState);
        assert!(is_simple_request(&Route::SoftwareState, case));

        // A vague case stays on the heavyweight model even if software-state.
        let vague = Interview::new("explorer.exe crashes");
        assert!(!is_simple_request(&Route::SoftwareState, vague.case()));

        // Hardware-evidenced and ambiguous routes always stay heavyweight.
        let hardware = Route::HardwareEvidenced {
            part_class: "psu".into(),
        };
        assert!(!is_simple_request(&hardware, case));
        assert!(!is_simple_request(&Route::Ambiguous, case));
    }

    #[test]
    fn the_intake_case_drives_the_verification_class() {
        // "Every time" earns deterministic verification...
        assert_eq!(
            verification_class_for(&Route::SoftwareState, Reproducibility::Always),
            VerificationClass::Deterministic
        );
        // ...anything weaker gets the conservative class (parole, not proof).
        for repro in [
            Reproducibility::Unknown,
            Reproducibility::Intermittent,
            Reproducibility::Once,
        ] {
            assert_eq!(
                verification_class_for(&Route::SoftwareState, repro),
                VerificationClass::Intermittent
            );
        }
        // The route outranks reproducibility: hardware verifies off-machine.
        let hardware = Route::HardwareEvidenced {
            part_class: "psu".into(),
        };
        assert_eq!(
            verification_class_for(&hardware, Reproducibility::Always),
            VerificationClass::Hardware
        );
    }

    #[test]
    fn an_interview_answer_changes_the_route() {
        // The vague statement is ambiguous; the exact-error answer carries
        // the WHEA evidence that routes it to hardware.
        let mut interview = Interview::new("my pc just dies");
        assert_eq!(route_for(&interview.case().signature()), Route::Ambiguous);
        interview.answer(
            intake::QuestionKind::ExactError,
            "WHEA Logger error 0x00000124",
        );
        assert_eq!(
            route_for(&interview.case().signature()),
            Route::HardwareEvidenced {
                part_class: "platform".into()
            }
        );
    }

    #[test]
    fn signature_is_structured_and_drops_identity() {
        let events = collect_diagnostics(
            "DESKTOP-NATHAN01: explorer.exe crashes on login, WER bucket 0x1234",
        );
        let signature = signature_of(&events);
        let symptoms: Vec<&str> = signature.symptoms.iter().map(|s| s.0.as_str()).collect();
        assert!(symptoms.contains(&"explorer.exe"));
        assert!(symptoms.contains(&"0x1234"));
        assert!(!symptoms.iter().any(|s| s.contains("nathan")));
    }

    #[tokio::test]
    async fn the_heuristic_generator_keeps_the_slate_non_empty() {
        let swarm = Swarm::new();
        let generators: Vec<Box<dyn Generator>> = vec![Box::new(HeuristicGenerator)];
        let gathered = swarm
            .gather(&generators, &collect_diagnostics("anything"))
            .await
            .expect("gather");
        assert_eq!(gathered.candidates.len(), 1);
        assert!(gathered.failures.is_empty());
    }

    #[test]
    fn hardware_route_labels_escalated_hardware_with_the_part_class() {
        let route = Route::HardwareEvidenced {
            part_class: "psu".into(),
        };
        let execution = ExecutionResult::new("p");
        let label = label_for(&route, &execution, &Verdict::OffMachine);
        assert_eq!(
            label,
            OutcomeLabel::EscalatedHardware {
                part_class: "psu".into()
            }
        );
    }

    #[test]
    fn halted_execution_labels_unresolved() {
        let execution = ExecutionResult::new("p"); // completed = false
        let label = label_for(&Route::SoftwareState, &execution, &Verdict::Pass);
        assert_eq!(label, OutcomeLabel::EscalatedHumanUnresolved);
    }

    #[test]
    fn verified_outcomes_label_resolved() {
        let mut execution = ExecutionResult::new("p");
        execution.completed = true;
        assert_eq!(
            label_for(&Route::SoftwareState, &execution, &Verdict::Pass),
            OutcomeLabel::ResolvedConfirmed
        );
        assert_eq!(
            label_for(&Route::SoftwareState, &execution, &Verdict::ProvisionalPass),
            OutcomeLabel::ResolvedProvisional
        );
        assert_eq!(
            label_for(
                &Route::SoftwareState,
                &execution,
                &Verdict::Fail { recurring: vec![] }
            ),
            OutcomeLabel::EscalatedHumanUnresolved
        );
    }

    #[tokio::test]
    async fn signed_off_execution_runs_through_the_dispatcher() {
        use agent_core::{execute_plan, Consent, Dispatcher};

        let mut dispatcher = Dispatcher::new();
        for tool in windows_tools() {
            dispatcher.register(tool);
        }
        let candidate = heuristic_candidate("disk is full");
        let result = execute_plan(&dispatcher, &candidate.plan, Consent::AllowReversible).await;

        // The executor returns a per-step record for the winning plan. On
        // non-Windows hosts the Windows tools report "unsupported", so execution
        // halts at the first step; on Windows it would proceed. Either way these
        // platform-independent invariants hold.
        assert_eq!(result.plan_id, candidate.plan.id);
        assert!(!result.steps.is_empty());
    }
}
