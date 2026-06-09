// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 The cec-support-agent authors
//! Judge panel.
//!
//! The panel has three jobs, in order. Routing first ([`route_for`]): classify
//! the case as software-state, hardware-evidenced, or ambiguous — the routing
//! verdict determines which gates are load-bearing. Scoring second: each
//! candidate is scored across a fixed set of [`ScoreAxis`]es and the panel
//! picks the [`best_of_n`]. Selection or escalation third: the
//! [`Escalation`] ladder ([`required_escalation`]) decides how far the winner
//! must be confirmed before it may run. The default [`HeuristicJudge`] is
//! model-free and deterministic, so the panel works at cold start with no
//! inference endpoint.

use std::cmp::Ordering;

use common::{Candidate, FaultSignature, Risk};
use serde::{Deserialize, Serialize};

/// The routing taxonomy: which kind of case this ticket is, decided before
/// any plan is scored, because the routing verdict determines which gates are
/// load-bearing.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Route {
    /// Sandbox-validatable and automatable end to end.
    SoftwareState,
    /// The evidence names a part. The winning "plan" is a diagnosis plus an
    /// RMA or bench action plus optional mitigations; the sandbox is moot. An
    /// evidence-backed hardware verdict is a successful outcome.
    HardwareEvidenced {
        /// The implicated part class (e.g. "psu", "storage", "memory").
        part_class: String,
    },
    /// The evidence does not clearly name software state or a part: escalate.
    Ambiguous,
}

impl Route {
    /// A plain-language explanation of the routing verdict, written for a
    /// non-technical user: what the evidence points at, what that part does,
    /// what the typical signs are, and what it means for them. Jargon-free by
    /// contract — this string goes on the user's screen, not in a log.
    pub fn explanation(&self) -> String {
        match self {
            Route::SoftwareState => "The evidence points at software — Windows itself, a \
                driver (the small program that runs a piece of hardware), or an installed \
                program — rather than a broken part. Problems like this can usually be \
                fixed without opening the computer."
                .to_string(),
            Route::HardwareEvidenced { part_class } => {
                let part = match part_class.as_str() {
                    "psu" => {
                        "the power supply — the part that turns wall power into the \
                        power every other part runs on. Sudden shut-offs, restarts with no \
                        error message, and the machine dying under load are its typical \
                        signs"
                    }
                    "gpu" => {
                        "the graphics card — the part that draws everything you see on \
                        screen. Crashes during games, visual glitches, and black screens \
                        while the fans keep running are its typical signs"
                    }
                    "storage" => {
                        "the storage drive — the part your files and Windows \
                        itself live on. Slowness, freezes, and files failing to open are \
                        its typical signs. As a precaution, copy your important files \
                        (documents, photos) somewhere else soon"
                    }
                    "cooling" => {
                        "cooling — the fans and airflow that keep the computer \
                        from overheating. Loud fans, very hot air, and shutdowns during \
                        heavy use are its typical signs. Check that the vents are not \
                        blocked with dust"
                    }
                    "memory" => {
                        "the memory (RAM) — the part the computer uses as its \
                        short-term workspace. Random crashes and blue screens that name a \
                        different cause each time are its typical signs"
                    }
                    "platform" => {
                        "the computer's core hardware — the processor or the \
                        main board everything plugs into. Errors reported by the hardware \
                        itself point here"
                    }
                    other => other,
                };
                format!(
                    "The evidence points at a physical part rather than software: {part}. \
                     A software change cannot repair a failing part — it needs to be \
                     inspected by a person and possibly replaced. Any steps suggested \
                     below are temporary measures to keep things stable until then."
                )
            }
            Route::Ambiguous => "The information so far does not clearly point at software \
                or at any specific part, so this case goes to a person for review instead \
                of letting the computer act on a guess. That is deliberate caution, not a \
                dead end."
                .to_string(),
        }
    }
}

/// Hardware-evidence markers and the part class each one names. Matched
/// against the structured symptoms of [`common::extract_symptoms`]; a prefix
/// entry (trailing `_`) matches id-bearing symptoms like `xid_79`.
const HARDWARE_MARKERS: &[(&str, &str)] = &[
    ("whea", "platform"),
    ("voltage", "psu"),
    ("smart", "storage"),
    ("thermal", "cooling"),
    ("overheat", "cooling"),
    ("xid_", "gpu"),
];

