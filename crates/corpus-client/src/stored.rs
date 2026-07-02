//! The corpus-serializable *stored* payload types.
//!
//! Layer-1 of the leak-prevention methodology (`docs/corpus-leak-prevention.md`
//! §2) splits the in-flight domain objects from the corpus row: the in-flight
//! [`common::Plan`] / [`crate::Outcome`] lose `Serialize`, and a value can only
//! reach a serialize boundary as one of the types here — every one of which is
//! produced by a de-identifying mint ([`crate::de_identify_plan`]), never a raw
//! request-derived object. So `serde_json::to_string(&candidate)` is a compile
//! error, and the only thing that *can* be written to a corpus row is a value
//! that already passed the de-id chokepoint.
//!
//! On-disk/wire shape is IDENTICAL to the pre-split row: every field name,
//! order, and serde attribute mirrors the raw type it replaces, so existing
//! JSONL corpora and hash chains still load and verify (pinned by a canned-row
//! wire-compatibility test in `store.rs`). This is a type-level split, not a
//! format change.
//!
//! The fields are `pub(crate)`: a struct-literal from outside the crate does
//! not compile (there is no way to forge a prose-bearing stored value), while
//! the de-id and gate code inside the crate reads them directly. Read access
//! for embedders is via the accessors and [`StoredPlan::to_plan`] rehydration.

use common::{FaultSignature, Plan, PlanStep, Risk, Symptom, Verification};
use serde::{Deserialize, Serialize};

use crate::schema::OutcomeLabel;

/// A de-identified symptom as stored on a corpus row. Serializes transparently
/// as a bare string, byte-identical to [`common::Symptom`]'s wire form.
///
/// Deserialization is VALIDATING (`#[serde(try_from)]`, Layer-1e/C4): a symptom
/// read from the wire ([`crate::HttpCorpus::query`]) or disk
/// ([`crate::FileCorpus::open`]) must be a member of the closed de-id grammar
/// ([`common::is_symptom_token`]), so a served or at-rest row whose signature
/// carries an identity-shaped token fails to *deserialize* — the wire/file path
/// is now identical to the construction path, and `serde` no longer bypasses the
/// grammar. In-crate construction ([`StoredSymptom::from_symptom`]) wraps a token
/// the extractor already produced; the write gate re-validates it.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(try_from = "String")]
pub struct StoredSymptom(pub(crate) String);

impl StoredSymptom {
    /// Wrap an already-extracted symptom for storage.
    pub(crate) fn from_symptom(symptom: &Symptom) -> Self {
        Self(symptom.0.clone())
    }

    /// The de-identified symptom token.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Consume into the owned token.
    pub fn into_inner(self) -> String {
        self.0
    }
}

impl TryFrom<String> for StoredSymptom {
    type Error = deid::Reject;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        deid::symptom(&value).map(|s| Self(s.0))
    }
}

/// A de-identified plan-step action as stored on a corpus row — the "ActionToken"
/// of `docs/corpus-leak-prevention.md` §2 Layer 1c/1e. Serializes transparently
/// as a bare string; deserialization is VALIDATING (`#[serde(try_from)]`): a wire
/// or at-rest action must be a member of the frozen [`deid::ACTION_VOCABULARY`],
/// so an out-of-vocabulary action (request prose a compromised server or a
/// hand-edited file could carry) fails to *deserialize*.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "String")]
pub struct StoredAction(pub(crate) String);

impl StoredAction {
    /// The de-identified action (tool-vocabulary) token.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<String> for StoredAction {
    type Error = deid::Reject;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        deid::action(&value).map(Self)
    }
}

// Test-only unchecked constructor: the adversarial gate/read-path tests FORGE a
// stored step whose action is request prose the mint would never emit (a struct
// literal is the only way to build an inadmissible stored value), then assert the
// gate or the deserializer refuses it. Not available outside the crate's tests.
#[cfg(test)]
impl From<&str> for StoredAction {
    fn from(value: &str) -> Self {
        Self(value.to_string())
    }
}

/// A de-identified plan id as stored on a corpus row. Serializes transparently
/// as a bare string; deserialization is VALIDATING (`#[serde(try_from)]`): a wire
/// or at-rest id must be a clean bounded slug ([`deid::plan_id`]), so a
/// path/host/email string that a server or hand-edited file routed into the id
/// fails to *deserialize*.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "String")]
pub struct StoredPlanId(pub(crate) String);

impl StoredPlanId {
    /// The plan id slug.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<String> for StoredPlanId {
    type Error = deid::Reject;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        deid::plan_id(&value).map(Self)
    }
}

// Test-only unchecked constructor (see [`StoredAction`]'s): forges a stored plan
// id for the adversarial gate/read-path tests. Not available outside tests.
#[cfg(test)]
impl From<&str> for StoredPlanId {
    fn from(value: &str) -> Self {
        Self(value.to_string())
    }
}

/// A de-identified fault signature as stored on a corpus row. Same field names
/// and order as [`common::FaultSignature`], so its JSON is byte-identical.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct StoredSignature {
    pub(crate) fingerprint: String,
    pub(crate) symptoms: Vec<StoredSymptom>,
}

