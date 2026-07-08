//! Authoring normalization: turn a staff-authored workflow into the canonical
//! de-identified corpus format, with a structural read-back so the author can
//! confirm that all their intent was captured with **no ambiguity** before the
//! workflow enters the gated submit path.
//!
//! This is the write-side complement to `docs/workflow-authoring-guide.md`. Shop
//! staff author in their own vocabulary; the corpus stores only the canonical,
//! de-identified action sequence (the frozen [`deid::ACTION_VOCABULARY`]). The gap
//! between the two is exactly where intent can be silently lost — an authored step
//! that maps to no registered action, or maps only after normalization. This
//! module makes that gap VISIBLE and forces the author to resolve it, rather than
//! dropping a step quietly.
//!
//! **What is actually stored.** The corpus row keeps the ordered canonical
//! *actions* and their risks — NOT the free-text descriptions (the de-id boundary
//! strips those). So "intent captured" means: the canonical action sequence
//! faithfully represents the author's workflow. The read-back states this plainly,
//! so the author confirms the sequence, not the prose.
//!
//! This is a PREVIEW/authoring aid. It is not the trust boundary: the gate
//! (`Contribution::new` + `ensure_evidence_integrity`) re-mints and re-validates
//! every action on submit, so a page/tool that got normalization wrong cannot
//! sneak content past admission. The normalizer just lets an author see and fix
//! the mapping first.

use common::{Plan, PlanStep, Risk};
use serde::{Deserialize, Serialize};

/// A single staff-authored step, in shop vocabulary / free text.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuthoredStep {
    /// What the step does and why, in the author's words (advisory — NOT stored
    /// on the corpus row; the de-id boundary keeps only the canonical action).
    pub description: String,
    /// The authored action phrase (e.g. `"driver_rollback"`, `"Roll back driver"`,
    /// or `"display_driver_uninstaller"`).
    pub action: String,
    /// The step's risk, as the author classified it.
    pub risk: Risk,
}

/// A staff-authored fix workflow, before normalization.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuthoredWorkflow {
    /// A proposed plan id (a slug). If it is not a clean slug the report flags it.
    pub id: String,
    /// The ordered steps.
    pub steps: Vec<AuthoredStep>,
}

/// How one authored step's action resolved against the frozen action vocabulary.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "resolution")]
pub enum StepResolution {
    /// The authored action is EXACTLY a canonical vocabulary action — stored as-is.
    Clean {
        /// The canonical action.
        action: String,
    },
    /// The authored action mapped to a canonical action only after normalization
    /// (lowercasing, spaces/hyphens → underscores). Surfaced in the read-back so
    /// the author confirms the mapping is what they meant, not a coincidence.
    Normalized {
        /// The author's original phrasing.
        authored: String,
        /// The canonical action it normalized to.
        action: String,
    },
    /// The authored action is NOT a registered action, so it cannot be stored.
    /// The author must pick a real one (or the action names a tool not yet
    /// implemented — e.g. a display-driver clean-uninstall — which this flags for
    /// the engine team). `suggestions` are vocabulary actions sharing a word with
    /// the authored phrase (possibly empty).
    Unmapped {
        /// The author's original phrasing.
        authored: String,
        /// Candidate canonical actions to choose from (may be empty).
        suggestions: Vec<String>,
    },
}

impl StepResolution {
    /// The canonical action this step will store, if it resolved cleanly.
    pub fn canonical(&self) -> Option<&str> {
        match self {
            StepResolution::Clean { action } | StepResolution::Normalized { action, .. } => {
                Some(action)
            }
            StepResolution::Unmapped { .. } => None,
        }
    }
}

/// The read-back for one step: what was authored, what will be stored, and whether
/// it resolved.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StepReadback {
    /// 1-based step position.
    pub index: usize,
    /// The author's description (advisory — not stored).
    pub description: String,
    /// The step's risk.
    pub risk: Risk,
    /// How the action resolved.
    pub resolution: StepResolution,
}

/// The normalization result for a whole workflow: the per-step read-back plus
/// whether the workflow is clean enough to submit.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NormalizationReport {
    /// The proposed plan id, and whether it is a clean slug.
    pub id: String,
    /// Whether `id` is a valid de-identified plan-id slug.
    pub id_is_clean: bool,
    /// Per-step read-back.
    pub steps: Vec<StepReadback>,
}

impl NormalizationReport {
    /// Whether every step resolved to a canonical action AND the id is clean — the
    /// workflow can be turned into a plan and submitted with no ambiguity.
    pub fn is_clean(&self) -> bool {
        self.id_is_clean
            && self
                .steps
                .iter()
                .all(|s| s.resolution.canonical().is_some())
    }

