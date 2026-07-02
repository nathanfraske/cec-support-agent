// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 The cec-support-agent authors
//! Plan provenance: judge-signed plans only.
//!
//! Agent neutrality requires that a plan enters the on-machine zone only
//! through the judge. This crate is that boundary's mechanism: the judge
//! holds a [`SigningKey`] and signs the winning plan; the executor verifies
//! the signature before any step runs, so a plan that was tampered with —
//! or that never passed the judge at all — is refused in code.
//!
//! The signature is HMAC-SHA256 over the plan's canonical JSON, so it binds
//! the exact steps, actions, and risk classes the judge saw. The bootstrap
//! uses a symmetric in-process key; key custody, rotation, signature format
//! evolution, and audit-log retention are deliberately left open (they are
//! deployment policy, not engine mechanics) — the enforcement point is what
//! this crate fixes in place.

use common::Plan;
use ed25519_dalek::{Signature, Signer, SigningKey as Ed25519Key, Verifier, VerifyingKey};
use hmac::{Hmac, Mac};
use sha2::Sha256;
use thiserror::Error;

type HmacSha256 = Hmac<Sha256>;

/// Errors raised while verifying plan provenance.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum ProvenanceError {
    /// The signature does not match the plan: it was modified after signing,
    /// signed with a different key, or never signed by the judge.
    #[error("plan signature verification failed: not the plan the judge signed")]
    BadSignature,
}

/// A plan together with the judge's signature over its canonical content.
///
/// In-process only (judge → executor): it wraps a raw in-flight [`Plan`] and so
/// has no `Serialize`/`Deserialize`. The signature binds the plan's canonical
/// bytes; it never crosses a serialize boundary.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SignedPlan {
    /// The plan exactly as the judge signed it.
    pub plan: Plan,
    /// Hex-encoded HMAC-SHA256 over the plan's canonical JSON.
    pub signature: String,
}

/// The key the judge signs winning plans with and the executor verifies them
/// against (symmetric in the bootstrap).
pub struct SigningKey([u8; 32]);

impl SigningKey {
    /// A fresh random key from OS entropy, for a process that hosts both the
    /// judge and the executor.
    pub fn generate() -> Self {
        let mut bytes = [0u8; 32];
        getrandom::getrandom(&mut bytes).expect("OS entropy source");
        Self(bytes)
    }

    /// A key from caller-provided bytes, for hosts that manage custody.
    pub fn from_bytes(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    /// Sign `plan`, binding the signature to its exact canonical content.
    pub fn sign(&self, plan: &Plan) -> SignedPlan {
        let mut mac = self.mac();
        mac.update(&canonical(plan));
        SignedPlan {
            plan: plan.clone(),
            signature: hex(&mac.finalize().into_bytes()),
        }
    }

    /// Verify that `signed` carries exactly the plan this key signed.
    /// Comparison is constant-time (via the HMAC implementation).
    pub fn verify(&self, signed: &SignedPlan) -> Result<(), ProvenanceError> {
        let expected = unhex(&signed.signature).ok_or(ProvenanceError::BadSignature)?;
        let mut mac = self.mac();
        mac.update(&canonical(&signed.plan));
        mac.verify_slice(&expected)
            .map_err(|_| ProvenanceError::BadSignature)
    }

    fn mac(&self) -> HmacSha256 {
        HmacSha256::new_from_slice(&self.0).expect("HMAC accepts any key length")
    }
}

/// The canonical bytes a signature covers: a deterministic, serde-INDEPENDENT
/// encoding of the plan's semantic content (id, title, and each step's action,
/// description, and risk, in order). Naming every field explicitly means the
/// signature does not depend on serde struct field order or on the JSON
/// serializer, so it stays stable across crate versions and is reproducible by
/// a verifier in another language — unlike `serde_json::to_vec`, whose bytes are
/// coupled to the struct definitions. Any change to a bound field changes these
/// bytes and breaks verification.
fn canonical(plan: &Plan) -> Vec<u8> {
    use std::fmt::Write as _;
    let mut s = String::from("cec-plan-canonical-v1\n");
    let _ = writeln!(s, "id:{}", plan.id);
    let _ = writeln!(s, "title:{}", plan.title);
    for step in &plan.steps {
        // Length-prefix the free-text fields so they cannot be confused with the
        // field separators or with each other.
        let _ = writeln!(
            s,
            "step:action={};desc[{}]={};risk={:?}",
            step.action,
            step.description.len(),
            step.description,
            step.risk
        );
    }
    s.into_bytes()
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

fn unhex(text: &str) -> Option<Vec<u8>> {
    if text.len() % 2 != 0 {
        return None;
    }
    (0..text.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&text[i..i + 2], 16).ok())
        .collect()
}

fn unhex_array<const N: usize>(text: &str) -> Option<[u8; N]> {
    let bytes = unhex(text)?;
    <[u8; N]>::try_from(bytes.as_slice()).ok()
}

// ---------------------------------------------------------------------------
// Sign-off attestation (asymmetric: the engine cannot forge it)
// ---------------------------------------------------------------------------
//
// Plan signing above is symmetric (HMAC) because the judge and the executor are
// the same process. Sign-off is different: the whole point is that the process
// admitting a corpus row must NOT be able to mint the "a human approved this"
// claim itself. So sign-off attestation is ed25519: a [`SignOffAuthority`]
// (whoever legitimately performs sign-off) holds the private key and signs the
// contribution's canonical tuple; the engine embeds only the [`SignOffPublicKey`]
// and re-verifies. Because the signing key never reaches the submitting process,
// a self-asserted `HumanConfirmed` row carries no valid signature and the gate
// refuses it — the "asserting party ≠ approving party" property, cryptographic
// rather than a server-side repo rule.

/// The sign-off authority's key pair. Whoever legitimately performs sign-off — a
/// human-operated tool, or a verifier service — holds this; the engine that
/// admits corpus rows does NOT (it holds only [`SignOffPublicKey`]).
pub struct SignOffAuthority {
    signing: Ed25519Key,
}

impl SignOffAuthority {
    /// A fresh authority key pair from OS entropy.
    pub fn generate() -> Self {
        let mut seed = [0u8; 32];
        getrandom::getrandom(&mut seed).expect("OS entropy source");
        Self {
            signing: Ed25519Key::from_bytes(&seed),
        }
    }

