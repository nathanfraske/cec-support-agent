//! Structured symptom extraction.
//!
//! De-identification by structured extraction, not scrubbing: a fault
//! signature is built from a fixed vocabulary of fault terms, structured codes
//! (hex stop codes and WER buckets, `event`/`xid`/`bucket`-prefixed numeric
//! ids), a FROZEN dictionary of Windows bugcheck/stop-code names, and a FROZEN
//! allowlist of OS/driver module names. Everything else in the input —
//! hostnames, usernames, paths, serials, asset tags, in-house binaries — is
//! dropped because it is not a member of a closed list, so the signature is
//! content-free by construction rather than by a scrub pass that has to
//! enumerate what to remove.
//!
//! The two former SHAPE heuristics (`is_stop_code_name` kept any
//! `ALL_CAPS_UNDERSCORE` token; `module_name` kept any `stem.exe`) were
//! denylists-of-shape, not dictionaries: they admitted asset tags, AD groups,
//! and custom binaries by shape (`docs/corpus-leak-prevention.md` §2 Layer
//! 1c/C5). They are replaced here by [`STOP_CODE_NAMES`] and [`MODULE_NAMES`],
//! curated and conservative — a missing entry is a false-negative (a real
//! symptom dropped), an over-broad entry is a leak, so the bias is toward
//! genuinely OS/Microsoft-published names only.

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

/// FROZEN Windows bugcheck / stop-code names, in Microsoft's canonical
/// `ALL_CAPS_UNDERSCORE` casing. A crash report names one of these verbatim and
/// users are told to type it exactly, so it is genuine, low-cardinality
/// evidence — unlike an arbitrary `ALL_CAPS_UNDERSCORE` token (an asset tag, an
/// AD group, a service name), which is identity. Membership is the allowlist;
/// the shape is not. Conservative and Microsoft-published only. MUST stay
/// sorted (binary search) — enforced by test.
const STOP_CODE_NAMES: &[&str] = &[
    "APC_INDEX_MISMATCH",
    "ATTEMPTED_EXECUTE_OF_NOEXECUTE_MEMORY",
    "ATTEMPTED_WRITE_TO_READONLY_MEMORY",
    "BAD_POOL_CALLER",
    "BAD_POOL_HEADER",
    "BAD_SYSTEM_CONFIG_INFO",
    "CLOCK_WATCHDOG_TIMEOUT",
    "CRITICAL_PROCESS_DIED",
    "CRITICAL_STRUCTURE_CORRUPTION",
    "DPC_WATCHDOG_VIOLATION",
    "DRIVER_IRQL_NOT_LESS_OR_EQUAL",
    "DRIVER_OVERRAN_STACK_BUFFER",
    "DRIVER_POWER_STATE_FAILURE",
    "DRIVER_VERIFIER_DETECTED_VIOLATION",
    "FAT_FILE_SYSTEM",
    "FAULTY_HARDWARE_CORRUPTED_PAGE",
    "HAL_INITIALIZATION_FAILED",
    "HYPERVISOR_ERROR",
    "INACCESSIBLE_BOOT_DEVICE",
    "IRQL_NOT_LESS_OR_EQUAL",
    "KERNEL_DATA_INPAGE_ERROR",
    "KERNEL_MODE_HEAP_CORRUPTION",
    "KERNEL_SECURITY_CHECK_FAILURE",
    "KMODE_EXCEPTION_NOT_HANDLED",
    "MACHINE_CHECK_EXCEPTION",
    "MEMORY_MANAGEMENT",
    "NTFS_FILE_SYSTEM",
    "PAGE_FAULT_IN_NONPAGED_AREA",
    "PFN_LIST_CORRUPT",
    "PROCESS1_INITIALIZATION_FAILED",
    "REFERENCE_BY_POINTER",
    "SYSTEM_PTE_MISUSE",
    "SYSTEM_SERVICE_EXCEPTION",
    "SYSTEM_THREAD_EXCEPTION_NOT_HANDLED",
    "THREAD_STUCK_IN_DEVICE_DRIVER",
    "UNEXPECTED_KERNEL_MODE_TRAP",
    "UNMOUNTABLE_BOOT_VOLUME",
    "VIDEO_DXGKRNL_FATAL_ERROR",
    "VIDEO_TDR_FAILURE",
    "VIDEO_TDR_TIMEOUT_DETECTED",
    "WHEA_UNCORRECTABLE_ERROR",
    "WORKER_THREAD_RETURNED_AT_BAD_IRQL",
];

