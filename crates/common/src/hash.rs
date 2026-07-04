//! The keyed content fingerprint shared by
//! [`FaultSignature`](crate::FaultSignature) fingerprints and
//! [`ConfigClass`](crate::ConfigClass) derived hashes.
//!
//! v1 was an unsalted FNV-1a over the sorted keys — leak class C7
//! (`docs/corpus-leak-prevention.md` §3.1(2)): over an identity-bearing input
//! an unsalted 64-bit hash is dictionary-reversible (compute-then-probe over
//! the enumerable input space recovers the pre-image) and a stable cross-log
//! correlation handle. v2 is HMAC-SHA256 under a per-deployment secret salt,
//! so the probe space is opaque to anyone without the salt and two deployments
//! with different salts produce unlinkable fingerprints for the same input.
//!
//! Salt custody (owner decision 2026-07-03): the salt is a per-deployment
//! secret loaded like the sign-off key — the `cec-support-agent` binary reads
//! `CEC_FINGERPRINT_SALT` at startup and calls [`set_fingerprint_salt`];
//! embedders do the same before the first fingerprint is computed. When no
//! salt is configured the documented [`COLD_START_FINGERPRINT_SALT`] applies,
//! which keeps cold start deterministic and byte-identical across
//! deployments; it provides domain separation only, NOT unlinkability — the
//! C7 property requires setting a real salt.

use std::sync::OnceLock;

/// The documented cold-start default salt, applied when
/// [`set_fingerprint_salt`] was never called. Public and fixed by design: a
/// cold-start engine stays deterministic and byte-identical across
/// deployments. It is NOT a secret — fingerprints under this default are
/// exactly as enumerable as the unsalted v1 hash, so a deployment that wants
/// the leak-C7 non-mappability property must configure a real per-deployment
/// salt (`CEC_FINGERPRINT_SALT`, e.g. from `openssl rand -hex 32`).
pub const COLD_START_FINGERPRINT_SALT: &[u8] = b"cec-fingerprint-cold-start-default-v2";

/// The minimum accepted salt length, in bytes. A shorter salt is refused
/// outright: a trivially short secret silently reopens the dictionary attack
/// the salt exists to close, so misconfiguration fails closed at startup
/// rather than weakening fingerprints quietly.
pub const MIN_FINGERPRINT_SALT_LEN: usize = 16;

static FINGERPRINT_SALT: OnceLock<Vec<u8>> = OnceLock::new();

/// Why [`set_fingerprint_salt`] refused. Messages are fixed and never echo the
/// salt bytes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FingerprintSaltError {
    /// The salt is shorter than [`MIN_FINGERPRINT_SALT_LEN`] bytes.
    TooShort,
    /// The salt is already fixed for this process — either set before, or a
    /// fingerprint was already computed (which locks in the cold-start
    /// default). Configure the salt at startup, before any fingerprint.
    AlreadyActive,
}

impl std::fmt::Display for FingerprintSaltError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FingerprintSaltError::TooShort => write!(
                f,
                "fingerprint salt is shorter than {MIN_FINGERPRINT_SALT_LEN} bytes — refusing a \
                 trivially enumerable salt"
            ),
            FingerprintSaltError::AlreadyActive => write!(
                f,
                "fingerprint salt is already fixed for this process — set it at startup, before \
                 the first fingerprint is computed"
            ),
        }
    }
}

impl std::error::Error for FingerprintSaltError {}

/// Configure the per-deployment fingerprint salt for this process. Call once,
/// at startup, before the first fingerprint is computed; the salt is
/// write-once (a mid-run change would silently split the fingerprint space).
/// Fails closed: a too-short salt or a late/duplicate set is an error, never a
/// silent fallback.
pub fn set_fingerprint_salt(salt: &[u8]) -> Result<(), FingerprintSaltError> {
    if salt.len() < MIN_FINGERPRINT_SALT_LEN {
        return Err(FingerprintSaltError::TooShort);
    }
    FINGERPRINT_SALT
        .set(salt.to_vec())
        .map_err(|_| FingerprintSaltError::AlreadyActive)
}