/// Software-state markers: evidence the case is diagnosable and remediable in
/// OS state alone.
const SOFTWARE_MARKERS: &[&str] = &[
    "crash",
    "crashes",
    "crashing",
    "hang",
    "freeze",
    "frozen",
    "slow",
    "boot",
    "login",
    "logon",
    "update",
    "driver",
    "corrupt",
    "corruption",
    "timeout",
    "tdr",
    "wer",
    "bsod",
    "bluescreen",
    "loop",
];

/// Route a ticket from its fault signature. Hardware evidence wins over
/// software evidence: a crash *with* a WHEA record is a hardware case that
/// happens to also crash. A signature carrying neither kind of marker — or no
/// symptoms at all — is [`Route::Ambiguous`] and escalates; early in
/// operation that is most tickets, by design.
pub fn route_for(signature: &FaultSignature) -> Route {
    let symptoms: Vec<&str> = signature.symptoms.iter().map(|s| s.0.as_str()).collect();

    for &(marker, part_class) in HARDWARE_MARKERS {
        let hit = if let Some(prefix) = marker.strip_suffix('_') {
            symptoms
                .iter()
                .any(|s| s.strip_prefix(prefix).is_some_and(|r| r.starts_with('_')))
        } else {
            // Exact match, or a substring of an underscore-joined code
            // symptom: "whea" must hit the recited stop-code name
            // "whea_uncorrectable_error", but never a module name like
            // "smartscreen.exe".
            symptoms
                .iter()
                .any(|s| *s == marker || (s.contains('_') && s.contains(marker)))
        };
        if hit {
            return Route::HardwareEvidenced {
                part_class: part_class.to_string(),
            };
        }
    }
    // A hard power event names the PSU path: Kernel-Power 41 is the
    // OS-visible fingerprint of power loss, not of software state.
    if symptoms.contains(&"kernel") && symptoms.iter().any(|s| s.starts_with("power")) {
        return Route::HardwareEvidenced {
            part_class: "psu".to_string(),
        };
    }

    if symptoms
        .iter()
        .any(|s| SOFTWARE_MARKERS.contains(s) || s.starts_with("event_") || s.ends_with(".exe"))
    {
        return Route::SoftwareState;
    }
    Route::Ambiguous
}

/// The axes a judge scores a candidate on. Higher is better on every axis.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ScoreAxis {
    /// How likely the plan resolves the fault.
    Likelihood,
    /// How safe the plan is (inverse of blast radius).
    Safety,
    /// How easily the plan can be undone.
    Reversibility,
    /// How cheap the plan is to run (time and disruption).
    Cost,
}

impl ScoreAxis {
    /// Every axis, for callers that iterate the full set.
    pub const ALL: [ScoreAxis; 4] = [
        ScoreAxis::Likelihood,
        ScoreAxis::Safety,
        ScoreAxis::Reversibility,
        ScoreAxis::Cost,
    ];
}

/// A candidate's scores. Each value is in `[0.0, 1.0]`.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Score {
    pub likelihood: f32,
    pub safety: f32,
    pub reversibility: f32,
    pub cost: f32,
}

impl Score {
    /// Weighted aggregate used to rank candidates. Safety and reversibility are
    /// weighted to bias the panel toward plans that are easy to undo.
    pub fn total(&self) -> f32 {
        0.4 * self.likelihood + 0.3 * self.safety + 0.2 * self.reversibility + 0.1 * self.cost
    }
}

/// Scores a candidate plan.
pub trait Judge {
    /// Produce a [`Score`] for `candidate`.
    fn score(&self, candidate: &Candidate) -> Score;
}

/// A deterministic, model-free judge usable at cold start. It rewards
/// lower-risk, reversible plans and penalizes destructive or sprawling ones.
#[derive(Debug, Default, Clone, Copy)]
pub struct HeuristicJudge;

impl Judge for HeuristicJudge {
    fn score(&self, candidate: &Candidate) -> Score {
        let (safety, reversibility) = match candidate.plan.risk() {
            Risk::ReadOnly => (1.0, 1.0),
            Risk::Reversible => (0.7, 0.9),
            Risk::Destructive => (0.2, 0.1),
        };
        let likelihood = if candidate.plan.steps.is_empty() {
            0.0
        } else {
            // Fix likelihood under corpus priors: a plan retrieved from
            // confirmed precedent outranks a cold guess of the same shape.
            match candidate.source {
                common::CandidateSource::CorpusPrimed => 0.8,
                _ => 0.6,
            }
        };
        let cost = 1.0 - (candidate.plan.steps.len() as f32 / 20.0).min(1.0);
        Score {
            likelihood,
            safety,
            reversibility,
            cost,
        }
    }
}

