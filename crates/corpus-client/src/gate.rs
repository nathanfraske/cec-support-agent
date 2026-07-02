use common::{Risk, VerificationResult};
use provenance::{SignOffPublicKey, SignOffSignature};
use thiserror::Error;

use crate::schema::{
    attestation_message, de_identify_plan, is_run_id_token, Contribution, OutcomeLabel, SignOff,
};

/// Why a contribution was refused at the evidence-integrity gate.
///
/// The gate is the inverted corpus's single truth-admission boundary, so its
/// refusals are structured: each names exactly which integrity property the row
/// failed, rather than a single opaque "refused".
#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum GateError {
    /// Sign-off has not cleared the gate (invariant 6): not verifier- or
    /// human-confirmed.
    #[error("refused: outcome is not verifier- or human-confirmed (sign-off gate)")]
    Unconfirmed,
    /// A resolved label carries no passing verification verdict — a "resolved"
    /// row with no evidence behind it is unauditable and must not become truth.
    #[error("refused: a resolved outcome must carry a passing verification verdict")]
    ResolvedWithoutPass,
    /// The resolved label and the verification verdict disagree (e.g.
    /// `ResolvedConfirmed` with only a provisional pass).
    #[error("refused: the resolved label does not match the verification verdict")]
    LabelVerdictMismatch,
    /// A resolved *destructive* fix carries less than human sign-off. A verifier
    /// may authorize reversible changes; only a human may authorize destructive
    /// ones — enforced here, not only at the CLI, so an embedder cannot mint a
    /// destructive "fix" with a verifier sign-off.
    #[error("refused: a resolved destructive plan requires human sign-off")]
    DestructiveFixNeedsHuman,
    /// The store requires a sign-off authority's attestation but the row carries
    /// none — a self-asserted sign-off, unsigned by any authority.
    #[error("refused: sign-off requires an authority attestation, but none is present")]
    AttestationMissing,
    /// The row's attestation does not verify against the configured authority:
    /// wrong key, a tampered tuple, or a malformed signature.
    #[error("refused: the sign-off attestation does not verify against the configured authority")]
    AttestationInvalid,
    /// A plan served by a remote corpus failed read-side de-identification
    /// re-validation: an out-of-vocabulary action, an inadmissible id, or
    /// free-text fields — content a compromised or buggy server could feed
    /// into the retrieval-first slate. The read path refuses the response.
    #[error("refused: a served plan failed read-side de-identification re-validation")]
    ServedPlanInadmissible,
    /// The row's stored plan is not its own de-identified image: an
    /// out-of-vocabulary action, an inadmissible id, or a title/description the
    /// de-id mint would never have produced (e.g. a hand-edited at-rest row).
    /// The write gate re-runs the de-id mint over the stored row and refuses
    /// anything whose content is not already extraction-clean — the only
    /// CONTENT check standing on the runtime corpus-write path.
    #[error("refused: the row's stored plan is not its own de-identified image")]
    RowNotDeIdentified,
    /// A stored symptom on the row (in the fault signature or a verification's
    /// recurring set) is not a member of the closed de-id grammar — an
    /// identity-shaped token an embedder or a hand-edited row could carry. The
    /// write gate re-runs the symptom mint over every stored symptom; the read
    /// path additionally refuses it at deserialize (`#[serde(try_from)]`).
    #[error("refused: a stored symptom is not a member of the de-id grammar")]
    SymptomNotDeIdentified,
    /// The row's run-provenance `run_id` is not a bounded opaque token — a
    /// free-text/path/email/prose string a hand-edited or embedder-built row could
    /// carry on the provenance pin, which rides onto the JSONL row unmodified. The
    /// write gate bounds it; the read path additionally refuses it at deserialize.
    #[error("refused: a run-provenance run_id is not a bounded opaque token")]
    RunIdNotDeIdentified,
}

