use common::{FaultSignature, Symptom};
use serde::{Deserialize, Serialize};

// The verification class vocabulary now lives in `common` (it is recorded on a
// corpus row), re-exported here so existing `agent_core::verify::VerificationClass`
// / `agent_core::VerificationClass` paths keep working.
pub use common::VerificationClass;

/// The verification verdict for an executed plan.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Verdict {
    /// The original failure signature is gone and the class is deterministic.
    Pass,
    /// The signature is gone but the class is intermittent: monitored over a
    /// horizon, auto-reopened on recurrence.
    ProvisionalPass,
    /// SOME original symptoms cleared, some remain, and the fix introduced no
    /// new ones — a beneficial-but-incomplete outcome (partial resolution).
    /// `cleared` is the proven benefit (attributable to this single signed
    /// plan); `remaining` is what is left and enters the retry context as
    /// PROGRESS — the next attempt works on the remainder, not from scratch.
    PartialPass {
        /// Original symptoms that cleared — the fix's proven benefit.
        cleared: Vec<Symptom>,
        /// Original symptoms still present — the remainder to keep working.
        remaining: Vec<Symptom>,
    },
    /// The fix INTRODUCED symptoms that were not present before (it may also
    /// have cleared some). Trading one problem for another is never autonomous
    /// credit — this escalates to a human.
    Regressed {
        /// Original symptoms that cleared (if any) — recorded, not credited.
        cleared: Vec<Symptom>,
        /// Symptoms present after that were not before — the regression.
        introduced: Vec<Symptom>,
    },
    /// The original signature (or part of it) recurred with nothing cleared. The
    /// recurring symptoms are the post-state diff — they enter the retry context
    /// as a hard negative, because a retry that does not know what failed is a
    /// coin flip.
    Fail {
        /// The original symptoms still present after execution.
        recurring: Vec<Symptom>,
    },
    /// Hardware class: the verdict belongs to the bench or the RMA, not to a
    /// machine-side diff.
    OffMachine,
    /// No independent re-collection was available to diff against the original,
    /// so the outcome could not be verified either way. It escalates for human
    /// review and can never back a resolved label — this is what a run gets when
    /// the post-fix state was never actually observed (the bootstrap echo).
    Unverified,
}

impl Verdict {
    /// The de-identified verification record this verdict contributes to a
    /// corpus row, under the class it was judged in, so a resolved outcome can
    /// be audited against — and gated on — the evidence that justified it. The
    /// recurring symptoms (a `Fail`'s post-state diff) are vocabulary terms, so
    /// this carries evidence, never identity.
    pub fn to_verification(&self, class: VerificationClass) -> common::Verification {
        use common::VerificationResult as R;
        // (result, recurring, cleared, introduced)
        let (result, recurring, cleared, introduced) = match self {
            Verdict::Pass => (R::Pass, Vec::new(), Vec::new(), Vec::new()),
            Verdict::ProvisionalPass => (R::ProvisionalPass, Vec::new(), Vec::new(), Vec::new()),
            Verdict::PartialPass { cleared, remaining } => (
                R::PartialPass,
                remaining.clone(),
                cleared.clone(),
                Vec::new(),
            ),
            Verdict::Regressed {
                cleared,
                introduced,
            } => (
                R::Regressed,
                Vec::new(),
                cleared.clone(),
                introduced.clone(),
            ),
            Verdict::Fail { recurring } => (R::Fail, recurring.clone(), Vec::new(), Vec::new()),
            Verdict::OffMachine => (R::OffMachine, Vec::new(), Vec::new(), Vec::new()),
            Verdict::Unverified => (R::Unverified, Vec::new(), Vec::new(), Vec::new()),
        };
        common::Verification {
            result,
            class: Some(class),
            recurring,
            cleared,
            introduced,
        }
    }
}

