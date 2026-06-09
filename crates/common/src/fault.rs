use serde::{Deserialize, Serialize};

/// A normalized symptom string extracted from diagnostics.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Symptom(pub String);

impl From<&str> for Symptom {
    fn from(s: &str) -> Self {
        Symptom(s.to_string())
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
        let fingerprint = Self::fingerprint_of(&symptoms);
        Self {
            fingerprint,
            symptoms,
        }
    }

    /// Compute a stable, order-independent fingerprint of the symptoms. This is
    /// a non-cryptographic FNV-1a content hash; it carries no identity data and
    /// pulls in no dependencies.
    fn fingerprint_of(symptoms: &[Symptom]) -> String {
        let mut keys: Vec<&str> = symptoms.iter().map(|s| s.0.as_str()).collect();
        keys.sort_unstable();
        let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
        for key in keys {
            for byte in key.as_bytes() {
                hash ^= u64::from(*byte);
                hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
            }
            // Separator so ["ab","c"] and ["a","bc"] do not collide.
            hash ^= 0xff;
            hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
        }
        format!("{hash:016x}")
    }
}
