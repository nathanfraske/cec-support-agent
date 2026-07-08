// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 The cec-support-agent authors
//! Windows diagnostic and remediation tools.
//!
//! Every tool here compiles on all platforms so `cargo build --workspace`
//! succeeds everywhere (a bootstrap invariant). The Windows implementations
//! live behind `#[cfg(windows)]` and shell out to scoped, well-known commands
//! (`Get-CimInstance`, `Get-WinEvent`, `reg`). On non-Windows hosts each tool
//! returns an "unsupported on this platform" outcome rather than failing to
//! build.
//!
//! State-changing tools (e.g. [`RegistrySet`]) declare a non-read-only
//! [`Risk`], so the `agent-core` dispatcher gates them behind explicit consent,
//! and they capture a restore point before they mutate anything.

mod advisory;
mod catalog;

use agent_core::{Tool, ToolError, ToolOutcome};
use async_trait::async_trait;
use common::Risk;

pub use advisory::{firmware_advisory, BoardIdentity, SupportAdvisory};
pub use catalog::{
    catalog_tools, safe_core_names, CatalogEntry, CatalogTool, DESTRUCTIVE_OPS, SAFE_CORE,
};

/// All Windows tools, ready to register with an `agent_core::Dispatcher`: the six
/// hand-written tools plus the data-driven [`SAFE_CORE`] catalog. The destructive
/// ops ([`DESTRUCTIVE_OPS`]) are deliberately NOT registered here — they are a
/// tracked, differentiated list that a Windows agent wires in behind human
/// sign-off, not something the engine auto-dispatches.
pub fn windows_tools() -> Vec<Box<dyn Tool>> {
    let mut tools: Vec<Box<dyn Tool>> = vec![
        Box::new(CimQuery),
        Box::new(EventLogQuery),
        Box::new(CreateRestorePoint),
        Box::new(RegistrySet),
        Box::new(BoardInfo),
        Box::new(DownloadFile),
    ];
    tools.extend(catalog_tools());
    tools
}

#[cfg(windows)]
fn run_powershell(script: &str) -> Result<String, ToolError> {
    let output = std::process::Command::new("powershell")
        .args(["-NoProfile", "-NonInteractive", "-Command", script])
        .output()
        .map_err(|e| ToolError::Execution(format!("failed to launch powershell: {e}")))?;
    if !output.status.success() {
        // Keep only the first meaningful line: PowerShell's full error
        // rendering (position markers, CategoryInfo, stack noise) means
        // nothing to the person reading the run output.
        let stderr = String::from_utf8_lossy(&output.stderr);
        let line = stderr
            .lines()
            .map(str::trim)
            .find(|line| !line.is_empty())
            .unwrap_or("command failed with no error text")
            .to_string();
        return Err(ToolError::Execution(line));
    }
    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

/// Reject anything that is not a bare identifier, so interpolated values cannot
/// inject extra PowerShell.
#[cfg(windows)]
fn safe_identifier(value: &str) -> Result<&str, ToolError> {
    if !value.is_empty() && value.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
        Ok(value)
    } else {
        Err(ToolError::Execution(format!(
            "refusing unsafe identifier: {value:?}"
        )))
    }
}

#[cfg(windows)]
fn json_or_text(raw: String) -> serde_json::Value {
    match serde_json::from_str::<serde_json::Value>(&raw) {
        Ok(value) => value,
        Err(_) => serde_json::Value::String(raw),
    }
}

#[cfg(not(windows))]
fn unsupported(tool: &str) -> ToolOutcome {
    ToolOutcome::failure(format!("{tool} requires Windows; unsupported on this host"))
}

/// Read-only CIM/WMI instance query (`Get-CimInstance`).
pub struct CimQuery;

#[async_trait]
impl Tool for CimQuery {
    fn name(&self) -> &str {
        "cim_query"
    }
    fn description(&self) -> &str {
        "Read-only CIM/WMI instance query via Get-CimInstance."
    }
    fn risk(&self) -> Risk {
        Risk::ReadOnly
    }
    async fn invoke(&self, args: serde_json::Value) -> Result<ToolOutcome, ToolError> {
        #[cfg(windows)]
        {
            let class = args
                .get("class")
                .and_then(|v| v.as_str())
                .unwrap_or("Win32_OperatingSystem");
            let class = safe_identifier(class)?;
            let script = format!("Get-CimInstance -ClassName {class} | ConvertTo-Json -Depth 4");
            let raw = run_powershell(&script)?;
            Ok(ToolOutcome::success(format!("queried {class}")).with_data(json_or_text(raw)))
        }
        #[cfg(not(windows))]
        {
            let _ = args;
            Ok(unsupported("cim_query"))
        }
    }
}

