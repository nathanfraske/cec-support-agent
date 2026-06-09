// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 The cec-support-agent authors
//! Agent runtime for the support agent.
//!
//! It provides the [`Tool`] trait, a [`Dispatcher`] that enforces a [`Consent`]
//! gate before any tool runs, and a minimal [`Agent`] execution loop that
//! reaches a model through the `inference` crate. The planning, scoring, and
//! corpus stages live in the `panel`, `swarm`, and `corpus-client` crates.

mod agent;
mod consent;
mod dispatch;
mod tool;

pub use agent::Agent;
pub use consent::Consent;
pub use dispatch::Dispatcher;
pub use tool::{Tool, ToolError, ToolOutcome};

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use common::Risk;

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
}