/// FROZEN OS / driver module-name allowlist (lowercase filenames). A crash
/// implicates one of these Windows kernel/OS/GPU-driver binaries; an arbitrary
/// `stem.exe` — a custom or in-house binary (`acmecorp_agent.dll`), a
/// user-named executable — is NOT a symptom, it is identity. Membership is the
/// allowlist; the `.exe/.dll/.sys` suffix is not. Conservative: only
/// well-known Windows/vendor modules. MUST stay sorted — enforced by test.
const MODULE_NAMES: &[&str] = &[
    "acpi.sys",
    "afd.sys",
    "atikmdag.sys",
    "audiodg.exe",
    "bthport.sys",
    "bthusb.sys",
    "classpnp.sys",
    "cng.sys",
    "combase.dll",
    "conhost.exe",
    "csrss.exe",
    "ctfmon.exe",
    "disk.sys",
    "dllhost.exe",
    "dwm.exe",
    "dxgkrnl.sys",
    "dxgmms1.sys",
    "dxgmms2.sys",
    "explorer.exe",
    "fastfat.sys",
    "fltmgr.sys",
    "fontdrvhost.exe",
    "gdi32.dll",
    "hal.dll",
    "http.sys",
    "iastor.sys",
    "iastorac.sys",
    "igdkmd64.sys",
    "kernel32.dll",
    "kernelbase.dll",
    "ksecdd.sys",
    "lsass.exe",
    "msrpc.sys",
    "msvcrt.dll",
    "ndis.sys",
    "netio.sys",
    "ntdll.dll",
    "ntfs.sys",
    "ntoskrnl.exe",
    "nvlddmkm.sys",
    "nvme.sys",
    "ole32.dll",
    "pci.sys",
    "rundll32.exe",
    "runtimebroker.exe",
    "searchindexer.exe",
    "services.exe",
    "sihost.exe",
    "smss.exe",
    "spoolsv.exe",
    "storahci.sys",
    "storport.sys",
    "svchost.exe",
    "taskhostw.exe",
    "tcpip.sys",
    "usbhub.sys",
    "usbxhci.sys",
    "user32.dll",
    "volmgr.sys",
    "volsnap.sys",
    "watchdog.sys",
    "wdf01000.sys",
    "win32k.sys",
    "win32kbase.sys",
    "win32kfull.sys",
    "wininit.exe",
    "winlogon.exe",
    "wuauclt.exe",
];