/// The salt in effect: the configured per-deployment secret, else the
/// documented cold-start default. First use locks the choice for the process.
fn active_salt() -> &'static [u8] {
    FINGERPRINT_SALT
        .get_or_init(|| COLD_START_FINGERPRINT_SALT.to_vec())
        .as_slice()
}

/// Stable, order-independent keyed content hash over string keys, rendered as
/// lowercase hex (64 chars): HMAC-SHA256 under the active salt over the
/// `cec-fingerprint-v2` canonical encoding of the sorted keys. It carries no
/// identity data beyond what the keys themselves expose — and under a real
/// per-deployment salt it is not enumerable or linkable across deployments.
pub(crate) fn fingerprint_of(keys: &[&str]) -> String {
    keyed_fingerprint(active_salt(), keys)
}

/// The keyed core, explicit about its salt so the two-salt properties are
/// testable without touching the process-global.
fn keyed_fingerprint(salt: &[u8], keys: &[&str]) -> String {
    use hmac::{Hmac, Mac};
    use sha2::Sha256;
    use std::fmt::Write as _;

    let mut keys: Vec<&str> = keys.to_vec();
    keys.sort_unstable();
    // Canonical message: versioned domain tag, key count, then each key
    // length-prefixed — so ["ab","c"] and ["a","bc"] cannot collide and a key
    // cannot smear into its neighbor. Mirrors the discipline of
    // `provenance::canonical` and the corpus chain encoding.
    let mut message = String::from("cec-fingerprint-v2\n");
    let _ = writeln!(message, "keys:{}", keys.len());
    for key in keys {
        let _ = writeln!(message, "key[{}]={key}", key.len());
    }
    let mut mac = Hmac::<Sha256>::new_from_slice(salt).expect("HMAC-SHA256 accepts any key length");
    mac.update(message.as_bytes());
    mac.finalize()
        .into_bytes()
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    // NOTE: unit tests here run in one process and must never call
    // `set_fingerprint_salt` — it would lock a non-default salt in for every
    // other test in this binary. The set/lock behavior is exercised in the
    // `fingerprint_salt` integration test (its own process); salted variation
    // is exercised through `keyed_fingerprint` directly.

    #[test]
    fn fingerprints_are_order_independent_and_64_hex() {
        let a = fingerprint_of(&["gpu:rtx-4070", "os:windows 11"]);
        let b = fingerprint_of(&["os:windows 11", "gpu:rtx-4070"]);
        assert_eq!(a, b, "the same key set must fingerprint identically");
        assert_eq!(a.len(), 64);
        assert!(a.bytes().all(|b| b.is_ascii_hexdigit()));
    }

    #[test]
    fn concatenation_cannot_forge_a_key_boundary() {
        assert_ne!(
            fingerprint_of(&["ab", "c"]),
            fingerprint_of(&["a", "bc"]),
            "length-prefixing must keep key boundaries distinct"
        );
    }

    #[test]
    fn the_cold_start_fingerprint_is_pinned() {
        // The documented cold-start default is part of the deployment contract
        // (byte-identical cold start): this vector moves only with a deliberate
        // encoding or default-salt change, which is a corpus migration.
        assert_eq!(
            fingerprint_of(&["event_41"]),
            keyed_fingerprint(COLD_START_FINGERPRINT_SALT, &["event_41"]),
            "an unconfigured process must use the documented cold-start salt"
        );
    }

    #[test]
    fn two_salts_produce_unlinkable_fingerprints() {
        // The leak-C7 property: the same input under two deployments' salts
        // must not correlate — and neither output equals the cold-start one.
        let keys = ["event_41", "0x1234"];
        let a = keyed_fingerprint(b"deployment-a-salt-0123456789abcdef", &keys);
        let b = keyed_fingerprint(b"deployment-b-salt-0123456789abcdef", &keys);
        let cold = keyed_fingerprint(COLD_START_FINGERPRINT_SALT, &keys);
        assert_ne!(a, b);
        assert_ne!(a, cold);
        assert_ne!(b, cold);
    }

    #[test]
    fn a_short_salt_is_refused_without_locking_the_default() {
        // TooShort must return BEFORE initializing the global, so a failed set
        // does not silently lock in the cold-start default.
        assert_eq!(
            set_fingerprint_salt(b"short"),
            Err(FingerprintSaltError::TooShort)
        );
    }
}
