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
    /// Optional structured payload (tool-specific), raw CIM among it.
    ///
    /// This is the untyped `serde_json::Value` the methodology (§2 Layer 2c)
    /// flags as the highest-fidelity raw field. It stays `Value` deliberately:
    /// Phase 1 removed `Serialize` from `ToolOutcome`, so `data` **has no path
    /// to a serialize/print sink** (a corpus row, the `--json` envelope, or a
    /// socket) — the 2c serialization boundary is already closed by the type
    /// split, so typing it into an allowlisted summary buys nothing there.
    /// Its residual exposure is the agent-loop / inference egress (leak class
    /// C2 — a model prompt built from tool output), the accepted-risk boundary
    /// handled by §3.1 / Phase 4 (`PromptPayload` + `--allow-remote-inference`),
    /// not the corpus/print sinks Phase 2 seals. `data` is consumed in-flight
    /// (parsed, e.g. `BoardIdentity::from_tool_data`), never re-emitted.
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
    /// If this tool installs software that carries an end-user license
    /// agreement, the stable id of that EULA (e.g. the product name); otherwise
    /// `None`. A step whose tool returns `Some` is REFUSED by
    /// [`crate::execute_plan`] unless the USER accepted that EULA on screen
    /// ([`crate::EulaAcceptances`]) — the engine never accepts a license on the
    /// user's behalf, so the liability stays with the user who accepted, not the
    /// shop that ran the installer. Default `None`: most tools install nothing.
    fn requires_eula(&self) -> Option<&str> {
        None
    }
    /// Run the tool with JSON arguments.
    async fn invoke(&self, args: serde_json::Value) -> Result<ToolOutcome, ToolError>;
}
