use common::{ConfigClass, FaultSignature, Plan};
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

/// The label sign-off binds to a ticket. Sign-off is the labeling event: it
/// always emits a label, including for unresolved and hardware-escalated
/// tickets, because an unlabeled ticket is corpus poison. A corpus of plans
/// without disciplined outcomes is a pile of plausible scripts.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OutcomeLabel {
    /// Verification passed against the original failure signature.
    ResolvedConfirmed,
    /// Provisional pass: the fault class is intermittent, so the ticket is
    /// monitored over a horizon and auto-reopens on signature recurrence.
    ResolvedProvisional,
    /// The signature recurred inside the monitoring horizon.
    Reopened,
    /// The evidence names a part; the deliverable is a diagnosis plus an RMA
    /// or bench action. An evidence-backed hardware verdict is a successful
    /// outcome, not a failed one.
    EscalatedHardware {
        /// The implicated part class (e.g. "psu", "storage", "memory").
        part_class: String,
    },
    /// Escalated to a human and not resolved by the pipeline.
    EscalatedHumanUnresolved,
    /// The customer withdrew the ticket.
    Withdrawn,
}

impl OutcomeLabel {
    /// Whether this label counts as a fix for retrieval purposes. Every other
    /// label still enters the corpus — a failure is a hard negative, not a
    /// discard — but only resolved outcomes back a [`FixMapping`].
    pub fn is_resolved(&self) -> bool {
        matches!(
            self,
            OutcomeLabel::ResolvedConfirmed | OutcomeLabel::ResolvedProvisional
        )
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
    /// The label emitted at sign-off.
    pub label: OutcomeLabel,
}

/// A de-identified outcome proposed for inclusion in the corpus: the
/// (signature, plan, outcome) triple plus its context.
///
/// [`Contribution::new`] de-identifies the plan by structured extraction
/// (see [`de_identify_plan`]) — free-text fields never reach a corpus row,
/// in code, regardless of what the caller passes in.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Contribution {
    /// The outcome being contributed.
    pub outcome: Outcome,
    /// The comparability key: a contribution is only retrieved for tickets of
    /// the same config class.
    pub config_class: ConfigClass,
    /// Sign-off state. Must be confirmed for [`crate::CorpusStore::submit`] to
    /// accept it.
    pub sign_off: SignOff,
}

impl Contribution {
    /// Build a contribution from an outcome, its config class, and its
    /// sign-off state. The outcome's plan is de-identified here, so a raw
    /// plan cannot enter a contribution at all.
    pub fn new(mut outcome: Outcome, config_class: ConfigClass, sign_off: SignOff) -> Self {
        outcome.plan = de_identify_plan(&outcome.plan);
        Self {
            outcome,
            config_class,
            sign_off,
        }
    }
}

/// De-identify a plan by structured extraction: keep only the vocabulary
/// fields (id, ordered actions, per-step risk) and drop every free-text field.
/// Step descriptions and the plan title are where model output and request
/// prose — and therefore hostnames, usernames, and paths — can hide; the
/// corpus row carries the action vocabulary instead.
pub fn de_identify_plan(plan: &Plan) -> Plan {
    let mut row = Plan::new(
        plan.id.clone(),
        plan.steps
            .iter()
            .map(|step| step.action.as_str())
            .collect::<Vec<_>>()
            .join(" -> "),
    );
    row.steps = plan
        .steps
        .iter()
        .map(|step| common::PlanStep {
            description: step.action.clone(),
            action: step.action.clone(),
            risk: step.risk,
        })
        .collect();
    row
}
