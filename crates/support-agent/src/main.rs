// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 The cec-support-agent authors
//! cec-support-agent: the headless CLI face of the engine.
//!
//! It assembles the pipeline from the diagram — collect diagnostics, generate
//! candidate plans, run the judge panel, and report the winner with its
//! required escalation — and stops before execution and corpus write-back,
//! which are gated on sign-off.
//!
//! Cold start (a bootstrap invariant): with no `--endpoint` (or with
//! `--offline`) the agent runs the whole pipeline using a model-free heuristic
//! candidate and an empty in-memory corpus. No CEC-hosted service is required.

use std::process::ExitCode;

use agent_core::{Consent, Dispatcher};
use common::{
    Candidate, CandidateSource, DiagnosticEvent, EventKind, FaultSignature, Plan, PlanStep, Risk,
    Severity, Symptom,
};
use corpus_client::{CorpusStore, LocalCorpus};
use inference::{ChatCompletionRequest, ChatMessage, Completer, Endpoint, OpenAiClient};
use panel::{best_of_n, escalation_for, HeuristicJudge};
use swarm::Swarm;
use tools_windows::windows_tools;

/// Parsed command-line arguments for the `diagnose` flow.
struct Args {
    describe: String,
    endpoint: Option<String>,
    model: String,
    offline: bool,
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
    let mut offline = false;

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
            "--endpoint" => {
                endpoint = Some(args.next().ok_or("--endpoint requires a value")?);
            }
            "--model" => {
                model = args.next().ok_or("--model requires a value")?;
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
        offline,
    }))
}

async fn run(args: Args) -> anyhow::Result<()> {
    println!("cec-support-agent: diagnose");
    println!("  request: {}", args.describe);

    // 1. Collect diagnostics (stubbed here from the request text) and derive a
    //    de-identified fault signature.
    let events = collect_diagnostics(&args.describe);
    let signature = signature_of(&events);
    println!("  fault signature: {}", signature.fingerprint);

    // 2. Cold-start corpus: empty in-memory store, no CEC service required.
    let corpus = LocalCorpus::new();
    let known = corpus.query(&signature).await?;
    println!(
        "  corpus: {} known mapping(s) for this signature (cold start = empty)",
        known.len()
    );

    // 3. Generate candidate plans. The swarm would fan this out to trusted
    //    nodes; here we always have a model-free heuristic candidate, and we add
    //    a model-proposed one when an endpoint is configured and reachable.
    let swarm = Swarm::new();
    println!(
        "  swarm: {} trusted node(s) registered",
        swarm.nodes().len()
    );

    let mut candidates = vec![heuristic_candidate(&args.describe)];
    if !args.offline {
        if let Some(base_url) = &args.endpoint {
            match model_candidate(base_url, &args.model, &args.describe).await {
                Ok(candidate) => candidates.push(candidate),
                Err(error) => eprintln!(
                    "  note: inference endpoint unavailable ({error}); \
                     continuing with the heuristic candidate"
                ),
            }
        }
    }

    // 4. Judge panel: score the slate, pick best-of-N, decide escalation.
    let judge = HeuristicJudge;
    let (index, best, score) =
        best_of_n(&judge, &candidates).expect("the heuristic candidate is always present");
    let escalation = escalation_for(best, &score);

    // 5. The tools that would execute the plan, behind the consent gate.
    let mut dispatcher = Dispatcher::new();
    for tool in windows_tools() {
        dispatcher.register(tool);
    }

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
    println!(
        "Execution and corpus write-back are gated on {escalation:?} sign-off and are NOT \
         performed by this run."
    );
    Ok(())
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

fn signature_of(events: &[DiagnosticEvent]) -> FaultSignature {
    let symptoms = events
        .iter()
        .map(|event| Symptom(normalize(&event.message)))
        .collect();
    FaultSignature::from_symptoms(symptoms)
}

fn normalize(text: &str) -> String {
    text.to_lowercase()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join("_")
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

async fn model_candidate(base_url: &str, model: &str, describe: &str) -> anyhow::Result<Candidate> {
    let client = OpenAiClient::new(Endpoint::new(base_url));
    let system = "You are a Windows support diagnostician. Given the user's problem, propose a \
                  single safe, reversible first step.";
    let request = ChatCompletionRequest::new(
        model,
        vec![ChatMessage::system(system), ChatMessage::user(describe)],
    );
    let response = client.complete(request).await?;
    let content = response
        .choices
        .into_iter()
        .next()
        .map(|choice| choice.message.content)
        .unwrap_or_default();

    let mut plan = Plan::new("model-1", "Model-proposed first step");
    plan.steps.push(PlanStep {
        description: content,
        action: "review".to_string(),
        risk: Risk::ReadOnly,
    });
    Ok(Candidate::new(
        plan,
        "Proposed by the configured inference endpoint".to_string(),
        CandidateSource::ColdModel,
    ))
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
    --offline            Skip the model call; use only the model-free heuristic
    -h, --help           Print help
    -V, --version        Print version

Cold start: with no --endpoint (or with --offline), the agent runs the full
diagnostic -> candidate -> judge pipeline using the model-free heuristic and an
empty in-memory corpus. No CEC-hosted service is required.",
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
    fn normalize_is_stable() {
        assert_eq!(normalize("Boot  LOOP"), "boot_loop");
    }
}
