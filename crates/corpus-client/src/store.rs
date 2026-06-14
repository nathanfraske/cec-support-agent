use async_trait::async_trait;
use common::{ConfigClass, FaultSignature};
use provenance::SignOffPublicKey;
use thiserror::Error;

use crate::gate::{ensure_attested, ensure_evidence_integrity, GateError};
use crate::schema::{Contribution, FixMapping};

/// Run the full admission gate for a store: the evidence-integrity checks
/// always, plus the sign-off **attestation** check when an authority is
/// configured (cold start has none).
fn admit(
    contribution: &Contribution,
    authority: &Option<SignOffPublicKey>,
) -> Result<(), GateError> {
    ensure_evidence_integrity(contribution)?;
    if let Some(authority) = authority {
        ensure_attested(contribution, authority)?;
    }
    Ok(())
}

/// Errors raised by a corpus backend.
#[derive(Debug, Error)]
pub enum CorpusError {
    /// The contribution failed the sign-off gate.
    #[error(transparent)]
    Gate(#[from] GateError),
    /// Transport-level failure talking to a remote corpus.
    #[error("corpus transport error: {0}")]
    Transport(String),
    /// Local storage failure (file-backed corpus).
    #[error("corpus storage error: {0}")]
    Storage(String),
}

/// Derive the fix mappings for a signature at a config class from a set of
/// corpus rows. Only resolved rows back a mapping (failures stay in the rows
/// as hard negatives); rows confirming the same plan for the same signature
/// aggregate into one mapping with a confirmation count.
fn fix_mappings(
    rows: &[Contribution],
    signature: &FaultSignature,
    config_class: &ConfigClass,
) -> Vec<FixMapping> {
    let mut mappings: Vec<FixMapping> = Vec::new();
    for row in rows {
        if row.outcome.signature.fingerprint != signature.fingerprint
            || row.config_class != *config_class
            || !row.outcome.label.is_resolved()
        {
            continue;
        }
        if let Some(existing) = mappings
            .iter_mut()
            .find(|m| m.plan.id == row.outcome.plan.id)
        {
            existing.confirmations += 1;
        } else {
            mappings.push(FixMapping {
                signature: row.outcome.signature.clone(),
                plan: row.outcome.plan.clone(),
                confirmations: 1,
            });
        }
    }
    mappings
}

/// A corpus backend: look up fix mappings and submit confirmed outcomes.
///
/// A ticket is matched only against like configs, so every query carries the
/// config class alongside the fault signature.
///
/// Implementors MUST enforce the sign-off gate in [`CorpusStore::submit`] by
/// calling [`ensure_signed_off`] before persisting or transmitting anything.
#[async_trait]
pub trait CorpusStore: Send + Sync {
    /// Look up known fix mappings for a fault signature at a config class.
    /// Empty at cold start.
    async fn query(
        &self,
        signature: &FaultSignature,
        config_class: &ConfigClass,
    ) -> Result<Vec<FixMapping>, CorpusError>;

    /// Submit a labeled outcome. Rejects unconfirmed contributions via the
    /// sign-off gate. Every label is accepted — a failure enters the corpus as
    /// a hard negative — but only resolved outcomes back future fix mappings.
    async fn submit(&self, contribution: &Contribution) -> Result<(), CorpusError>;
}

/// In-memory corpus used for cold start and self-hosting. Ships no data; it
/// starts empty and only ever holds contributions that cleared the gate.
#[derive(Default)]
pub struct LocalCorpus {
    rows: std::sync::Mutex<Vec<Contribution>>,
    authority: Option<SignOffPublicKey>,
}

impl LocalCorpus {
    /// A new, empty corpus.
    pub fn new() -> Self {
        Self::default()
    }

    /// Require every confirmed row to carry a valid sign-off attestation by
    /// `authority`. The store holds only the public key, so it can verify but
    /// not forge an attestation.
    pub fn with_authority(mut self, authority: SignOffPublicKey) -> Self {
        self.authority = Some(authority);
        self
    }

    /// Number of accepted rows held in memory (all labels, including hard
    /// negatives).
    pub fn len(&self) -> usize {
        self.rows.lock().expect("corpus mutex poisoned").len()
    }

    /// Whether the corpus holds no rows (true at cold start).
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

#[async_trait]
impl CorpusStore for LocalCorpus {
    async fn query(
        &self,
        signature: &FaultSignature,
        config_class: &ConfigClass,
    ) -> Result<Vec<FixMapping>, CorpusError> {
        let guard = self.rows.lock().expect("corpus mutex poisoned");
        Ok(fix_mappings(&guard, signature, config_class))
    }

