// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 The cec-support-agent authors
//! Corpus client, schema, and cold-start store.
//!
//! This crate is the only contact point with the private corpus service. It
//! ships **no corpus data** (invariant 2): at cold start it runs against an
//! empty in-memory [`LocalCorpus`] and a local inference endpoint, with no
//! CEC-hosted service required (invariant 3).
//!
//! # Sign-off gate (invariant 6)
//! [`CorpusStore::submit`] refuses any [`Contribution`] whose [`SignOff`] is not
//! `VerifierConfirmed` or `HumanConfirmed`. The refusal is enforced here in
//! code via [`ensure_signed_off`] — not in documentation — and every submit
//! path (local and remote) calls it before persisting or transmitting anything.

//! # De-identification (invariant 1)
//! A corpus row is de-identified by structured extraction, not scrubbing:
//! fault signatures are built from a fixed vocabulary
//! ([`common::extract_symptoms`]) and [`Contribution::new`] strips every
//! free-text plan field down to the action vocabulary ([`de_identify_plan`]).
//! The adversarial leakage suite below seeds known identifiers through the
//! whole path and asserts zero leakage into the serialized row — a scrubbing
//! pass that is never adversarially tested is a scrubbing pass that leaks.

mod gate;
mod schema;
mod store;
mod stored;

pub use gate::{ensure_attested, ensure_evidence_integrity, ensure_signed_off, GateError};
pub use schema::{
    de_identify_plan, Contribution, FixMapping, Outcome, OutcomeLabel, RowIntegrity, RowProvenance,
    SignOff, SignOffAttestation,
};
pub use store::{CorpusError, CorpusStore, FileCorpus, HttpCorpus, LocalCorpus};
pub use stored::{StoredOutcome, StoredPlan, StoredSignature, StoredStep, StoredSymptom};

// The sign-off authority types live in `provenance`; re-export them so a caller
// configuring a store with `.with_authority(...)` (and attesting via
// `Contribution::attested_by(...)`) does not need a separate import.
pub use provenance::{SignOffAuthority, SignOffPublicKey, SignOffSignature};

#[cfg(test)]
mod leakage_tests {
    //! Adversarial de-identification suite: seed bundles with known
    //! identifiers and assert zero leakage into corpus rows.

    use super::*;
    use common::{extract_symptoms, ConfigClass, FaultSignature, Plan, PlanStep, Risk};

    /// Identity tokens seeded through the pipeline — the SINGLE source of truth
    /// from `leakguard`, so a new test cannot quietly narrow the set (the gap that
    /// let the old suite pass green while `action`/`id` leaked verbatim).
    use leakguard::POISON;

    /// A support request and a model-written plan saturated with identity in the
    /// FREE-TEXT fields (describe/title/description) but with a CLEAN, in-vocabulary
    /// `action`/`id` — the case that legitimately de-identifies (the free text is
    /// stripped to the action vocabulary). `Contribution::new` must succeed and the
    /// row must leak nothing.
    fn clean_contribution() -> Contribution {
        let describe = "DESKTOP-NATHAN01 (user nathan, MAC 00:1A:2B:3C:4D:5E, \
                        serial SN12345678): explorer.exe crashes on login, WER \
                        bucket 0x1234; logs under C:\\Users\\nathan; contact \
                        nathan@example.com or 192.168.1.20";
        let signature = FaultSignature::from_symptoms(extract_symptoms(describe));

        let mut plan = Plan::new("model-1", format!("Fix {describe}"));
        plan.steps.push(PlanStep {
            description: format!("On {describe}, roll back the display driver").into(),
            action: "driver_rollback".into(),
            risk: Risk::Reversible,
        });
        Contribution::new(
            Outcome {
                signature,
                plan,
                label: OutcomeLabel::ResolvedConfirmed,
                verification: Some(common::Verification::pass()),
            },
            ConfigClass::from_inventory(["os:windows 11 23h2", "gpu:rtx-4070"]),
            SignOff::HumanConfirmed,
        )
        .expect("a clean-action plan de-identifies")
    }

    #[test]
    fn no_seeded_identifier_reaches_a_serialized_corpus_row() {
        let row = serde_json::to_string(&clean_contribution()).expect("contribution serializes");
        leakguard::assert_no_poison(&row, "corpus row");
    }