/// Read-only Windows event log query (`Get-WinEvent`). Covers event log, WER,
/// and WHEA providers, which all surface through the unified log.
pub struct EventLogQuery;

#[async_trait]
impl Tool for EventLogQuery {
    fn name(&self) -> &str {
        "event_log_query"
    }
    fn description(&self) -> &str {
        "Read-only Windows event log query via Get-WinEvent."
    }
    fn risk(&self) -> Risk {
        Risk::ReadOnly
    }
    async fn invoke(&self, args: serde_json::Value) -> Result<ToolOutcome, ToolError> {
        #[cfg(windows)]
        {
            let log = args.get("log").and_then(|v| v.as_str()).unwrap_or("System");
            let log = safe_identifier(log)?;
            let max = args
                .get("max")
                .and_then(|v| v.as_u64())
                .unwrap_or(20)
                .min(500);
            let script = format!(
                "Get-WinEvent -LogName {log} -MaxEvents {max} | \
                 Select-Object TimeCreated, Id, LevelDisplayName, ProviderName, Message | \
                 ConvertTo-Json -Depth 3"
            );
            let raw = run_powershell(&script)?;
            Ok(
                ToolOutcome::success(format!("read {max} events from {log}"))
                    .with_data(json_or_text(raw)),
            )
        }
        #[cfg(not(windows))]
        {
            let _ = args;
            Ok(unsupported("event_log_query"))
        }
    }
}

/// A unique restore-point path for a `reg export`, in the system temp dir:
/// `cec_registry_backup_<keyhash>_<pid>_<nanos>_<seq>.reg`. Keyed to the target
/// key path (a stable FNV-1a discriminator, so two different keys can never share
/// one file) AND to this invocation (process id + a high-resolution timestamp + a
/// process-monotonic counter, so a same-key retry or a concurrent execution each
/// get their own file). This is what makes the Reversible guarantee real: every
/// registry write keeps a distinct, restorable export instead of clobbering the
/// last one's backup under a single fixed filename.
#[cfg_attr(not(windows), allow(dead_code))]
fn unique_backup_path(key: &str) -> std::path::PathBuf {
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::{SystemTime, UNIX_EPOCH};

    // Process-monotonic sequence: distinguishes two backups minted in the same
    // nanosecond (a same-key retry loop) within one process.
    static COUNTER: AtomicU64 = AtomicU64::new(0);

    // Non-cryptographic FNV-1a over the key path — carries no identity, just a
    // short fixed-width discriminator so distinct keys never collide on one file.
    let mut key_hash: u64 = 0xcbf2_9ce4_8422_2325;
    for byte in key.as_bytes() {
        key_hash ^= u64::from(*byte);
        key_hash = key_hash.wrapping_mul(0x0000_0100_0000_01b3);
    }

    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let seq = COUNTER.fetch_add(1, Ordering::Relaxed);
    std::env::temp_dir().join(format!(
        "cec_registry_backup_{key_hash:016x}_{}_{nanos}_{seq}.reg",
        std::process::id()
    ))
}

/// Set a registry value, first exporting the key as a restore point. This is a
/// reversible change: the backup is written before the value is touched, and
/// the tool aborts if the backup fails.
pub struct RegistrySet;

