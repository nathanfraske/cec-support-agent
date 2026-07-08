use serde::{Deserialize, Serialize};

use crate::Symptom;

/// How an outcome for a fault class can be verified. Decided before execution
/// from the route and the reproducibility the user reported; recorded on the
/// corpus row so a resolved outcome can be audited against the instrument that
/// judged it (an intermittent fault is paroled, not confirmed).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VerificationClass {
    /// Deterministic: re-run the collection and the verdict is pass/fail, now.
    Deterministic,
    /// Intermittent: a clean re-collection earns only a provisional pass under a
    /// monitoring horizon with auto-reopen.
    Intermittent,
    /// Hardware-evidenced: verification is the bench/RMA outcome, not a
    /// machine-side check.
    Hardware,
}

/// The verifier's verdict for an executed plan, as a de-identified record.
///
/// This mirrors the result of `agent-core`'s `verify_outcome` (a diff of the
/// re-collected signature against the original) but carries no free text — only
/// the verdict kind and, on a failure, the recurring vocabulary symptoms that
/// were the post-state diff. Carrying it on the corpus row is what lets a
/// "resolved" outcome be audited against the evidence that justified it, and
/// lets the sign-off gate refuse a resolved label with no passing verdict
/// behind it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VerificationResult {
    /// The original failure signature is gone and the class is deterministic.
    Pass,
    /// The signature is gone but the class is intermittent: a provisional pass
    /// under a monitoring horizon with auto-reopen.
    ProvisionalPass,
    /// Some — but not all — of the original symptoms cleared, and the fix
    /// introduced no new ones: a beneficial-but-incomplete outcome (partial
    /// resolution). The `cleared` set is the proven benefit; `recurring` is what
    /// is left. Backs a `ResolvedPartial` label — an improvement, not a fix.
    PartialPass,
    /// The fix INTRODUCED symptoms that were not present before (it may also
    /// have cleared some). Trading one problem for another is never autonomous
    /// credit: this escalates to a human. `introduced` names the new symptoms.
    Regressed,
    /// The original signature (or part of it) recurred after execution.
    Fail,
    /// Hardware class: the verdict belongs to the bench or RMA, not a
    /// machine-side diff.
    OffMachine,
    /// No independent re-collection was available, so the outcome could not be
    /// verified either way (e.g. the bootstrap collector only re-reads the
    /// request text — not an observation of the post-fix state). An unverified
    /// outcome can never back a resolved label; it escalates for human review.
    Unverified,
}

impl VerificationResult {
    /// Whether this verdict counts as a FULL passing verification (the only
    /// verdicts that may back a fully-resolved outcome). A `PartialPass` is a
    /// beneficial improvement but NOT a full pass.
    pub fn is_pass(self) -> bool {
        matches!(
            self,
            VerificationResult::Pass | VerificationResult::ProvisionalPass
        )
    }

    /// Whether this verdict recorded a proven benefit — some original symptoms
    /// cleared, attributable to the fix. A full pass and a partial pass are both
    /// beneficial; a partial pass is beneficial without being a full pass.
    pub fn is_beneficial(self) -> bool {
        self.is_pass() || matches!(self, VerificationResult::PartialPass)
    }
}

/// A de-identified verification record bound to a corpus outcome.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Verification {
    /// The verdict kind.
    pub result: VerificationResult,
    /// The class the outcome was judged under (so a `ResolvedProvisional` is
    /// visibly an intermittent parole, not a deterministic confirmation).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub class: Option<VerificationClass>,
    /// Original symptoms still present after execution — what is left. Empty on
    /// a full `Pass`. On a `PartialPass` these are the symptoms the fix did not
    /// clear; on a `Fail` they are all the originals that recurred. Symptoms are
    /// vocabulary terms, so this carries evidence, never identity.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub recurring: Vec<Symptom>,
    /// Original symptoms that CLEARED after execution — the fix's proven
    /// benefit, attributable to this single signed plan (the pre/post signatures
    /// bracket only this plan). Non-empty on a `PartialPass`; also populated on a
    /// full `Pass` where it equals the whole original set. Vocabulary terms only.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub cleared: Vec<Symptom>,
    /// Symptoms present AFTER execution that were NOT present before — a
    /// regression the fix introduced. Non-empty only on a `Regressed` verdict;
    /// its presence is why that outcome escalates instead of earning credit.
    /// Vocabulary terms only.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub introduced: Vec<Symptom>,
}

impl Verification {
    /// A full passing verdict: the whole original set cleared, nothing left.
    pub fn pass() -> Self {
        Self {
            result: VerificationResult::Pass,
            class: None,
            recurring: Vec::new(),
            cleared: Vec::new(),
            introduced: Vec::new(),
        }
    }

    /// A provisional-pass verdict with no recurring symptoms and no class.
    pub fn provisional() -> Self {
        Self {
            result: VerificationResult::ProvisionalPass,
            class: None,
            recurring: Vec::new(),
            cleared: Vec::new(),
            introduced: Vec::new(),
        }
    }

    /// A partial-resolution verdict: `cleared` is the proven benefit, `recurring`
    /// is what is left. Both non-empty for a real partial.
    pub fn partial(cleared: Vec<Symptom>, recurring: Vec<Symptom>) -> Self {
        Self {
            result: VerificationResult::PartialPass,
            class: None,
            recurring,
            cleared,
            introduced: Vec::new(),
        }
    }

    /// A regression verdict: the fix introduced `introduced` symptoms (and may
    /// have cleared some). Escalates — never autonomous credit.
    pub fn regressed(cleared: Vec<Symptom>, introduced: Vec<Symptom>) -> Self {
        Self {
            result: VerificationResult::Regressed,
            class: None,
            recurring: Vec::new(),
            cleared,
            introduced,
        }
    }
}