    /// The steps that still need the author's resolution (unmapped actions).
    pub fn unresolved(&self) -> Vec<&StepReadback> {
        self.steps
            .iter()
            .filter(|s| matches!(s.resolution, StepResolution::Unmapped { .. }))
            .collect()
    }

    /// A plain-language read-back for author confirmation — one line per step,
    /// stating exactly what will be STORED (the canonical action), plus a leading
    /// note that descriptions are not stored. An unresolved step is called out.
    pub fn readback_lines(&self) -> Vec<String> {
        let mut lines = vec![
            "The corpus will store ONLY the ordered actions below (your descriptions \
             are for review and are not stored). Confirm this sequence captures the \
             workflow:"
                .to_string(),
        ];
        if !self.id_is_clean {
            lines.push(format!(
                "  ⚠ id \"{}\" is not a clean slug [a-z0-9_-]{{1,40}} — rename it.",
                self.id
            ));
        }
        for step in &self.steps {
            let line = match &step.resolution {
                StepResolution::Clean { action } => {
                    format!("  {}. {} [{:?}]", step.index, action, step.risk)
                }
                StepResolution::Normalized { authored, action } => format!(
                    "  {}. {} [{:?}]  (from \"{}\" — confirm)",
                    step.index, action, step.risk, authored
                ),
                StepResolution::Unmapped {
                    authored,
                    suggestions,
                } => {
                    let hint = if suggestions.is_empty() {
                        "no registered action matches — pick one, or this tool is not \
                         implemented yet"
                            .to_string()
                    } else {
                        format!("did you mean: {}", suggestions.join(", "))
                    };
                    format!(
                        "  {}. ⚠ UNRESOLVED \"{}\" [{:?}] — {}",
                        step.index, authored, step.risk, hint
                    )
                }
            };
            lines.push(line);
        }
        lines
    }

    /// The canonical in-flight plan, IF the workflow is clean. The plan's actions
    /// are the resolved canonical tokens; descriptions are set to the action (the
    /// de-id boundary keeps no prose), so `Contribution::new`/`de_identify_plan`
    /// accept it idempotently. `None` when anything is unresolved — an ambiguous
    /// or unmapped workflow cannot become a plan.
    pub fn to_plan(&self) -> Option<Plan> {
        if !self.is_clean() {
            return None;
        }
        let mut plan = Plan::new(self.id.clone(), String::new());
        for step in &self.steps {
            let action = step.resolution.canonical()?.to_string();
            plan.steps.push(PlanStep {
                description: action.clone().into(),
                action,
                risk: step.risk,
            });
        }
        Some(plan)
    }
}

/// Normalize a value to the canonical action shape: trim, lowercase, and map
/// spaces and hyphens to underscores (so `"Roll back driver"` → `"roll_back_driver"`,
/// `"driver-rollback"` → `"driver_rollback"`).
fn normalize_action(value: &str) -> String {
    value
        .trim()
        .to_lowercase()
        .chars()
        .map(|c| if c == ' ' || c == '-' { '_' } else { c })
        .collect()
}

/// Vocabulary actions that share a word with the authored phrase — hints for an
/// unmapped step. A "word" is an alphanumeric run of length ≥ 3 in the authored
/// phrase; a vocabulary action is suggested if it contains that word.
fn suggestions_for(authored: &str) -> Vec<String> {
    let words: Vec<String> = authored
        .to_lowercase()
        .split(|c: char| !c.is_ascii_alphanumeric())
        .filter(|w| w.len() >= 3)
        .map(str::to_string)
        .collect();
    deid::ACTION_VOCABULARY
        .iter()
        .filter(|action| words.iter().any(|w| action.contains(w.as_str())))
        .map(|a| a.to_string())
        .collect()
}

/// Resolve one authored action against the frozen vocabulary.
fn resolve_action(authored: &str) -> StepResolution {
    if deid::ACTION_VOCABULARY.binary_search(&authored).is_ok() {
        return StepResolution::Clean {
            action: authored.to_string(),
        };
    }
    let normalized = normalize_action(authored);
    if deid::ACTION_VOCABULARY
        .binary_search(&normalized.as_str())
        .is_ok()
    {
        return StepResolution::Normalized {
            authored: authored.to_string(),
            action: normalized,
        };
    }
    StepResolution::Unmapped {
        authored: authored.to_string(),
        suggestions: suggestions_for(authored),
    }
}

