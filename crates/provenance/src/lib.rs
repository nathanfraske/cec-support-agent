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
use hmac::{Hmac, Mac};
use serde::{Deserialize, Serialize};
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
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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

/// The canonical bytes a signature covers: the plan's JSON serialization
/// (field order is fixed by the struct definitions in `common`).
fn canonical(plan: &Plan) -> Vec<u8> {
    serde_json::to_vec(plan).expect("plans serialize")
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
}
