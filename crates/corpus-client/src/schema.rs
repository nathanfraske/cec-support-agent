use common::{ConfigClass, FaultSignature, Plan, Verification};
use serde::{Deserialize, Serialize};

/// Whether an outcome has cleared the sign-off gate.
///
/// Ordered from weakest to strongest (`Unconfirmed` < `VerifierConfirmed` <
/// `HumanConfirmed`), so the gate can require a *minimum* level for a given
/// risk — a destructive fix needs at least `HumanConfirmed`.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Serialize, Deserialize,
)]
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
    /// The verifier's de-identified verdict for this outcome, bound to the row
    /// so a resolved label can be audited against — and gated on — the evidence
    /// that justified it. `None` for outcomes with no machine verification
    /// (e.g. a withdrawn ticket, or a plan that never executed). The sign-off
    /// gate requires a *matching passing* verdict for any resolved label.
    #[serde(default)]
    pub verification: Option<Verification>,
}

/// An attestation that a recognized sign-off authority — NOT the submitting
/// process — performed this sign-off. It is an ed25519 signature over the
/// contribution's canonical tuple ([`attestation_message`]), by a key the
/// engine does not hold. When a store is configured with a
/// [`provenance::SignOffPublicKey`], the gate refuses any confirmed row whose
/// attestation is missing or does not verify — so a self-asserted
/// `HumanConfirmed` cannot be minted by the process that admits rows.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SignOffAttestation {
    /// Short id of the authority public key that signed (diagnostic).
    pub authority_id: String,
    /// Hex-encoded ed25519 signature over the canonical tuple.
    pub signature: String,
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
    /// The sign-off authority's attestation over this row's canonical tuple.
    /// `None` at cold start (no authority configured). When a store carries an
    /// authority public key, this must be present and valid for a confirmed row.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub attestation: Option<SignOffAttestation>,
}

impl Contribution {
    /// Build a contribution from an outcome, its config class, and its
    /// sign-off state. The outcome's plan is de-identified here, so a raw
    /// plan cannot enter a contribution at all. The attestation is unset;
    /// attach one with [`Contribution::attested_by`].
    pub fn new(mut outcome: Outcome, config_class: ConfigClass, sign_off: SignOff) -> Self {
        outcome.plan = de_identify_plan(&outcome.plan);
        Self {
            outcome,
            config_class,
            sign_off,
            attestation: None,
        }
    }

    /// Attach a sign-off authority's attestation over this contribution's
    /// canonical tuple. Call after [`Contribution::new`] (so the attestation
    /// covers the de-identified plan that is actually stored). The authority
    /// holds the private key; the engine that admits the row holds only the
    /// matching public key and re-verifies at the gate.
    pub fn attested_by(mut self, authority: &provenance::SignOffAuthority) -> Self {
        let signature = authority.attest(&attestation_message(&self));
        self.attestation = Some(SignOffAttestation {
            authority_id: authority.public_key().id(),
            signature: signature.to_hex(),
        });
        self
    }
}

/// The canonical bytes a sign-off attestation covers: the contribution's
/// `(signature, plan, label, sign_off, config_class)` tuple in a stable,
/// serde-independent encoding. The attestation field itself is excluded (it
/// signs everything else). Built from the de-identified plan that is stored, so
/// the gate re-derives exactly what the authority signed. Tampering with any
/// covered field changes these bytes and breaks verification.
pub(crate) fn attestation_message(c: &Contribution) -> Vec<u8> {
    use std::fmt::Write as _;
    let o = &c.outcome;
    let mut s = String::from("cec-signoff-attestation-v1\n");
    let _ = writeln!(s, "fp:{}", o.signature.fingerprint);
    let mut symptoms: Vec<&str> = o.signature.symptoms.iter().map(|x| x.0.as_str()).collect();
    symptoms.sort_unstable();
    let _ = writeln!(s, "sym:{}", symptoms.join(","));
    let _ = writeln!(s, "plan:{}", o.plan.id);
    for step in &o.plan.steps {
        let _ = writeln!(s, "step:{}:{:?}", step.action, step.risk);
    }
    let _ = writeln!(s, "label:{}", label_tag(&o.label));
    let _ = writeln!(s, "signoff:{:?}", c.sign_off);
    let _ = writeln!(s, "class:{}", c.config_class.key());
    s.into_bytes()
}

/// A stable tag for an outcome label (its data, not its `Debug` formatting).
fn label_tag(label: &OutcomeLabel) -> String {
    match label {
        OutcomeLabel::ResolvedConfirmed => "resolved_confirmed".into(),
        OutcomeLabel::ResolvedProvisional => "resolved_provisional".into(),
        OutcomeLabel::Reopened => "reopened".into(),
        OutcomeLabel::EscalatedHardware { part_class } => {
            format!("escalated_hardware:{part_class}")
        }
        OutcomeLabel::EscalatedHumanUnresolved => "escalated_human_unresolved".into(),
        OutcomeLabel::Withdrawn => "withdrawn".into(),
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
