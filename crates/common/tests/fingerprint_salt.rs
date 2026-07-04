//! Process-level test of the fingerprint-salt lifecycle (leak-C7).
//!
//! The salt is a write-once process-global, so this lives in its own
//! integration-test binary (its own process): the crate's unit tests must
//! never set a salt, or they would poison each other's fingerprints. One
//! sequential test exercises the whole lifecycle deterministically.

use common::{set_fingerprint_salt, ConfigClass, FaultSignature, FingerprintSaltError, Symptom};

const DEPLOYMENT_SALT: &[u8] = b"an-integration-test-deployment-salt";

#[test]
fn the_salt_lifecycle_is_fail_closed_and_write_once() {
    // 1. A too-short salt is refused and must NOT lock the global (a failed
    //    set later accepts a real salt).
    assert_eq!(
        set_fingerprint_salt(b"short"),
        Err(FingerprintSaltError::TooShort)
    );

    // 2. A real salt is accepted after a refused one.
    set_fingerprint_salt(DEPLOYMENT_SALT).expect("valid salt accepted");

    // 3. Write-once: a second set is refused, even with the same value.
    assert_eq!(
        set_fingerprint_salt(DEPLOYMENT_SALT),
        Err(FingerprintSaltError::AlreadyActive)
    );

    // 4. The configured salt is the one actually in effect: the fingerprint
    //    equals the documented construction under DEPLOYMENT_SALT, and no
    //    longer matches the cold-start construction (unlinkability across
    //    deployments — the leak-C7 property).
    let salted = FaultSignature::from_symptoms(vec![Symptom("event_41".into())]);
    assert_eq!(
        salted.fingerprint,
        reference(DEPLOYMENT_SALT, &["event_41"]),
        "the configured salt must be the salt in effect"
    );
    assert_ne!(
        salted.fingerprint,
        reference(common::COLD_START_FINGERPRINT_SALT, &["event_41"]),
        "a configured salt must move the fingerprint off the cold-start value"
    );
    let class = ConfigClass::from_inventory(["os:windows 11", "gpu:rtx-4070"]);
    assert_eq!(
        class.key(),
        reference(DEPLOYMENT_SALT, &["gpu:rtx-4070", "os:windows 11"]),
        "the config class derives from the same configured salt"
    );
}

/// The documented `cec-fingerprint-v2` construction (HMAC-SHA256 over the
/// versioned, count-framed, length-prefixed sorted keys), reimplemented from
/// the spec so this test does not depend on the crate's own encoder.
fn reference(salt: &[u8], sorted_keys: &[&str]) -> String {
    use hmac::{Hmac, Mac};
    use sha2::Sha256;
    let mut message = String::from("cec-fingerprint-v2\n");
    message.push_str(&format!("keys:{}\n", sorted_keys.len()));
    for key in sorted_keys {
        message.push_str(&format!("key[{}]={key}\n", key.len()));
    }
    let mut mac = Hmac::<Sha256>::new_from_slice(salt).expect("HMAC-SHA256 accepts any key length");
    mac.update(message.as_bytes());
    mac.finalize()
        .into_bytes()
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}