    async fn submit(&self, contribution: &Contribution) -> Result<(), CorpusError> {
        // Gate enforced before any state change.
        admit(contribution, &self.authority)?;
        let mut guard = self.rows.lock().expect("corpus mutex poisoned");
        guard.push(contribution.clone());
        Ok(())
    }
}

/// File-backed corpus for self-hosting: one JSON row per line at a local
/// path, loaded at open and appended on submit. This is what makes the
/// flywheel turn across runs with no service at all — the next run facing a
/// known signature starts from this run's outcome. Ships no data; the file
/// begins empty and only ever holds contributions that cleared the gate.
pub struct FileCorpus {
    path: std::path::PathBuf,
    rows: std::sync::Mutex<Vec<Contribution>>,
    authority: Option<SignOffPublicKey>,
}

impl FileCorpus {
    /// Open (or start) a corpus file. A missing file is an empty corpus; an
    /// unparseable one is an error rather than silent data loss.
    pub fn open(path: impl Into<std::path::PathBuf>) -> Result<Self, CorpusError> {
        let path = path.into();
        let rows = match std::fs::read_to_string(&path) {
            Ok(text) => text
                .lines()
                .filter(|line| !line.trim().is_empty())
                .map(serde_json::from_str)
                .collect::<Result<Vec<Contribution>, _>>()
                .map_err(|e| CorpusError::Storage(format!("parse {}: {e}", path.display())))?,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Vec::new(),
            Err(e) => {
                return Err(CorpusError::Storage(format!(
                    "read {}: {e}",
                    path.display()
                )))
            }
        };
        Ok(Self {
            path,
            rows: std::sync::Mutex::new(rows),
            authority: None,
        })
    }

    /// Require every confirmed row to carry a valid sign-off attestation by
    /// `authority`. The store holds only the public key.
    pub fn with_authority(mut self, authority: SignOffPublicKey) -> Self {
        self.authority = Some(authority);
        self
    }

    /// Number of rows held (all labels, including hard negatives).
    pub fn len(&self) -> usize {
        self.rows.lock().expect("corpus mutex poisoned").len()
    }

    /// Whether the corpus holds no rows.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

#[async_trait]
impl CorpusStore for FileCorpus {
    async fn query(
        &self,
        signature: &FaultSignature,
        config_class: &ConfigClass,
    ) -> Result<Vec<FixMapping>, CorpusError> {
        let guard = self.rows.lock().expect("corpus mutex poisoned");
        Ok(fix_mappings(&guard, signature, config_class))
    }

    async fn submit(&self, contribution: &Contribution) -> Result<(), CorpusError> {
        // Gate enforced before any state change — nothing that fails the
        // evidence-integrity gate is written to disk or held in memory.
        admit(contribution, &self.authority)?;
        let line =
            serde_json::to_string(contribution).map_err(|e| CorpusError::Storage(e.to_string()))?;
        use std::io::Write as _;
        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)
            .map_err(|e| CorpusError::Storage(format!("open {}: {e}", self.path.display())))?;
        writeln!(file, "{line}").map_err(|e| CorpusError::Storage(e.to_string()))?;
        let mut guard = self.rows.lock().expect("corpus mutex poisoned");
        guard.push(contribution.clone());
        Ok(())
    }
}

/// HTTP client for a self-hosted or CEC-hosted corpus service. Optional: the
/// engine runs without it, since cold start uses [`LocalCorpus`].
pub struct HttpCorpus {
    base_url: String,
    http: reqwest::Client,
    authority: Option<SignOffPublicKey>,
}

impl HttpCorpus {
    /// Build a client for the corpus service at `base_url`.
    pub fn new(base_url: impl Into<String>) -> Self {
        Self {
            base_url: base_url.into().trim_end_matches('/').to_string(),
            http: reqwest::Client::new(),
            authority: None,
        }
    }

