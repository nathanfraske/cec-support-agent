//! Structured symptom extraction.
//!
//! De-identification by structured extraction, not scrubbing: a fault
//! signature is built from a fixed vocabulary of fault terms, structured codes
//! (hex stop codes and WER buckets, `event`/`xid`/`bucket`-prefixed numeric
//! ids), and bare module names. Everything else in the input — hostnames,
//! usernames, paths, serials — is dropped because it never matches a rule, so
//! the signature is content-free by construction rather than by a scrub pass
//! that has to enumerate what to remove.

use crate::fault::Symptom;

/// Fault-domain vocabulary. A token is kept verbatim only when it appears
/// here. Sorted; `binary_search` relies on it (enforced by test).
const VOCABULARY: &[&str] = &[
    "bluescreen",
    "boot",
    "bsod",
    "bucket",
    "bugcheck",
    "corrupt",
    "corruption",
    "crash",
    "crashes",
    "crashing",
    "disk",
    "driver",
    "error",
    "event",
    "freeze",
    "frozen",
    "hang",
    "kernel",
    "login",
    "logon",
    "loop",
    "memory",
    "overheat",
    "power",
    "reboot",
    "restart",
    "shutdown",
    "slow",
    "smart",
    "stop",
    "tdr",
    "thermal",
    "timeout",
    "update",
    "voltage",
    "wer",
    "whea",
    "xid",
];

/// Vocabulary words that may carry a numeric id: a decimal token directly
/// after one of these is kept as `prefix_number` (e.g. `event_4101`,
/// `xid_79`, `power_41`). A bare number with no such prefix is dropped — it
/// could be anything, including part of a hostname.
const ID_PREFIXES: &[&str] = &[
    "bucket", "bugcheck", "error", "event", "id", "kernel", "power", "stop", "xid",
];

/// Module-name suffixes. A token ending in one of these is kept as a bare
/// module name (any path components were already shed by tokenization).
const MODULE_SUFFIXES: &[&str] = &[".dll", ".exe", ".sys"];

/// Extract the structured, de-identified symptoms from free text.
///
/// The result is sorted and deduplicated, so the same evidence always produces
/// the same symptom set (and therefore the same
/// [`FaultSignature`](crate::FaultSignature) fingerprint).
pub fn extract_symptoms(text: &str) -> Vec<Symptom> {
    let lowered = text.to_lowercase();
    let mut symptoms: Vec<String> = Vec::new();
    let mut previous_kept: Option<&str> = None;

    // Pre-pass on the ORIGINAL casing: bluescreen stop-code names are
    // ALL-CAPS words joined by underscores (WHEA_UNCORRECTABLE_ERROR,
    // CRITICAL_PROCESS_DIED) — a fixed Microsoft vocabulary, and exactly what
    // users are told to type verbatim. The shape only counts in the original
    // casing, so lowercase snake_case (e.g. a username inside a path) never
    // qualifies.
    for raw in text.split(|c: char| !(c.is_ascii_alphanumeric() || c == '_')) {
        let token = raw.trim_matches('_');
        if is_stop_code_name(token) {
            symptoms.push(token.to_lowercase());
        }
    }

    for raw in lowered.split(|c: char| !(c.is_ascii_alphanumeric() || c == '.' || c == '_')) {
        let token = raw.trim_matches(|c| c == '.' || c == '_');
        if token.is_empty() {
            previous_kept = None;
            continue;
        }

        if is_hex_code(token) {
            symptoms.push(token.to_string());
            previous_kept = Some(token);
        } else if let Some(module) = module_name(token) {
            symptoms.push(module.to_string());
            previous_kept = Some(module);
        } else if VOCABULARY.binary_search(&token).is_ok() {
            symptoms.push(token.to_string());
            previous_kept = Some(token);
        } else if is_decimal_id(token) {
            // A bare number is kept only in the context of an id-bearing
            // vocabulary word; otherwise it is dropped as potential identity.
            if let Some(prefix) = previous_kept.filter(|p| ID_PREFIXES.contains(p)) {
                symptoms.push(format!("{prefix}_{token}"));
            }
            previous_kept = None;
        } else {
            previous_kept = None;
        }
    }

    symptoms.sort_unstable();
    symptoms.dedup();
    symptoms.into_iter().map(Symptom).collect()
}

/// A `0x`-prefixed hex code (stop code, WER bucket, bugcheck parameter).
fn is_hex_code(token: &str) -> bool {
    token.len() > 2 && token.starts_with("0x") && token[2..].chars().all(|c| c.is_ascii_hexdigit())
}

