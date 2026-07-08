// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 The cec-support-agent authors
//! The safe-core tool catalog: the built-in Windows diagnostic/repair primitives
//! the engine can plan and dispatch, expressed as DATA (plain-language, de-jargoned)
//! rather than one hand-written struct per tool.
//!
//! Two lists, deliberately DIFFERENTIATED:
//! - [`SAFE_CORE`] — read-only and reversible primitives. These are registered as
//!   real dispatcher tools ([`catalog_tools`]) and their names are members of the
//!   frozen `deid::ACTION_VOCABULARY`, so the engine may plan, store, and (on a
//!   Windows target) run them.
//! - [`DESTRUCTIVE_OPS`] — the data-erasing / can-brick primitives. They are NOT
//!   registered and NOT in the action vocabulary: the engine cannot silently plan
//!   or auto-run one. They are catalogued here (with the correct risk and the
//!   reversal/caution) so they are tracked and visible, and so that WIRING one in
//!   later is a deliberate act, gated behind human sign-off (the corpus's
//!   `DestructiveFixNeedsHuman` rule already exists for exactly this).
//!
//! **Windows execution is a drop-in** (owner rule: leave Windows-box work to a
//! Windows agent). An entry marked [`CatalogEntry::runnable_as_is`] carries a
//! COMPLETE, injection-safe fixed command and runs directly on a Windows host; an
//! entry that needs arguments declares its contract and returns a clear "wire
//! validated argument handling" outcome until a Windows agent implements it (see
//! `docs/safe-core-windows-execution-playbook.md`). On non-Windows hosts every
//! catalog tool returns "unsupported", matching the hand-written tools.

use agent_core::{Tool, ToolError, ToolOutcome};
use async_trait::async_trait;
use common::Risk;

/// One catalogued primitive. Fields are plain-language on purpose — `title` and
/// `summary` are what a shop tech or the owner reads, not raw command jargon.
#[derive(Debug, Clone, Copy)]
pub struct CatalogEntry {
    /// The de-identified action token (a `deid::ACTION_VOCABULARY` member for
    /// [`SAFE_CORE`]). Lowercase `[a-z0-9_]`.
    pub name: &'static str,
    /// Plain-language name a person reads (de-jargoned).
    pub title: &'static str,
    /// What it does, in plain language.
    pub summary: &'static str,
    /// The actual Windows invocation (the spec the Windows drop-in implements).
    pub command: &'static str,
    /// Risk class — drives the consent/sign-off gate.
    pub risk: Risk,
    /// Coarse grouping for display.
    pub category: &'static str,
    /// True when `command` is a COMPLETE, argument-free, injection-safe command
    /// that can run as-is on a Windows host. False when the primitive needs
    /// validated arguments (a target host, a drive, a registry key) — those
    /// declare their contract and defer execution to the Windows drop-in.
    pub runnable_as_is: bool,
    /// For [`DESTRUCTIVE_OPS`] only: how to undo it (or why it is irreversible) /
    /// what to snapshot first. Empty for [`SAFE_CORE`].
    pub reversal_note: &'static str,
}

