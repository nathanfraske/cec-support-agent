use serde::{Deserialize, Serialize};

/// Severity of a diagnostic event, ordered from least to most urgent.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Serialize, Deserialize,
)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    #[default]
    Info,
    Warning,
    Error,
    Critical,
}

/// The kind of source a diagnostic event was collected from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EventKind {
    #[default]
    Log,
    /// Windows Error Reporting record.
    Wer,
    /// Windows Hardware Error Architecture record.
    Whea,
    /// CIM / WMI instance state.
    CimState,
    /// Windows event log entry.
    EventLog,
}

/// A single de-identified diagnostic observation gathered from a machine.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiagnosticEvent {
    /// What kind of source produced this event.
    pub kind: EventKind,
    /// Origin of the event (component, provider, or subsystem name).
    pub source: String,
    /// The de-identified message body.
    pub message: String,
    /// Severity of the event.
    pub severity: Severity,
    /// Milliseconds since the Unix epoch, as observed at collection time.
    pub timestamp_ms: u64,
}

impl DiagnosticEvent {
    /// Construct a diagnostic event.
    pub fn new(
        kind: EventKind,
        source: impl Into<String>,
        message: impl Into<String>,
        severity: Severity,
        timestamp_ms: u64,
    ) -> Self {
        Self {
            kind,
            source: source.into(),
            message: message.into(),
            severity,
            timestamp_ms,
        }
    }
}
