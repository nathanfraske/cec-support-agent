use common::{ConfigClass, FaultSignature, Plan, Verification};
use serde::{Deserialize, Serialize};

use crate::stored::{
    StoredAction, StoredOutcome, StoredPlan, StoredPlanId, StoredSignature, StoredStep,
};

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
        /// The implicated part class (e.g. "psu", "storage", "memory"). Comes
        /// from the frozen HARDWARE_MARKERS taxonomy, so it is a bounded lowercase
        /// slug; deserialization is VALIDATING ([`deserialize_part_class`]) — a
        /// wire or at-rest row whose part_class carries identity/prose fails to
        /// deserialize, matching the other stored leaves.
        #[serde(deserialize_with = "deserialize_part_class")]
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
///
/// Carries STORED (de-identified) payload types — the served plan is a
/// [`StoredPlan`], never a raw in-flight `Plan` — so a served mapping cannot
/// hand raw prose to the pipeline. Rehydrate the plan for the retrieval-first
/// slate with [`StoredPlan::to_plan`] (Phase 2 adds `from_served`
/// re-validation before that).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FixMapping {
    /// The fault this mapping addresses.
    pub signature: StoredSignature,
    /// The de-identified plan known to resolve it.
    pub plan: StoredPlan,
    /// How many confirmed outcomes back this mapping.
    pub confirmations: u32,
}

/// The result of executing a plan against a fault — the **in-flight** triple a
/// caller assembles and hands to [`Contribution::new`]. It carries a raw
/// [`Plan`] and deliberately has **no `Serialize`**: a raw outcome cannot reach
/// a corpus row directly, only through `Contribution::new`, which de-identifies
/// the plan into a [`StoredPlan`]. The stored form is [`StoredOutcome`].
#[derive(Debug, Clone, PartialEq, Eq)]
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
    pub verification: Option<Verification>,
}

/// Per-row tamper-evidence: a hash chain linking this row to the previous one
/// in a [`crate::FileCorpus`]. `hash = sha256(chain_canonical(prev, row))` — a
/// serde-independent, field-by-field canonical encoding (`cec-corpus-chain-v2`)
/// with the `integrity` field itself excluded. The store fills this in on
/// write; on open it recomputes the chain and refuses a file where any row has
/// been edited, reordered, or removed mid-stream — so a hand-edited "confirmed"
/// precedent is never served. (The known residual is truncation of the tail,
/// which a hash chain cannot detect without an external anchor; see FOLLOWUPS.)
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
    /// independent confirmations; the same id is a single observation. Production
    /// run ids are 32 hex chars from OS entropy; deserialization is VALIDATING
    /// ([`deserialize_run_id`]) — a wire or at-rest row whose run_id is not a
    /// bounded opaque token (a smuggled path/email/prose on the provenance pin)
    /// fails to deserialize.
    #[serde(deserialize_with = "deserialize_run_id")]
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

impl RowProvenance {
    /// The provenance COMMITMENT: `sha256` (hex) over a canonical, serde-
    /// independent encoding of this provenance (`cec-provenance-commitment-v1`:
    /// length-prefixed run id, the retrieval-first bit, and the SORTED
    /// primed-from set, count-framed). The v4 attestation binds THIS value
    /// instead of the raw fields, so a served corpus row can prove its
    /// attestation covers its provenance without shipping the run id or the
    /// priming graph (RFC Q6, owner-decided provenance minimization) — equal
    /// provenance ⇒ equal commitment (still a usable independence key), while
    /// the raw fields stay at rest only.
    pub fn commitment(&self) -> String {
        use sha2::{Digest, Sha256};
        use std::fmt::Write as _;
        let mut s = String::from("cec-provenance-commitment-v1\n");
        lp(&mut s, "run", &self.run_id);
        let _ = writeln!(s, "rf:{}", self.retrieval_first);
        let mut primed: Vec<&str> = self.primed_from.iter().map(|x| x.as_str()).collect();
        primed.sort_unstable();
        let _ = writeln!(s, "primed:{}", primed.len());
        for id in &primed {
            lp(&mut s, "primed", id);
        }
        let mut hasher = Sha256::new();
        hasher.update(s.as_bytes());
        hasher
            .finalize()
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect()
    }
}

