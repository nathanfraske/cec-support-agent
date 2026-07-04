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
mod config_class;
mod diagnostic;
mod execution;
mod extract;
mod fault;
mod fluency;
mod hash;
mod inventory;
mod plan;
mod prose;
mod verification;

pub use candidate::{Candidate, CandidateSource};
pub use config_class::ConfigClass;
pub use diagnostic::{DiagnosticEvent, EventKind, Severity};
pub use execution::{ExecutionResult, StepResult};
pub use extract::{extract_symptoms, is_symptom_token};
pub use fault::{FaultSignature, Symptom};
pub use fluency::Fluency;
pub use hash::{
    set_fingerprint_salt, FingerprintSaltError, COLD_START_FINGERPRINT_SALT,
    MIN_FINGERPRINT_SALT_LEN,
};
pub use inventory::{CoarseHostInventory, ExternalInventory, InventoryProvider};
pub use plan::{Plan, PlanStep, Risk};
pub use prose::Prose;
pub use verification::{Verification, VerificationClass, VerificationResult};

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

    #[test]
    fn config_class_unifies_bom_and_derived_hash() {
        let cec = ConfigClass::from_bom("BOM-2026.06-r3");
        assert_eq!(cec.key(), "BOM-2026.06-r3");

        // Same inventory, any order and casing, yields the same class.
        let a = ConfigClass::from_inventory(["os:windows 11 23h2", "GPU:rtx-4070"]);
        let b = ConfigClass::from_inventory(["gpu:rtx-4070", " OS:Windows 11 23H2 "]);
        assert_eq!(a, b);
        let c = ConfigClass::from_inventory(["os:windows 10"]);
        assert_ne!(a, c);
    }

    #[test]
    fn recurring_symptoms_diff_post_against_original() {
        let original = FaultSignature::from_symptoms(vec![
            Symptom("crash".into()),
            Symptom("event_41".into()),
        ]);
        let still_broken = FaultSignature::from_symptoms(vec![Symptom("event_41".into())]);
        assert_eq!(
            original.recurring_in(&still_broken),
            vec![Symptom("event_41".into())]
        );

        let healthy = FaultSignature::from_symptoms(vec![]);
        assert!(original.recurring_in(&healthy).is_empty());
    }

    #[test]
    fn execution_result_tracks_step_success() {
        let mut result = ExecutionResult::new("p1");
        assert_eq!(result.plan_id, "p1");
        assert!(!result.completed);
        assert!(result.all_ok(), "no steps is vacuously all-ok");

        result.steps.push(StepResult {
            step: 1,
            action: "read".into(),
            ok: true,
            summary: "ok".into(),
        });
        assert!(result.all_ok());
        result.steps.push(StepResult {
            step: 2,
            action: "registry_set".into(),
            ok: false,
            summary: "consent denied".into(),
        });
        assert!(!result.all_ok());
    }
}