/// Normalize a staff-authored workflow into a [`NormalizationReport`]: map each
/// step's action to the frozen vocabulary, flag anything unresolved, and produce
/// the read-back for author confirmation. Deterministic — no model, no I/O.
pub fn normalize_workflow(workflow: &AuthoredWorkflow) -> NormalizationReport {
    let id_is_clean = deid::plan_id(&workflow.id).is_ok();
    let steps = workflow
        .steps
        .iter()
        .enumerate()
        .map(|(i, step)| StepReadback {
            index: i + 1,
            description: step.description.clone(),
            risk: step.risk,
            resolution: resolve_action(&step.action),
        })
        .collect();
    NormalizationReport {
        id: workflow.id.clone(),
        id_is_clean,
        steps,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn step(description: &str, action: &str, risk: Risk) -> AuthoredStep {
        AuthoredStep {
            description: description.into(),
            action: action.into(),
            risk,
        }
    }

    #[test]
    fn an_exact_vocabulary_action_is_clean() {
        let wf = AuthoredWorkflow {
            id: "driver-fix".into(),
            steps: vec![step("look at the board", "board_info", Risk::ReadOnly)],
        };
        let report = normalize_workflow(&wf);
        assert!(report.is_clean());
        assert!(matches!(
            report.steps[0].resolution,
            StepResolution::Clean { .. }
        ));
    }

    #[test]
    fn casing_and_spaces_normalize_with_a_confirmable_note() {
        let wf = AuthoredWorkflow {
            id: "driver-fix".into(),
            steps: vec![step("roll it back", "Driver Rollback", Risk::Reversible)],
        };
        let report = normalize_workflow(&wf);
        assert!(report.is_clean(), "normalized to a real action");
        match &report.steps[0].resolution {
            StepResolution::Normalized { authored, action } => {
                assert_eq!(authored, "Driver Rollback");
                assert_eq!(action, "driver_rollback");
            }
            other => panic!("expected Normalized, got {other:?}"),
        }
    }

    #[test]
    fn an_unregistered_action_is_flagged_not_dropped() {
        // The DDU case: `display_driver_uninstaller` is not a registered tool yet,
        // so it must be FLAGGED (surfacing the missing-tool gap), never silently
        // dropped — and the workflow is not clean.
        let wf = AuthoredWorkflow {
            id: "gpu-swap".into(),
            steps: vec![
                step(
                    "clean uninstall",
                    "display_driver_uninstaller",
                    Risk::Destructive,
                ),
                step("reinstall driver", "driver_rollback", Risk::Reversible),
            ],
        };
        let report = normalize_workflow(&wf);
        assert!(!report.is_clean(), "an unmapped step blocks submission");
        assert_eq!(
            report.to_plan(),
            None,
            "cannot become a plan while unresolved"
        );
        let unresolved = report.unresolved();
        assert_eq!(unresolved.len(), 1);
        assert_eq!(unresolved[0].index, 1);
    }

    #[test]
    fn an_unmapped_action_suggests_vocabulary_by_shared_word() {
        let wf = AuthoredWorkflow {
            id: "driver-fix".into(),
            steps: vec![step(
                "undo the driver",
                "roll back the driver",
                Risk::Reversible,
            )],
        };
        let report = normalize_workflow(&wf);
        match &report.steps[0].resolution {
            StepResolution::Unmapped { suggestions, .. } => {
                assert!(
                    suggestions.contains(&"driver_rollback".to_string()),
                    "shared word 'driver' should suggest driver_rollback: {suggestions:?}"
                );
            }
            other => panic!("expected Unmapped, got {other:?}"),
        }
    }

    #[test]
    fn a_non_slug_id_is_flagged() {
        let wf = AuthoredWorkflow {
            id: "Fix For The Shop PC".into(), // spaces + caps → not a slug
            steps: vec![step("x", "board_info", Risk::ReadOnly)],
        };
        let report = normalize_workflow(&wf);
        assert!(!report.id_is_clean);
        assert!(!report.is_clean(), "a non-slug id blocks submission");
    }

    #[test]
    fn a_clean_workflow_produces_a_submittable_plan() {
        let wf = AuthoredWorkflow {
            id: "restore-and-check".into(),
            steps: vec![
                step("snapshot first", "create_restore_point", Risk::Reversible),
                step("check the logs", "event_log_query", Risk::ReadOnly),
            ],
        };
        let report = normalize_workflow(&wf);
        assert!(report.is_clean());
        let plan = report.to_plan().expect("clean workflow yields a plan");
        assert_eq!(plan.steps.len(), 2);
        assert_eq!(plan.steps[0].action, "create_restore_point");
        assert_eq!(plan.steps[1].action, "event_log_query");
        // The read-back names what is stored and flags nothing.
        let lines = report.readback_lines();
        assert!(lines.iter().all(|l| !l.contains("UNRESOLVED")));
    }
}