/// The evidence-integrity gate: the single checkpoint that admits a row into the
/// inverted corpus. Returns `Ok(())` only when the contribution is sign-off
/// confirmed AND — for a resolved outcome (the rows that become retrievable
/// fixes) — its truth claim is bound to its evidence:
///
/// 1. **Sign-off confirmed** (invariant 6): verifier or human.
/// 2. **De-identified image** (1f): the stored plan equals its own
///    `de_identify_plan` re-mint — a content check that refuses an
///    out-of-vocabulary action, an inadmissible id, or a hand-edited
///    title/description on any row, including one loaded from disk.
/// 3. **Resolved ⇒ passing verdict** that *matches* the label: `ResolvedConfirmed`
///    needs a `Pass`, `ResolvedProvisional` a `ProvisionalPass`.
/// 4. **Destructive resolved fix ⇒ human sign-off.**
///
/// Hard negatives (non-resolved labels) are admitted regardless — a failure is
/// truth too, and an unlabeled ticket is corpus poison — they just never back a
/// [`FixMapping`].
///
/// Every [`CorpusStore::submit`](crate::CorpusStore::submit) implementation MUST
/// call this before persisting or transmitting a contribution.
pub fn ensure_evidence_integrity(contribution: &Contribution) -> Result<(), GateError> {
    // (1) Sign-off must clear the gate.
    if !contribution.sign_off.is_confirmed() {
        return Err(GateError::Unconfirmed);
    }

    // (1b / 1f) The stored plan must be its OWN de-identified image. Re-run the
    // de-id mint over the row (rehydrate → re-mint) and require an exact match:
    // an out-of-vocabulary action or inadmissible id fails the mint, and a
    // hand-edited title/description the mint would never have produced fails the
    // idempotence equality. `Contribution::new` guarantees this for a freshly
    // built row, but a row loaded from disk or handed in by an embedder has NOT
    // been through the constructor — and this is the only content check on the
    // runtime `/mnt/e` corpus-write path, which no git/CI/CODEOWNERS layer sees.
    match de_identify_plan(&contribution.outcome.plan.to_plan()) {
        Ok(reminted) if reminted == contribution.outcome.plan => {}
        _ => return Err(GateError::RowNotDeIdentified),
    }

    // (1c/C5) Every stored symptom must be a member of the closed de-id grammar.
    // Phase 2 wired the symptom mint (now that the closed grammar admits a
    // legitimate `<prefix>_<digits>` token like `event_41`, which the earlier
    // round-trip form rejected). This catches an embedder-built or hand-edited
    // row whose signature — or a verification's recurring set — carries an
    // identity-shaped "symptom" that never came from the extractor.
    let outcome = &contribution.outcome;
    for symptom in outcome.signature.symptoms() {
        if !common::is_symptom_token(symptom.as_str()) {
            return Err(GateError::SymptomNotDeIdentified);
        }
    }
    if let Some(verification) = outcome.verification() {
        for symptom in &verification.recurring {
            if !common::is_symptom_token(&symptom.0) {
                return Err(GateError::SymptomNotDeIdentified);
            }
        }
    }

    // (1d) The run-provenance pin carries a `run_id` onto the row unmodified. It
    // is OS entropy on shipped paths, but an embedder-built or hand-edited row
    // could smuggle a path/email/prose here; bound it to an opaque token on the
    // write boundary, symmetric with the read-side deserialize guard.
    if let Some(provenance) = &contribution.provenance {
        if !is_run_id_token(&provenance.run_id) {
            return Err(GateError::RunIdNotDeIdentified);
        }
    }

    if outcome.label.is_resolved() {
        // (2) A resolved label must be backed by a matching passing verdict.
        match (
            &outcome.label,
            outcome.verification.as_ref().map(|v| v.result),
        ) {
            (OutcomeLabel::ResolvedConfirmed, Some(VerificationResult::Pass))
            | (OutcomeLabel::ResolvedProvisional, Some(VerificationResult::ProvisionalPass)) => {}
            // A passing verdict that disagrees with the label (e.g. a confirmed
            // label over a provisional pass) is a mismatch, not missing evidence.
            (_, Some(r)) if r.is_pass() => return Err(GateError::LabelVerdictMismatch),
            // Missing, or a non-passing (Fail / OffMachine) verdict.
            _ => return Err(GateError::ResolvedWithoutPass),
        }

        // (3) A resolved destructive fix requires human sign-off.
        if matches!(outcome.plan.risk(), Risk::Destructive)
            && contribution.sign_off != SignOff::HumanConfirmed
        {
            return Err(GateError::DestructiveFixNeedsHuman);
        }
    }

    Ok(())
}