/// Verify an outcome by diffing the re-collected signature against the
/// original failure signature.
///
/// The claim "fixed" is only valid against the same instrument that established
/// "broken": `post` must come from re-running the same targeted collection that
/// produced `original`. `post` is therefore an `Option`: `None` means **no
/// independent re-collection was available** (e.g. the bootstrap collector only
/// re-reads the request text, which is not an observation of the post-fix
/// state) — that yields [`Verdict::Unverified`], never a pass, so a run that
/// never actually observed the machine afterwards cannot be recorded as
/// resolved. A hardware-class outcome is always `OffMachine` (the verdict
/// belongs to the bench), with or without a re-collection.
pub fn verify_outcome(
    original: &FaultSignature,
    post: Option<&FaultSignature>,
    class: VerificationClass,
) -> Verdict {
    if class == VerificationClass::Hardware {
        return Verdict::OffMachine;
    }
    let Some(post) = post else {
        return Verdict::Unverified;
    };
    // Reason ONLY about the original fault's symptoms — did they clear? Post-fix
    // re-collections routinely surface benign, incidental vocabulary (a `reboot`
    // log, a `boot` entry) that has nothing to do with the fault; treating any
    // post-only symptom as a regression would fire false alarms on ordinary
    // noise. Detecting a genuine regression (a NEW fault the fix caused, not log
    // noise) needs a fault-vs-noise signal the naive diff does not have — that is
    // the collector's job and is deferred (see `OutcomeLabel::Regressed`, a
    // recordable outcome, and FOLLOWUPS). So this autonomous verifier partitions
    // the ORIGINAL symptoms into cleared and remaining, and nothing more.
    let remaining = original.recurring_in(post); // originals still there
    let cleared = original.cleared_in(post); // originals now gone (the benefit)

    if !remaining.is_empty() {
        // Some originals remain. If NONE cleared, nothing improved → a full
        // Fail (the hard negative). If SOME cleared, it is a partial
        // resolution: a proven benefit plus a remainder to keep working.
        return if cleared.is_empty() {
            Verdict::Fail {
                recurring: remaining,
            }
        } else {
            Verdict::PartialPass { cleared, remaining }
        };
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
            verify_outcome(&original, Some(&post), VerificationClass::Deterministic),
            Verdict::Pass
        );
    }

    #[test]
    fn a_clean_recollection_only_paroles_intermittent_faults() {
        let original = signature(&["freeze"]);
        let post = signature(&[]);
        assert_eq!(
            verify_outcome(&original, Some(&post), VerificationClass::Intermittent),
            Verdict::ProvisionalPass
        );
    }

    #[test]
    fn recurrence_with_nothing_cleared_is_a_full_fail() {
        // All originals recur and nothing cleared → a plain Fail (hard
        // negative), naming the recurring symptoms for the retry context.
        let original = signature(&["crash", "event_41"]);
        let post = signature(&["crash", "event_41"]);
        match verify_outcome(&original, Some(&post), VerificationClass::Deterministic) {
            Verdict::Fail { recurring } => {
                assert_eq!(
                    recurring,
                    vec![Symptom("crash".into()), Symptom("event_41".into())]
                );
            }
            other => panic!("expected Fail, got {other:?}"),
        }
    }

    #[test]
    fn some_cleared_some_remaining_is_a_partial_pass() {
        // `crash` cleared, `event_41` remains, nothing new → a partial
        // resolution: a proven benefit (cleared) plus a remainder to keep working.
        let original = signature(&["crash", "event_41"]);
        let post = signature(&["event_41"]);
        match verify_outcome(&original, Some(&post), VerificationClass::Deterministic) {
            Verdict::PartialPass { cleared, remaining } => {
                assert_eq!(cleared, vec![Symptom("crash".into())], "the proven benefit");
                assert_eq!(remaining, vec![Symptom("event_41".into())], "the remainder");
            }
            other => panic!("expected PartialPass, got {other:?}"),
        }
    }

    #[test]
    fn a_post_only_symptom_is_ignored_not_treated_as_a_regression() {
        // A post-fix re-collection carrying a symptom that was NOT in the
        // original fault (`whea` here — could equally be a benign `reboot` log)
        // must NOT be flagged as a regression by the naive diff: the autonomous
        // verifier judges only the ORIGINAL symptoms. Here `crash` cleared and
        // `event_41` remains → a partial pass; the post-only `whea` is ignored.
        let original = signature(&["crash", "event_41"]);
        let post = signature(&["event_41", "whea"]);
        match verify_outcome(&original, Some(&post), VerificationClass::Deterministic) {
            Verdict::PartialPass { cleared, remaining } => {
                assert_eq!(cleared, vec![Symptom("crash".into())]);
                assert_eq!(remaining, vec![Symptom("event_41".into())]);
            }
            other => panic!("expected PartialPass (post-only symptom ignored), got {other:?}"),
        }
    }

    #[test]
    fn hardware_verification_is_off_machine_even_when_clean() {
        let original = signature(&["whea"]);
        let post = signature(&[]);
        assert_eq!(
            verify_outcome(&original, Some(&post), VerificationClass::Hardware),
            Verdict::OffMachine
        );
    }

    #[test]
    fn no_recollection_is_unverified_not_a_pass() {
        // The bootstrap case: nothing was observed after execution, so the
        // outcome cannot be confirmed — it must escalate, never resolve.
        let original = signature(&["crash"]);
        assert_eq!(
            verify_outcome(&original, None, VerificationClass::Deterministic),
            Verdict::Unverified
        );
        assert_eq!(
            verify_outcome(&original, None, VerificationClass::Intermittent),
            Verdict::Unverified
        );
        // Hardware is still off-machine regardless.
        assert_eq!(
            verify_outcome(&original, None, VerificationClass::Hardware),
            Verdict::OffMachine
        );
    }
}
