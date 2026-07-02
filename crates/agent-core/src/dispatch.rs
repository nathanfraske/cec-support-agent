use std::collections::HashMap;

use common::{Plan, Risk};

use crate::consent::Consent;
use crate::tool::{Tool, ToolError, ToolOutcome};

/// A correction the dispatcher made to a plan whose step under-stated its real
/// risk — e.g. a model labeling a `registry_set` step `ReadOnly`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RiskCorrection {
    /// The action whose claimed risk was raised.
    pub action: String,
    /// The risk the plan claimed.
    pub claimed: Risk,
    /// The action's true risk, taken from the registered tool.
    pub actual: Risk,
}

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

    /// Whether a tool with this name is registered. The registered set is the
    /// agent's operation vocabulary: a plan step whose action is not in it is
    /// advisory-only and cannot be executed.
    pub fn contains(&self, name: &str) -> bool {
        self.tools.contains_key(name)
    }

    /// The declared risk of a registered tool, or `None` if it is not in the
    /// vocabulary.
    pub fn risk_of(&self, name: &str) -> Option<Risk> {
        self.tools.get(name).map(|tool| tool.risk())
    }

    /// Reconcile a (possibly model-generated) plan's claimed risks against the
    /// real risk of each registered tool. A model cannot mislabel a
    /// state-changing action as `ReadOnly` to slip it past the consent gate or
    /// understate it in the rendered consent: any step that under-states its
    /// tool's risk is RAISED to the true risk (never lowered), and the
    /// correction is reported. Out-of-vocabulary steps are left untouched — they
    /// are advisory-only and never executed anyway.
    pub fn reconcile_risk(&self, plan: &Plan) -> (Plan, Vec<RiskCorrection>) {
        let mut corrected = plan.clone();
        let mut corrections = Vec::new();
        for step in &mut corrected.steps {
            if let Some(actual) = self.risk_of(&step.action) {
                if step.risk < actual {
                    corrections.push(RiskCorrection {
                        action: step.action.clone(),
                        claimed: step.risk,
                        actual,
                    });
                    step.risk = actual;
                }
            }
        }
        (corrected, corrections)
    }

    /// A stable, model-readable catalog of registered tools — one
    /// `- name (risk: …): description` line each, sorted so the prompt the
    /// agent loop builds is deterministic.
    pub fn catalog(&self) -> String {
        let mut lines: Vec<String> = self
            .tools
            .values()
            .map(|tool| {
                format!(
                    "- {} (risk: {:?}): {}",
                    tool.name(),
                    tool.risk(),
                    tool.description()
                )
            })
            .collect();
        lines.sort();
        lines.join("\n")
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