/// Extract the structured, de-identified symptoms from free text.
///
/// The result is sorted and deduplicated, so the same evidence always produces
/// the same symptom set (and therefore the same
/// [`FaultSignature`](crate::FaultSignature) fingerprint). Every token it emits
/// is a member of the closed grammar ([`is_symptom_token`]) — the extractor and
/// the grammar are the same allowlist, so a value round-trips iff the extractor
/// itself would have produced it.
pub fn extract_symptoms(text: &str) -> Vec<Symptom> {
    let lowered = text.to_lowercase();
    let mut symptoms: Vec<String> = Vec::new();
    let mut previous_kept: Option<&str> = None;

    for raw in lowered.split(|c: char| !(c.is_ascii_alphanumeric() || c == '.' || c == '_')) {
        let token = raw.trim_matches(|c| c == '.' || c == '_');
        if token.is_empty() {
            previous_kept = None;
            continue;
        }

        if is_hex_code(token) {
            symptoms.push(token.to_string());
            previous_kept = Some(token);
        } else if MODULE_NAMES.binary_search(&token).is_ok() {
            symptoms.push(token.to_string());
            previous_kept = None;
        } else if is_stop_code_token(token) {
            // A bugcheck name is a whole token in its own right; it never
            // prefixes a following number.
            symptoms.push(token.to_string());
            previous_kept = None;
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

/// Whether `token` is a single, canonical (lowercase) member of the closed de-id
/// symptom grammar:
///
/// `VOCABULARY ∪ 0x-hex ∪ <known-prefix>_<digits> ∪ STOP_CODE_NAMES ∪
/// MODULE_NAMES`
///
/// This is the exact set [`extract_symptoms`] can emit as a single token, so it
/// is the membership predicate the de-id symptom mint ([`crate`] re-exported to
/// `deid::symptom`) and the read-side deserialization guards check: a stored or
/// served symptom is admissible iff it is something the extractor itself would
/// have produced. An identity-shaped token (`desktop-nathan01`, an asset tag, a
/// custom binary) is not a member and is refused.
pub fn is_symptom_token(token: &str) -> bool {
    is_hex_code(token)
        || VOCABULARY.binary_search(&token).is_ok()
        || MODULE_NAMES.binary_search(&token).is_ok()
        || is_stop_code_token(token)
        || is_prefixed_id(token)
}

/// A `0x`-prefixed hex code (stop code, WER bucket, bugcheck parameter).
fn is_hex_code(token: &str) -> bool {
    token.len() > 2 && token.starts_with("0x") && token[2..].chars().all(|c| c.is_ascii_hexdigit())
}

/// Whether `token` (in the canonical lowercase symptom form) names a member of
/// the frozen [`STOP_CODE_NAMES`] dictionary. Bugcheck names are published in
/// `ALL_CAPS`; the stored/extracted symptom form is lowercase, so probe the
/// dictionary with an uppercase copy. Case-insensitive against a closed list is
/// strictly safer than the old ALL-CAPS *shape* rule — a lowercase username in a
/// path (`john_smith`) is not in the list and is refused regardless of casing.
fn is_stop_code_token(token: &str) -> bool {
    // Fast reject: bugcheck names are ASCII letters, digits, and underscores.
    if token.is_empty()
        || !token
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'_')
    {
        return false;
    }
    let upper = token.to_ascii_uppercase();
    STOP_CODE_NAMES.binary_search(&upper.as_str()).is_ok()
}

/// A `<known-prefix>_<digits>` id (`event_41`, `xid_79`, `power_41`): a
/// vocabulary id-prefix, an underscore, then a short decimal id. This is the one
/// grammar member the extractor produces from TWO input tokens, so it does not
/// round-trip as a single token through `extract_symptoms` — the grammar admits
/// it directly.
fn is_prefixed_id(token: &str) -> bool {
    match token.split_once('_') {
        Some((prefix, digits)) => ID_PREFIXES.contains(&prefix) && is_decimal_id(digits),
        None => false,
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
    fn dictionaries_are_sorted_for_binary_search() {
        let mut stop = STOP_CODE_NAMES.to_vec();
        stop.sort_unstable();
        assert_eq!(STOP_CODE_NAMES, stop.as_slice(), "STOP_CODE_NAMES unsorted");
        let mut modules = MODULE_NAMES.to_vec();
        modules.sort_unstable();
        assert_eq!(MODULE_NAMES, modules.as_slice(), "MODULE_NAMES unsorted");
        // The module allowlist is lowercase; the stop-code dictionary uppercase.
        for m in MODULE_NAMES {
            assert_eq!(*m, m.to_lowercase(), "module {m:?} must be lowercase");
        }
        for s in STOP_CODE_NAMES {
            assert_eq!(*s, s.to_uppercase(), "stop code {s:?} must be uppercase");
        }
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
    fn keeps_stop_code_names_from_the_frozen_dictionary() {
        let symptoms =
            strings("blue screen said WHEA_UNCORRECTABLE_ERROR then CRITICAL_PROCESS_DIED");
        assert!(symptoms.contains(&"whea_uncorrectable_error".to_string()));
        assert!(symptoms.contains(&"critical_process_died".to_string()));
        // Two-segment bugcheck names in the dictionary qualify too.
        assert!(strings("MEMORY_MANAGEMENT bsod").contains(&"memory_management".to_string()));
        // Lowercase snake_case (a username in a path) is not in the dictionary.
        assert!(!strings("C:\\Users\\john_smith\\ntoskrnl crashed")
            .iter()
            .any(|s| s.contains("john")));
        // A made-up ALL-CAPS token is NOT a bugcheck name — the C5 fix: the old
        // shape heuristic kept any ALL_CAPS_UNDERSCORE token (asset tags, AD
        // groups). The dictionary refuses it.
        assert!(strings("asset RIG_NATHAN_DESK failed").is_empty());
        assert!(strings("group DOMAIN_ADMINS_EAST").is_empty());
    }

    #[test]
    fn keeps_only_allowlisted_module_names() {
        // A real OS/driver module survives.
        assert!(strings("fault in nvlddmkm.sys during tdr").contains(&"nvlddmkm.sys".to_string()));
        // A custom / in-house binary by shape does NOT — the C5 fix: the old
        // rule kept any `stem.exe/.dll/.sys`, so `acmecorp_agent.dll` (an
        // in-house binary name = identity) or a user-named exe rode through.
        assert!(strings("acmecorp_agent.dll loaded").is_empty());
        assert!(strings("C:\\Users\\nathan\\app.exe crashed")
            .iter()
            .all(|s| s != "app.exe"));
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
        // The structured evidence survives extraction; the non-allowlisted
        // `app.exe` does NOT (it is a potential in-house/user binary name).
        assert!(symptoms.contains(&"crash".to_string()));
        assert!(!symptoms.contains(&"app.exe".to_string()));
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
    fn is_symptom_token_matches_what_the_extractor_emits() {
        // Every member class round-trips through the grammar predicate.
        for token in [
            "crash",
            "0x1234",
            "event_41",
            "xid_79",
            "explorer.exe",
            "ntoskrnl.exe",
            "whea_uncorrectable_error",
            "memory_management",
        ] {
            assert!(is_symptom_token(token), "{token:?} should be a member");
        }
        // Identity shapes are refused.
        for token in [
            "desktop-nathan01",
            "nathan",
            "rig_nathan_desk",
            "acmecorp_agent.dll",
            "app.exe",
            "sn12345678",
            "boot_loop",
            "explorer.exe crashes",
        ] {
            assert!(!is_symptom_token(token), "{token:?} must not be a member");
        }
        // Everything the extractor emits is a grammar member (self-consistency).
        for s in extract_symptoms(
            "explorer.exe crashes on login, WER bucket 0x1234, Kernel-Power event 41, \
             WHEA_UNCORRECTABLE_ERROR, nvlddmkm.sys tdr",
        ) {
            assert!(
                is_symptom_token(&s.0),
                "extractor emitted a non-grammar token: {:?}",
                s.0
            );
        }
    }

    #[test]
    fn is_deterministic_and_order_independent() {
        let a = extract_symptoms("crash on boot, WHEA error");
        let b = extract_symptoms("WHEA error; boot crash");
        assert_eq!(a, b);
    }
}