/// SAFE CORE — read-only and reversible built-in Windows primitives. Registered as
/// dispatcher tools; every `name` here is in `deid::ACTION_VOCABULARY`. Excludes
/// the six hand-written tools already registered (cim_query, event_log_query,
/// create_restore_point, registry_set, board_info, download_file).
pub const SAFE_CORE: &[CatalogEntry] = &[
    // --- Storage & disk ---
    CatalogEntry { name: "disk_check", title: "Check a disk for errors (read-only)", summary: "Scans a drive's file system for logical and physical errors without changing anything.", command: "chkdsk <drive>:", risk: Risk::ReadOnly, category: "storage", runnable_as_is: false, reversal_note: "" },
    CatalogEntry { name: "disk_health", title: "Report drive health", summary: "Lists each physical drive with its health status (Healthy / Warning / Unhealthy).", command: "Get-PhysicalDisk | ConvertTo-Json -Depth 3", risk: Risk::ReadOnly, category: "storage", runnable_as_is: true, reversal_note: "" },
    CatalogEntry { name: "smart_data", title: "Read drive SMART data", summary: "Reads a drive's reliability counters: temperature, power-on hours, wear, and error counts.", command: "Get-PhysicalDisk | Get-StorageReliabilityCounter | ConvertTo-Json -Depth 3", risk: Risk::ReadOnly, category: "storage", runnable_as_is: true, reversal_note: "" },
    CatalogEntry { name: "filesystem_info", title: "Show file-system details", summary: "Reports drive type, volume properties, NTFS metadata locations, and sector sizes.", command: "fsutil fsinfo sectorinfo <drive>:", risk: Risk::ReadOnly, category: "storage", runnable_as_is: false, reversal_note: "" },
    CatalogEntry { name: "dirty_bit_query", title: "Check a drive's 'needs-check' flag", summary: "Reports whether a volume is marked dirty (which schedules a disk check at next restart).", command: "fsutil dirty query <drive>:", risk: Risk::ReadOnly, category: "storage", runnable_as_is: false, reversal_note: "" },
    CatalogEntry { name: "list_disks", title: "List disks and partitions", summary: "Enumerates disks, partitions, and volumes with their sizes and layout.", command: "diskpart > list disk / list volume", risk: Risk::ReadOnly, category: "storage", runnable_as_is: false, reversal_note: "" },
    CatalogEntry { name: "disk_details", title: "Show a disk's details", summary: "Displays the detailed properties of a selected disk, partition, or volume.", command: "diskpart > detail disk", risk: Risk::ReadOnly, category: "storage", runnable_as_is: false, reversal_note: "" },
    CatalogEntry { name: "optimize_volume", title: "Optimize a drive (defrag / TRIM)", summary: "Defragments an HDD or issues TRIM to an SSD; a maintenance operation that does not erase data.", command: "Optimize-Volume -DriveLetter <X> -Defrag", risk: Risk::Reversible, category: "storage", runnable_as_is: false, reversal_note: "" },
    CatalogEntry { name: "schedule_disk_check", title: "Schedule a startup disk check", summary: "Displays or changes which drives are checked automatically at startup.", command: "chkntfs /c <drive>:", risk: Risk::Reversible, category: "storage", runnable_as_is: false, reversal_note: "" },

    // --- Windows OS integrity ---
    CatalogEntry { name: "verify_system_files", title: "Verify system files (no repair)", summary: "Scans protected Windows system files for corruption without changing anything.", command: "sfc /verifyonly", risk: Risk::ReadOnly, category: "os_integrity", runnable_as_is: true, reversal_note: "" },
    CatalogEntry { name: "repair_system_files", title: "Repair system files", summary: "Scans and repairs corrupted protected Windows system files from a known-good source.", command: "sfc /scannow", risk: Risk::Reversible, category: "os_integrity", runnable_as_is: true, reversal_note: "" },
    CatalogEntry { name: "check_image_health", title: "Check Windows image health", summary: "Reports whether the Windows component store is healthy, repairable, or not — no repair.", command: "DISM /Online /Cleanup-Image /CheckHealth", risk: Risk::ReadOnly, category: "os_integrity", runnable_as_is: true, reversal_note: "" },
    CatalogEntry { name: "scan_image_health", title: "Scan Windows image for corruption", summary: "Scans the Windows component store for corruption and logs what it finds — no repair.", command: "DISM /Online /Cleanup-Image /ScanHealth", risk: Risk::ReadOnly, category: "os_integrity", runnable_as_is: true, reversal_note: "" },
    CatalogEntry { name: "repair_image_health", title: "Repair the Windows image", summary: "Repairs corrupted or missing Windows components using Windows Update or a given source.", command: "DISM /Online /Cleanup-Image /RestoreHealth", risk: Risk::Reversible, category: "os_integrity", runnable_as_is: true, reversal_note: "" },

    // --- Drivers ---
    CatalogEntry { name: "list_devices", title: "List devices", summary: "Enumerates devices with their status, drivers, and any problem codes.", command: "pnputil /enum-devices", risk: Risk::ReadOnly, category: "drivers", runnable_as_is: true, reversal_note: "" },
    CatalogEntry { name: "list_drivers", title: "List installed driver packages", summary: "Lists third-party driver packages in the Windows driver store.", command: "pnputil /enum-drivers", risk: Risk::ReadOnly, category: "drivers", runnable_as_is: true, reversal_note: "" },
    CatalogEntry { name: "disable_device", title: "Disable a device", summary: "Disables a specific device to isolate or work around problematic hardware.", command: "pnputil /disable-device <instance-id>", risk: Risk::Reversible, category: "drivers", runnable_as_is: false, reversal_note: "" },

    // --- Memory / hardware ---
    CatalogEntry { name: "memory_diagnostic", title: "Test system memory", summary: "Schedules the built-in Windows Memory Diagnostic to test RAM at the next restart.", command: "mdsched.exe", risk: Risk::ReadOnly, category: "memory_hardware", runnable_as_is: false, reversal_note: "" },

    // --- Boot / recovery ---
    CatalogEntry { name: "backup_boot_config", title: "Back up boot configuration", summary: "Exports the Boot Configuration Data (BCD) store to a file so it can be restored.", command: "bcdedit /export <path>", risk: Risk::ReadOnly, category: "boot", runnable_as_is: false, reversal_note: "" },
    CatalogEntry { name: "rebuild_bcd", title: "Rebuild the boot menu", summary: "Scans the disks for Windows installations and rebuilds the boot configuration store.", command: "bootrec /rebuildbcd", risk: Risk::Reversible, category: "boot", runnable_as_is: true, reversal_note: "" },
    CatalogEntry { name: "rebuild_boot_files", title: "Rebuild boot files (modern UEFI)", summary: "Recreates the EFI System Partition boot files and BCD store — the modern boot-repair path.", command: "bcdboot C:\\Windows", risk: Risk::Reversible, category: "boot", runnable_as_is: false, reversal_note: "" },

    // --- Networking ---
    CatalogEntry { name: "flush_dns", title: "Clear the DNS cache", summary: "Clears the local DNS resolver cache to fix stale name-resolution problems.", command: "ipconfig /flushdns", risk: Risk::Reversible, category: "networking", runnable_as_is: true, reversal_note: "" },
    CatalogEntry { name: "release_ip", title: "Release the network address", summary: "Releases the current DHCP-assigned IP address.", command: "ipconfig /release", risk: Risk::Reversible, category: "networking", runnable_as_is: true, reversal_note: "" },
    CatalogEntry { name: "renew_ip", title: "Renew the network address", summary: "Requests a fresh IP address from the DHCP server.", command: "ipconfig /renew", risk: Risk::Reversible, category: "networking", runnable_as_is: true, reversal_note: "" },
    CatalogEntry { name: "trace_route", title: "Trace the network route", summary: "Traces the network path to a destination, hop by hop, to locate where traffic stops.", command: "tracert <target>", risk: Risk::ReadOnly, category: "networking", runnable_as_is: false, reversal_note: "" },
    CatalogEntry { name: "path_ping", title: "Measure path latency and loss", summary: "Reports latency and packet loss at each hop between here and a destination.", command: "pathping <target>", risk: Risk::ReadOnly, category: "networking", runnable_as_is: false, reversal_note: "" },
    CatalogEntry { name: "ping_host", title: "Ping a host", summary: "Sends echo requests to confirm basic network reachability to a host.", command: "ping <target>", risk: Risk::ReadOnly, category: "networking", runnable_as_is: false, reversal_note: "" },
    CatalogEntry { name: "dns_lookup", title: "Look up a DNS name", summary: "Queries DNS servers to diagnose name-resolution problems.", command: "nslookup <hostname>", risk: Risk::ReadOnly, category: "networking", runnable_as_is: false, reversal_note: "" },
    CatalogEntry { name: "network_connections", title: "Show network connections", summary: "Displays active connections, listening ports, and per-connection owning process.", command: "netstat -ano", risk: Risk::ReadOnly, category: "networking", runnable_as_is: true, reversal_note: "" },
    CatalogEntry { name: "list_network_adapters", title: "List network adapters", summary: "Lists network adapters and their status and properties.", command: "Get-NetAdapter | ConvertTo-Json -Depth 3", risk: Risk::ReadOnly, category: "networking", runnable_as_is: true, reversal_note: "" },
    CatalogEntry { name: "restart_network_adapter", title: "Restart a network adapter", summary: "Disables and re-enables a network adapter to clear a stuck connection.", command: "Restart-NetAdapter -Name <adapter>", risk: Risk::Reversible, category: "networking", runnable_as_is: false, reversal_note: "" },
    CatalogEntry { name: "reset_firewall", title: "Reset the firewall to defaults", summary: "Resets Windows Firewall policy to defaults; the built-in export makes it restorable.", command: "netsh advfirewall reset", risk: Risk::Reversible, category: "networking", runnable_as_is: true, reversal_note: "" },

    // --- Security / malware ---
    CatalogEntry { name: "defender_scan", title: "Run a malware scan", summary: "Runs a Microsoft Defender scan of the system or a chosen path.", command: "Start-MpScan -ScanType QuickScan", risk: Risk::ReadOnly, category: "security", runnable_as_is: true, reversal_note: "" },
    CatalogEntry { name: "defender_offline_scan", title: "Run an offline malware scan", summary: "Reboots into a trusted environment and scans for rootkits and boot-record malware.", command: "Start-MpWDOScan", risk: Risk::Reversible, category: "security", runnable_as_is: true, reversal_note: "" },
    CatalogEntry { name: "defender_status", title: "Show antivirus status", summary: "Reports Defender's protection state, real-time protection, and signature freshness.", command: "Get-MpComputerStatus | ConvertTo-Json -Depth 3", risk: Risk::ReadOnly, category: "security", runnable_as_is: true, reversal_note: "" },
    CatalogEntry { name: "defender_threats", title: "List detected threats", summary: "Lists malware threats Defender has detected, past and active.", command: "Get-MpThreatDetection | ConvertTo-Json -Depth 3", risk: Risk::ReadOnly, category: "security", runnable_as_is: true, reversal_note: "" },
    CatalogEntry { name: "suspend_bitlocker", title: "Suspend drive encryption", summary: "Temporarily suspends BitLocker (the required safe step before clearing the TPM).", command: "manage-bde -protectors -disable <drive>", risk: Risk::Reversible, category: "security", runnable_as_is: false, reversal_note: "" },

    // --- Performance ---
    CatalogEntry { name: "power_report", title: "Generate a power/energy report", summary: "Analyzes power settings and battery health and writes a report.", command: "powercfg /energy", risk: Risk::ReadOnly, category: "performance", runnable_as_is: true, reversal_note: "" },
    CatalogEntry { name: "performance_report", title: "Generate a performance report", summary: "Collects performance counters and produces a system diagnostics report.", command: "perfmon /report", risk: Risk::ReadOnly, category: "performance", runnable_as_is: true, reversal_note: "" },

    // --- Firmware / Secure Boot (read-only checks; writes are in DESTRUCTIVE_OPS) ---
    CatalogEntry { name: "secure_boot_status", title: "Check Secure Boot", summary: "Reports whether Secure Boot is enabled on a UEFI system.", command: "Confirm-SecureBootUEFI", risk: Risk::ReadOnly, category: "firmware", runnable_as_is: true, reversal_note: "" },

    // --- Registry (read + reversible; delete is in DESTRUCTIVE_OPS) ---
    CatalogEntry { name: "registry_query", title: "Read a registry key", summary: "Reads registry subkeys and values, with optional recursive search.", command: "reg query <key> [/s]", risk: Risk::ReadOnly, category: "registry", runnable_as_is: false, reversal_note: "" },
    CatalogEntry { name: "backup_registry_key", title: "Back up a registry key", summary: "Exports a registry key to a file so it can be restored after edits.", command: "reg export <key> <file>", risk: Risk::ReadOnly, category: "registry", runnable_as_is: false, reversal_note: "" },
    CatalogEntry { name: "restore_registry_key", title: "Restore a registry key", summary: "Restores registry keys/values from a previously exported file.", command: "reg import <file>", risk: Risk::Reversible, category: "registry", runnable_as_is: false, reversal_note: "" },
    CatalogEntry { name: "compare_registry", title: "Compare registry keys", summary: "Compares two registry locations to find differences.", command: "reg compare <key1> <key2>", risk: Risk::ReadOnly, category: "registry", runnable_as_is: false, reversal_note: "" },

    // --- Backup / restore ---
    CatalogEntry { name: "backup_system", title: "Back up the system", summary: "Creates an image, volume, file, or system-state backup.", command: "wbadmin start backup -backupTarget:<loc> -include:<vols>", risk: Risk::Reversible, category: "backup", runnable_as_is: false, reversal_note: "" },
    CatalogEntry { name: "list_shadow_copies", title: "List restore snapshots", summary: "Lists Volume Shadow Copy snapshots available for restore.", command: "vssadmin list shadows", risk: Risk::ReadOnly, category: "backup", runnable_as_is: true, reversal_note: "" },

    // --- Process / handles / text ---
    CatalogEntry { name: "process_list", title: "List running processes (detailed)", summary: "Lists running processes with handle, thread, memory, and module detail, as JSON.", command: "Get-CimInstance Win32_Process | ConvertTo-Json -Depth 4", risk: Risk::ReadOnly, category: "process", runnable_as_is: true, reversal_note: "" },
    CatalogEntry { name: "process_list_basic", title: "List running processes (basic)", summary: "Lists running processes in CSV form (built-in, no dependencies).", command: "tasklist /v /fo csv", risk: Risk::ReadOnly, category: "process", runnable_as_is: true, reversal_note: "" },
    CatalogEntry { name: "open_files", title: "List open files", summary: "Lists files currently held open, locally or from the network.", command: "openfiles /query /fo csv /v", risk: Risk::ReadOnly, category: "process", runnable_as_is: true, reversal_note: "" },
    CatalogEntry { name: "search_text", title: "Search text in files", summary: "Searches files for a literal or regex pattern (built-in log/text triage).", command: "findstr /s /i /n <pattern> <files>", risk: Risk::ReadOnly, category: "process", runnable_as_is: false, reversal_note: "" },
];