impl StoredSignature {
    /// Build the stored signature from an in-flight one. The fingerprint is a
    /// content hash and the symptoms are vocabulary tokens produced by the
    /// extractor, so this is a structural move, not a scrub.
    pub(crate) fn from_signature(signature: &FaultSignature) -> Self {
        Self {
            fingerprint: signature.fingerprint.clone(),
            symptoms: signature
                .symptoms
                .iter()
                .map(StoredSymptom::from_symptom)
                .collect(),
        }
    }

    /// The stable content fingerprint of the fault.
    pub fn fingerprint(&self) -> &str {
        &self.fingerprint
    }

    /// The de-identified symptom tokens.
    pub fn symptoms(&self) -> &[StoredSymptom] {
        &self.symptoms
    }

    /// Rehydrate the stored signature into an in-flight [`FaultSignature`] — the
    /// read-side counterpart used when a served signature must be compared or
    /// re-queried. The tokens are already de-identified, so this is a structural
    /// move.
    pub fn to_signature(&self) -> FaultSignature {
        FaultSignature {
            fingerprint: self.fingerprint.clone(),
            symptoms: self.symptoms.iter().map(|s| Symptom(s.0.clone())).collect(),
        }
    }
}

/// A single de-identified plan step as stored on a corpus row. Same field names
/// and order as [`common::PlanStep`]; every field is a minted/validated token.
/// `description` equals `action` for a stored row (the de-id mint sets it so), so
/// both are the validating [`StoredAction`] type — a served/at-rest step whose
/// action OR description is out-of-vocabulary fails to deserialize.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StoredStep {
    pub(crate) description: StoredAction,
    pub(crate) action: StoredAction,
    pub(crate) risk: Risk,
}

impl StoredStep {
    /// The de-identified action (tool-vocabulary) token.
    pub fn action(&self) -> &str {
        self.action.as_str()
    }

    /// The de-identified step description (equal to the action for a stored row).
    pub fn description(&self) -> &str {
        self.description.as_str()
    }

    /// The step's risk classification.
    pub fn risk(&self) -> Risk {
        self.risk
    }
}

/// A de-identified plan as stored on a corpus row — the ONLY serializable plan
/// payload. Produced by [`crate::de_identify_plan`], which mints the id and
/// every action against the frozen vocabulary and drops all free text; a value
/// of this type is de-identified by construction. Same field names and order as
/// [`common::Plan`], so the JSON is byte-identical.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StoredPlan {
    pub(crate) id: StoredPlanId,
    pub(crate) title: String,
    pub(crate) steps: Vec<StoredStep>,
}

impl StoredPlan {
    /// Construct a stored plan from already-minted pieces. Crate-internal so the
    /// only path to a `StoredPlan` outside the crate is `de_identify_plan`.
    pub(crate) fn from_minted(id: StoredPlanId, steps: Vec<StoredStep>) -> Self {
        let title = steps
            .iter()
            .map(|step| step.action.as_str())
            .collect::<Vec<_>>()
            .join(" -> ");
        Self { id, title, steps }
    }

    /// The plan id (a validated slug).
    pub fn id(&self) -> &str {
        self.id.as_str()
    }

    /// The de-identified title (the joined action vocabulary).
    pub fn title(&self) -> &str {
        &self.title
    }

    /// The stored steps.
    pub fn steps(&self) -> &[StoredStep] {
        &self.steps
    }

    /// The overall risk of the plan: the maximum risk of any step.
    pub fn risk(&self) -> Risk {
        self.steps
            .iter()
            .map(|step| step.risk)
            .max()
            .unwrap_or(Risk::ReadOnly)
    }

    /// Rehydrate the stored plan into an in-flight [`common::Plan`] for the
    /// retrieval-first pipeline (judge → consent → execute). The stored fields
    /// are already de-identified, so wrapping them back into `Prose` leaves is
    /// safe — this is the read-side counterpart of `de_identify_plan`. Phase 2
    /// hardens the served path with `from_served` re-validation before this.
    pub fn to_plan(&self) -> Plan {
        Plan {
            id: self.id.0.clone(),
            title: self.title.clone().into(),
            steps: self
                .steps
                .iter()
                .map(|step| PlanStep {
                    description: step.description.0.clone().into(),
                    action: step.action.0.clone(),
                    risk: step.risk,
                })
                .collect(),
        }
    }
}

/// A de-identified outcome as stored on a corpus row: the (signature, plan,
/// label, verification) tuple, carrying only stored/validated types. Replaces
/// the in-flight [`crate::Outcome`] (which loses `Serialize`) on a
/// [`crate::Contribution`]. Same field names and order, so its JSON is
/// byte-identical to the pre-split row.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StoredOutcome {
    pub(crate) signature: StoredSignature,
    pub(crate) plan: StoredPlan,
    pub(crate) label: OutcomeLabel,
    #[serde(default)]
    pub(crate) verification: Option<Verification>,
}

impl StoredOutcome {
    /// The de-identified fault signature.
    pub fn signature(&self) -> &StoredSignature {
        &self.signature
    }

    /// The de-identified plan.
    pub fn plan(&self) -> &StoredPlan {
        &self.plan
    }

    /// The sign-off label bound to the row.
    pub fn label(&self) -> &OutcomeLabel {
        &self.label
    }

    /// The bound verification verdict, if any.
    pub fn verification(&self) -> Option<&Verification> {
        self.verification.as_ref()
    }
}
