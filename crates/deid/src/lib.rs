//! Validating de-identification mints for the corpus boundary.
//!
//! The corpus is de-identified by structured EXTRACTION, not scrubbing. But
//! extraction is only a guarantee if the values that reach a corpus row are
//! *validated* against a positive allowlist — not merely produced by a trusted
//! chokepoint. These mints are that validation: each takes a candidate value and
//! returns `Ok` only if the value is, by content, admissible. An out-of-vocabulary
//! action, a prose-bearing plan id, or a non-extractable "symptom" is rejected, so
//! a leak aborts the row instead of being copied through.
//!
//! This closes the keystone gap the leak-prevention methodology identified:
//! `de_identify_plan` historically copied `step.action` and `plan.id` through
//! VERBATIM, and the adversarial de-id test never seeded those two fields — so
//! identity placed there shipped unflagged. See `docs/corpus-leak-prevention.md`.

use common::{is_symptom_token, Symptom};

/// A value rejected by a de-id mint: it would have carried unvalidated content
/// into a corpus row. Holds only the field name and the reason — never the raw
/// offending value, so a `Reject` is itself safe to surface.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Reject {
    /// The field whose value was rejected (`"action"`, `"plan_id"`, `"symptom"`).
    pub field: &'static str,
    /// Why it was rejected.
    pub reason: &'static str,
}

impl std::fmt::Display for Reject {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "de-id rejected {}: {}", self.field, self.reason)
    }
}

impl std::error::Error for Reject {}

/// The frozen action vocabulary admissible into a corpus row: the dispatcher's
/// registered tool names plus the sanctioned advisory token `review`. MUST stay
/// sorted (binary search) and `[a-z0-9_]`-only. `support-agent` carries a drift
/// test asserting every registered dispatcher tool is a member, so the frozen
/// list and the live registry cannot silently diverge.
pub const ACTION_VOCABULARY: &[&str] = &[
    "backup_boot_config",
    "backup_registry_key",
    "backup_system",
    "board_info",
    "check_image_health",
    "cim_query",
    "compare_registry",
    "create_restore_point",
    "defender_offline_scan",
    "defender_scan",
    "defender_status",
    "defender_threats",
    "dirty_bit_query",
    "disable_device",
    "disk_check",
    "disk_details",
    "disk_health",
    "dns_lookup",
    "download_file",
    "driver_rollback",
    "event_log_query",
    "filesystem_info",
    "flush_dns",
    "list_devices",
    "list_disks",
    "list_drivers",
    "list_network_adapters",
    "list_shadow_copies",
    "memory_diagnostic",
    "network_connections",
    "open_files",
    "optimize_volume",
    "path_ping",
    "performance_report",
    "ping_host",
    "power_report",
    "process_list",
    "process_list_basic",
    "rebuild_bcd",
    "rebuild_boot_files",
    "registry_query",
    "registry_set",
    "release_ip",
    "renew_ip",
    "repair_image_health",
    "repair_system_files",
    "reset_firewall",
    "restart_network_adapter",
    "restore_registry_key",
    "review",
    "scan_image_health",
    "schedule_disk_check",
    "search_text",
    "secure_boot_status",
    "smart_data",
    "suspend_bitlocker",
    "trace_route",
    "verify_system_files",
];

/// Mint a plan-step action: it must be a member of the frozen [`ACTION_VOCABULARY`].
/// This is the keystone C1 fix — a generator that sets `action = <model prose>`
/// (or any request-derived text) is rejected here, not copied into the row.
pub fn action(value: &str) -> Result<String, Reject> {
    if ACTION_VOCABULARY.binary_search(&value).is_ok() {
        Ok(value.to_string())
    } else {
        Err(Reject {
            field: "action",
            reason: "not a member of the frozen action vocabulary",
        })
    }
}

