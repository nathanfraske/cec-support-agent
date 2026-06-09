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

use agent_core::{Tool, ToolError, ToolOutcome};
use async_trait::async_trait;
use common::Risk;

/// All Windows tools, ready to register with an `agent_core::Dispatcher`.
pub fn windows_tools() -> Vec<Box<dyn Tool>> {
    vec![
        Box::new(CimQuery),
        Box::new(EventLogQuery),
        Box::new(RegistrySet),
    ]
}

#[cfg(windows)]
fn run_powershell(script: &str) -> Result<String, ToolError> {
    let output = std::process::Command::new("powershell")
        .args(["-NoProfile", "-NonInteractive", "-Command", script])
        .output()
        .map_err(|e| ToolError::Execution(format!("failed to launch powershell: {e}")))?;
    if !output.status.success() {
        return Err(ToolError::Execution(
            String::from_utf8_lossy(&output.stderr).trim().to_string(),
        ));
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

            // 1. Capture a restore point: export the key before any change.
            let backup = std::env::temp_dir().join("cec_registry_backup.reg");
            let backup = backup.to_string_lossy().into_owned();
            let exported = std::process::Command::new("reg")
                .args(["export", key, backup.as_str(), "/y"])
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builder_exposes_all_tools() {
        let tools = windows_tools();
        assert_eq!(tools.len(), 3);
        let names: Vec<&str> = tools.iter().map(|t| t.name()).collect();
        assert!(names.contains(&"cim_query"));
        assert!(names.contains(&"registry_set"));
    }

    #[test]
    fn state_changing_tool_is_not_read_only() {
        assert_eq!(RegistrySet.risk(), Risk::Reversible);
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
