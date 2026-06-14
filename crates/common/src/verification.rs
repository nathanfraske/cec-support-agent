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
    /// The original signature (or part of it) recurred after execution.
    Fail,
    /// Hardware class: the verdict belongs to the bench or RMA, not a
    /// machine-side diff.
    OffMachine,
}

impl VerificationResult {
    /// Whether this verdict counts as a passing verification (the only verdicts
    /// that may back a resolved outcome).
    pub fn is_pass(self) -> bool {
        matches!(
            self,
            VerificationResult::Pass | VerificationResult::ProvisionalPass
        )
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
    /// Original symptoms still present after execution — the post-state diff.
    /// Empty unless `result` is `Fail`. Symptoms are vocabulary terms, so this
    /// carries evidence, never identity.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub recurring: Vec<Symptom>,
}

impl Verification {
    /// A passing verdict with no recurring symptoms and no recorded class.
    pub fn pass() -> Self {
        Self {
            result: VerificationResult::Pass,
            class: None,
            recurring: Vec::new(),
        }
    }

    /// A provisional-pass verdict with no recurring symptoms and no class.
    pub fn provisional() -> Self {
        Self {
            result: VerificationResult::ProvisionalPass,
            class: None,
            recurring: Vec::new(),
        }
    }
}
