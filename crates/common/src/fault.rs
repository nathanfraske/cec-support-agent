use serde::{Deserialize, Serialize};

/// A normalized symptom string extracted from diagnostics.
///
/// Serializes transparently as a bare string. Deserialization is VALIDATING
/// (`#[serde(try_from)]`): a symptom read from the wire or disk must be a member
/// of the closed de-id grammar ([`crate::is_symptom_token`]), so a served or
/// at-rest row cannot carry an identity-shaped "symptom" (`desktop-nathan01`,
/// an asset tag) in the fault signature or a verification's recurring set. This
/// is the read-side counterpart of the write-time extractor — Layer-1e/C4 of
/// `docs/corpus-leak-prevention.md`. In-flight construction (`From<&str>`, the
/// public tuple field) is unchecked; the extractor only ever produces grammar
/// members, and the write gate re-validates.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(try_from = "String")]
pub struct Symptom(pub String);

impl From<&str> for Symptom {
    fn from(s: &str) -> Self {
        Symptom(s.to_string())
    }
}

impl TryFrom<String> for Symptom {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        if crate::is_symptom_token(&value) {
            Ok(Symptom(value))
        } else {
            Err(format!(
                "'{value}' is not a member of the de-id symptom grammar"
            ))
        }
    }
}

/// A de-identified fingerprint of a fault, used to look up fix mappings in the
/// corpus. Contains no machine- or user-identifying information — only the
/// normalized symptoms and a stable content hash of them.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct FaultSignature {
    /// Stable, order-independent content hash of the normalized symptoms.
    pub fingerprint: String,
    /// The normalized symptoms that produced the fingerprint.
    pub symptoms: Vec<Symptom>,
}

impl FaultSignature {
    /// Build a signature from symptoms, computing a stable fingerprint.
    pub fn from_symptoms(symptoms: Vec<Symptom>) -> Self {
        let keys: Vec<&str> = symptoms.iter().map(|s| s.0.as_str()).collect();
        let fingerprint = crate::hash::fingerprint_of(&keys);
        Self {
            fingerprint,
            symptoms,
        }
    }

    /// Whether any of this signature's symptoms recur in `post`. Verification
    /// diffs a re-collected signature against the original failure signature
    /// with this: the claim "fixed" is only valid against the same instrument
    /// that established "broken".
    pub fn recurring_in(&self, post: &FaultSignature) -> Vec<Symptom> {
        self.symptoms
            .iter()
            .filter(|symptom| post.symptoms.contains(symptom))
            .cloned()
            .collect()
    }
}
