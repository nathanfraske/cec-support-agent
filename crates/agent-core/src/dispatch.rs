use std::collections::HashMap;

use crate::consent::Consent;
use crate::tool::{Tool, ToolError, ToolOutcome};

/// Holds registered tools and enforces the consent gate before dispatch.
#[derive(Default)]
pub struct Dispatcher {
    tools: HashMap<String, Box<dyn Tool>>,
}

impl Dispatcher {
    /// An empty dispatcher.
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a tool, keyed by its [`Tool::name`].
    pub fn register(&mut self, tool: Box<dyn Tool>) -> &mut Self {
        self.tools.insert(tool.name().to_string(), tool);
        self
    }

    /// Names of all registered tools.
    pub fn tool_names(&self) -> Vec<&str> {
        self.tools.keys().map(String::as_str).collect()
    }

    /// Dispatch a tool by name. Refuses to run when `consent` does not permit
    /// the tool's declared risk — the consent gate is enforced here, not at the
    /// call site.
    pub async fn dispatch(
        &self,
        name: &str,
        args: serde_json::Value,
        consent: Consent,
    ) -> Result<ToolOutcome, ToolError> {
        let tool = self
            .tools
            .get(name)
            .ok_or_else(|| ToolError::NotFound(name.to_string()))?;
        let risk = tool.risk();
        if !consent.permits(risk) {
            return Err(ToolError::ConsentDenied { risk });
        }
        tool.invoke(args).await
    }
}
