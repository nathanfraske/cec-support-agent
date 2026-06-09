use async_trait::async_trait;
use common::FaultSignature;
use thiserror::Error;

use crate::gate::{ensure_signed_off, GateError};
use crate::schema::{Contribution, FixMapping};

/// Errors raised by a corpus backend.
#[derive(Debug, Error)]
pub enum CorpusError {
    /// The contribution failed the sign-off gate.
    #[error(transparent)]
    Gate(#[from] GateError),
    /// Transport-level failure talking to a remote corpus.
    #[error("corpus transport error: {0}")]
    Transport(String),
}

/// A corpus backend: look up fix mappings and submit confirmed outcomes.
///
/// Implementors MUST enforce the sign-off gate in [`CorpusStore::submit`] by
/// calling [`ensure_signed_off`] before persisting or transmitting anything.
#[async_trait]
pub trait CorpusStore: Send + Sync {
    /// Look up known fix mappings for a fault signature. Empty at cold start.
    async fn query(&self, signature: &FaultSignature) -> Result<Vec<FixMapping>, CorpusError>;

    /// Submit a confirmed outcome. Rejects unconfirmed contributions via the
    /// sign-off gate.
    async fn submit(&self, contribution: &Contribution) -> Result<(), CorpusError>;
}

/// In-memory corpus used for cold start and self-hosting. Ships no data; it
/// starts empty and only ever holds contributions that cleared the gate.
#[derive(Default)]
pub struct LocalCorpus {
    mappings: std::sync::Mutex<Vec<FixMapping>>,
}

impl LocalCorpus {
    /// A new, empty corpus.
    pub fn new() -> Self {
        Self::default()
    }

    /// Number of accepted mappings held in memory.
    pub fn len(&self) -> usize {
        self.mappings.lock().expect("corpus mutex poisoned").len()
    }

    /// Whether the corpus holds no mappings (true at cold start).
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

#[async_trait]
impl CorpusStore for LocalCorpus {
    async fn query(&self, signature: &FaultSignature) -> Result<Vec<FixMapping>, CorpusError> {
        let guard = self.mappings.lock().expect("corpus mutex poisoned");
        let hits = guard
            .iter()
            .filter(|m| m.signature.fingerprint == signature.fingerprint)
            .cloned()
            .collect();
        Ok(hits)
    }

    async fn submit(&self, contribution: &Contribution) -> Result<(), CorpusError> {
        // Gate enforced before any state change.
        ensure_signed_off(contribution)?;
        let mut guard = self.mappings.lock().expect("corpus mutex poisoned");
        guard.push(FixMapping {
            signature: contribution.outcome.signature.clone(),
            plan: contribution.outcome.plan.clone(),
            confirmations: 1,
        });
        Ok(())
    }
}

/// HTTP client for a self-hosted or CEC-hosted corpus service. Optional: the
/// engine runs without it, since cold start uses [`LocalCorpus`].
pub struct HttpCorpus {
    base_url: String,
    http: reqwest::Client,
}

impl HttpCorpus {
    /// Build a client for the corpus service at `base_url`.
    pub fn new(base_url: impl Into<String>) -> Self {
        Self {
            base_url: base_url.into().trim_end_matches('/').to_string(),
            http: reqwest::Client::new(),
        }
    }
}

#[async_trait]
impl CorpusStore for HttpCorpus {
    async fn query(&self, signature: &FaultSignature) -> Result<Vec<FixMapping>, CorpusError> {
        let url = format!("{}/v1/mappings/{}", self.base_url, signature.fingerprint);
        let response = self
            .http
            .get(&url)
            .send()
            .await
            .map_err(|e| CorpusError::Transport(e.to_string()))?;
        if !response.status().is_success() {
            return Err(CorpusError::Transport(format!(
                "status {}",
                response.status()
            )));
        }
        response
            .json()
            .await
            .map_err(|e| CorpusError::Transport(e.to_string()))
    }

    async fn submit(&self, contribution: &Contribution) -> Result<(), CorpusError> {
        // Gate enforced before the network call: an unconfirmed contribution
        // never leaves the process.
        ensure_signed_off(contribution)?;
        let url = format!("{}/v1/contributions", self.base_url);
        let response = self
            .http
            .post(&url)
            .json(contribution)
            .send()
            .await
            .map_err(|e| CorpusError::Transport(e.to_string()))?;
        if !response.status().is_success() {
            return Err(CorpusError::Transport(format!(
                "status {}",
                response.status()
            )));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema::{Outcome, SignOff};
    use common::{FaultSignature, Plan, Symptom};

    fn contribution(sign_off: SignOff) -> Contribution {
        let signature = FaultSignature::from_symptoms(vec![Symptom("boot_loop".into())]);
        let plan = Plan::new("p1", "restart service");
        Contribution::new(
            Outcome {
                signature,
                plan,
                resolved: true,
            },
            sign_off,
        )
    }

    #[tokio::test]
    async fn submit_refuses_unconfirmed_and_keeps_store_empty() {
        let corpus = LocalCorpus::new();
        let error = corpus
            .submit(&contribution(SignOff::Unconfirmed))
            .await
            .expect_err("gate must reject");
        assert!(matches!(error, CorpusError::Gate(_)));
        assert!(corpus.is_empty(), "rejected outcome must not be stored");
    }

    #[tokio::test]
    async fn submit_accepts_confirmed_then_queryable() {
        let corpus = LocalCorpus::new();
        let confirmed = contribution(SignOff::HumanConfirmed);
        corpus.submit(&confirmed).await.expect("confirmed accepted");
        assert_eq!(corpus.len(), 1);
        let hits = corpus
            .query(&confirmed.outcome.signature)
            .await
            .expect("query");
        assert_eq!(hits.len(), 1);
    }

    #[tokio::test]
    async fn verifier_confirmation_also_clears_the_gate() {
        let corpus = LocalCorpus::new();
        corpus
            .submit(&contribution(SignOff::VerifierConfirmed))
            .await
            .expect("verifier-confirmed accepted");
        assert_eq!(corpus.len(), 1);
    }
}