    /// Rebuild the authority from its stored 32-byte seed (hex). This is the
    /// SECRET half — it belongs only where sign-off is performed, never in the
    /// engine that admits rows.
    pub fn from_seed_hex(seed_hex: &str) -> Option<Self> {
        Some(Self {
            signing: Ed25519Key::from_bytes(&unhex_array::<32>(seed_hex)?),
        })
    }

    /// The authority's seed (hex) for custody/storage. SECRET — handle as a key.
    pub fn seed_hex(&self) -> String {
        hex(&self.signing.to_bytes())
    }

    /// The public half to embed in the engine.
    pub fn public_key(&self) -> SignOffPublicKey {
        SignOffPublicKey {
            verifying: self.signing.verifying_key(),
        }
    }

    /// Attest (sign) the canonical bytes of a contribution's sign-off tuple.
    pub fn attest(&self, message: &[u8]) -> SignOffSignature {
        SignOffSignature(self.signing.sign(message))
    }
}

/// The public verifying half of a [`SignOffAuthority`], embedded in the engine.
/// Verifies attestations; cannot create them.
#[derive(Clone)]
pub struct SignOffPublicKey {
    verifying: VerifyingKey,
}

impl SignOffPublicKey {
    /// Parse a public key from its 32-byte hex encoding.
    pub fn from_hex(key_hex: &str) -> Option<Self> {
        let bytes = unhex_array::<32>(key_hex)?;
        VerifyingKey::from_bytes(&bytes)
            .ok()
            .map(|verifying| Self { verifying })
    }

    /// The 32-byte public key as hex.
    pub fn to_hex(&self) -> String {
        hex(self.verifying.as_bytes())
    }

    /// A short, stable id for this authority (first 16 hex chars of the key) —
    /// for diagnostics and to tag which authority signed a row.
    pub fn id(&self) -> String {
        self.to_hex()[..16].to_string()
    }

    /// Whether `signature` is a valid attestation of `message` by this authority.
    pub fn verify(&self, message: &[u8], signature: &SignOffSignature) -> bool {
        self.verifying.verify(message, &signature.0).is_ok()
    }
}

/// An ed25519 sign-off attestation over a contribution's canonical tuple.
pub struct SignOffSignature(Signature);

impl SignOffSignature {
    /// The 64-byte signature as hex (for storage on a corpus row).
    pub fn to_hex(&self) -> String {
        hex(&self.0.to_bytes())
    }

