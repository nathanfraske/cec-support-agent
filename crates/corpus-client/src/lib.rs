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

pub use gate::{ensure_evidence_integrity, ensure_signed_off, GateError};
pub use schema::{de_identify_plan, Contribution, FixMapping, Outcome, OutcomeLabel, SignOff};
pub use store::{CorpusError, CorpusStore, FileCorpus, HttpCorpus, LocalCorpus};

#[cfg(test)]
mod leakage_tests {
    //! Adversarial de-identification suite: seed bundles with known
    //! identifiers and assert zero leakage into corpus rows.

    use super::*;
    use common::{extract_symptoms, ConfigClass, FaultSignature, Plan, PlanStep, Risk};

    /// Identifiers seeded through the pipeline. None may survive into a
    /// serialized corpus row, under any casing.
    const SEEDED_IDENTIFIERS: &[&str] = &[
        "desktop-nathan01",
        "nathan",
        "c:\\users",
        "nathan@example.com",
        "192.168.1.20",
        "00:1a:2b:3c:4d:5e",
        "sn12345678",
    ];

    /// A support request and a (hostile) model-written plan, both saturated
    /// with identity, as the worst realistic input.
    fn seeded_contribution() -> Contribution {
        let describe = "DESKTOP-NATHAN01 (user nathan, MAC 00:1A:2B:3C:4D:5E, \
                        serial SN12345678): explorer.exe crashes on login, WER \
                        bucket 0x1234; logs under C:\\Users\\nathan; contact \
                        nathan@example.com or 192.168.1.20";
        let signature = FaultSignature::from_symptoms(extract_symptoms(describe));

        let mut plan = Plan::new("model-1", format!("Fix {describe}"));
        plan.steps.push(PlanStep {
            description: format!("On {describe}, roll back the display driver"),
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
    }

    #[test]
    fn no_seeded_identifier_reaches_a_serialized_corpus_row() {
        let row = serde_json::to_string(&seeded_contribution())
            .expect("contribution serializes")
            .to_lowercase();
        for identifier in SEEDED_IDENTIFIERS {
            assert!(
                !row.contains(identifier),
                "identifier {identifier:?} leaked into the corpus row: {row}"
            );
        }
    }

    #[test]
    fn the_row_still_carries_the_structured_evidence() {
        let row = seeded_contribution();
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
        let row = de_identify_plan(&plan);
        assert_eq!(row.id, "p");
        assert_eq!(row.title, "cim_query -> registry_set");
        assert_eq!(row.risk(), Risk::Reversible);
        assert_eq!(row.steps.len(), 2);
    }
}