#[async_trait]
impl Tool for RegistrySet {
    fn name(&self) -> &str {
        "registry_set"
    }
    fn description(&self) -> &str {
        "Set a registry value after exporting the key as a backup (reversible)."
    }
    fn risk(&self) -> Risk {
        Risk::Reversible
    }
    async fn invoke(&self, args: serde_json::Value) -> Result<ToolOutcome, ToolError> {
        #[cfg(windows)]
        {
            let key = args
                .get("key")
                .and_then(|v| v.as_str())
                .ok_or_else(|| ToolError::Execution("missing 'key'".to_string()))?;
            let name = args
                .get("name")
                .and_then(|v| v.as_str())
                .ok_or_else(|| ToolError::Execution("missing 'name'".to_string()))?;
            let value = args
                .get("value")
                .and_then(|v| v.as_str())
                .ok_or_else(|| ToolError::Execution("missing 'value'".to_string()))?;

            // 1. Capture a restore point: export the key to a UNIQUE backup file
            //    before any change. A single fixed filename let a second write (a
            //    two-key plan, a same-key retry, or two concurrent serve
            //    executions) clobber the first change's restore point while still
            //    reporting success — the Reversible guarantee was silently false
            //    for every write but the last. The name is keyed to this key path
            //    and this invocation, and we refuse to reuse an existing file
            //    rather than overwrite it, so each write keeps its own export.
            let backup_path = unique_backup_path(key);
            if backup_path.exists() {
                return Err(ToolError::Execution(format!(
                    "registry backup path already exists ({}); refusing to overwrite an \
                     existing restore point",
                    backup_path.display()
                )));
            }
            let backup = backup_path.to_string_lossy().into_owned();
            // No `/y`: with a unique, non-existent target there is nothing to
            // overwrite, and dropping the force flag makes an accidental clobber
            // impossible rather than merely unlikely.
            let exported = std::process::Command::new("reg")
                .args(["export", key, backup.as_str()])
                .status()
                .map_err(|e| ToolError::Execution(format!("failed to run reg export: {e}")))?;
            if !exported.success() {
                return Err(ToolError::Execution(
                    "registry backup failed; aborting before any write".to_string(),
                ));
            }

            // 2. Apply the change only after the backup succeeded.
            let applied = std::process::Command::new("reg")
                .args(["add", key, "/v", name, "/d", value, "/f"])
                .status()
                .map_err(|e| ToolError::Execution(format!("failed to run reg add: {e}")))?;
            if !applied.success() {
                return Err(ToolError::Execution("registry write failed".to_string()));
            }

            Ok(ToolOutcome::success(format!(
                "set {key}\\{name}; restore point at {backup}"
            )))
        }
        #[cfg(not(windows))]
        {
            let _ = args;
            Ok(unsupported("registry_set"))
        }
    }
}

/// Create a System Restore point before any change (`Checkpoint-Computer`),
/// then positively verify that a checkpoint was actually created — not merely
/// requested. Reversible: it establishes a rollback target rather than
/// mutating live state, so a plan can take it before a riskier step.
///
/// Two failure modes make the verification load-bearing: System Restore may
/// be disabled entirely, and Windows silently skips creation within 24 hours
/// of the last point unless the `SystemRestorePointCreationFrequency`
/// registry override is 0. In both cases `Checkpoint-Computer` returns
/// without a new checkpoint, so "restore point created" as an unverified
/// assumption is silently false often enough to be a design defect. This tool
/// reports success only after reading the newest restore point back and
/// matching it to this request.
///
/// Coverage boundary (state it in any consent rendering): a restore point
/// covers system files, the registry, and drivers; it does not cover BIOS,
/// firmware, EC state, or user files.
pub struct CreateRestorePoint;

#[async_trait]
impl Tool for CreateRestorePoint {
    fn name(&self) -> &str {
        "create_restore_point"
    }
    fn description(&self) -> &str {
        "Create a System Restore point via Checkpoint-Computer and verify it was \
         actually created (covers system files/registry/drivers; not firmware or \
         user files)."
    }
    fn risk(&self) -> Risk {
        Risk::Reversible
    }
    async fn invoke(&self, args: serde_json::Value) -> Result<ToolOutcome, ToolError> {
        #[cfg(windows)]
        {
            let description = args
                .get("description")
                .and_then(|v| v.as_str())
                .unwrap_or("cec-support-agent checkpoint");
            // Pass the description as a single-quoted literal so it cannot break
            // out of the argument; doubling any embedded quote escapes it.
            let safe = description.replace('\'', "''");
            // Request the checkpoint, then verify it exists: the newest restore
            // point must carry this run's description. Checkpoint-Computer
            // throws if System Restore is disabled; the read-back catches the
            // silent 24-hour skip.
            let script = format!(
                "$ErrorActionPreference = 'Stop'\n\
                 Checkpoint-Computer -Description '{safe}' -RestorePointType MODIFY_SETTINGS\n\
                 $rp = Get-ComputerRestorePoint | Sort-Object SequenceNumber | Select-Object -Last 1\n\
                 if ($null -eq $rp -or $rp.Description -ne '{safe}') {{\n\
                     throw 'restore point was requested but not created (Windows skips creation within 24 hours of the last point unless SystemRestorePointCreationFrequency is 0)'\n\
                 }}\n\
                 \"sequence=$($rp.SequenceNumber)\""
            );
            let raw = run_powershell(&script)?;
            Ok(ToolOutcome::success(format!(
                "created and verified restore point: {description} ({})",
                raw.trim()
            )))
        }
        #[cfg(not(windows))]
        {
            let _ = args;
            Ok(unsupported("create_restore_point"))
        }
    }
}