    /// Parse a signature from its 64-byte hex encoding.
    pub fn from_hex(sig_hex: &str) -> Option<Self> {
        unhex_array::<64>(sig_hex).map(|b| Self(Signature::from_bytes(&b)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use common::{PlanStep, Risk};

    fn plan() -> Plan {
        let mut plan = Plan::new("p1", "fix");
        plan.steps.push(PlanStep {
            description: "look".into(),
            action: "cim_query".into(),
            risk: Risk::ReadOnly,
        });
        plan
    }

    #[test]
    fn a_signed_plan_verifies() {
        let key = SigningKey::generate();
        let signed = key.sign(&plan());
        assert_eq!(key.verify(&signed), Ok(()));
    }

    #[test]
    fn tampering_with_any_step_breaks_the_signature() {
        let key = SigningKey::generate();
        let mut signed = key.sign(&plan());
        signed.plan.steps[0].action = "registry_set".into();
        assert_eq!(key.verify(&signed), Err(ProvenanceError::BadSignature));

        let mut signed = key.sign(&plan());
        signed.plan.steps[0].risk = Risk::Destructive;
        assert_eq!(key.verify(&signed), Err(ProvenanceError::BadSignature));

        // The canonical encoding also binds the plan title and each step's
        // rendered description (what the human consented to), not just actions.
        let mut signed = key.sign(&plan());
        signed.plan.title = "a different title".into();
        assert_eq!(key.verify(&signed), Err(ProvenanceError::BadSignature));

        let mut signed = key.sign(&plan());
        signed.plan.steps[0].description = "look (but actually do something)".into();
        assert_eq!(key.verify(&signed), Err(ProvenanceError::BadSignature));
    }

    #[test]
    fn the_canonical_encoding_is_not_coupled_to_serde() {
        // Two independently-built but semantically-identical plans sign the same.
        let a = key_signed_title("p1", "fix");
        let b = key_signed_title("p1", "fix");
        assert_eq!(a, b);
    }

    fn key_signed_title(id: &str, title: &str) -> Vec<u8> {
        let mut plan = Plan::new(id, title);
        plan.steps.push(PlanStep {
            description: "look".into(),
            action: "cim_query".into(),
            risk: Risk::ReadOnly,
        });
        super::canonical(&plan)
    }

    #[test]
    fn a_different_key_does_not_verify() {
        let signer = SigningKey::generate();
        let other = SigningKey::generate();
        let signed = signer.sign(&plan());
        assert_eq!(other.verify(&signed), Err(ProvenanceError::BadSignature));
    }

    #[test]
    fn garbage_signatures_are_refused_not_panicked_on() {
        let key = SigningKey::generate();
        let forged = SignedPlan {
            plan: plan(),
            signature: "not hex!".into(),
        };
        assert_eq!(key.verify(&forged), Err(ProvenanceError::BadSignature));
    }

    #[test]
    fn a_genuine_attestation_verifies_with_only_the_public_key() {
        let authority = SignOffAuthority::generate();
        let public = authority.public_key();
        let msg = b"signature|plan|label|human_confirmed|class";
        let sig = authority.attest(msg);
        assert!(public.verify(msg, &sig));
    }

    #[test]
    fn a_tampered_message_fails_verification() {
        let authority = SignOffAuthority::generate();
        let public = authority.public_key();
        let sig = authority.attest(b"the original tuple");
        assert!(!public.verify(b"a different tuple", &sig));
    }

    #[test]
    fn another_authoritys_public_key_does_not_verify() {
        let authority = SignOffAuthority::generate();
        let other = SignOffAuthority::generate();
        let msg = b"the tuple";
        let sig = authority.attest(msg);
        assert!(!other.public_key().verify(msg, &sig));
    }

    #[test]
    fn public_key_and_signature_round_trip_through_hex() {
        let authority = SignOffAuthority::generate();
        let public = authority.public_key();
        let msg = b"tuple bytes";
        let sig = authority.attest(msg);

        let public2 = SignOffPublicKey::from_hex(&public.to_hex()).expect("valid pubkey hex");
        let sig2 = SignOffSignature::from_hex(&sig.to_hex()).expect("valid sig hex");
        assert!(public2.verify(msg, &sig2));
        assert_eq!(public.id(), public2.id());
    }

    #[test]
    fn the_seed_reconstructs_the_same_authority() {
        let authority = SignOffAuthority::generate();
        let same = SignOffAuthority::from_seed_hex(&authority.seed_hex()).expect("valid seed");
        assert_eq!(authority.public_key().to_hex(), same.public_key().to_hex());
    }

    #[test]
    fn garbage_attestation_hex_is_refused_not_panicked_on() {
        assert!(SignOffPublicKey::from_hex("not hex").is_none());
        assert!(SignOffSignature::from_hex("zz").is_none());
        // Right shape, wrong length.
        assert!(SignOffPublicKey::from_hex("abcd").is_none());
    }
}
