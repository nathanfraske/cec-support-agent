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

/// Per-row tamper-evidence: a hash chain linking this row to the previous one
/// in a [`crate::FileCorpus`]. `hash = sha256(prev || canonical-row-without-
/// integrity)`. The store fills this in on write; on open it recomputes the
/// chain and refuses a file where any row has been edited, reordered, or removed
/// mid-stream — so a hand-edited "confirmed" precedent is never served. (The
/// known residual is truncation of the tail, which a hash chain cannot detect
/// without an external anchor; see FOLLOWUPS.)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RowIntegrity {
    /// The previous row's `hash` ("" for the first row).
    pub prev: String,
    /// `sha256(prev || canonical(row-without-integrity))`, hex.
    pub hash: String,
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

/// Run-provenance of a corpus row: which run produced it and how the plan was
/// generated. Lets confirmations be counted only from INDEPENDENT runs — a
/// re-submitted row (same `run_id`) or a row whose plan was corpus-primed from
/// the very mapping it would confirm cannot inflate that mapping's confidence —
/// and lets a confirmation's origin (de-novo vs corpus-primed) be audited. It
/// carries no identity: a `run_id` is an opaque token, plan ids are vocabulary.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RowProvenance {
    /// Opaque id of the run that produced this outcome. Distinct runs are
    /// independent confirmations; the same id is a single observation.
    pub run_id: String,
    /// Whether the plan came from a corpus precedent (retrieval-first) rather
    /// than de-novo generation.
    #[serde(default)]
    pub retrieval_first: bool,
    /// Plan ids of the corpus precedents that primed this run (empty for a
    /// de-novo/control run). A plan primed from itself is not independent
    /// support for itself.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub primed_from: Vec<String>,
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
    /// Run-provenance: which run produced this row and how its plan was
    /// generated. `None` on legacy rows; when present, confirmation counting
    /// uses it to admit only independent confirmations.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provenance: Option<RowProvenance>,
    /// Tamper-evidence chain link, filled in by a [`crate::FileCorpus`] on write.
    /// Not part of the signed/attested tuple (the store adds it after admission).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub integrity: Option<RowIntegrity>,
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
            provenance: None,
            integrity: None,
        }
    }

    /// Record how this row was produced (run id, retrieval-first, primed-from)
    /// so confirmation counting can admit only independent confirmations.
    pub fn with_provenance(mut self, provenance: RowProvenance) -> Self {
        self.provenance = Some(provenance);
        self
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
/// `(signature, plan, label, sign_off, config_class)` tuple AND its
/// run-provenance, in a stable, serde-independent encoding. The attestation
/// field itself is excluded (it signs everything else). Built from the
/// de-identified plan that is stored, so the gate re-derives exactly what the
/// authority signed. Binding the provenance pin (the `run_id` especially) means
/// one valid attestation cannot be replayed onto clones with fabricated run ids
/// to inflate a mapping's independent-confirmation count. Tampering with any
/// covered field changes these bytes and breaks verification.
pub(crate) fn attestation_message(c: &Contribution) -> Vec<u8> {
    use std::fmt::Write as _;

    // Append a length-prefixed `tag[len]=value\n` line. Length-prefixing every
    // attacker-controlled value is what makes the encoding UNAMBIGUOUS: a value
    // can no longer carry the field separators or a newline to forge a different
    // structure with the same signed bytes (a `plan.id` of "p\nstep:rm:Destructive"
    // is byte-distinct from a genuine extra step). Field *names* and counts are
    // fixed literals we emit, never free text, so they need no prefix. This
    // mirrors the discipline in `provenance::canonical`.
    fn lp(s: &mut String, tag: &str, value: &str) {
        let _ = writeln!(s, "{tag}[{}]={value}", value.len());
    }

    let o = &c.outcome;
    let mut s = String::from("cec-signoff-attestation-v3\n");
    // Fault signature: fingerprint + sorted symptoms, each length-prefixed.
    lp(&mut s, "fp", &o.signature.fingerprint);
    let mut symptoms: Vec<&str> = o.signature.symptoms.iter().map(|x| x.0.as_str()).collect();
    symptoms.sort_unstable();
    let _ = writeln!(s, "syms:{}", symptoms.len());
    for sym in &symptoms {
        lp(&mut s, "sym", sym);
    }
    // Plan: id + each step (action length-prefixed, risk by discriminant).
    lp(&mut s, "plan", &o.plan.id);
    let _ = writeln!(s, "steps:{}", o.plan.steps.len());
    for step in &o.plan.steps {
        lp(&mut s, "act", &step.action);
        let _ = writeln!(s, "risk:{:?}", step.risk);
    }
    // Label (length-prefixed tag — covers the EscalatedHardware part_class).
    lp(&mut s, "label", &label_tag(&o.label));
    // Verification verdict — the evidence the gate keys a resolved label on, so
    // the authority must sign the verdict it approved, not just the label. A
    // swapped or fabricated verdict changes these bytes and breaks the signature.
    match &o.verification {
        None => {
            let _ = writeln!(s, "ver:none");
        }
        Some(v) => {
            let mut recurring: Vec<&str> = v.recurring.iter().map(|x| x.0.as_str()).collect();
            recurring.sort_unstable();
            let _ = writeln!(
                s,
                "ver:{:?};class={:?};rec:{}",
                v.result,
                v.class,
                recurring.len()
            );
            for r in &recurring {
                lp(&mut s, "rec", r);
            }
        }
    }
    let _ = writeln!(s, "signoff:{:?}", c.sign_off);
    // Config class — bind the VARIANT discriminant, not just the shared inner key.
    // `BomRevision("x")` and `DerivedHash("x")` both have key "x" but are distinct
    // comparability classes for retrieval; without the tag, one valid attestation
    // would verify for the other and replay a row across classes.
    let class_tag = match &c.config_class {
        ConfigClass::BomRevision(_) => "bom",
        ConfigClass::DerivedHash(_) => "hash",
    };
    let _ = writeln!(s, "class:{class_tag}");
    lp(&mut s, "classkey", c.config_class.key());
    // Run-provenance: binds the run_id so one attestation cannot be replayed onto
    // clones with fabricated run ids to inflate the independent-confirmation count.
    match &c.provenance {
        None => {
            let _ = writeln!(s, "prov:none");
        }
        Some(p) => {
            lp(&mut s, "run", &p.run_id);
            let _ = writeln!(s, "rf:{}", p.retrieval_first);
            let mut primed: Vec<&str> = p.primed_from.iter().map(|x| x.as_str()).collect();
            primed.sort_unstable();
            let _ = writeln!(s, "primed:{}", primed.len());
            for id in &primed {
                lp(&mut s, "primed", id);
            }
        }
    }
    s.into_bytes()
}

/// The chain hash for a row given the previous row's hash: `sha256(prev ||
/// canonical(row-without-integrity))`, hex. The row is canonicalized with its
/// `integrity` field cleared so the hash never depends on itself; every other
/// field (including the attestation and provenance) is covered, so any edit
/// breaks the chain.
pub(crate) fn chain_hash(prev: &str, row: &Contribution) -> String {
    use sha2::{Digest, Sha256};
    let mut bare = row.clone();
    bare.integrity = None;
    let payload = serde_json::to_vec(&bare).expect("contribution serializes");
    let mut hasher = Sha256::new();
    // A versioned domain prefix so the chain encoding can evolve without silently
    // colliding with a future scheme, consistent with the canonicalization tags
    // used elsewhere. The payload is the same-code serde_json image of the row
    // (recomputed only by this crate), so cross-language reproducibility is not a
    // requirement here as it is for the attestation message.
    hasher.update(b"cec-corpus-chain-v1\n");
    hasher.update(prev.as_bytes());
    hasher.update(b"\n");
    hasher.update(&payload);
    hasher
        .finalize()
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
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