/// Read-only motherboard and firmware identity. Selects configuration fields
/// only — manufacturer, product, versions — and never serial numbers, asset
/// tags, or service tags, so the payload is safe to show, log, and reason
/// over. Feeds [`BoardIdentity`] and [`firmware_advisory`].
pub struct BoardInfo;

#[async_trait]
impl Tool for BoardInfo {
    fn name(&self) -> &str {
        "board_info"
    }
    fn description(&self) -> &str {
        "Read-only motherboard, BIOS, and system model identity via CIM \
         (configuration fields only; no serial numbers)."
    }
    fn risk(&self) -> Risk {
        Risk::ReadOnly
    }
    async fn invoke(&self, args: serde_json::Value) -> Result<ToolOutcome, ToolError> {
        #[cfg(windows)]
        {
            let _ = args;
            // Selects are explicit allowlists: identity-bearing fields
            // (SerialNumber, tags) are never queried in the first place.
            let script = "$board = Get-CimInstance Win32_BaseBoard | \
                              Select-Object Manufacturer, Product, Version\n\
                          $bios = Get-CimInstance Win32_BIOS | \
                              Select-Object SMBIOSBIOSVersion, \
                              @{n='ReleaseDate';e={'{0:yyyy-MM-dd}' -f $_.ReleaseDate}}\n\
                          $system = Get-CimInstance Win32_ComputerSystem | \
                              Select-Object Manufacturer, Model\n\
                          [pscustomobject]@{ board = $board; bios = $bios; system = $system } | \
                              ConvertTo-Json -Depth 3";
            let raw = run_powershell(script)?;
            Ok(
                ToolOutcome::success("read board, BIOS, and system identity")
                    .with_data(json_or_text(raw)),
            )
        }
        #[cfg(not(windows))]
        {
            let _ = args;
            Ok(unsupported("board_info"))
        }
    }
}

/// Download a file over HTTPS into `Downloads\cec-support` and report its
/// SHA-256 and size. Reversible: the remedy is deleting the file. The tool
/// fetches and verifies only — installing or flashing what was downloaded is
/// a separate, consent-gated (or advisory-only) concern.
pub struct DownloadFile;

/// Accept only an `https://` URL made of unsurprising characters, so an
/// interpolated value cannot break out of the PowerShell argument or smuggle
/// a plaintext download.
#[cfg_attr(not(windows), allow(dead_code))]
fn validated_url(url: &str) -> Result<&str, ToolError> {
    let ok_char = |c: char| c.is_ascii_alphanumeric() || ":/.?=&%#+_-~".contains(c);
    if url.starts_with("https://") && url.len() > "https://".len() && url.chars().all(ok_char) {
        Ok(url)
    } else {
        Err(ToolError::Execution(format!(
            "refusing URL (must be https:// and plain characters): {url:?}"
        )))
    }
}

/// Accept only a bare file name — no path separators, no leading dots — so
/// the download cannot escape the dedicated folder.
#[cfg_attr(not(windows), allow(dead_code))]
fn validated_file_name(name: &str) -> Result<&str, ToolError> {
    let ok_char = |c: char| c.is_ascii_alphanumeric() || "._-".contains(c);
    if !name.is_empty() && !name.starts_with('.') && name.chars().all(ok_char) {
        Ok(name)
    } else {
        Err(ToolError::Execution(format!(
            "refusing file name (bare name only): {name:?}"
        )))
    }
}

