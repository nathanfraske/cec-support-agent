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

impl Symptom {
    /// If this symptom names a Windows bug-check (stop) code — either the
    /// symbolic name (`irql_not_less_or_equal`) or the hex value (`0x0000000a`)
    /// — resolve it to the [`StopCode`](crate::StopCode) so a report or the
    /// diagnostic brain can show the operator what it means. Symptoms are stored
    /// lowercase; [`crate::describe`] matches names and hex case-insensitively.
    /// Returns `None` for symptoms that are not stop codes (`explorer.exe`,
    /// `event_41`, `crashes`).
    pub fn stop_code(&self) -> Option<&'static crate::StopCode> {
        crate::describe(&self.0)
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
        let fingerprint = crate::hash::fingerprint_of(crate::hash::FingerprintDomain::Fault, &keys);
        Self {
            fingerprint,
            symptoms,
        }
    }

    /// Every Windows bug-check (stop) code named among this signature's symptoms,
    /// in symptom order, de-duplicated by code (so `0x0000000a` and
    /// `irql_not_less_or_equal` in the same fault resolve to one entry). The
    /// diagnostic brain and operator-facing reports use this to attach a
    /// plain-English meaning to each stop code without re-parsing free text.
    pub fn stop_codes(&self) -> Vec<&'static crate::StopCode> {
        let mut out: Vec<&'static crate::StopCode> = Vec::new();
        for symptom in &self.symptoms {
            if let Some(sc) = symptom.stop_code() {
                if !out.iter().any(|e| e.code == sc.code) {
                    out.push(sc);
                }
            }
        }
        out
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

    /// The original symptoms that CLEARED in `post` (present here, absent there)
    /// — the fix's proven benefit. The other half of [`recurring_in`]: together
    /// they partition this signature's symptoms into "still there" and "gone".
    pub fn cleared_in(&self, post: &FaultSignature) -> Vec<Symptom> {
        self.symptoms
            .iter()
            .filter(|symptom| !post.symptoms.contains(symptom))
            .cloned()
            .collect()
    }

    /// The symptoms INTRODUCED in `post` (present there, absent here) — new
    /// problems that were not part of the original fault. A non-empty result is
    /// a regression the fix may have caused.
    pub fn introduced_in(&self, post: &FaultSignature) -> Vec<Symptom> {
        post.symptoms
            .iter()
            .filter(|symptom| !self.symptoms.contains(symptom))
            .cloned()
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn symptom_resolves_stop_codes_by_name_and_hex() {
        assert_eq!(
            Symptom("irql_not_less_or_equal".into())
                .stop_code()
                .map(|s| s.code),
            Some(0x0000_000A)
        );
        assert_eq!(
            Symptom("0x0000000a".into()).stop_code().map(|s| s.name),
            Some("IRQL_NOT_LESS_OR_EQUAL")
        );
        // Non-stop-code symptoms resolve to nothing.
        assert!(Symptom("explorer.exe".into()).stop_code().is_none());
        assert!(Symptom("crashes".into()).stop_code().is_none());
        assert!(Symptom("event_41".into()).stop_code().is_none());
    }

    #[test]
    fn signature_collects_stop_codes_deduped_by_code() {
        // Same stop code named twice — once by hex, once by symbolic name —
        // plus a non-stop-code symptom, resolves to a single explained entry.
        let sig = FaultSignature::from_symptoms(vec![
            Symptom("0x0000000a".into()),
            Symptom("explorer.exe".into()),
            Symptom("irql_not_less_or_equal".into()),
        ]);
        let codes = sig.stop_codes();
        assert_eq!(codes.len(), 1, "0xA named twice must dedupe to one");
        assert_eq!(codes[0].code, 0x0000_000A);
        assert!(!codes[0].meaning.trim().is_empty());
    }
}
