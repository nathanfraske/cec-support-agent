use serde::{Deserialize, Serialize};

/// Risk of applying a plan or step, ordered from least to most dangerous.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Serialize, Deserialize,
)]
#[serde(rename_all = "snake_case")]
pub enum Risk {
    /// Read-only inspection; makes no system state changes.
    #[default]
    ReadOnly,
    /// Reversible change, guarded by a restore point.
    Reversible,
    /// Change that is hard or impossible to reverse.
    Destructive,
}

/// A single ordered step within a [`Plan`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlanStep {
    /// Human-readable description of what the step does.
    pub description: String,
    /// The concrete action to run (e.g. a scoped cmdlet or a tool name).
    pub action: String,
    /// Risk classification for this individual step.
    pub risk: Risk,
}

/// A candidate remediation: an ordered list of steps with an overall risk.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Plan {
    /// Stable identifier for the plan within a run.
    pub id: String,
    /// Short title summarizing the remediation.
    pub title: String,
    /// Ordered steps to execute.
    pub steps: Vec<PlanStep>,
}

impl Plan {
    /// Create an empty plan with the given id and title.
    pub fn new(id: impl Into<String>, title: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            steps: Vec::new(),
        }
    }

    /// The overall risk of the plan: the maximum risk of any step.
    pub fn risk(&self) -> Risk {
        self.steps
            .iter()
            .map(|s| s.risk)
            .max()
            .unwrap_or(Risk::ReadOnly)
    }

    /// Whether the plan requires explicit consent before execution. Anything
    /// beyond read-only inspection requires consent.
    pub fn requires_consent(&self) -> bool {
        self.risk() > Risk::ReadOnly
    }
}