    #[test]
    fn the_row_still_carries_the_structured_evidence() {
        let row = clean_contribution();
        let symptoms: Vec<&str> = row
            .outcome
            .signature
            .symptoms
            .iter()
            .map(|s| s.0.as_str())
            .collect();
        assert!(symptoms.contains(&"explorer.exe"));
        assert!(symptoms.contains(&"0x1234"));
        assert_eq!(row.outcome.plan.steps[0].action, "driver_rollback");
        assert_eq!(row.outcome.plan.steps[0].description, "driver_rollback");
    }

    // --- The C1 regression guards: the two fields `de_identify_plan` historically
    //     copied VERBATIM and the old suite never seeded. Identity placed here must
    //     now ABORT the row, not ride into it.

    #[test]
    fn a_poisoned_action_is_refused_not_copied_through() {
        let signature = FaultSignature::from_symptoms(extract_symptoms("explorer.exe 0x1234"));
        let mut plan = Plan::new("heuristic-1", "title");
        plan.steps.push(PlanStep {
            description: "x".into(),
            // The documented agent mistake: a generator routes prose into `action`
            // ("powershell ... DESKTOP-NATHAN01") — identity in the action field.
            action: format!("powershell on {}", POISON[0]),
            risk: Risk::ReadOnly,
        });
        let refused = Contribution::new(
            Outcome {
                signature,
                plan,
                label: OutcomeLabel::ResolvedConfirmed,
                verification: Some(common::Verification::pass()),
            },
            ConfigClass::from_inventory(["os:windows"]),
            SignOff::HumanConfirmed,
        );
        assert!(
            refused.is_err(),
            "a poisoned action must be REFUSED by the de-id mint, not copied into the row"
        );
    }

    #[test]
    fn a_poisoned_plan_id_is_refused() {
        let signature = FaultSignature::from_symptoms(extract_symptoms("explorer.exe 0x1234"));
        // An id built from request text: format!("fix for {describe}") — spaces/caps.
        let mut plan = Plan::new("fix for DESKTOP-NATHAN01", "title");
        plan.steps.push(PlanStep {
            description: "x".into(),
            action: "cim_query".into(),
            risk: Risk::ReadOnly,
        });
        let refused = Contribution::new(
            Outcome {
                signature,
                plan,
                label: OutcomeLabel::ResolvedConfirmed,
                verification: Some(common::Verification::pass()),
            },
            ConfigClass::from_inventory(["os:windows"]),
            SignOff::HumanConfirmed,
        );
        assert!(
            refused.is_err(),
            "a poisoned plan id must be REFUSED by the de-id mint"
        );
    }

    #[test]
    fn de_identify_plan_keeps_order_and_risk() {
        let mut plan = Plan::new("p", "free text title");
        plan.steps.push(PlanStep {
            description: "prose".into(),
            action: "cim_query".into(),
            risk: Risk::ReadOnly,
        });
        plan.steps.push(PlanStep {
            description: "more prose".into(),
            action: "registry_set".into(),
            risk: Risk::Reversible,
        });
        let row = de_identify_plan(&plan).expect("clean actions de-identify");
        assert_eq!(row.id(), "p");
        assert_eq!(row.title(), "cim_query -> registry_set");
        assert_eq!(row.risk(), Risk::Reversible);
        assert_eq!(row.steps().len(), 2);
    }

    // --- 1d sealed Debug: `format!("{:?}", outcome)` must never spill request
    //     prose. The raw Outcome holds a Plan whose title/description are Prose,
    //     so a stray `dbg!(outcome)` / `tracing::info!(?outcome)` is redacted by
    //     construction — no manual per-struct Debug impl needed.

    #[test]
    fn debug_of_a_raw_outcome_never_leaks_planted_prose() {
        let host = "ZZHOSTZZ";
        let user = "ZZUSERZZ";
        let mut plan = Plan::new("heuristic-1", format!("fix for {host}"));
        plan.steps.push(PlanStep {
            description: format!("act against {user}").into(),
            action: "cim_query".into(),
            risk: Risk::ReadOnly,
        });
        let outcome = Outcome {
            signature: FaultSignature::from_symptoms(extract_symptoms("explorer.exe 0x1234")),
            plan,
            label: OutcomeLabel::ResolvedConfirmed,
            verification: Some(common::Verification::pass()),
        };
        let shown = format!("{outcome:?}");
        assert!(
            !shown.contains(host),
            "Debug leaked the planted host: {shown}"
        );
        assert!(
            !shown.contains(user),
            "Debug leaked the planted user: {shown}"
        );
        // Still structurally useful: the redaction marker and the clean action
        // vocabulary survive, so Debug remains diagnostic, just not leaky.
        assert!(shown.contains("redacted"));
        assert!(shown.contains("cim_query"));
    }
}