/// DESTRUCTIVE OPS — the data-erasing / can-brick primitives. DIFFERENTIATED from
/// the safe core: NOT registered as tools and NOT in `deid::ACTION_VOCABULARY`, so
/// the engine cannot silently plan or auto-run one. Catalogued (with correct risk
/// and the reversal/caution) so they are tracked and so that wiring one in later
/// is a deliberate act gated behind human sign-off.
pub const DESTRUCTIVE_OPS: &[CatalogEntry] = &[
    CatalogEntry { name: "wipe_disk", title: "Wipe a disk", summary: "Erases the partition table (and with 'all', zero-fills every sector) of a disk.", command: "diskpart > clean [all]", risk: Risk::Destructive, category: "storage", runnable_as_is: false, reversal_note: "IRREVERSIBLE — all data is destroyed. Confirm the correct disk and that data is backed up." },
    CatalogEntry { name: "repair_disk", title: "Repair a disk (chkdsk /r)", summary: "Repairs file-system errors and recovers bad sectors — repair mode can orphan or truncate files.", command: "chkdsk <drive>: /r", risk: Risk::Destructive, category: "storage", runnable_as_is: false, reversal_note: "Can lose data while repairing. Back up first; run read-only 'disk_check' before deciding." },
    CatalogEntry { name: "cleanup_component_store", title: "Clean up the Windows image", summary: "Deletes superseded Windows components; those specific updates can no longer be uninstalled.", command: "DISM /Online /Cleanup-Image /StartComponentCleanup [/ResetBase]", risk: Risk::Destructive, category: "os_integrity", runnable_as_is: false, reversal_note: "Removes update rollback capability. Not reversible without reinstalling the affected updates." },
    CatalogEntry { name: "remove_driver", title: "Remove a driver package", summary: "Permanently removes a driver package from the store, optionally ripping it off live devices.", command: "pnputil /delete-driver <oem#.inf> [/uninstall /force]", risk: Risk::Destructive, category: "drivers", runnable_as_is: false, reversal_note: "Keep the original driver package to reinstall. /force can leave hardware without a driver." },
    CatalogEntry { name: "clean_gpu_drivers", title: "Clean-remove GPU drivers (DDU)", summary: "Fully strips graphics drivers and leftovers (the GPU-swap / driver-corruption case).", command: "Display Driver Uninstaller — Safe Mode — Clean and restart", risk: Risk::Destructive, category: "drivers", runnable_as_is: false, reversal_note: "High-effort recovery: stage the GPU driver installer BEFORE running; no display output until reinstalled." },
    CatalogEntry { name: "set_secure_boot", title: "Change Secure Boot keys", summary: "Writes Secure Boot key material (PK/KEK/DB/DBX).", command: "Set-SecureBootUEFI ...", risk: Risk::Destructive, category: "firmware", runnable_as_is: false, reversal_note: "A bad write can make the machine unbootable with no simple recovery. Expert-only." },
    CatalogEntry { name: "clear_tpm", title: "Clear the security chip (TPM)", summary: "Invalidates all TPM-sealed keys.", command: "tpm.msc — Clear TPM", risk: Risk::Destructive, category: "security", runnable_as_is: false, reversal_note: "SUSPEND BitLocker first ('suspend_bitlocker') or you cause a recovery-key lockout = permanent data loss." },
    CatalogEntry { name: "reset_tcpip", title: "Reset the TCP/IP stack", summary: "Rewrites the TCP/IP registry subsystem to defaults.", command: "netsh int ip reset", risk: Risk::Destructive, category: "networking", runnable_as_is: false, reversal_note: "Windows makes NO automatic backup — export the config first; needs a reboot." },
    CatalogEntry { name: "reset_winsock", title: "Reset the network sockets catalog", summary: "Resets the Winsock catalog and strips custom network layers (LSPs).", command: "netsh winsock reset", risk: Risk::Destructive, category: "networking", runnable_as_is: false, reversal_note: "Only reversible if 'netsh winsock dump' was captured beforehand; needs a reboot." },
    CatalogEntry { name: "fix_master_boot_record", title: "Rewrite the master boot record (legacy)", summary: "Writes the MBR/boot sector — legacy BIOS only; a no-op or error on modern UEFI systems.", command: "bootrec /fixmbr | /fixboot", risk: Risk::Destructive, category: "boot", runnable_as_is: false, reversal_note: "Writes boot structures; back up the BCD first. Prefer 'rebuild_boot_files' (bcdboot) on UEFI." },
    CatalogEntry { name: "delete_registry_key", title: "Delete a registry key", summary: "Permanently removes registry subkeys or values.", command: "reg delete <key> /f", risk: Risk::Destructive, category: "registry", runnable_as_is: false, reversal_note: "Back up first with 'backup_registry_key' (reg export); deletion has no undo." },
    CatalogEntry { name: "reset_pc", title: "Reset this PC", summary: "Reinstalls Windows; can keep files or remove everything depending on the option chosen.", command: "systemreset  (Settings > Recovery > Reset this PC)", risk: Risk::Destructive, category: "recovery", runnable_as_is: false, reversal_note: "'Remove everything' wipes the device. Confirm the option and back up first." },
    CatalogEntry { name: "flash_firmware", title: "Update firmware / BIOS", summary: "Flashes system or device firmware (via vendor tool or Windows UpdateCapsule).", command: "vendor tool (e.g. dcu-cli /applyUpdates) or UpdateCapsule", risk: Risk::Destructive, category: "firmware", runnable_as_is: false, reversal_note: "Power loss mid-flash can BRICK the board. Ensure stable power; have the recovery procedure ready." },
];