    /// Require every confirmed row to carry a valid sign-off attestation by
    /// `authority` before it leaves the process.
    pub fn with_authority(mut self, authority: SignOffPublicKey) -> Self {
        self.authority = Some(authority);
        self
    }
}

#[async_trait]
impl CorpusStore for HttpCorpus {
    async fn query(
        &self,
        signature: &FaultSignature,
        config_class: &ConfigClass,
    ) -> Result<Vec<FixMapping>, CorpusError> {
        let url = format!(
            "{}/v1/mappings/{}/{}",
            self.base_url,
            config_class.key(),
            signature.fingerprint
        );
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
        // Gate enforced before the network call: a contribution that fails the
        // admission gate never leaves the process.
        admit(contribution, &self.authority)?;
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
    use crate::schema::{Outcome, OutcomeLabel, SignOff};
    use common::{FaultSignature, Plan, Symptom, Verification};

    fn config_class() -> ConfigClass {
        ConfigClass::from_inventory(["os:windows 11", "gpu:rtx-4070"])
    }

    /// A resolved label is bound to a matching passing verdict (the gate now
    /// requires one); any other label needs no verdict.
    fn verification_for(label: &OutcomeLabel) -> Option<Verification> {
        match label {
            OutcomeLabel::ResolvedConfirmed => Some(Verification::pass()),
            OutcomeLabel::ResolvedProvisional => Some(Verification::provisional()),
            _ => None,
        }
    }

    fn contribution(label: OutcomeLabel, sign_off: SignOff) -> Contribution {
        let signature = FaultSignature::from_symptoms(vec![Symptom("boot_loop".into())]);
        let plan = Plan::new("p1", "restart service");
        let verification = verification_for(&label);
        Contribution::new(
            Outcome {
                signature,
                plan,
                label,
                verification,
            },
            config_class(),
            sign_off,
        )
    }

    #[tokio::test]
    async fn submit_refuses_unconfirmed_and_keeps_store_empty() {
        let corpus = LocalCorpus::new();
        let error = corpus
            .submit(&contribution(
                OutcomeLabel::ResolvedConfirmed,
                SignOff::Unconfirmed,
            ))
            .await
            .expect_err("gate must reject");
        assert!(matches!(error, CorpusError::Gate(_)));
        assert!(corpus.is_empty(), "rejected outcome must not be stored");
    }

    #[tokio::test]
    async fn submit_accepts_confirmed_then_queryable() {
        let corpus = LocalCorpus::new();
        let confirmed = contribution(OutcomeLabel::ResolvedConfirmed, SignOff::HumanConfirmed);
        corpus.submit(&confirmed).await.expect("confirmed accepted");
        assert_eq!(corpus.len(), 1);
        let hits = corpus
            .query(&confirmed.outcome.signature, &config_class())
            .await
            .expect("query");
        assert_eq!(hits.len(), 1);
    }

    #[tokio::test]
    async fn verifier_confirmation_also_clears_the_gate() {
        let corpus = LocalCorpus::new();
        corpus
            .submit(&contribution(
                OutcomeLabel::ResolvedProvisional,
                SignOff::VerifierConfirmed,
            ))
            .await
            .expect("verifier-confirmed accepted");
        assert_eq!(corpus.len(), 1);
    }

    #[tokio::test]
    async fn hard_negatives_are_stored_but_not_retrieved_as_fixes() {
        let corpus = LocalCorpus::new();
        let negative = contribution(
            OutcomeLabel::EscalatedHumanUnresolved,
            SignOff::HumanConfirmed,
        );
        corpus.submit(&negative).await.expect("labeled and signed");
        assert_eq!(corpus.len(), 1, "the failure is kept as a hard negative");
        let hits = corpus
            .query(&negative.outcome.signature, &config_class())
            .await
            .expect("query");
        assert!(hits.is_empty(), "a failure must not be offered as a fix");
    }

    /// A unique temp path per test; removed on drop.
    struct TempPath(std::path::PathBuf);

    impl TempPath {
        fn new(tag: &str) -> Self {
            Self(std::env::temp_dir().join(format!(
                "cec-corpus-test-{}-{tag}.jsonl",
                std::process::id()
            )))
        }
    }

    impl Drop for TempPath {
        fn drop(&mut self) {
            let _ = std::fs::remove_file(&self.0);
        }
    }

    #[tokio::test]
    async fn file_corpus_persists_across_reopen() {
        let path = TempPath::new("roundtrip");
        {
            let corpus = FileCorpus::open(&path.0).expect("open empty");
            assert!(corpus.is_empty());
            corpus
                .submit(&contribution(
                    OutcomeLabel::ResolvedConfirmed,
                    SignOff::HumanConfirmed,
                ))
                .await
                .expect("accepted");
        }
        // A new process would see the row: reopen and query.
        let reopened = FileCorpus::open(&path.0).expect("reopen");
        assert_eq!(reopened.len(), 1);
        let row = contribution(OutcomeLabel::ResolvedConfirmed, SignOff::HumanConfirmed);
        let hits = reopened
            .query(&row.outcome.signature, &config_class())
            .await
            .expect("query");
        assert_eq!(hits.len(), 1);
    }

    #[tokio::test]
    async fn file_corpus_gate_rejects_before_anything_touches_disk() {
        let path = TempPath::new("gate");
        let corpus = FileCorpus::open(&path.0).expect("open");
        let error = corpus
            .submit(&contribution(
                OutcomeLabel::ResolvedConfirmed,
                SignOff::Unconfirmed,
            ))
            .await
            .expect_err("gate must reject");
        assert!(matches!(error, CorpusError::Gate(_)));
        assert!(
            !path.0.exists(),
            "an unconfirmed contribution must never reach disk"
        );
    }

    #[tokio::test]
    async fn confirmations_aggregate_per_plan() {
        let corpus = LocalCorpus::new();
        let row = contribution(OutcomeLabel::ResolvedConfirmed, SignOff::HumanConfirmed);
        corpus.submit(&row).await.expect("first");
        corpus.submit(&row).await.expect("second");
        let hits = corpus
            .query(&row.outcome.signature, &config_class())
            .await
            .expect("query");
        assert_eq!(hits.len(), 1, "same plan aggregates into one mapping");
        assert_eq!(hits[0].confirmations, 2);
    }

    // --- Sign-off attestation (MH-1): the engine cannot forge a confirmed row
    //     when an authority is configured. ------------------------------------

    #[tokio::test]
    async fn authority_store_rejects_an_unattested_confirmed_row() {
        let authority = provenance::SignOffAuthority::generate();
        let corpus = LocalCorpus::new().with_authority(authority.public_key());
        // A self-asserted HumanConfirmed with no attestation: the exact forgery.
        let forged = contribution(OutcomeLabel::ResolvedConfirmed, SignOff::HumanConfirmed);
        let error = corpus.submit(&forged).await.expect_err("must be refused");
        assert!(matches!(
            error,
            CorpusError::Gate(GateError::AttestationMissing)
        ));
        assert!(corpus.is_empty(), "a forged row must not be stored");
    }

    #[tokio::test]
    async fn authority_store_accepts_a_genuinely_attested_row() {
        let authority = provenance::SignOffAuthority::generate();
        let corpus = LocalCorpus::new().with_authority(authority.public_key());
        let attested = contribution(OutcomeLabel::ResolvedConfirmed, SignOff::HumanConfirmed)
            .attested_by(&authority);
        corpus.submit(&attested).await.expect("genuine attestation");
        assert_eq!(corpus.len(), 1);
    }

    #[tokio::test]
    async fn an_attestation_by_another_authority_is_refused() {
        let authority = provenance::SignOffAuthority::generate();
        let attacker = provenance::SignOffAuthority::generate();
        let corpus = LocalCorpus::new().with_authority(authority.public_key());
        // Signed by a key the store does not trust.
        let row = contribution(OutcomeLabel::ResolvedConfirmed, SignOff::HumanConfirmed)
            .attested_by(&attacker);
        let error = corpus.submit(&row).await.expect_err("untrusted authority");
        assert!(matches!(
            error,
            CorpusError::Gate(GateError::AttestationInvalid)
        ));
    }

    #[tokio::test]
    async fn tampering_with_the_tuple_after_attestation_is_refused() {
        let authority = provenance::SignOffAuthority::generate();
        let corpus = LocalCorpus::new().with_authority(authority.public_key());
        // Attest a verifier-level row, then forge it up to human: the attestation
        // covers sign_off, so the signature no longer matches.
        let mut row = contribution(
            OutcomeLabel::ResolvedProvisional,
            SignOff::VerifierConfirmed,
        )
        .attested_by(&authority);
        row.sign_off = SignOff::HumanConfirmed;
        let error = corpus.submit(&row).await.expect_err("tampered tuple");
        assert!(matches!(
            error,
            CorpusError::Gate(GateError::AttestationInvalid)
        ));
    }

    #[tokio::test]
    async fn without_an_authority_an_unattested_confirmed_row_is_accepted() {
        // Cold start: no authority configured, so attestation is not required
        // (Increment-1 behavior is preserved).
        let corpus = LocalCorpus::new();
        let row = contribution(OutcomeLabel::ResolvedConfirmed, SignOff::HumanConfirmed);
        corpus.submit(&row).await.expect("accepted at cold start");
        assert_eq!(corpus.len(), 1);
    }

    #[tokio::test]
    async fn retrieval_is_scoped_to_the_config_class() {
        let corpus = LocalCorpus::new();
        let row = contribution(OutcomeLabel::ResolvedConfirmed, SignOff::HumanConfirmed);
        corpus.submit(&row).await.expect("accepted");
        let other_class = ConfigClass::from_inventory(["os:windows 10"]);
        let hits = corpus
            .query(&row.outcome.signature, &other_class)
            .await
            .expect("query");
        assert!(hits.is_empty(), "a ticket matches only like configs");
    }
}
