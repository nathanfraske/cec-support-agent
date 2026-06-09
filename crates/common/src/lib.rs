// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 The cec-support-agent authors
//! Shared domain types for the cec-support-agent engine.
//!
//! These types are the vocabulary that the `inference`, `panel`, `swarm`, and
//! `corpus-client` crates speak. They carry no corpus data and no provider
//! coupling — they are plain serializable records. Keeping them here lets the
//! host app (MyOwnLLM) and the AllMyStuff brain link the engine without pulling
//! in any private corpus schema.

mod candidate;
mod diagnostic;
mod fault;
mod plan;

pub use candidate::{Candidate, CandidateSource};
pub use diagnostic::{DiagnosticEvent, EventKind, Severity};
pub use fault::{FaultSignature, Symptom};
pub use plan::{Plan, PlanStep, Risk};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plan_risk_is_max_step_risk() {
        let mut plan = Plan::new("p1", "mixed");
        plan.steps.push(PlanStep {
            description: "look".into(),
            action: "cim_query".into(),
            risk: Risk::ReadOnly,
        });
        plan.steps.push(PlanStep {
            description: "change".into(),
            action: "registry_set".into(),
            risk: Risk::Reversible,
        });
        assert_eq!(plan.risk(), Risk::Reversible);
        assert!(plan.requires_consent());
    }

    #[test]
    fn empty_plan_is_read_only() {
        let plan = Plan::new("p0", "noop");
        assert_eq!(plan.risk(), Risk::ReadOnly);
        assert!(!plan.requires_consent());
    }

    #[test]
    fn fingerprint_is_order_independent_and_deterministic() {
        let a = FaultSignature::from_symptoms(vec![
            Symptom("boot_loop".into()),
            Symptom("bsod".into()),
        ]);
        let b = FaultSignature::from_symptoms(vec![
            Symptom("bsod".into()),
            Symptom("boot_loop".into()),
        ]);
        assert_eq!(a.fingerprint, b.fingerprint);
        let c = FaultSignature::from_symptoms(vec![Symptom("disk_full".into())]);
        assert_ne!(a.fingerprint, c.fingerprint);
    }
}