#[async_trait]
impl Tool for DownloadFile {
    fn name(&self) -> &str {
        "download_file"
    }
    fn description(&self) -> &str {
        "Download a file over HTTPS into Downloads\\cec-support and report its \
         SHA-256 (reversible: delete the file; never installs anything)."
    }
    fn risk(&self) -> Risk {
        Risk::Reversible
    }
    async fn invoke(&self, args: serde_json::Value) -> Result<ToolOutcome, ToolError> {
        #[cfg(windows)]
        {
            let url = args
                .get("url")
                .and_then(|v| v.as_str())
                .ok_or_else(|| ToolError::Execution("missing 'url'".to_string()))?;
            let url = validated_url(url)?;
            let name = args
                .get("file_name")
                .and_then(|v| v.as_str())
                .ok_or_else(|| ToolError::Execution("missing 'file_name'".to_string()))?;
            let name = validated_file_name(name)?;

            let script = format!(
                "$ErrorActionPreference = 'Stop'\n\
                 $dir = Join-Path $env:USERPROFILE 'Downloads\\cec-support'\n\
                 New-Item -ItemType Directory -Force -Path $dir | Out-Null\n\
                 $dest = Join-Path $dir '{name}'\n\
                 Invoke-WebRequest -Uri '{url}' -OutFile $dest -MaximumRedirection 5\n\
                 $hash = (Get-FileHash -Algorithm SHA256 -Path $dest).Hash\n\
                 $bytes = (Get-Item $dest).Length\n\
                 \"path=$dest sha256=$hash bytes=$bytes\""
            );
            let raw = run_powershell(&script)?;
            Ok(ToolOutcome::success(format!(
                "downloaded {name}: {}",
                raw.trim()
            )))
        }
        #[cfg(not(windows))]
        {
            let _ = args;
            Ok(unsupported("download_file"))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builder_exposes_all_tools() {
        let tools = windows_tools();
        // The six hand-written tools plus the data-driven SAFE_CORE catalog.
        assert_eq!(tools.len(), 6 + SAFE_CORE.len());
        let names: Vec<&str> = tools.iter().map(|t| t.name()).collect();
        for hand in [
            "cim_query",
            "event_log_query",
            "create_restore_point",
            "registry_set",
            "board_info",
            "download_file",
        ] {
            assert!(names.contains(&hand), "missing hand-written tool {hand}");
        }
        // Every safe-core catalog entry is registered too.
        for e in SAFE_CORE {
            assert!(names.contains(&e.name), "missing catalog tool {}", e.name);
        }
        // Destructive ops are NOT registered (differentiated, human-gated).
        for e in DESTRUCTIVE_OPS {
            assert!(
                !names.contains(&e.name),
                "destructive op {} must not be auto-registered",
                e.name
            );
        }
    }

    #[test]
    fn download_url_validation_is_https_only_and_injection_safe() {
        assert!(validated_url("https://vendor.example/bios/PRIME-X570-PRO-4021.zip").is_ok());
        assert!(validated_url("http://vendor.example/file.zip").is_err());
        assert!(validated_url("https://").is_err());
        assert!(validated_url("https://x' ; Remove-Item -Recurse / '").is_err());
    }

    #[test]
    fn download_file_name_must_be_a_bare_name() {
        assert!(validated_file_name("bios-4021.zip").is_ok());
        assert!(validated_file_name("..\\..\\evil.exe").is_err());
        assert!(validated_file_name("a/b.zip").is_err());
        assert!(validated_file_name(".hidden").is_err());
        assert!(validated_file_name("").is_err());
    }

    #[test]
    fn board_info_is_read_only_and_download_is_reversible() {
        assert_eq!(BoardInfo.risk(), Risk::ReadOnly);
        assert_eq!(DownloadFile.risk(), Risk::Reversible);
    }

    #[test]
    fn registry_backup_paths_are_unique_per_key_and_invocation() {
        // The Reversible guarantee requires each registry write to keep its OWN
        // restore point: a single fixed filename let a second write clobber the
        // first's backup while still reporting success. Two exports of the same
        // key must land on DISTINCT files, and different keys must not share one.
        let a = unique_backup_path(r"HKLM\Software\CEC\Foo");
        let b = unique_backup_path(r"HKLM\Software\CEC\Foo");
        assert_ne!(
            a, b,
            "a same-key retry must not reuse the first write's backup file"
        );

        let c = unique_backup_path(r"HKLM\Software\CEC\Bar");
        assert_ne!(a, c, "distinct keys must not collide on one backup file");

        // Each backup lands under the tool's prefix, keeps the .reg extension a
        // re-import needs, and is never the old fixed name that caused the clobber.
        for path in [&a, &b, &c] {
            let name = path
                .file_name()
                .and_then(|n| n.to_str())
                .expect("backup has a file name");
            assert!(
                name.starts_with("cec_registry_backup_"),
                "unexpected backup name: {name}"
            );
            assert!(
                name.ends_with(".reg"),
                "backup must be a .reg export: {name}"
            );
            assert_ne!(
                name, "cec_registry_backup.reg",
                "must not use the old single fixed filename"
            );
        }
    }

    #[test]
    fn state_changing_tools_are_not_read_only() {
        assert_eq!(RegistrySet.risk(), Risk::Reversible);
        assert_eq!(CreateRestorePoint.risk(), Risk::Reversible);
        assert_eq!(CimQuery.risk(), Risk::ReadOnly);
    }

    #[cfg(not(windows))]
    #[tokio::test]
    async fn tools_report_unsupported_off_windows() {
        let outcome = CimQuery
            .invoke(serde_json::json!({ "class": "Win32_OperatingSystem" }))
            .await
            .expect("stub returns Ok");
        assert!(!outcome.ok);
        assert!(outcome.summary.contains("Windows"));
    }
}