/// Select the highest-scoring candidate, returning its index, the candidate,
/// and its score. Returns `None` for an empty slate.
pub fn best_of_n<'a, J: Judge>(
    judge: &J,
    candidates: &'a [Candidate],
) -> Option<(usize, &'a Candidate, Score)> {
    candidates
        .iter()
        .enumerate()
        .map(|(index, candidate)| (index, candidate, judge.score(candidate)))
        .max_by(|a, b| {
            a.2.total()
                .partial_cmp(&b.2.total())
                .unwrap_or(Ordering::Equal)
        })
}

/// How far a winning plan must be confirmed before it may run.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Serialize, Deserialize,
)]
#[serde(rename_all = "snake_case")]
pub enum Escalation {
    /// Safe enough to run automatically.
    #[default]
    Auto,
    /// An automated verifier must confirm the outcome.
    VerifierConfirm,
    /// A human must confirm before and/or after execution.
    HumanConfirm,
}

/// Decide the escalation level for a scored candidate. Destructive plans always
/// require a human; low-confidence reversible plans do too.
pub fn escalation_for(candidate: &Candidate, score: &Score) -> Escalation {
    match candidate.plan.risk() {
        Risk::Destructive => Escalation::HumanConfirm,
        Risk::Reversible if score.total() < 0.6 => Escalation::HumanConfirm,
        Risk::Reversible => Escalation::VerifierConfirm,
        Risk::ReadOnly => Escalation::Auto,
    }
}

/// The full escalation decision: the risk/score ladder of [`escalation_for`],
/// raised by the routing verdict and the validation state.
///
/// - A hardware-evidenced or ambiguous route always requires a human (the
///   hardware class never auto-signs in v0; ambiguity escalates by design).
/// - A state-changing plan with no sandbox validation report requires a human:
///   unvalidated equals escalate.
/// - Escalation triggers independent of confidence — a high judge score does
///   not lower it.
pub fn required_escalation(
    route: &Route,
    sandbox_validated: bool,
    candidate: &Candidate,
    score: &Score,
) -> Escalation {
    let mut level = escalation_for(candidate, score);
    if !matches!(route, Route::SoftwareState) {
        level = level.max(Escalation::HumanConfirm);
    }
    if candidate.plan.requires_consent() && !sandbox_validated {
        level = level.max(Escalation::HumanConfirm);
    }
    level
}

#[cfg(test)]
mod tests {
    use super::*;
    use common::{CandidateSource, Plan, PlanStep};

    fn candidate_with(risk: Risk) -> Candidate {
        let mut plan = Plan::new("p", "t");
        plan.steps.push(PlanStep {
            description: "do".into(),
            action: "act".into(),
            risk,
        });
        Candidate::new(plan, "because", CandidateSource::ColdModel)
    }

    #[test]
    fn best_of_n_prefers_the_safer_plan() {
        let judge = HeuristicJudge;
        let candidates = vec![
            candidate_with(Risk::Destructive),
            candidate_with(Risk::Reversible),
        ];
        let (index, _, _) = best_of_n(&judge, &candidates).expect("non-empty");
        assert_eq!(index, 1);
    }

    #[test]
    fn corpus_primed_precedent_outranks_a_cold_guess_of_the_same_shape() {
        let judge = HeuristicJudge;
        let cold = candidate_with(Risk::Reversible);
        let mut primed = candidate_with(Risk::Reversible);
        primed.source = CandidateSource::CorpusPrimed;
        let candidates = vec![cold, primed];
        let (index, _, _) = best_of_n(&judge, &candidates).expect("non-empty");
        assert_eq!(index, 1, "precedent wins under corpus priors");
    }

    #[test]
    fn best_of_n_on_empty_is_none() {
        let judge = HeuristicJudge;
        assert!(best_of_n(&judge, &[]).is_none());
    }

    #[test]
    fn destructive_plans_require_a_human() {
        let judge = HeuristicJudge;
        let candidate = candidate_with(Risk::Destructive);
        let score = judge.score(&candidate);
        assert_eq!(escalation_for(&candidate, &score), Escalation::HumanConfirm);
    }

    #[test]
    fn read_only_plans_run_automatically() {
        let judge = HeuristicJudge;
        let candidate = candidate_with(Risk::ReadOnly);
        let score = judge.score(&candidate);
        assert_eq!(escalation_for(&candidate, &score), Escalation::Auto);
    }

    fn signature(symptoms: &[&str]) -> FaultSignature {
        FaultSignature::from_symptoms(symptoms.iter().map(|s| Symptom(s.to_string())).collect())
    }

    use common::Symptom;

