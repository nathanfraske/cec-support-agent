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

#[async_trait]
impl<T: Generator + ?Sized> Generator for Box<T> {
    async fn generate(&self, events: &[DiagnosticEvent]) -> Result<Vec<Candidate>, SwarmError> {
        (**self).generate(events).await
    }
}

/// The escalation evidence from applying a candidate inside a sandbox VM with no
/// user data. Deliberately NOT a corpus verdict: it reports only whether the
/// plan applied cleanly, never a post-fix signature. The corpus verdict is a
/// separate quantity, computed from a real re-collection on the real target —
/// see the [`SandboxValidator`] contract. Do not extend this type with a
/// post-state signature or a pass/fail verdict; that would blur the two.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ValidationReport {
    /// The candidate plan id this report is about.
    pub candidate_id: String,
    /// Whether the plan applied cleanly in the sandbox. Escalation evidence
    /// only — a clean apply LOWERS a reversible software-state plan's bar; it is
    /// never a `Verdict::Pass` and never mints a resolved corpus row.
    pub applied_cleanly: bool,
    /// Free-text notes from the sandbox run.
    pub notes: String,
}

/// Coordinates running a candidate in a disposable sandbox VM. The actual VM
/// backend is provided by the host; this is only the coordination surface.
///
/// # Contract: a sandbox LOWERS an escalation, it never MINTS truth
///
/// A clean [`ValidationReport`] (`applied_cleanly == true`) is *escalation
/// evidence only*. Its entire authority is to lower a **reversible,
/// software-state** plan's bar from human sign-off to verifier sign-off (via
/// `panel::required_escalation`). It does nothing else. Specifically, a clean
/// report:
///
/// - is NOT a `Verdict::Pass`. The corpus verdict comes only from
///   `agent_core::verify_outcome` over a *real* post-fix re-collection on the
///   actual target machine — never from the sandbox. Sandbox evidence and the
///   corpus verdict are different quantities from different machines.
/// - cannot lower a destructive plan (always human) or a hardware/ambiguous
///   route (always human).
/// - cannot, by itself, produce a resolved corpus row. A resolved row is gated
///   on a matching passing verdict + sign-off + ed25519 attestation, none of
///   which the sandbox supplies.
///
/// A rigged or flaky sandbox can therefore waste a verifier's time; it can never
/// admit a fix. A dirty apply or a validation error is treated conservatively
/// (unvalidated ⇒ escalate). See `docs/test-validation-fleet-design.md` §3.
#[async_trait]
pub trait SandboxValidator: Send + Sync {
    /// Validate `candidate` in a disposable sandbox, returning escalation
    /// evidence (a clean/dirty apply), never a corpus verdict — see the trait
    /// contract above.
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
    ///
    /// Degrades gracefully: a generator that fails (e.g. an unreachable
    /// inference endpoint) is recorded in [`Gathered::failures`] and the
    /// gather continues with the rest, so one dead node never empties the
    /// slate. Zero viable plans is a judge-panel escalation, not a gather
    /// error; only an empty generator set is an error here.
    pub async fn gather<G: Generator>(
        &self,
        generators: &[G],
        events: &[DiagnosticEvent],
    ) -> Result<Gathered, SwarmError> {
        if generators.is_empty() {
            return Err(SwarmError::NoGenerators);
        }
        let mut gathered = Gathered::default();
        for generator in generators {
            match generator.generate(events).await {
                Ok(candidates) => gathered.candidates.extend(candidates),
                Err(error) => gathered.failures.push(error),
            }
        }
        Ok(gathered)
    }
}

/// The result of a swarm gather: every candidate the surviving generators
/// produced, plus the failures from the ones that did not.
#[derive(Debug, Default)]
pub struct Gathered {
    /// All candidate plans, in generator order.
    pub candidates: Vec<Candidate>,
    /// One error per failed generator.
    pub failures: Vec<SwarmError>,
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

    struct AlwaysRejects;

    #[async_trait]
    impl Generator for AlwaysRejects {
        async fn generate(
            &self,
            _events: &[DiagnosticEvent],
        ) -> Result<Vec<Candidate>, SwarmError> {
            Err(SwarmError::Rejected("down".to_string()))
        }
    }

    #[tokio::test]
    async fn gather_collects_from_each_generator() {
        let swarm = Swarm::new();
        let generators = [OneShot, OneShot];
        let gathered = swarm.gather(&generators, &[]).await.expect("gather");
        assert_eq!(gathered.candidates.len(), 2);
        assert!(gathered.failures.is_empty());
    }

    #[tokio::test]
    async fn gather_without_generators_errors() {
        let swarm = Swarm::new();
        let empty: [OneShot; 0] = [];
        let result = swarm.gather(&empty, &[]).await;
        assert!(matches!(result, Err(SwarmError::NoGenerators)));
    }

    #[tokio::test]
    async fn gather_degrades_past_a_failed_generator() {
        let swarm = Swarm::new();
        let generators: Vec<Box<dyn Generator>> = vec![Box::new(AlwaysRejects), Box::new(OneShot)];
        let gathered = swarm.gather(&generators, &[]).await.expect("gather");
        assert_eq!(gathered.candidates.len(), 1, "the survivor still produces");
        assert_eq!(gathered.failures.len(), 1, "the failure is recorded");
    }
}
