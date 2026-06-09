// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 The cec-support-agent authors
//! Swarm coordination.
//!
//! The swarm fans the same diagnostics out to a set of trusted [`Generator`]
//! nodes, collects their candidate plans, and coordinates validating those
//! candidates inside disposable sandbox VMs that hold no user data
//! ([`SandboxValidator`]). The traits are the coordination surface; the host
//! supplies the concrete node transport and VM backend.

use async_trait::async_trait;
use common::{Candidate, DiagnosticEvent};
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Errors raised while coordinating the swarm.
#[derive(Debug, Error)]
pub enum SwarmError {
    /// No generators were available to handle the task.
    #[error("no trusted generators are available")]
    NoGenerators,
    /// A node rejected the dispatched task.
    #[error("node '{0}' rejected the task")]
    Rejected(String),
}

/// A trusted worker that can generate candidate plans from diagnostics.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TrustedNode {
    /// Stable node identifier.
    pub id: String,
    /// Where the node can be reached (transport-specific).
    pub endpoint: String,
}

/// Anything that can produce candidate plans for a set of diagnostics.
#[async_trait]
pub trait Generator: Send + Sync {
    /// Generate zero or more candidate plans for `events`.
    async fn generate(&self, events: &[DiagnosticEvent]) -> Result<Vec<Candidate>, SwarmError>;
}

/// The verdict from validating a candidate inside a sandbox VM with no user
/// data.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ValidationReport {
    /// The candidate plan id this report is about.
    pub candidate_id: String,
    /// Whether the plan applied cleanly in the sandbox.
    pub applied_cleanly: bool,
    /// Free-text notes from the sandbox run.
    pub notes: String,
}

/// Coordinates running a candidate in a disposable sandbox VM. The actual VM
/// backend is provided by the host; this is only the coordination surface.
#[async_trait]
pub trait SandboxValidator: Send + Sync {
    /// Validate `candidate`, returning the sandbox's verdict.
    async fn validate(&self, candidate: &Candidate) -> Result<ValidationReport, SwarmError>;
}

/// The set of trusted nodes the swarm knows about, plus the fan-out logic that
/// gathers candidates from a slate of generators.
#[derive(Debug, Default, Clone)]
pub struct Swarm {
    nodes: Vec<TrustedNode>,
}

impl Swarm {
    /// An empty swarm with no registered nodes.
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a trusted node.
    pub fn add_node(&mut self, node: TrustedNode) -> &mut Self {
        self.nodes.push(node);
        self
    }

    /// The registered trusted nodes.
    pub fn nodes(&self) -> &[TrustedNode] {
        &self.nodes
    }

    /// Fan `events` out to every generator and gather all candidate plans.
    pub async fn gather<G: Generator>(
        &self,
        generators: &[G],
        events: &[DiagnosticEvent],
    ) -> Result<Vec<Candidate>, SwarmError> {
        if generators.is_empty() {
            return Err(SwarmError::NoGenerators);
        }
        let mut candidates = Vec::new();
        for generator in generators {
            candidates.extend(generator.generate(events).await?);
        }
        Ok(candidates)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use common::{CandidateSource, Plan};

    struct OneShot;

    #[async_trait]
    impl Generator for OneShot {
        async fn generate(
            &self,
            _events: &[DiagnosticEvent],
        ) -> Result<Vec<Candidate>, SwarmError> {
            Ok(vec![Candidate::new(
                Plan::new("g1", "generated"),
                "stub",
                CandidateSource::ColdModel,
            )])
        }
    }

    #[tokio::test]
    async fn gather_collects_from_each_generator() {
        let swarm = Swarm::new();
        let generators = [OneShot, OneShot];
        let candidates = swarm.gather(&generators, &[]).await.expect("gather");
        assert_eq!(candidates.len(), 2);
    }

    #[tokio::test]
    async fn gather_without_generators_errors() {
        let swarm = Swarm::new();
        let empty: [OneShot; 0] = [];
        let result = swarm.gather(&empty, &[]).await;
        assert!(matches!(result, Err(SwarmError::NoGenerators)));
    }
}
