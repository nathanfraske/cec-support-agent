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
    extract_symptoms, Candidate, CandidateSource, ConfigClass, DiagnosticEvent, EventKind,
    ExecutionResult, FaultSignature, Plan, PlanStep, Risk, Severity,
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
use swarm::{Generator, Swarm, SwarmError};
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
            // The single verb; accepted for readability of the command line.
            "diagnose" => {}
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
    }))
}

async fn run(args: Args) -> anyhow::Result<()> {
    println!("cec-support-agent: diagnose");
    println!("  request: {}", args.describe);

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
            println!("  ? {prompt}");
            print!("  > ");
            std::io::stdout().flush()?;
            let mut line = String::new();
            if std::io::stdin().read_line(&mut line)? == 0 {
                break; // EOF: proceed with what we have.
            }
            interview.answer(question.kind, line.trim());
        }
    }
    let case = interview.into_case();
    println!("  case: {}", case.brief());
    // The register check: how reasoned the person's explanation was decides
    // how measured the response is. It calibrates teaching (definitions,
    // examples, walkthroughs), never safety or what gets checked.
    println!(
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
    println!(
        "  fault signature: {} ({} structured symptom(s))",
        signature.fingerprint,
        signature.symptoms.len()
    );

    // 2. The config class scopes every corpus row and query to like configs:
    //    the BOM revision on a CEC build, a derived inventory hash otherwise.
    let config_class = host_config_class();
    println!("  config class: {}", config_class.key());

    // 3. Routing precedes scoring: the routing verdict determines which gates
    //    are load-bearing. A hardware-evidenced case's deliverable is a
    //    diagnosis plus a parts action; an ambiguous case escalates.
    let route = route_for(&signature);
    println!("  route: {route:?}");
    if case.fluency == common::Fluency::Guided {
        println!("  what this means: {}", route.explanation());
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
                    println!(
                        "  board: {} {} — BIOS {} ({})",
                        board.manufacturer, board.product, board.bios_version, board.bios_date
                    );
                    let advisory = firmware_advisory(&board, case.fluency);
                    println!("  firmware advisory ({}):", advisory.vendor);
                    for step in &advisory.steps {
                        println!("    {step}");
                    }
                }
                None => println!("  board: identity payload unrecognized"),
            },
            Ok(outcome) => println!("  board: unavailable ({})", outcome.summary),
            Err(error) => println!("  board: unavailable ({error})"),
        }
    }

    // 4. The corpus: file-backed when `--corpus` names a path — the
    //    self-hosted flywheel, where the next run facing a known signature
    //    starts from this run's outcome — and in-memory otherwise. Cold start
    //    either way: no CEC service is required.
    let corpus: Box<dyn CorpusStore> = match &args.corpus {
        Some(path) => {
            let file = FileCorpus::open(path)?;
            println!("  corpus: file-backed at {path} ({} row(s))", file.len());
            Box::new(file)
        }
        None => Box::new(LocalCorpus::new()),
    };
    let known = corpus.query(&signature, &config_class).await?;
    println!(
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
        println!(
            "  retrieval-first: adapting {} precedent plan(s); skipping de novo generation",
            candidates.len()
        );
    }

    let swarm = Swarm::new();
    println!(
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
            println!("  generation model: {generation_model}");
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
            println!(
                "  risk reconciled: '{}' claimed {:?} but is {:?} — raised before consent",
                correction.action, correction.claimed, correction.actual
            );
        }
        candidate.plan = reconciled;
    }

    // 6. Sandbox validation. The CLI configures no VM backend, so every plan
    //    is unvalidated here — and unvalidated equals escalate: the judge
    //    requires human sign-off for any state-changing unvalidated plan.
    let sandbox_validated = false;
    println!("  sandbox: no validator configured; plans are unvalidated (unvalidated = escalate)");

    // 7. Judge panel: score the slate, pick best-of-N, decide the escalation
    //    from the route, the validation state, and the risk/score ladder.
    let judge = HeuristicJudge;
    let (index, best, score) =
        best_of_n(&judge, &candidates).expect("the heuristic candidate is always present");
    let escalation = required_escalation(&route, sandbox_validated, best, &score);

    let consent_needed = match best.plan.risk() {
        Risk::ReadOnly => Consent::ReadOnlyOnly,
        Risk::Reversible => Consent::AllowReversible,
        Risk::Destructive => Consent::AllowDestructive,
    };

    println!();
    println!(
        "selected candidate #{index} of {} (source: {:?})",
        candidates.len(),
        best.source
    );
    println!("  title:      {}", best.plan.title);
    println!("  rationale:  {}", best.rationale);
    println!("  risk:       {:?}", best.plan.risk());
    println!("  score:      {:.3}", score.total());
    println!("  escalation: {escalation:?}");
    println!("  steps:");
    for (i, step) in best.plan.steps.iter().enumerate() {
        println!(
            "    {}. [{:?}] {} -> {}",
            i + 1,
            step.risk,
            step.description,
            step.action
        );
    }
    println!("  tools available: {:?}", dispatcher.tool_names());
    println!("  consent needed:  {consent_needed:?}");

    println!();
    match args.sign_off {
        None => {
            println!(
                "Execution and corpus write-back are gated on {escalation:?} sign-off and are NOT \
                 performed by this run. Re-run with --sign-off <verifier|human> to execute the \
                 winning plan, verify the outcome, and record the labeled result."
            );
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
                println!(
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
            println!("sign-off: {sign_off:?} -> executing under consent {granted:?}");

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
                println!(
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
                )
                .await;
                return Ok(());
            }
            if !is_executable(&best.plan, &dispatcher) {
                println!(
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
                    println!();
                    println!("retry {attempt}: next-best plan '{}'", candidate.plan.title);
                }

                // Consent is to a rendered plan, never an opaque script:
                // plain-language steps, risk class, and the restore-point
                // coverage boundary.
                println!("{}", render_consent(&candidate.plan));
                if std::io::stdin().is_terminal() {
                    print!("  Type 'yes' to consent, anything else to decline: ");
                    std::io::stdout().flush()?;
                    let mut line = String::new();
                    std::io::stdin().read_line(&mut line)?;
                    if !line.trim().eq_ignore_ascii_case("yes") {
                        println!("  consent declined; ticket withdrawn");
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
                        )
                        .await;
                        final_label = Some(label);
                        break;
                    }
                } else {
                    println!("  (headless run: --sign-off {sign_off:?} is the recorded consent)");
                }

                let signed = signer.sign(&candidate.plan);
                println!("  plan signature: {}…", &signed.signature[..16]);
                let execution =
                    agent_core::execute_signed_plan(&dispatcher, &signed, &signer, granted)
                        .await
                        .map_err(|error| anyhow::anyhow!(error))?;
                for step in &execution.steps {
                    println!(
                        "  step {} [{}] {} -> {}",
                        step.step,
                        if step.ok { "ok" } else { "fail" },
                        step.action,
                        step.summary
                    );
                }
                println!(
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
                    println!(
                        "  verification: no live re-collection available — the outcome cannot be \
                         confirmed and will escalate for human verification (NR-1)"
                    );
                }
                let verdict = verify_outcome(&signature, post.as_ref(), class);
                println!("  verification ({class:?}): {verdict:?}");
                if let Verdict::Fail { recurring } = &verdict {
                    println!(
                        "  hard negative: {} original symptom(s) recurred; the failed plan \
                         and this diff enter the retry context",
                        recurring.len()
                    );
                }

                // Sign-off is the labeling event: every attempt emits a
                // label — a failure enters the corpus as a hard negative, not
                // a discard — because an unlabeled ticket is corpus poison.
                let label = label_for(&route, &execution, &verdict);
                println!("  outcome label: {label:?}");
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
                println!();
                println!("ticket label: {label:?}");
                if case.fluency == common::Fluency::Guided {
                    println!("  what this means: {}", explain_label(&label));
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
) {
    let mut contribution = Contribution::new(
        Outcome {
            signature: signature.clone(),
            plan: plan.clone(),
            label: label.clone(),
            verification,
        },
        config_class.clone(),
        sign_off,
    );
    if let Some(provenance) = provenance {
        contribution = contribution.with_provenance(provenance);
    }
    match corpus.submit(&contribution).await {
        Ok(()) => println!("  corpus: outcome recorded (label={label:?}, sign-off={sign_off:?})"),
        Err(error) => println!("  corpus: submit refused: {error}"),
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

/// The config class for this host. A real collector derives it from the CIM
/// hardware and driver inventory (or the BOM revision on a CEC build); the
/// bootstrap uses the host's coarse identity so corpus rows are still scoped
/// to like configs.
fn host_config_class() -> ConfigClass {
    ConfigClass::from_inventory(host_inventory())
}

/// The inventory facts that scope a corpus row to "like configs". A real CEC
/// build keys on the BOM revision; a general host derives the class from its
/// hardware and driver inventory. Today this is the cross-platform coarse set
/// (OS, arch, OS family) — deterministic and identity-free. A Windows build
/// SHOULD enrich it here with CIM **configuration** fields (board vendor/model,
/// BIOS version/date, chipset, GPU model, driver versions) — never serial
/// numbers or service tags — so retrieval is scoped to genuinely-like hardware
/// rather than to every machine sharing an OS/arch. That enrichment needs a
/// Windows host to build and verify and is tracked in FOLLOWUPS; the config
/// class is already bound into a row's sign-off attestation, so whatever it is
/// derived from is tamper-evident.
fn host_inventory() -> Vec<String> {
    vec![
        format!("os:{}", std::env::consts::OS),
        format!("arch:{}", std::env::consts::ARCH),
        format!("family:{}", std::env::consts::FAMILY),
    ]
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

Cold start: with no --endpoint (or with --offline), the agent runs the full
diagnostic -> route -> candidate -> judge pipeline using the model-free
heuristic and an empty in-memory corpus. No CEC-hosted service is required.",
        version = env!("CARGO_PKG_VERSION")
    );
}

#[cfg(test)]
mod tests {
    use super::*;

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
        let config_class = host_config_class();
        record_outcome(
            &corpus,
            &signature,
            &heuristic_candidate("x").plan,
            OutcomeLabel::ResolvedConfirmed,
            &config_class,
            SignOff::HumanConfirmed,
            Some(common::Verification::pass()),
            None,
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