/// A bugcheck/stop-code name in its original casing: at least two ALL-CAPS
/// segments joined by underscores (`MEMORY_MANAGEMENT`,
/// `WHEA_UNCORRECTABLE_ERROR`).
fn is_stop_code_name(token: &str) -> bool {
    token.len() >= 5
        && token.contains('_')
        && token
            .chars()
            .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || c == '_')
        && token.split('_').filter(|seg| !seg.is_empty()).count() >= 2
}

/// A bare module name (`explorer.exe`), or `None`. Tokens with interior dots
/// beyond a single `stem.suffix` (e.g. dotted hostnames) are rejected.
fn module_name(token: &str) -> Option<&str> {
    let suffix = MODULE_SUFFIXES
        .iter()
        .find(|suffix| token.ends_with(*suffix))?;
    let stem = &token[..token.len() - suffix.len()];
    if !stem.is_empty() && !stem.contains('.') {
        Some(token)
    } else {
        None
    }
}

/// A short, purely decimal token — an event id, never a serial-length number.
fn is_decimal_id(token: &str) -> bool {
    (1..=6).contains(&token.len()) && token.chars().all(|c| c.is_ascii_digit())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn strings(text: &str) -> Vec<String> {
        extract_symptoms(text).into_iter().map(|s| s.0).collect()
    }

    #[test]
    fn vocabulary_is_sorted_for_binary_search() {
        let mut sorted = VOCABULARY.to_vec();
        sorted.sort_unstable();
        assert_eq!(VOCABULARY, sorted.as_slice());
    }

    #[test]
    fn keeps_structured_evidence() {
        let symptoms = strings("explorer.exe crashes on login with WER bucket 0x1234");
        assert!(symptoms.contains(&"explorer.exe".to_string()));
        assert!(symptoms.contains(&"crashes".to_string()));
        assert!(symptoms.contains(&"login".to_string()));
        assert!(symptoms.contains(&"wer".to_string()));
        assert!(symptoms.contains(&"0x1234".to_string()));
    }

    #[test]
    fn keeps_stop_code_names_by_their_all_caps_shape() {
        let symptoms =
            strings("blue screen said WHEA_UNCORRECTABLE_ERROR then CRITICAL_PROCESS_DIED");
        assert!(symptoms.contains(&"whea_uncorrectable_error".to_string()));
        assert!(symptoms.contains(&"critical_process_died".to_string()));
        // Two-segment bugcheck names qualify too.
        assert!(strings("MEMORY_MANAGEMENT bsod").contains(&"memory_management".to_string()));
        // Lowercase snake_case (a username in a path) never qualifies.
        assert!(!strings("C:\\Users\\john_smith\\app crashed")
            .iter()
            .any(|s| s.contains("john")));
        // Mixed case does not qualify either.
        assert!(strings("told by John_Smith").is_empty());
    }

    #[test]
    fn binds_numeric_ids_to_their_prefix() {
        let symptoms = strings("Kernel-Power event 41 after Xid 79");
        assert!(symptoms.contains(&"event_41".to_string()));
        assert!(symptoms.contains(&"xid_79".to_string()));
        assert!(symptoms.contains(&"kernel".to_string()));
        assert!(symptoms.contains(&"power".to_string()));
    }

    #[test]
    fn drops_identity_bearing_text() {
        let symptoms = strings(
            "DESKTOP-NATHAN01 crash in C:\\Users\\nathan\\AppData\\app.exe, \
             contact nathan@example.com or 192.168.1.20, serial SN12345678",
        );
        let joined = symptoms.join(" ");
        assert!(!joined.contains("nathan"), "username leaked: {joined}");
        assert!(!joined.contains("desktop"), "hostname leaked: {joined}");
        assert!(!joined.contains("example"), "email leaked: {joined}");
        assert!(!joined.contains("192"), "address leaked: {joined}");
        assert!(!joined.contains("sn12345678"), "serial leaked: {joined}");
        // The structured evidence survives extraction.
        assert!(symptoms.contains(&"crash".to_string()));
        assert!(symptoms.contains(&"app.exe".to_string()));
    }

    #[test]
    fn bare_numbers_without_an_id_prefix_are_dropped() {
        assert!(strings("machine 1234 in room 42").is_empty());
    }

    #[test]
    fn dotted_hostnames_are_not_module_names() {
        assert!(strings("host01.corp.example.sys").is_empty());
    }

    #[test]
    fn is_deterministic_and_order_independent() {
        let a = extract_symptoms("crash on boot, WHEA error");
        let b = extract_symptoms("WHEA error; boot crash");
        assert_eq!(a, b);
    }
}
