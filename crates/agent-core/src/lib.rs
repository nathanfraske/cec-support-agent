// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 The cec-support-agent authors
//! Agent runtime for the support agent.
//!
//! It provides the [`Tool`] trait, a [`Dispatcher`] that enforces a [`Consent`]
//! gate before any tool runs, a minimal [`Agent`] execution loop that
//! reaches a model through the `inference` crate, and the [`verify_outcome`]
//! stage that diffs a re-collected signature against the original failure
//! signature. The planning, scoring, and corpus stages live in the `panel`,
//! `swarm`, and `corpus-client` crates.

mod agent;
mod consent;
mod dispatch;
mod execute;
mod tool;
mod verify;

pub use agent::{Agent, AgentRun, AgentStep};
pub use consent::Consent;
pub use dispatch::{Dispatcher, RiskCorrection};
pub use execute::{execute_plan, execute_signed_plan};
pub use tool::{Tool, ToolError, ToolOutcome};
pub use verify::{verify_outcome, Verdict, VerificationClass};

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use common::{Plan, PlanStep, Risk};
    use inference::{
        ChatCompletionRequest, ChatCompletionResponse, ChatMessage, Choice, Completer,
        InferenceError,
    };
    use std::collections::VecDeque;
    use std::sync::Mutex;

    struct Destructive;

    #[async_trait]
    impl Tool for Destructive {
        fn name(&self) -> &str {
            "wipe"
        }
        fn description(&self) -> &str {
            "destructive test tool"
        }
        fn risk(&self) -> Risk {
            Risk::Destructive
        }
        async fn invoke(&self, _args: serde_json::Value) -> Result<ToolOutcome, ToolError> {
            Ok(ToolOutcome::success("wiped"))
        }
    }

    struct Reader;

    #[async_trait]
    impl Tool for Reader {
        fn name(&self) -> &str {
            "read"
        }
        fn description(&self) -> &str {
            "read-only test tool"
        }
        fn risk(&self) -> Risk {
            Risk::ReadOnly
        }
        async fn invoke(&self, _args: serde_json::Value) -> Result<ToolOutcome, ToolError> {
            Ok(ToolOutcome::success("data"))
        }
    }

    /// A [`Completer`] that replays a fixed script of assistant replies.
    struct ScriptedModel {
        replies: Mutex<VecDeque<String>>,
    }

    impl ScriptedModel {
        fn new(replies: &[&str]) -> Self {
            Self {
                replies: Mutex::new(replies.iter().map(|s| s.to_string()).collect()),
            }
        }
    }

    #[async_trait]
    impl Completer for ScriptedModel {
        async fn complete(
            &self,
            _request: ChatCompletionRequest,
        ) -> Result<ChatCompletionResponse, InferenceError> {
            let content = self
                .replies
                .lock()
                .expect("scripted model mutex")
                .pop_front()
                .unwrap_or_else(|| r#"{"final":""}"#.to_string());
            Ok(ChatCompletionResponse {
                model: String::new(),
                choices: vec![Choice {
                    message: ChatMessage::assistant(content),
                    finish_reason: None,
                }],
                usage: None,
            })
        }
    }

    #[test]
    fn risk_reconciliation_raises_an_understated_step_and_leaves_advisory_steps() {
        let mut dispatcher = Dispatcher::new();
        dispatcher.register(Box::new(Destructive)); // "wipe" is Destructive
        dispatcher.register(Box::new(Reader)); // "read" is ReadOnly

        let mut plan = Plan::new("p", "model plan");
        // A model mislabels the destructive tool as read-only...
        plan.steps.push(PlanStep {
            description: "just a peek".into(),
            action: "wipe".into(),
            risk: Risk::ReadOnly,
        });
        // ...an honest read-only step is correct...
        plan.steps.push(PlanStep {
            description: "look".into(),
            action: "read".into(),
            risk: Risk::ReadOnly,
        });
        // ...and an out-of-vocabulary "review" step is advisory, left untouched.
        plan.steps.push(PlanStep {
            description: "think about it".into(),
            action: "review".into(),
            risk: Risk::ReadOnly,
        });

        let (reconciled, corrections) = dispatcher.reconcile_risk(&plan);
        assert_eq!(corrections.len(), 1);
        assert_eq!(corrections[0].action, "wipe");
        assert_eq!(corrections[0].claimed, Risk::ReadOnly);
        assert_eq!(corrections[0].actual, Risk::Destructive);
        assert_eq!(reconciled.steps[0].risk, Risk::Destructive, "raised");
        assert_eq!(reconciled.steps[1].risk, Risk::ReadOnly, "honest step kept");
        assert_eq!(
            reconciled.steps[2].risk,
            Risk::ReadOnly,
            "advisory left as-is"
        );
        assert_eq!(
            reconciled.risk(),
            Risk::Destructive,
            "plan risk now reflects it"
        );
    }

    #[test]
    fn risk_reconciliation_never_lowers_a_step() {
        let mut dispatcher = Dispatcher::new();
        dispatcher.register(Box::new(Reader)); // "read" is ReadOnly
        let mut plan = Plan::new("p", "over-cautious");
        // A step claims a HIGHER risk than the tool: reconciliation never lowers.
        plan.steps.push(PlanStep {
            description: "read carefully".into(),
            action: "read".into(),
            risk: Risk::Destructive,
        });
        let (reconciled, corrections) = dispatcher.reconcile_risk(&plan);
        assert!(corrections.is_empty());
        assert_eq!(reconciled.steps[0].risk, Risk::Destructive, "not lowered");
    }

    #[tokio::test]
    async fn consent_gate_blocks_unconsented_destructive_tool() {
        let mut dispatcher = Dispatcher::new();
        dispatcher.register(Box::new(Destructive));

        let blocked = dispatcher
            .dispatch("wipe", serde_json::Value::Null, Consent::ReadOnlyOnly)
            .await;
        assert!(matches!(blocked, Err(ToolError::ConsentDenied { .. })));

        let allowed = dispatcher
            .dispatch("wipe", serde_json::Value::Null, Consent::AllowDestructive)
            .await;
        assert!(allowed.is_ok());
    }

    #[tokio::test]
    async fn dispatch_reports_unknown_tools() {
        let dispatcher = Dispatcher::new();
        let missing = dispatcher
            .dispatch("nope", serde_json::Value::Null, Consent::AllowDestructive)
            .await;
        assert!(matches!(missing, Err(ToolError::NotFound(_))));
    }

    #[tokio::test]
    async fn run_loops_through_a_tool_then_finishes() {
        let mut dispatcher = Dispatcher::new();
        dispatcher.register(Box::new(Reader));
        let model =
            ScriptedModel::new(&[r#"{"tool":"read","args":{}}"#, r#"{"final":"all good"}"#]);
        let agent = Agent::new(&model, &dispatcher, "m");

        let run = agent.run("system", "task").await.expect("run");
        assert!(run.finished);
        assert_eq!(run.answer, "all good");
        assert_eq!(run.steps.len(), 1);
        assert_eq!(run.steps[0].tool, "read");
        assert!(run.steps[0].outcome.as_ref().expect("ok outcome").ok);
    }

    #[tokio::test]
    async fn run_records_a_consent_denied_tool_then_finishes() {
        let mut dispatcher = Dispatcher::new();
        dispatcher.register(Box::new(Destructive));
        // Default consent is ReadOnlyOnly, so the destructive call is refused.
        let model = ScriptedModel::new(&[r#"{"tool":"wipe","args":{}}"#, r#"{"final":"stopped"}"#]);
        let agent = Agent::new(&model, &dispatcher, "m");

        let run = agent.run("s", "t").await.expect("run");
        assert_eq!(run.steps.len(), 1);
        assert!(run.steps[0].outcome.is_err());
        assert!(run.finished);
    }

    #[tokio::test]
    async fn run_stops_at_max_steps() {
        let mut dispatcher = Dispatcher::new();
        dispatcher.register(Box::new(Reader));
        // The model never finishes; the cap must stop the loop.
        let model = ScriptedModel::new(&[
            r#"{"tool":"read","args":{}}"#,
            r#"{"tool":"read","args":{}}"#,
            r#"{"tool":"read","args":{}}"#,
        ]);
        let agent = Agent::new(&model, &dispatcher, "m").with_max_steps(2);

        let run = agent.run("s", "t").await.expect("run");
        assert!(!run.finished);
        assert_eq!(run.steps.len(), 2);
    }

    #[tokio::test]
    async fn execute_plan_runs_read_only_to_completion() {
        let mut dispatcher = Dispatcher::new();
        dispatcher.register(Box::new(Reader));
        let mut plan = Plan::new("p", "t");
        plan.steps.push(PlanStep {
            description: "look".into(),
            action: "read".into(),
            risk: Risk::ReadOnly,
        });

        let result = execute_plan(&dispatcher, &plan, Consent::ReadOnlyOnly).await;
        assert!(result.completed);
        assert!(result.all_ok());
        assert_eq!(result.steps.len(), 1);
    }

    #[tokio::test]
    async fn execute_plan_stops_on_an_unknown_tool() {
        let dispatcher = Dispatcher::new();
        let mut plan = Plan::new("p", "t");
        plan.steps.push(PlanStep {
            description: "?".into(),
            action: "nope".into(),
            risk: Risk::ReadOnly,
        });
        plan.steps.push(PlanStep {
            description: "?".into(),
            action: "read".into(),
            risk: Risk::ReadOnly,
        });

        let result = execute_plan(&dispatcher, &plan, Consent::ReadOnlyOnly).await;
        assert!(!result.completed);
        assert_eq!(result.steps.len(), 1, "must stop at the first failing step");
        assert!(!result.steps[0].ok);
    }

    #[tokio::test]
    async fn execute_signed_plan_runs_a_genuinely_signed_plan() {
        use provenance::SigningKey;

        let mut dispatcher = Dispatcher::new();
        dispatcher.register(Box::new(Reader));
        let mut plan = Plan::new("p", "t");
        plan.steps.push(PlanStep {
            description: "look".into(),
            action: "read".into(),
            risk: Risk::ReadOnly,
        });

        let key = SigningKey::generate();
        let signed = key.sign(&plan);
        let result = execute_signed_plan(&dispatcher, &signed, &key, Consent::ReadOnlyOnly)
            .await
            .expect("valid signature");
        assert!(result.completed);
    }

    #[tokio::test]
    async fn execute_signed_plan_refuses_a_tampered_plan_before_any_tool_runs() {
        use provenance::SigningKey;

        let mut dispatcher = Dispatcher::new();
        dispatcher.register(Box::new(Reader));
        let mut plan = Plan::new("p", "t");
        plan.steps.push(PlanStep {
            description: "look".into(),
            action: "read".into(),
            risk: Risk::ReadOnly,
        });

        let key = SigningKey::generate();
        let mut signed = key.sign(&plan);
        // Tamper after signing: swap in a different action.
        signed.plan.steps[0].action = "wipe".into();
        let refused = execute_signed_plan(&dispatcher, &signed, &key, Consent::AllowDestructive)
            .await
            .expect_err("tampered plan must be refused");
        assert_eq!(refused, provenance::ProvenanceError::BadSignature);
    }

    #[test]
    fn the_dispatcher_exposes_its_operation_vocabulary() {
        let mut dispatcher = Dispatcher::new();
        dispatcher.register(Box::new(Reader));
        assert!(dispatcher.contains("read"));
        assert!(!dispatcher.contains("review"));
    }

    #[tokio::test]
    async fn execute_plan_consent_gate_halts_destructive_under_read_only() {
        let mut dispatcher = Dispatcher::new();
        dispatcher.register(Box::new(Destructive));
        let mut plan = Plan::new("p", "t");
        plan.steps.push(PlanStep {
            description: "wipe".into(),
            action: "wipe".into(),
            risk: Risk::Destructive,
        });

        let result = execute_plan(&dispatcher, &plan, Consent::ReadOnlyOnly).await;
        assert!(!result.completed);
        assert!(!result.steps[0].ok);
        assert!(result.steps[0].summary.contains("consent"));
    }
}