/// A de-identified outcome proposed for inclusion in the corpus: the
/// (signature, plan, outcome) triple plus its context.
///
/// [`Contribution::new`] de-identifies the plan by structured extraction
/// (see [`de_identify_plan`]) — free-text fields never reach a corpus row,
/// in code, regardless of what the caller passes in.
///
/// The fields are **private**: [`Contribution::new`] is the sole constructor, so
/// a struct-literal cannot bypass the de-id mint (pinned by a `trybuild`
/// compile-fail test), and the stored `outcome` is a [`StoredOutcome`] — a raw
/// `Outcome` can never sit on a row. Read the row through the accessors.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Contribution {
    /// The de-identified outcome being contributed.
    pub(crate) outcome: StoredOutcome,
    /// The comparability key: a contribution is only retrieved for tickets of
    /// the same config class.
    pub(crate) config_class: ConfigClass,
    /// Sign-off state. Must be confirmed for [`crate::CorpusStore::submit`] to
    /// accept it.
    pub(crate) sign_off: SignOff,
    /// The sign-off authority's attestation over this row's canonical tuple.
    /// `None` at cold start (no authority configured). When a store carries an
    /// authority public key, this must be present and valid for a confirmed row.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) attestation: Option<SignOffAttestation>,
    /// Run-provenance: which run produced this row and how its plan was
    /// generated. `None` on legacy rows; when present, confirmation counting
    /// uses it to admit only independent confirmations.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) provenance: Option<RowProvenance>,
    /// Tamper-evidence chain link, filled in by a [`crate::FileCorpus`] on write.
    /// Not part of the signed/attested tuple (the store adds it after admission).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) integrity: Option<RowIntegrity>,
}

/// Whether a `part_class` is an admissible taxonomy token: a bounded lowercase
/// slug `[a-z0-9_-]{1,32}`, the shape the frozen `panel::HARDWARE_MARKERS` table
/// emits (psu, storage, memory, cooling, platform, gpu). Rejects the identity a
/// hand-edited row could route into the label (spaces, uppercase, `@`, `.`, `/`)
/// — e.g. `"psu on DESKTOP-NATHAN01 for nathan@cec.direct"`.
pub(crate) fn is_part_class_token(value: &str) -> bool {
    (1..=32).contains(&value.len())
        && value
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-' || b == b'_')
}

/// Whether a `run_id` is an admissible opaque token: a bounded
/// `[A-Za-z0-9_-]{1,64}` string. Production run ids are 32 hex chars from OS
/// entropy; this bounds length and charset so a hand-edited or embedder-built row
/// cannot smuggle a path, email, or free-text prose through the provenance pin.
pub(crate) fn is_run_id_token(value: &str) -> bool {
    (1..=64).contains(&value.len())
        && value
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
}

/// Read-side validating deserializer for `OutcomeLabel::EscalatedHardware`'s
/// `part_class`: an at-rest/wire value outside the bounded taxonomy slug fails to
/// deserialize, so a hand-edited row is refused at `FileCorpus::open` rather than
/// riding onto the API wire via `wire_label`.
fn deserialize_part_class<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let value = String::deserialize(deserializer)?;
    if is_part_class_token(&value) {
        Ok(value)
    } else {
        Err(serde::de::Error::custom(
            "part_class is not a bounded taxonomy slug [a-z0-9_-]{1,32}",
        ))
    }
}

/// Read-side validating deserializer for `RowProvenance::run_id`: an
/// at-rest/wire value that is not a bounded opaque token fails to deserialize.
fn deserialize_run_id<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let value = String::deserialize(deserializer)?;
    if is_run_id_token(&value) {
        Ok(value)
    } else {
        Err(serde::de::Error::custom(
            "run_id is not a bounded opaque token [A-Za-z0-9_-]{1,64}",
        ))
    }
}

impl Contribution {
    /// Build a contribution from an outcome, its config class, and its
    /// sign-off state. The outcome's plan is de-identified here, so a raw
    /// plan cannot enter a contribution at all — and the de-id is validating:
    /// an out-of-vocabulary action or a prose-bearing plan id refuses the row
    /// (see [`de_identify_plan`]) instead of being copied through. A hardware
    /// label's `part_class` is validated to the taxonomy slug on the same
    /// boundary. The attestation is unset; attach one with
    /// [`Contribution::attested_by`].
    pub fn new(
        outcome: Outcome,
        config_class: ConfigClass,
        sign_off: SignOff,
    ) -> Result<Self, deid::Reject> {
        // Validate the one free-text field the label carries onto the row: a
        // hardware part_class. It rides onto the serialized row unmodified and
        // egresses to the API wire via `wire_label`, so it is minted here, not
        // copied through — matching the plan's de-id discipline.
        if let OutcomeLabel::EscalatedHardware { part_class } = &outcome.label {
            if !is_part_class_token(part_class) {
                return Err(deid::Reject {
                    field: "part_class",
                    reason: "not a bounded taxonomy slug [a-z0-9_-]{1,32}",
                });
            }
        }
        let stored = StoredOutcome {
            signature: StoredSignature::from_signature(&outcome.signature),
            plan: de_identify_plan(&outcome.plan)?,
            label: outcome.label,
            verification: outcome.verification,
        };
        Ok(Self {
            outcome: stored,
            config_class,
            sign_off,
            attestation: None,
            provenance: None,
            integrity: None,
        })
    }