/// A data-driven [`Tool`] backed by a [`CatalogEntry`]. Its identity, description,
/// and risk come from the entry; execution runs the entry's fixed command on a
/// Windows host when [`CatalogEntry::runnable_as_is`], and otherwise declares its
/// contract for the Windows drop-in.
pub struct CatalogTool(pub &'static CatalogEntry);

#[async_trait]
impl Tool for CatalogTool {
    fn name(&self) -> &str {
        self.0.name
    }
    fn description(&self) -> &str {
        self.0.summary
    }
    fn risk(&self) -> Risk {
        self.0.risk
    }
    async fn invoke(&self, args: serde_json::Value) -> Result<ToolOutcome, ToolError> {
        let _ = args;
        #[cfg(windows)]
        {
            if self.0.runnable_as_is {
                let raw = crate::run_powershell(self.0.command)?;
                return Ok(ToolOutcome::success(format!("ran {}", self.0.name))
                    .with_data(crate::json_or_text(raw)));
            }
            // Needs validated arguments — the Windows agent wires arg handling per
            // docs/safe-core-windows-execution-playbook.md. Failing here is safe:
            // an unwired arg-taking tool never runs an unvalidated command.
            Err(ToolError::Execution(format!(
                "{} needs argument wiring (command: {}) — see the safe-core Windows execution playbook",
                self.0.name, self.0.command
            )))
        }
        #[cfg(not(windows))]
        {
            Ok(crate::unsupported(self.0.name))
        }
    }
}

/// The safe-core catalog as registrable dispatcher tools (one [`CatalogTool`] per
/// [`SAFE_CORE`] entry).
pub fn catalog_tools() -> Vec<Box<dyn Tool>> {
    SAFE_CORE
        .iter()
        .map(|e| Box::new(CatalogTool(e)) as Box<dyn Tool>)
        .collect()
}

/// Every safe-core action name (for the `deid::ACTION_VOCABULARY` drift check).
pub fn safe_core_names() -> Vec<&'static str> {
    SAFE_CORE.iter().map(|e| e.name).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn names_are_clean_tokens_and_unique() {
        let mut seen = HashSet::new();
        for e in SAFE_CORE.iter().chain(DESTRUCTIVE_OPS) {
            assert!(
                e.name
                    .bytes()
                    .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_'),
                "{:?} is not a clean [a-z0-9_] token",
                e.name
            );
            assert!(seen.insert(e.name), "duplicate catalog name {:?}", e.name);
        }
    }

    #[test]
    fn safe_core_is_never_destructive() {
        for e in SAFE_CORE {
            assert_ne!(
                e.risk,
                Risk::Destructive,
                "{:?} is in SAFE_CORE but is Destructive — move it to DESTRUCTIVE_OPS",
                e.name
            );
        }
    }

    #[test]
    fn destructive_ops_are_all_destructive_and_carry_a_reversal_note() {
        for e in DESTRUCTIVE_OPS {
            assert_eq!(
                e.risk,
                Risk::Destructive,
                "{:?} must be Destructive",
                e.name
            );
            assert!(
                !e.reversal_note.is_empty(),
                "destructive op {:?} must document its reversal/caution",
                e.name
            );
        }
    }

    #[test]
    fn the_two_lists_are_disjoint() {
        let safe: HashSet<_> = SAFE_CORE.iter().map(|e| e.name).collect();
        for e in DESTRUCTIVE_OPS {
            assert!(
                !safe.contains(e.name),
                "{:?} is in BOTH lists — a destructive op must not be in the safe core",
                e.name
            );
        }
    }

    #[test]
    fn every_catalog_entry_has_plain_language_text() {
        for e in SAFE_CORE.iter().chain(DESTRUCTIVE_OPS) {
            assert!(
                !e.title.is_empty() && !e.summary.is_empty(),
                "{:?} needs a title+summary",
                e.name
            );
        }
    }
}