/// Mint a plan id: a bounded lowercase slug `[a-z0-9_-]{1,40}`. This blocks the
/// realistic agent mistake `format!("...{describe}")` — spaces, colons, uppercase,
/// `@`, `/`, and backslashes all fail the charset, so a path/email/host string
/// cannot become an id. A slug that is itself a pre-lowercased identity token
/// (a bare hostname) is NOT distinguishable by charset alone; that residual is
/// closed by the frozen-prefix dictionary in a later phase (see the methodology).
pub fn plan_id(value: &str) -> Result<String, Reject> {
    let ok = (1..=40).contains(&value.len())
        && value
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-' || b == b'_');
    if ok {
        Ok(value.to_string())
    } else {
        Err(Reject {
            field: "plan_id",
            reason: "not a clean slug [a-z0-9_-]{1,40}",
        })
    }
}

/// Mint a symptom: the value must be a single, canonical member of the closed
/// de-id symptom grammar ([`common::is_symptom_token`]):
///
/// `VOCABULARY ∪ 0x-hex ∪ <known-prefix>_<digits> ∪ frozen stop-code dictionary
/// ∪ frozen OS/driver-module allowlist`
///
/// This is exactly the set the allowlisting extractor can emit as one token, so
/// a value is an admissible symptom iff the extractor itself would have produced
/// it; arbitrary identity fails because it is not a member of any list.
///
/// Phase 2 replaced the earlier round-trip check (`extract_symptoms(value) ==
/// [value]`) with the closed grammar: the round-trip form rejected a
/// legitimately-extracted `<id-prefix>_<digits>` symptom (produced from two
/// input tokens, e.g. `event_41`), which is why it could not be wired into the
/// write path in Phase 1. The grammar admits `event_41` directly, so this mint
/// now backs the write-gate symptom check and the read-side deserialization
/// guards as well as the leak-probe harness.
pub fn symptom(value: &str) -> Result<Symptom, Reject> {
    if is_symptom_token(value) {
        Ok(Symptom(value.to_string()))
    } else {
        Err(Reject {
            field: "symptom",
            reason: "not a member of the closed de-id symptom grammar",
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn action_vocabulary_is_sorted_and_charset_clean() {
        let mut sorted = ACTION_VOCABULARY.to_vec();
        sorted.sort_unstable();
        assert_eq!(
            sorted, ACTION_VOCABULARY,
            "ACTION_VOCABULARY must stay sorted for binary_search"
        );
        for a in ACTION_VOCABULARY {
            assert!(
                a.bytes()
                    .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_'),
                "{a:?} is not [a-z0-9_]"
            );
        }
    }

    #[test]
    fn action_admits_vocabulary_and_rejects_prose() {
        assert!(action("cim_query").is_ok());
        assert!(action("review").is_ok());
        assert!(action("powershell:Get-CimInstance on DESKTOP-NATHAN01").is_err());
        assert!(action("rm -rf /home/nathan").is_err());
        assert!(action("driver_rollback ").is_err()); // trailing space
        assert!(action("").is_err());
    }

    #[test]
    fn plan_id_blocks_prose_paths_and_emails() {
        assert!(plan_id("heuristic-1").is_ok());
        assert!(plan_id("model-displaydriver").is_ok());
        assert!(plan_id("fix for DESKTOP-NATHAN01").is_err()); // spaces + caps
        assert!(plan_id("c:\\users\\nathan").is_err()); // backslash + colon
        assert!(plan_id("fix-for-nathan@example.com").is_err()); // @ and .
        assert!(plan_id("").is_err());
    }

    #[test]
    fn symptom_admits_the_closed_grammar_and_rejects_identity() {
        assert!(symptom("explorer.exe").is_ok()); // module allowlist
        assert!(symptom("0x1234").is_ok()); // hex code
        assert!(symptom("crash").is_ok()); // vocabulary
        assert!(symptom("event_41").is_ok()); // <prefix>_<digits> — the Phase-1 blocker
        assert!(symptom("xid_79").is_ok());
        assert!(symptom("whea_uncorrectable_error").is_ok()); // stop-code dictionary
        assert!(symptom("desktop-nathan01").is_err()); // hostname shape
        assert!(symptom("nathan").is_err());
        assert!(symptom("boot_loop").is_err()); // not a real extracted token
        assert!(symptom("acmecorp_agent.dll").is_err()); // in-house binary
        assert!(symptom("explorer.exe crashes").is_err()); // multi-token
    }
}
