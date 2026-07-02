use serde::{Deserialize, Serialize};

use crate::Prose;

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
///
/// An **in-flight** type: it carries free-text prose and therefore has no
/// `Serialize` — it cannot reach a corpus row or any serialize boundary. The
/// de-identified, serializable counterpart is [`corpus-client`'s `StoredStep`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlanStep {
    /// Human-readable description of what the step does. Free-text prose (a
    /// generator often echoes the request into it), so it is [`Prose`]: it has
    /// no `Serialize`/`Display` and cannot reach a sink.
    pub description: Prose,
    /// The concrete action to run (a registered tool name). Not prose — it is
    /// validated against the frozen action vocabulary before it reaches a row.
    pub action: String,
    /// Risk classification for this individual step.
    pub risk: Risk,
}

/// A candidate remediation: an ordered list of steps with an overall risk.
///
/// An **in-flight** type: `title` (and each step's `description`) is free-text
/// prose, so `Plan` has no `Serialize` — a raw plan cannot be written to a
/// corpus row, printed, or put on the `--json`/API wire. It reaches a row only
/// via `de_identify_plan`, which mints the serializable `StoredPlan`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Plan {
    /// Stable identifier for the plan within a run. Not prose — validated to a
    /// bounded slug before it reaches a row.
    pub id: String,
    /// Short title summarizing the remediation. Free-text prose, so it is
    /// [`Prose`]: it has no `Serialize`/`Display`. `de_identify_plan` drops it
    /// entirely and reconstructs the stored title from the action vocabulary.
    pub title: Prose,
    /// Ordered steps to execute.
    pub steps: Vec<PlanStep>,
}

impl Plan {
    /// Create an empty plan with the given id and title.
    pub fn new(id: impl Into<String>, title: impl Into<Prose>) -> Self {
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
