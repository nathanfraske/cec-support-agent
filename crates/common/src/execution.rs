use serde::{Deserialize, Serialize};

/// The outcome of running a single plan step through a tool.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StepResult {
    /// 1-based index of the step within the plan.
    pub step: usize,
    /// The action (tool name) that was run.
    pub action: String,
    /// Whether the step succeeded.
    pub ok: bool,
    /// One-line summary: the tool's own summary, or the dispatch/error message.
    pub summary: String,
}

/// The outcome of executing a whole [`Plan`](crate::Plan): a per-step record
/// plus whether the plan ran to completion.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct ExecutionResult {
    /// The id of the plan that was executed.
    pub plan_id: String,
    /// Per-step results in execution order. A run that stops early holds only
    /// the steps it reached.
    pub steps: Vec<StepResult>,
    /// Whether every step ran and succeeded. `false` if a step failed, was
    /// refused by the consent gate, or the plan was otherwise halted.
    pub completed: bool,
}

impl ExecutionResult {
    /// An empty result for `plan_id`, not yet completed.
    pub fn new(plan_id: impl Into<String>) -> Self {
        Self {
            plan_id: plan_id.into(),
            steps: Vec::new(),
            completed: false,
        }
    }

    /// Whether every recorded step succeeded (vacuously true with no steps).
    pub fn all_ok(&self) -> bool {
        self.steps.iter().all(|s| s.ok)
    }
}
