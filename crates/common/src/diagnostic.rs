use serde::{Deserialize, Serialize};

use crate::Prose;

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

/// A single diagnostic observation gathered from a machine.
///
/// An **in-flight** type: `message` is the raw observation body (request text in
/// the bootstrap), so `DiagnosticEvent` has no `Serialize` — it is reduced to a
/// [`crate::FaultSignature`] by `extract_symptoms` before anything is stored or
/// emitted.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiagnosticEvent {
    /// What kind of source produced this event.
    pub kind: EventKind,
    /// Origin of the event (component, provider, or subsystem name).
    pub source: String,
    /// The raw observation body. Free-text prose (request text in the
    /// bootstrap), so it is [`Prose`]: it is reduced to a
    /// [`FaultSignature`](crate::FaultSignature) by `extract_symptoms` before
    /// anything is stored or emitted, and cannot reach a sink directly.
    pub message: Prose,
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
        message: impl Into<Prose>,
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