    #[test]
    fn routing_prefers_hardware_evidence_over_software_evidence() {
        // A crash that also carries a WHEA record is a hardware case.
        let route = route_for(&signature(&["crash", "whea"]));
        assert_eq!(
            route,
            Route::HardwareEvidenced {
                part_class: "platform".into()
            }
        );
    }

    #[test]
    fn kernel_power_routes_to_the_psu_path() {
        let route = route_for(&signature(&["kernel", "power", "power_41", "shutdown"]));
        assert_eq!(
            route,
            Route::HardwareEvidenced {
                part_class: "psu".into()
            }
        );
    }

    #[test]
    fn recited_stop_code_names_route_to_hardware() {
        // A user quoting WHEA_UNCORRECTABLE_ERROR off the bluescreen carries
        // hardware evidence even if they have no idea what it means.
        assert_eq!(
            route_for(&signature(&["whea_uncorrectable_error", "bsod"])),
            Route::HardwareEvidenced {
                part_class: "platform".into()
            }
        );
        // But a module name containing a marker substring must not match.
        assert_ne!(
            route_for(&signature(&["smartscreen.exe", "crash"])),
            Route::HardwareEvidenced {
                part_class: "storage".into()
            }
        );
    }

    #[test]
    fn xid_ids_route_to_the_gpu_path() {
        let route = route_for(&signature(&["xid_79", "crash"]));
        assert_eq!(
            route,
            Route::HardwareEvidenced {
                part_class: "gpu".into()
            }
        );
    }

    #[test]
    fn software_evidence_routes_software_state() {
        assert_eq!(
            route_for(&signature(&["explorer.exe", "crashes", "login"])),
            Route::SoftwareState
        );
    }

    #[test]
    fn no_recognizable_evidence_is_ambiguous() {
        assert_eq!(route_for(&signature(&[])), Route::Ambiguous);
        assert_eq!(route_for(&signature(&["0x1234"])), Route::Ambiguous);
    }

    #[test]
    fn every_route_explains_itself_in_plain_language() {
        // Each known part class names the part, what it does, and its signs.
        for (class, expect) in [
            ("psu", "power supply"),
            ("gpu", "graphics card"),
            ("storage", "storage drive"),
            ("cooling", "fans"),
            ("memory", "short-term workspace"),
            ("platform", "main board"),
        ] {
            let text = Route::HardwareEvidenced {
                part_class: class.into(),
            }
            .explanation();
            assert!(text.contains(expect), "{class}: {text}");
            assert!(text.contains("typical signs") || text.contains("point here"));
        }
        // Software and ambiguous routes explain what happens next.
        assert!(Route::SoftwareState
            .explanation()
            .contains("a driver (the small program"));
        assert!(Route::Ambiguous.explanation().contains("goes to a person"));
        // The storage explanation tells the user to back up — the one piece
        // of advice that must never be omitted.
        let storage = Route::HardwareEvidenced {
            part_class: "storage".into(),
        }
        .explanation();
        assert!(storage.contains("copy your important files"));
    }

    #[test]
    fn hardware_and_ambiguous_routes_always_require_a_human() {
        let judge = HeuristicJudge;
        let candidate = candidate_with(Risk::ReadOnly);
        let score = judge.score(&candidate);
        let hardware = Route::HardwareEvidenced {
            part_class: "psu".into(),
        };
        assert_eq!(
            required_escalation(&hardware, true, &candidate, &score),
            Escalation::HumanConfirm
        );
        assert_eq!(
            required_escalation(&Route::Ambiguous, true, &candidate, &score),
            Escalation::HumanConfirm
        );
    }

    #[test]
    fn unvalidated_state_changing_plans_require_a_human() {
        let judge = HeuristicJudge;
        let candidate = candidate_with(Risk::Reversible);
        let score = judge.score(&candidate);
        // Validated: the normal ladder applies.
        assert_eq!(
            required_escalation(&Route::SoftwareState, true, &candidate, &score),
            Escalation::VerifierConfirm
        );
        // Unvalidated equals escalate.
        assert_eq!(
            required_escalation(&Route::SoftwareState, false, &candidate, &score),
            Escalation::HumanConfirm
        );
    }

    #[test]
    fn unvalidated_read_only_plans_may_still_run_automatically() {
        let judge = HeuristicJudge;
        let candidate = candidate_with(Risk::ReadOnly);
        let score = judge.score(&candidate);
        assert_eq!(
            required_escalation(&Route::SoftwareState, false, &candidate, &score),
            Escalation::Auto
        );
    }
}
