use common::{FaultSignature, Symptom};
use serde::{Deserialize, Serialize};

/// How an outcome for this fault class can be verified.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VerificationClass {
    /// The fault reproduces deterministically: re-run the collection and the
    /// verdict is pass or fail, now.
    Deterministic,
    /// The fault is intermittent: absence of evidence over minutes is not a
    /// fix, so a clean re-collection earns only a provisional pass under a
    /// monitoring horizon with auto-reopen.
    Intermittent,
    /// The fault is hardware-evidenced: verification is the bench or RMA
    /// outcome, not a machine-side check.
    Hardware,
}

/// The verification verdict for an executed plan.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Verdict {
    /// The original failure signature is gone and the class is deterministic.
    Pass,
    /// The signature is gone but the class is intermittent: monitored over a
    /// horizon, auto-reopened on recurrence.
    ProvisionalPass,
    /// The original signature (or part of it) recurred. The recurring
    /// symptoms are the post-state diff — they enter the retry context as a
    /// hard negative, because a retry that does not know what failed is a
    /// coin flip.
    Fail {
        /// The original symptoms still present after execution.
        recurring: Vec<Symptom>,
    },
    /// Hardware class: the verdict belongs to the bench or the RMA, not to a
    /// machine-side diff.
    OffMachine,
}

/// Verify an outcome by diffing the re-collected signature against the
/// original failure signature.
///
/// The claim "fixed" is only valid against the same instrument that
/// established "broken": `post` must come from re-running the same targeted
/// collection that produced `original`.
pub fn verify_outcome(
    original: &FaultSignature,
    post: &FaultSignature,
    class: VerificationClass,
) -> Verdict {
    if class == VerificationClass::Hardware {
        return Verdict::OffMachine;
    }
    let recurring = original.recurring_in(post);
    if !recurring.is_empty() {
        return Verdict::Fail { recurring };
    }
    match class {
        VerificationClass::Deterministic => Verdict::Pass,
        VerificationClass::Intermittent => Verdict::ProvisionalPass,
        VerificationClass::Hardware => unreachable!("handled above"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn signature(symptoms: &[&str]) -> FaultSignature {
        FaultSignature::from_symptoms(symptoms.iter().map(|s| Symptom(s.to_string())).collect())
    }

    #[test]
    fn a_clean_recollection_passes_deterministic_faults() {
        let original = signature(&["crash", "event_41"]);
        let post = signature(&[]);
        assert_eq!(
            verify_outcome(&original, &post, VerificationClass::Deterministic),
            Verdict::Pass
        );
    }

    #[test]
    fn a_clean_recollection_only_paroles_intermittent_faults() {
        let original = signature(&["freeze"]);
        let post = signature(&[]);
        assert_eq!(
            verify_outcome(&original, &post, VerificationClass::Intermittent),
            Verdict::ProvisionalPass
        );
    }

    #[test]
    fn recurrence_fails_and_names_the_recurring_symptoms() {
        let original = signature(&["crash", "event_41"]);
        let post = signature(&["event_41", "reboot"]);
        match verify_outcome(&original, &post, VerificationClass::Deterministic) {
            Verdict::Fail { recurring } => {
                assert_eq!(recurring, vec![Symptom("event_41".into())]);
            }
            other => panic!("expected Fail, got {other:?}"),
        }
    }

    #[test]
    fn hardware_verification_is_off_machine_even_when_clean() {
        let original = signature(&["whea"]);
        let post = signature(&[]);
        assert_eq!(
            verify_outcome(&original, &post, VerificationClass::Hardware),
            Verdict::OffMachine
        );
    }
}
