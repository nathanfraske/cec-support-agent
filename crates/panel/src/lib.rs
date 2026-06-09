// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 The cec-support-agent authors
//! Judge panel.
//!
//! The panel scores candidate plans across a fixed set of [`ScoreAxis`]es,
//! picks the [`best_of_n`], and decides how far the winner must be confirmed
//! before it may run via the [`Escalation`] ladder. The default
//! [`HeuristicJudge`] is model-free and deterministic, so the panel works at
//! cold start with no inference endpoint.

use std::cmp::Ordering;

use common::{Candidate, Risk};
use serde::{Deserialize, Serialize};

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
            0.6
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
}
