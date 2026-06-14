use common::{Risk, VerificationResult};
use provenance::{SignOffPublicKey, SignOffSignature};
use thiserror::Error;

use crate::schema::{attestation_message, Contribution, OutcomeLabel, SignOff};

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
}

/// The evidence-integrity gate: the single checkpoint that admits a row into the
/// inverted corpus. Returns `Ok(())` only when the contribution is sign-off
/// confirmed AND — for a resolved outcome (the rows that become retrievable
/// fixes) — its truth claim is bound to its evidence:
///
/// 1. **Sign-off confirmed** (invariant 6): verifier or human.
/// 2. **Resolved ⇒ passing verdict** that *matches* the label: `ResolvedConfirmed`
///    needs a `Pass`, `ResolvedProvisional` a `ProvisionalPass`.
/// 3. **Destructive resolved fix ⇒ human sign-off.**
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

    let outcome = &contribution.outcome;
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
    use crate::schema::{Contribution, Outcome};
    use common::{ConfigClass, FaultSignature, Plan, PlanStep, Symptom, Verification};

    fn config_class() -> ConfigClass {
        ConfigClass::from_inventory(["os:windows 11"])
    }

    fn plan_with(risk: Risk) -> Plan {
        let mut plan = Plan::new("p1", "fix");
        plan.steps.push(PlanStep {
            description: "do".into(),
            action: "act".into(),
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
                signature: FaultSignature::from_symptoms(vec![Symptom("boot_loop".into())]),
                plan: plan_with(risk),
                label,
                verification,
            },
            config_class(),
            sign_off,
        )
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
}