/// Back-compat name for the sign-off gate. Prefer [`ensure_evidence_integrity`],
/// which this delegates to: the gate now enforces more than sign-off confirmation.
pub fn ensure_signed_off(contribution: &Contribution) -> Result<(), GateError> {
    ensure_evidence_integrity(contribution)
}

/// Verify the sign-off **attestation** against a configured authority: a
/// confirmed row must carry a valid ed25519 signature, by `authority`, over its
/// canonical tuple. This is the cryptographic half of the truth-admission
/// boundary — the engine holds only the public key, so it cannot mint a passing
/// attestation, and a self-asserted `HumanConfirmed` (no/invalid signature) is
/// refused. Called by [`CorpusStore::submit`](crate::CorpusStore::submit) only
/// when the store was configured with an authority (cold start has none).
pub fn ensure_attested(
    contribution: &Contribution,
    authority: &SignOffPublicKey,
) -> Result<(), GateError> {
    let attestation = contribution
        .attestation
        .as_ref()
        .ok_or(GateError::AttestationMissing)?;
    let signature =
        SignOffSignature::from_hex(&attestation.signature).ok_or(GateError::AttestationInvalid)?;
    if authority.verify(&attestation_message(contribution), &signature) {
        Ok(())
    } else {
        Err(GateError::AttestationInvalid)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema::{Contribution, Outcome, RowProvenance};
    use common::{ConfigClass, FaultSignature, Plan, PlanStep, Symptom, Verification};

    fn config_class() -> ConfigClass {
        ConfigClass::from_inventory(["os:windows 11"])
    }

    fn plan_with(risk: Risk) -> Plan {
        let mut plan = Plan::new("p1", "fix");
        plan.steps.push(PlanStep {
            description: "do".into(),
            // A real tool name — the validating de-id mint rejects out-of-vocabulary
            // actions, and these tests exercise the gate, not the action vocabulary.
            action: "cim_query".into(),
            risk,
        });
        plan
    }

    fn contribution(
        label: OutcomeLabel,
        sign_off: SignOff,
        risk: Risk,
        verification: Option<Verification>,
    ) -> Contribution {
        Contribution::new(
            Outcome {
                signature: FaultSignature::from_symptoms(vec![Symptom("event_41".into())]),
                plan: plan_with(risk),
                label,
                verification,
            },
            config_class(),
            sign_off,
        )
        .expect("test contribution de-identifies")
    }

    #[test]
    fn unconfirmed_is_refused() {
        let c = contribution(
            OutcomeLabel::ResolvedConfirmed,
            SignOff::Unconfirmed,
            Risk::Reversible,
            Some(Verification::pass()),
        );
        assert_eq!(ensure_evidence_integrity(&c), Err(GateError::Unconfirmed));
    }

    #[test]
    fn resolved_confirmed_needs_a_pass_verdict() {
        // Missing verdict.
        let missing = contribution(
            OutcomeLabel::ResolvedConfirmed,
            SignOff::HumanConfirmed,
            Risk::Reversible,
            None,
        );
        assert_eq!(
            ensure_evidence_integrity(&missing),
            Err(GateError::ResolvedWithoutPass)
        );
        // A failing verdict under a resolved label.
        let failing = contribution(
            OutcomeLabel::ResolvedConfirmed,
            SignOff::HumanConfirmed,
            Risk::Reversible,
            Some(Verification {
                result: VerificationResult::Fail,
                class: None,
                recurring: vec![Symptom("event_41".into())],
            }),
        );
        assert_eq!(
            ensure_evidence_integrity(&failing),
            Err(GateError::ResolvedWithoutPass)
        );
    }

    #[test]
    fn confirmed_label_over_a_provisional_pass_is_a_mismatch() {
        let c = contribution(
            OutcomeLabel::ResolvedConfirmed,
            SignOff::HumanConfirmed,
            Risk::Reversible,
            Some(Verification::provisional()),
        );
        assert_eq!(
            ensure_evidence_integrity(&c),
            Err(GateError::LabelVerdictMismatch)
        );
    }

    #[test]
    fn a_matching_resolved_outcome_passes() {
        let confirmed = contribution(
            OutcomeLabel::ResolvedConfirmed,
            SignOff::VerifierConfirmed,
            Risk::Reversible,
            Some(Verification::pass()),
        );
        assert!(ensure_evidence_integrity(&confirmed).is_ok());
        let provisional = contribution(
            OutcomeLabel::ResolvedProvisional,
            SignOff::VerifierConfirmed,
            Risk::Reversible,
            Some(Verification::provisional()),
        );
        assert!(ensure_evidence_integrity(&provisional).is_ok());
    }

    #[test]
    fn a_destructive_fix_needs_human_sign_off() {
        let verifier = contribution(
            OutcomeLabel::ResolvedConfirmed,
            SignOff::VerifierConfirmed,
            Risk::Destructive,
            Some(Verification::pass()),
        );
        assert_eq!(
            ensure_evidence_integrity(&verifier),
            Err(GateError::DestructiveFixNeedsHuman)
        );
        let human = contribution(
            OutcomeLabel::ResolvedConfirmed,
            SignOff::HumanConfirmed,
            Risk::Destructive,
            Some(Verification::pass()),
        );
        assert!(ensure_evidence_integrity(&human).is_ok());
    }

    #[test]
    fn provisional_label_over_a_full_pass_is_a_mismatch() {
        // The mirror of `confirmed_label_over_a_provisional_pass_is_a_mismatch`:
        // a provisional label backed by a *deterministic* Pass is also a verdict
        // that disagrees with its label, not a match.
        let c = contribution(
            OutcomeLabel::ResolvedProvisional,
            SignOff::HumanConfirmed,
            Risk::Reversible,
            Some(Verification::pass()),
        );
        assert_eq!(
            ensure_evidence_integrity(&c),
            Err(GateError::LabelVerdictMismatch)
        );
    }

    #[test]
    fn an_unverified_or_offmachine_verdict_cannot_back_a_resolved_label() {
        // A resolved label needs a *passing* verdict; Unverified (no real
        // re-collection) and OffMachine (the verdict belongs to the bench) are
        // not passes, so they fall through to ResolvedWithoutPass rather than
        // being silently admitted.
        for result in [
            VerificationResult::Unverified,
            VerificationResult::OffMachine,
            VerificationResult::Fail,
        ] {
            let c = contribution(
                OutcomeLabel::ResolvedConfirmed,
                SignOff::HumanConfirmed,
                Risk::Reversible,
                Some(Verification {
                    result,
                    class: None,
                    recurring: Vec::new(),
                }),
            );
            assert_eq!(
                ensure_evidence_integrity(&c),
                Err(GateError::ResolvedWithoutPass),
                "{result:?} must not back a resolved label"
            );
        }
    }

    #[test]
    fn hard_negatives_are_admitted_without_a_verdict() {
        // A failure is truth too: it needs only a confirmed sign-off, no verdict,
        // and is unaffected by the destructive-fix rule (it is not a fix).
        let negative = contribution(
            OutcomeLabel::EscalatedHumanUnresolved,
            SignOff::VerifierConfirmed,
            Risk::Destructive,
            None,
        );
        assert!(ensure_evidence_integrity(&negative).is_ok());
    }

    // --- 1f write-gate idempotence: the stored plan must be its own de-id image.
    //     `Contribution::new` guarantees this, but a row loaded from /mnt/e or
    //     handed in by an embedder bypasses the constructor — the gate re-runs the
    //     mint and refuses content that is not already extraction-clean. These
    //     forge the stored plan by struct literal (only possible in-crate) to
    //     simulate exactly that off-constructor path. Proven red-on-revert.

    use crate::stored::{StoredPlan, StoredStep};

    #[test]
    fn a_row_with_an_out_of_vocab_action_is_refused_by_the_write_gate() {
        // A hand-edited at-rest row whose stored step action is request prose the
        // action mint would never emit. de_identify_plan returns Err → refused.
        let mut c = contribution(
            OutcomeLabel::ResolvedConfirmed,
            SignOff::HumanConfirmed,
            Risk::Reversible,
            Some(Verification::pass()),
        );
        c.outcome.plan = StoredPlan {
            id: "p1".into(),
            title: "cim_query".into(),
            steps: vec![StoredStep {
                description: "powershell on DESKTOP-NATHAN01".into(),
                action: "powershell -c whoami on DESKTOP-NATHAN01".into(),
                risk: Risk::ReadOnly,
            }],
        };
        assert_eq!(
            ensure_evidence_integrity(&c),
            Err(GateError::RowNotDeIdentified)
        );
    }

    #[test]
    fn a_row_with_a_hand_edited_title_fails_the_idempotence_check() {
        // Every action is clean vocabulary, but the stored title is prose the
        // mint would never produce (the mint reconstructs it as the joined
        // actions). The re-mint succeeds yet does not EQUAL the stored plan, so
        // the idempotence equality — not the mint — catches the tamper.
        let mut c = contribution(
            OutcomeLabel::ResolvedConfirmed,
            SignOff::HumanConfirmed,
            Risk::Reversible,
            Some(Verification::pass()),
        );
        c.outcome.plan = StoredPlan {
            id: "p1".into(),
            title: "Fix DESKTOP-NATHAN01 for nathan".into(), // != "cim_query"
            steps: vec![StoredStep {
                description: "cim_query".into(),
                action: "cim_query".into(),
                risk: Risk::ReadOnly,
            }],
        };
        assert_eq!(
            ensure_evidence_integrity(&c),
            Err(GateError::RowNotDeIdentified)
        );
    }

    #[test]
    fn a_genuinely_de_identified_row_passes_the_idempotence_check() {
        // The positive control: a row built through Contribution::new IS its own
        // de-id image, so the new content check does not regress admission.
        let c = contribution(
            OutcomeLabel::ResolvedConfirmed,
            SignOff::HumanConfirmed,
            Risk::Reversible,
            Some(Verification::pass()),
        );
        assert!(ensure_evidence_integrity(&c).is_ok());
    }

    // --- #4: the label's part_class and the provenance run_id are validated
    //     (they ride onto the row unmodified; part_class egresses to the API
    //     wire). part_class is minted at Contribution::new; run_id is bounded at
    //     the write gate — both additionally refused at deserialize (see store.rs).

    #[test]
    fn a_poisoned_part_class_is_refused_at_construction() {
        // A hardware label carrying identity/prose in part_class must be refused
        // where the plan's de-id mint refuses one — at construction — not copied
        // onto the row and out to `wire_label`.
        let poisoned = Contribution::new(
            Outcome {
                signature: FaultSignature::from_symptoms(vec![Symptom("event_41".into())]),
                plan: plan_with(Risk::ReadOnly),
                label: OutcomeLabel::EscalatedHardware {
                    part_class: "psu on DESKTOP-NATHAN01 for nathan@cec.direct".into(),
                },
                verification: None,
            },
            config_class(),
            SignOff::HumanConfirmed,
        );
        let reject = poisoned.expect_err("a poisoned part_class must be refused at construction");
        assert_eq!(reject.field, "part_class");

        // The frozen taxonomy vocabulary still admits.
        let clean = Contribution::new(
            Outcome {
                signature: FaultSignature::from_symptoms(vec![Symptom("event_41".into())]),
                plan: plan_with(Risk::ReadOnly),
                label: OutcomeLabel::EscalatedHardware {
                    part_class: "psu".into(),
                },
                verification: None,
            },
            config_class(),
            SignOff::HumanConfirmed,
        );
        assert!(
            clean.is_ok(),
            "a taxonomy part_class must still be admitted"
        );
    }

    #[test]
    fn a_poisoned_run_id_is_refused_by_the_write_gate() {
        // A free-text/identity run_id on the provenance pin is bounded to an
        // opaque token at the write boundary.
        let c = contribution(
            OutcomeLabel::ResolvedConfirmed,
            SignOff::HumanConfirmed,
            Risk::Reversible,
            Some(Verification::pass()),
        )
        .with_provenance(RowProvenance {
            run_id: "nathan@cec.direct C:/Users/nathan".into(),
            retrieval_first: false,
            primed_from: Vec::new(),
        });
        assert_eq!(
            ensure_evidence_integrity(&c),
            Err(GateError::RunIdNotDeIdentified)
        );

        // A genuine opaque run id still admits.
        let clean = contribution(
            OutcomeLabel::ResolvedConfirmed,
            SignOff::HumanConfirmed,
            Risk::Reversible,
            Some(Verification::pass()),
        )
        .with_provenance(RowProvenance {
            run_id: "a1b2c3d4e5f60718293a4b5c6d7e8f90".into(),
            retrieval_first: false,
            primed_from: Vec::new(),
        });
        assert!(ensure_evidence_integrity(&clean).is_ok());
    }

    // --- #7: the write-gate symptom guard (SymptomNotDeIdentified) bites through
    //     the blessed construct+submit path. Symptom is a public tuple struct and
    //     Contribution::new wraps the caller's FaultSignature without validating
    //     its tokens, so this gate loop is the only write-path symptom guard —
    //     drive an identity-shaped token through it and assert BOTH halves fire.

    #[test]
    fn a_non_de_identified_symptom_is_refused_by_the_write_gate() {
        // (a) A poisoned token in the fault SIGNATURE (the first half of the loop).
        let in_signature = Contribution::new(
            Outcome {
                signature: FaultSignature::from_symptoms(vec![Symptom("DESKTOP-NATHAN01".into())]),
                plan: plan_with(Risk::ReadOnly),
                label: OutcomeLabel::EscalatedHumanUnresolved,
                verification: None,
            },
            config_class(),
            SignOff::HumanConfirmed,
        )
        .expect("construction wraps the symptom without validating it");
        assert_eq!(
            ensure_evidence_integrity(&in_signature),
            Err(GateError::SymptomNotDeIdentified)
        );

        // (b) A poisoned token in the verification's RECURRING set, over a CLEAN
        //     signature — so only the second half of the loop can catch it.
        let in_recurring = Contribution::new(
            Outcome {
                signature: FaultSignature::from_symptoms(vec![Symptom("event_41".into())]),
                plan: plan_with(Risk::ReadOnly),
                label: OutcomeLabel::EscalatedHumanUnresolved,
                verification: Some(Verification {
                    result: VerificationResult::Fail,
                    class: None,
                    recurring: vec![Symptom("nathan@cec.direct".into())],
                }),
            },
            config_class(),
            SignOff::HumanConfirmed,
        )
        .expect("construction wraps the recurring symptom without validating it");
        assert_eq!(
            ensure_evidence_integrity(&in_recurring),
            Err(GateError::SymptomNotDeIdentified)
        );
    }
}
