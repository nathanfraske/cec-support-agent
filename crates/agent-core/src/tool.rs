use async_trait::async_trait;
use common::Risk;
use serde::Deserialize;
use thiserror::Error;

/// The result of running a tool.
///
/// In-flight only: `summary` is tool prose and `data` is raw tool payload, so
/// `ToolOutcome` has no `Serialize` — it cannot be written to a row or the wire.
/// (It keeps `Deserialize` for reading tool fixtures; nothing serializes it.)
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct ToolOutcome {
    /// Whether the tool achieved its effect.
    pub ok: bool,
    /// One-line human-readable summary of what happened.
    pub summary: String,
    /// Optional structured payload (tool-specific).
    #[serde(default)]
    pub data: serde_json::Value,
}

impl ToolOutcome {
    /// A successful outcome with no structured data.
    pub fn success(summary: impl Into<String>) -> Self {
        Self {
            ok: true,
            summary: summary.into(),
            data: serde_json::Value::Null,
        }
    }

    /// A failed outcome with no structured data.
    pub fn failure(summary: impl Into<String>) -> Self {
        Self {
            ok: false,
            summary: summary.into(),
            data: serde_json::Value::Null,
        }
    }

    /// Attach a structured payload.
    pub fn with_data(mut self, data: serde_json::Value) -> Self {
        self.data = data;
        self
    }
}

/// Errors raised while dispatching or running a tool.
#[derive(Debug, Error)]
pub enum ToolError {
    /// The granted consent does not permit a tool of this risk.
    #[error("consent denied: action risk {risk:?} exceeds the granted consent")]
    ConsentDenied { risk: Risk },
    /// No tool with the requested name is registered.
    #[error("tool '{0}' not found")]
    NotFound(String),
    /// The tool ran but failed.
    #[error("tool execution failed: {0}")]
    Execution(String),
}

/// A capability the agent can invoke. Implementations declare their [`Risk`] so
/// the dispatcher can enforce the consent gate before running them.
#[async_trait]
pub trait Tool: Send + Sync {
    /// Unique name used for dispatch.
    fn name(&self) -> &str;
    /// One-line human description of the tool.
    fn description(&self) -> &str;
    /// The risk of invoking this tool; the dispatcher enforces consent on it.
    fn risk(&self) -> Risk;
    /// Run the tool with JSON arguments.
    async fn invoke(&self, args: serde_json::Value) -> Result<ToolOutcome, ToolError>;
}