    /// The de-identified outcome on this row.
    pub fn outcome(&self) -> &StoredOutcome {
        &self.outcome
    }

    /// The comparability class this row is scoped to.
    pub fn config_class(&self) -> &ConfigClass {
        &self.config_class
    }

    /// The sign-off state of this row.
    pub fn sign_off(&self) -> SignOff {
        self.sign_off
    }

    /// The sign-off authority's attestation, if attached.
    pub fn attestation(&self) -> Option<&SignOffAttestation> {
        self.attestation.as_ref()
    }

    /// The run-provenance of this row, if recorded.
    pub fn provenance(&self) -> Option<&RowProvenance> {
        self.provenance.as_ref()
    }

    /// The tamper-evidence chain link, if the row has been persisted.
    pub fn integrity(&self) -> Option<&RowIntegrity> {
        self.integrity.as_ref()
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

/// The canonical bytes a sign-off attestation covers (`cec-signoff-attestation-v4`):
/// the contribution's `(signature, plan, label, sign_off, config_class)` tuple
/// AND its run-provenance COMMITMENT, in a stable, serde-independent encoding.
/// The attestation field itself is excluded (it signs everything else). Built
/// from the de-identified plan that is stored, so the gate re-derives exactly
/// what the authority signed. v4 binds [`RowProvenance::commitment`] instead of
/// the raw provenance fields: replay protection is unchanged (a fabricated
/// run id changes the commitment and breaks the signature) but the signed
/// bytes no longer embed the run id or priming graph, so a Q6-minimized served
/// row (attested outcome + commitment, no raw provenance) is verifiable by a
/// consumer. Tampering with any covered field changes these bytes and breaks
/// verification.
pub(crate) fn attestation_message(c: &Contribution) -> Vec<u8> {
    use std::fmt::Write as _;

    let o = &c.outcome;
    let mut s = String::from("cec-signoff-attestation-v4\n");
    // Fault signature: fingerprint + sorted symptoms, each length-prefixed.
    lp(&mut s, "fp", &o.signature.fingerprint);
    let mut symptoms: Vec<&str> = o.signature.symptoms.iter().map(|x| x.0.as_str()).collect();
    symptoms.sort_unstable();
    let _ = writeln!(s, "syms:{}", symptoms.len());
    for sym in &symptoms {
        lp(&mut s, "sym", sym);
    }
    // Plan: id + each step (action length-prefixed, risk by discriminant).
    lp(&mut s, "plan", o.plan.id.as_str());
    let _ = writeln!(s, "steps:{}", o.plan.steps.len());
    for step in &o.plan.steps {
        lp(&mut s, "act", step.action.as_str());
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
    // Run-provenance, bound as its COMMITMENT (v4): the sha256 of the
    // provenance canonical still makes one attestation unreplayable onto
    // clones with fabricated run ids (a different run id ⇒ a different
    // commitment ⇒ different signed bytes), while the signed message itself
    // no longer embeds the raw run id or priming graph — so a Q6-minimized
    // served row can carry just this commitment and still let a consumer
    // verify the signature (RFC Q6 DECIDED note's design wrinkle, resolved).
    match &c.provenance {
        None => {
            let _ = writeln!(s, "prov:none");
        }
        Some(p) => {
            lp(&mut s, "provcommit", &p.commitment());
        }
    }
    s.into_bytes()
}

/// Append a length-prefixed `tag[len]=value\n` line to a canonical encoding.
/// Length-prefixing every attacker-controlled value is what makes the encoding
/// UNAMBIGUOUS: a value can no longer carry the field separators or a newline
/// to forge a different structure with the same bytes (a `plan.id` of
/// "p\nstep:rm:Destructive" is byte-distinct from a genuine extra step). Field
/// *names* and counts are fixed literals we emit, never free text, so they need
/// no prefix (and the `&'static str` tag pins tags-are-literals at the type
/// level). This mirrors the discipline in `provenance::canonical`; shared by
/// [`attestation_message`] and [`chain_canonical`].
fn lp(s: &mut String, tag: &'static str, value: &str) {
    use std::fmt::Write as _;
    let _ = writeln!(s, "{tag}[{}]={value}", value.len());
}

/// The chain hash for a row given the previous row's hash:
/// `sha256(chain_canonical(prev, row))`, hex. Every field except `integrity`
/// (which holds this hash, so the hash never depends on itself) is bound
/// explicitly — including the attestation and provenance — so any edit breaks
/// the chain.
pub(crate) fn chain_hash(prev: &str, row: &Contribution) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(chain_canonical(prev, row));
    hasher
        .finalize()
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

/// The canonical bytes the tamper-evidence chain covers: the previous row's
/// hash plus a deterministic, serde-INDEPENDENT, field-by-field encoding of the
/// row with its `integrity` field excluded (never read, so the hash cannot
/// depend on itself).
///
/// v1 hashed the row's `serde_json` image, which coupled the chain to struct
/// field order and serde attributes — a pure struct-layout refactor silently
/// broke every stored chain. v2 names every bound field explicitly (the same
/// discipline as [`attestation_message`] and `provenance::canonical`), so the
/// encoding is stable across crate versions and reproducible by a verifier in
/// another language. Two deliberate divergences from the attestation message:
/// list fields (symptoms, recurring, primed_from) are bound in STORED order,
/// not sorted — the chain tamper-evidences the row as written, it does not
/// canonicalize sets — and the fields the attestation leaves derived or
/// excluded (`plan.title`, each step's `description`, and the attestation
/// itself) are bound here, because a hand-edit to any of them must break the
/// chain.
fn chain_canonical(prev: &str, c: &Contribution) -> Vec<u8> {
    use std::fmt::Write as _;

    let o = &c.outcome;
    let mut s = String::from("cec-corpus-chain-v2\n");
    lp(&mut s, "prev", prev);
    // Fault signature: fingerprint + symptoms in stored order.
    lp(&mut s, "fp", &o.signature.fingerprint);
    let _ = writeln!(s, "syms:{}", o.signature.symptoms.len());
    for sym in &o.signature.symptoms {
        lp(&mut s, "sym", sym.as_str());
    }
    // Plan: id, derived title, and each step's description/action/risk.
    lp(&mut s, "plan", o.plan.id.as_str());
    lp(&mut s, "title", &o.plan.title);
    let _ = writeln!(s, "steps:{}", o.plan.steps.len());
    for step in &o.plan.steps {
        lp(&mut s, "desc", step.description.as_str());
        lp(&mut s, "act", step.action.as_str());
        let _ = writeln!(s, "risk:{:?}", step.risk);
    }
    lp(&mut s, "label", &label_tag(&o.label));
    match &o.verification {
        None => {
            let _ = writeln!(s, "ver:none");
        }
        Some(v) => {
            let _ = writeln!(
                s,
                "ver:{:?};class={:?};rec:{}",
                v.result,
                v.class,
                v.recurring.len()
            );
            for r in &v.recurring {
                lp(&mut s, "rec", &r.0);
            }
        }
    }
    let _ = writeln!(s, "signoff:{:?}", c.sign_off);
    let class_tag = match &c.config_class {
        ConfigClass::BomRevision(_) => "bom",
        ConfigClass::DerivedHash(_) => "hash",
    };
    let _ = writeln!(s, "class:{class_tag}");
    lp(&mut s, "classkey", c.config_class.key());
    match &c.attestation {
        None => {
            let _ = writeln!(s, "att:none");
        }
        Some(a) => {
            lp(&mut s, "attid", &a.authority_id);
            lp(&mut s, "attsig", &a.signature);
        }
    }
    match &c.provenance {
        None => {
            let _ = writeln!(s, "prov:none");
        }
        Some(p) => {
            lp(&mut s, "run", &p.run_id);
            let _ = writeln!(s, "rf:{}", p.retrieval_first);
            let _ = writeln!(s, "primed:{}", p.primed_from.len());
            for id in &p.primed_from {
                lp(&mut s, "primed", id);
            }
        }
    }
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
///
/// The kept fields are minted, not copied: `step.action` must be a member of
/// the frozen [`deid::ACTION_VOCABULARY`] and `plan.id` a clean bounded slug.
/// Both were historically copied through verbatim — the keystone leak vector
/// (C1) of `docs/corpus-leak-prevention.md` — so identity routed into either
/// field now aborts the row instead of riding into it.
pub fn de_identify_plan(plan: &Plan) -> Result<StoredPlan, deid::Reject> {
    let id = StoredPlanId(deid::plan_id(&plan.id)?);
    let steps = plan
        .steps
        .iter()
        .map(|step| {
            let action = StoredAction(deid::action(&step.action)?);
            Ok(StoredStep {
                description: action.clone(),
                action,
                risk: step.risk,
            })
        })
        .collect::<Result<Vec<_>, deid::Reject>>()?;
    Ok(StoredPlan::from_minted(id, steps))
}
