use common::{FaultSignature, Plan};
use serde::{Deserialize, Serialize};

/// Whether an outcome has cleared the sign-off gate.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SignOff {
    /// Not yet confirmed. Contributions in this state are rejected on submit.
    #[default]
    Unconfirmed,
    /// Confirmed by an automated verifier.
    VerifierConfirmed,
    /// Confirmed by a human operator.
    HumanConfirmed,
}

impl SignOff {
    /// Whether this sign-off clears the gate.
    pub fn is_confirmed(self) -> bool {
        matches!(self, SignOff::VerifierConfirmed | SignOff::HumanConfirmed)
    }
}

/// A fault-signature → plan mapping retrieved from the corpus.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FixMapping {
    /// The fault this mapping addresses.
    pub signature: FaultSignature,
    /// The plan known to resolve it.
    pub plan: Plan,
    /// How many confirmed outcomes back this mapping.
    pub confirmations: u32,
}

/// The result of executing a plan against a fault.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Outcome {
    /// The fault that was diagnosed.
    pub signature: FaultSignature,
    /// The plan that was executed.
    pub plan: Plan,
    /// Whether the plan resolved the fault.
    pub resolved: bool,
}

/// A de-identified outcome proposed for inclusion in the corpus.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Contribution {
    /// The outcome being contributed.
    pub outcome: Outcome,
    /// Sign-off state. Must be confirmed for [`crate::CorpusStore::submit`] to
    /// accept it.
    pub sign_off: SignOff,
}

impl Contribution {
    /// Build a contribution from an outcome and its sign-off state.
    pub fn new(outcome: Outcome, sign_off: SignOff) -> Self {
        Self { outcome, sign_off }
    }
}
