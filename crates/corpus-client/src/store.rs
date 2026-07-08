use std::collections::HashSet;

use async_trait::async_trait;
use common::{ConfigClass, FaultSignature};
use provenance::SignOffPublicKey;
use thiserror::Error;

use crate::gate::{ensure_attested, ensure_evidence_integrity, GateError};
use crate::schema::{
    attestation_message, chain_hash, de_identify_plan, Contribution, FixMapping, OutcomeLabel,
    RowIntegrity,
};
use crate::stored::{StoredPlan, StoredSignature};

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

/// The independence key a resolved row contributes to a plan's confirmation
/// count, or `None` if it is not an independent confirmation of that plan.
///
/// EI-03/A5: a confirmation counts only if it came from an independent run.
/// A row whose plan was corpus-primed *from the very mapping it would confirm*
/// is circular and contributes nothing. Otherwise rows are deduplicated by
/// `run_id`, so re-submitting the same run cannot inflate confidence. A row with
/// no provenance is deduplicated by a CONTENT HASH of its canonical bytes, so
/// byte-identical no-provenance submissions collapse to a single key — a verbatim
/// replay is one observation, not N. (The prior positional `row:{index}` counted
/// every resubmission as distinct, so a replayed no-provenance row inflated
/// confirmations, and its Reopened mirror over-demoted a multiply-confirmed fix.)
fn confirmation_key(row: &Contribution, plan_id: &str) -> Option<String> {
    match &row.provenance {
        Some(p) if p.retrieval_first && p.primed_from.iter().any(|id| id == plan_id) => None,
        Some(p) => Some(format!("run:{}", p.run_id)),
        None => Some(format!("content:{}", content_hash(row))),
    }
}

/// A stable content hash of a row's canonical bytes, used as the independence key
/// for a row that carries no run provenance. It reuses the attestation
/// canonicalization — covering the signature, plan, label, verification,
/// sign-off, and config class (everything that makes two rows the *same*
/// observation) and excluding the attestation and the tamper-evidence link — so
/// two byte-identical no-provenance rows hash equal and count once.
fn content_hash(row: &Contribution) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(b"cec-corpus-confirmation-content-v1\n");
    hasher.update(attestation_message(row));
    hasher
        .finalize()
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

/// Derive the fix mappings for a signature at a config class from a set of
/// corpus rows. Only resolved rows back a mapping (other hard negatives stay in
/// the rows but never offer a fix); rows confirming the same plan aggregate into
/// one mapping whose confirmation count is the number of INDEPENDENT
/// confirmations ([`confirmation_key`]), **net of `Reopened` events** — a fix
/// that recurred inside its monitoring horizon is demoted (EI-06 / T-104). A
/// plan whose id is in `revoked` (an owner-only retraction list) is never
/// offered, and a plan with no net independent confirmation is not offered.
fn fix_mappings(
    rows: &[Contribution],
    signature: &FaultSignature,
    config_class: &ConfigClass,
    revoked: &HashSet<String>,
) -> Vec<FixMapping> {
    use std::collections::HashMap;

    struct Acc {
        signature: StoredSignature,
        plan: StoredPlan,
        contributors: HashSet<String>,
        // Reopens are deduplicated by the SAME independence key as confirmations
        // (a set, not a counter), so a single reopen run replayed N times cancels
        // one confirmation, not N. Counting raw reopen rows was asymmetric with
        // the run-deduped confirmation count and let a duplicated `Reopened` line
        // bury a multiply-confirmed fix out of retrieval.
        reopens: HashSet<String>,
    }

    let mut order: Vec<String> = Vec::new();
    let mut accs: HashMap<String, Acc> = HashMap::new();

    for row in rows {
        if row.outcome.signature.fingerprint != signature.fingerprint
            || row.config_class != *config_class
        {
            continue;
        }
        let resolved = row.outcome.label.is_resolved();
        let reopened = matches!(row.outcome.label, OutcomeLabel::Reopened);
        if !resolved && !reopened {
            continue; // a plain hard negative affects no mapping
        }
        let plan_id = row.outcome.plan.id().to_string();
        let acc = accs.entry(plan_id.clone()).or_insert_with(|| {
            order.push(plan_id.clone());
            Acc {
                signature: row.outcome.signature.clone(),
                plan: row.outcome.plan.clone(),
                contributors: HashSet::new(),
                reopens: HashSet::new(),
            }
        });
        // Both sides key on the same independence function: distinct runs count,
        // re-submissions of one run collapse, and a circular (self-primed) row
        // contributes to neither — symmetric, so a reopen can never out-weigh a
        // confirmation through replay alone.
        if let Some(key) = confirmation_key(row, &plan_id) {
            if resolved {
                acc.contributors.insert(key);
            } else {
                acc.reopens.insert(key);
            }
        }
    }

    order
        .iter()
        .filter_map(|plan_id| {
            if revoked.contains(plan_id) {
                return None; // owner-revoked: never offered as a fix
            }
            let acc = &accs[plan_id];
            // Each DISTINCT reopen run cancels one independent confirmation;
            // circular (self-primed) rows never counted on either side.
            let confirmations =
                (acc.contributors.len() as u32).saturating_sub(acc.reopens.len() as u32);
            (confirmations > 0).then(|| FixMapping {
                signature: acc.signature.clone(),
                plan: acc.plan.clone(),
                confirmations,
            })
        })
        .collect()
}

/// Verify a loaded file's tamper-evidence hash chain and return the chain head
/// (the last row's hash). A file is either fully chained (every row carries
/// [`RowIntegrity`] and the chain verifies in order) or fully unchained (a
/// legacy/empty file, no integrity anywhere). A mix — or any edited, reordered,
/// or mid-stream-removed row — is a tamper and an error, not silent acceptance.
fn verify_chain(rows: &[Contribution]) -> Result<String, CorpusError> {
    let with = rows.iter().filter(|r| r.integrity.is_some()).count();
    if with == 0 {
        return Ok(String::new()); // unchained legacy/empty file
    }
    if with != rows.len() {
        return Err(CorpusError::Storage(
            "corpus integrity: some rows have no chain link (tampered)".into(),
        ));
    }
    let mut prev = String::new();
    for (i, row) in rows.iter().enumerate() {
        let integ = row.integrity.as_ref().expect("checked all present");
        let expected = chain_hash(&prev, row);
        if integ.prev != prev || integ.hash != expected {
            return Err(CorpusError::Storage(format!(
                "corpus integrity: row {i} fails the tamper-evidence chain"
            )));
        }
        prev = integ.hash.clone();
    }
    Ok(prev)
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
    revoked: HashSet<String>,
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

    /// Owner-only retraction: plan ids in `revoked` are never offered as a fix
    /// (a proven-wrong precedent withdrawn by the owner). Hard negatives and the
    /// rows themselves are untouched; only retrieval is suppressed.
    pub fn with_revoked(mut self, revoked: impl IntoIterator<Item = String>) -> Self {
        self.revoked = revoked.into_iter().collect();
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
        Ok(fix_mappings(&guard, signature, config_class, &self.revoked))
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
    revoked: HashSet<String>,
    /// The tamper-evidence chain head (the last row's hash).
    chain_head: std::sync::Mutex<String>,
}

impl FileCorpus {
    /// Open (or start) a corpus file. A missing file is an empty corpus; an
    /// unparseable one is an error rather than silent data loss; a file whose
    /// tamper-evidence chain does not verify (an edited/reordered/removed row)
    /// is an error, so a hand-edited precedent is never served.
    ///
    /// The chain is keyless, so it proves internal consistency but not that the
    /// rows were ever attested. When an authority is required, configure it with
    /// [`FileCorpus::with_authority`], which additionally re-verifies the ed25519
    /// attestation on every at-rest row — opening alone (cold start) does not.
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
        let head = verify_chain(&rows)?;
        // De-id image check at the disk boundary (Layer-1f), matching
        // `HttpCorpus::query`. Every stored LEAF validates at deserialize
        // (`#[serde(try_from)]`), but `StoredPlan.title` is a plain string with no
        // read-side guard, so a hand-edited row with valid-vocabulary leaves but an
        // identity-bearing title deserializes clean. Enforce here — independent of
        // whether an authority is configured — that every at-rest plan is its own
        // de-identified image, so a cold-start open never serves such a title into
        // the human trace or the consent screen.
        for (i, row) in rows.iter().enumerate() {
            match de_identify_plan(&row.outcome.plan.to_plan()) {
                Ok(reminted) if reminted == row.outcome.plan => {}
                _ => {
                    return Err(CorpusError::Storage(format!(
                        "corpus integrity: at-rest row {i}'s stored plan is not its own \
                         de-identified image (a title or leaf the de-id mint would never emit)"
                    )))
                }
            }
        }
        Ok(Self {
            path,
            rows: std::sync::Mutex::new(rows),
            authority: None,
            revoked: HashSet::new(),
            chain_head: std::sync::Mutex::new(head),
        })
    }

    /// Require every confirmed row to carry a valid sign-off attestation by
    /// `authority`, **including the rows already loaded from disk**. The store
    /// holds only the public key.
    ///
    /// This re-runs the full admission gate over every at-rest row under
    /// `authority` and refuses the corpus (a `Storage` error) if any row does not
    /// clear it. That closes the open-time bypass: the tamper-evidence hash chain
    /// is keyless (recomputable by anyone with write access), so it proves the
    /// file is internally consistent, NOT that its rows were ever attested. Only
    /// re-verifying the ed25519 attestation makes the on-disk history face the
    /// same boundary as a freshly submitted row — otherwise an authority
    /// configured here would gate future submits while a file-rewrite of forged
    /// "confirmed fixes" was served unchecked from `query`.
    ///
    /// Consequence: a corpus accreted at cold start (rows with no attestation)
    /// cannot be opened under an authority — turning on enforcement requires a
    /// corpus built under that authority. That is the intended fail-closed stance.
    pub fn with_authority(mut self, authority: SignOffPublicKey) -> Result<Self, CorpusError> {
        self.authority = Some(authority);
        {
            let rows = self.rows.lock().expect("corpus mutex poisoned");
            for (i, row) in rows.iter().enumerate() {
                admit(row, &self.authority).map_err(|e| {
                    CorpusError::Storage(format!(
                        "corpus integrity: at-rest row {i} fails the admission gate \
                         under the configured sign-off authority ({e})"
                    ))
                })?;
            }
        }
        Ok(self)
    }

    /// Owner-only retraction: plan ids in `revoked` are never offered as a fix.
    pub fn with_revoked(mut self, revoked: impl IntoIterator<Item = String>) -> Self {
        self.revoked = revoked.into_iter().collect();
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
        Ok(fix_mappings(&guard, signature, config_class, &self.revoked))
    }

    async fn submit(&self, contribution: &Contribution) -> Result<(), CorpusError> {
        // Gate enforced before any state change — nothing that fails the
        // evidence-integrity gate is written to disk or held in memory.
        admit(contribution, &self.authority)?;
        // Attach the tamper-evidence chain link, computed from the current head
        // over everything else in the row (`chain_canonical` never reads the
        // `integrity` field), then append the linked row.
        let mut head = self.chain_head.lock().expect("chain mutex poisoned");
        let mut linked = contribution.clone();
        let hash = chain_hash(&head, &linked);
        linked.integrity = Some(RowIntegrity {
            prev: head.clone(),
            hash: hash.clone(),
        });
        let line =
            serde_json::to_string(&linked).map_err(|e| CorpusError::Storage(e.to_string()))?;
        use std::io::Write as _;
        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)
            .map_err(|e| CorpusError::Storage(format!("open {}: {e}", self.path.display())))?;
        writeln!(file, "{line}").map_err(|e| CorpusError::Storage(e.to_string()))?;
        let mut guard = self.rows.lock().expect("corpus mutex poisoned");
        guard.push(linked);
        *head = hash;
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
        // The retrieval keys travel in the REQUEST BODY, never the URL
        // (cartography control E / AGENTS.md non-mappability rule 6): URL paths
        // land verbatim in proxy/server access logs, where even an opaque keyed
        // fingerprint is a stable per-fault correlation handle; bodies do not.
        // Honest scope: this defeats ACCESS-LOG capture only. The corpus server
        // itself still learns the keys (a rostered caller is permitted to —
        // cartography §0), a body-logging middlebox still sees the body, and
        // on-the-wire privacy comes from the loopback/mesh transport posture,
        // not from this placement.
        let url = format!("{}/v1/mappings/query", self.base_url);
        let body = serde_json::json!({
            "config_class": config_class.key(),
            "fingerprint": signature.fingerprint,
        });
        let response = self
            .http
            .post(&url)
            .json(&body)
            .send()
            .await
            .map_err(|e| CorpusError::Transport(e.to_string()))?;
        if !response.status().is_success() {
            return Err(CorpusError::Transport(format!(
                "status {}",
                response.status()
            )));
        }
        // Read-side re-validation, in two layers (Layer-1e/C4). Read the raw body
        // as a TRANSPORT concern, then DESERIALIZE it as an ADMISSION concern.
        //
        // Layer 1 — leaf validation at the wire boundary. The stored leaf types
        // validate on deserialize (`#[serde(try_from)]`): a served action or
        // description not in the frozen vocabulary, an inadmissible plan id, or a
        // symptom outside the closed grammar makes `from_str` FAIL — so the
        // wire/file path is identical to the construction path and `serde` no
        // longer bypasses the mints. A parse failure on the mappings endpoint is
        // therefore a de-identification refusal, not a transport fault.
        let body = response
            .text()
            .await
            .map_err(|e| CorpusError::Transport(e.to_string()))?;
        let mappings: Vec<FixMapping> =
            serde_json::from_str(&body).map_err(|_| GateError::ServedPlanInadmissible)?;
        // Layer 2 — the plan's DERIVED `title` is a plain string (not leaf-typed),
        // so re-validate that the served plan is exactly its own de-identified
        // image: a hand-edited title a mint would never produce is refused even
        // though its actions/id/symptoms parsed. Fails closed: one bad mapping
        // refuses the whole response (a poisoned server is not a
        // partially-trustworthy one). Cryptographic re-verification of row
        // attestations on this path needs attested rows on the wire — the
        // mappings aggregate carries none; that lands with the corpus-service
        // wire contract (see FOLLOWUPS).
        for mapping in &mappings {
            match de_identify_plan(&mapping.plan.to_plan()) {
                Ok(sanitized) if sanitized == mapping.plan => {}
                _ => return Err(GateError::ServedPlanInadmissible.into()),
            }
        }
        Ok(mappings)
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
    use crate::schema::{
        chain_hash, Outcome, OutcomeLabel, RowIntegrity, RowProvenance, SignOff, SignOffAttestation,
    };
    use crate::stored::{StoredAction, StoredPlanId, StoredStep, StoredSymptom};
    use common::{
        FaultSignature, Plan, Symptom, Verification, VerificationClass, VerificationResult,
    };

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
        let signature = FaultSignature::from_symptoms(vec![Symptom("event_41".into())]);
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
        .expect("test contribution de-identifies")
    }

    // --- Partial resolution: a beneficial-but-incomplete outcome -------------

    #[tokio::test]
    async fn a_partial_resolution_with_cleared_evidence_is_admitted_as_beneficial() {
        // A fix that cleared SOME original symptoms (a proven benefit) but left
        // others is admitted to the corpus as beneficial truth — under a verifier
        // sign-off, no human — even though it is not a full resolution.
        let corpus = LocalCorpus::new();
        let signature = FaultSignature::from_symptoms(vec![
            Symptom("event_41".into()),
            Symptom("0x1234".into()),
        ]);
        let verification = Verification::partial(
            vec![Symptom("0x1234".into())],   // cleared — the proven benefit
            vec![Symptom("event_41".into())], // remaining — the remainder
        );
        let row = Contribution::new(
            Outcome {
                signature,
                plan: Plan::new("p1", "restart service"),
                label: OutcomeLabel::ResolvedPartial,
                verification: Some(verification),
            },
            config_class(),
            SignOff::VerifierConfirmed,
        )
        .expect("de-identifies");
        corpus
            .submit(&row)
            .await
            .expect("a partial with cleared evidence is admitted");
        assert_eq!(corpus.len(), 1);
        // Beneficial, but NOT a resolved fix — it is recorded, not yet retrieved.
        assert!(!row.outcome().label().is_resolved());
        assert!(row.outcome().label().is_beneficial());
    }

    #[tokio::test]
    async fn a_partial_label_without_cleared_evidence_is_refused() {
        // A `ResolvedPartial` label with no `PartialPass`+cleared behind it is a
        // partial claim with no proven improvement — the gate refuses it, so a
        // fabricated "improvement" cannot earn a beneficial precedent.
        let corpus = LocalCorpus::new();
        let row = Contribution::new(
            Outcome {
                signature: FaultSignature::from_symptoms(vec![Symptom("event_41".into())]),
                plan: Plan::new("p1", "restart service"),
                label: OutcomeLabel::ResolvedPartial,
                verification: Some(Verification::pass()), // wrong verdict, no cleared
            },
            config_class(),
            SignOff::VerifierConfirmed,
        )
        .expect("de-identifies");
        let error = corpus
            .submit(&row)
            .await
            .expect_err("a partial without proven benefit is refused");
        assert!(matches!(
            error,
            CorpusError::Gate(GateError::PartialWithoutBenefit)
        ));
        assert!(corpus.is_empty());
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
            .query(&confirmed.outcome.signature.to_signature(), &config_class())
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
            .query(&negative.outcome.signature.to_signature(), &config_class())
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
            .query(&row.outcome.signature.to_signature(), &config_class())
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
        // Rows for the same plan aggregate into ONE mapping. Two byte-identical
        // no-provenance submissions are one observation, not two: keyed on a
        // content hash of the row's canonical bytes, a verbatim replay collapses
        // to a single confirmation (the #2 replay-inflation fix). Independence
        // between distinct runs is established by provenance, not by resubmission.
        let corpus = LocalCorpus::new();
        let row = contribution(OutcomeLabel::ResolvedConfirmed, SignOff::HumanConfirmed);
        corpus.submit(&row).await.expect("first");
        corpus.submit(&row).await.expect("second (verbatim replay)");
        let hits = corpus
            .query(&row.outcome.signature.to_signature(), &config_class())
            .await
            .expect("query");
        assert_eq!(hits.len(), 1, "same plan aggregates into one mapping");
        assert_eq!(
            hits[0].confirmations, 1,
            "byte-identical no-provenance rows collapse to one confirmation"
        );
    }

    #[tokio::test]
    async fn verbatim_no_provenance_resubmissions_do_not_inflate_or_over_demote() {
        // #2 regression (replay integrity): a no-provenance attested row's
        // confirmation key is a CONTENT HASH of its canonical bytes, so N verbatim
        // resubmissions of a ResolvedConfirmed row yield exactly ONE confirmation —
        // the replay the Some(p) run-dedup branch stops but the None branch used to
        // skip. Symmetrically, N verbatim Reopened replays demote by ONE, not N, so
        // a duplicated Reopened line cannot bury a multiply-confirmed fix.
        let corpus = LocalCorpus::new();
        let sig = FaultSignature::from_symptoms(vec![Symptom("event_41".into())]);
        let confirmed = contribution(OutcomeLabel::ResolvedConfirmed, SignOff::HumanConfirmed);
        for _ in 0..5 {
            corpus.submit(&confirmed).await.expect("verbatim confirm");
        }
        let hits = corpus.query(&sig, &config_class()).await.expect("query");
        assert_eq!(hits.len(), 1, "same plan → one mapping");
        assert_eq!(
            hits[0].confirmations, 1,
            "5 byte-identical no-provenance confirmations are ONE independent confirmation"
        );

        // Five verbatim Reopened replays cancel exactly one confirmation (net
        // zero), not five — otherwise a single duplicated reopen buries the fix.
        let reopened = contribution(OutcomeLabel::Reopened, SignOff::HumanConfirmed);
        for _ in 0..5 {
            corpus.submit(&reopened).await.expect("verbatim reopen");
        }
        let demoted = corpus.query(&sig, &config_class()).await.expect("query");
        assert!(
            demoted.is_empty(),
            "one confirmation net one distinct reopen is zero — demoted out of retrieval"
        );
    }

    fn provenance(run_id: &str, retrieval_first: bool, primed_from: &[&str]) -> RowProvenance {
        RowProvenance {
            run_id: run_id.into(),
            retrieval_first,
            primed_from: primed_from.iter().map(|s| s.to_string()).collect(),
        }
    }

    #[tokio::test]
    async fn the_same_run_is_one_confirmation_not_many() {
        // EI-03/A5: re-submitting the same run id must not inflate confidence.
        let corpus = LocalCorpus::new();
        let base = contribution(OutcomeLabel::ResolvedConfirmed, SignOff::HumanConfirmed);
        let r1 = base
            .clone()
            .with_provenance(provenance("run-A", false, &[]));
        let r2 = base
            .clone()
            .with_provenance(provenance("run-A", false, &[]));
        corpus.submit(&r1).await.expect("first");
        corpus.submit(&r2).await.expect("second (same run)");
        let hits = corpus
            .query(&base.outcome.signature.to_signature(), &config_class())
            .await
            .expect("query");
        assert_eq!(hits[0].confirmations, 1, "one run = one confirmation");
    }

    #[tokio::test]
    async fn distinct_independent_runs_each_confirm() {
        let corpus = LocalCorpus::new();
        let base = contribution(OutcomeLabel::ResolvedConfirmed, SignOff::HumanConfirmed);
        corpus
            .submit(
                &base
                    .clone()
                    .with_provenance(provenance("run-A", false, &[])),
            )
            .await
            .expect("A");
        corpus
            .submit(
                &base
                    .clone()
                    .with_provenance(provenance("run-B", false, &[])),
            )
            .await
            .expect("B");
        let hits = corpus
            .query(&base.outcome.signature.to_signature(), &config_class())
            .await
            .expect("query");
        assert_eq!(hits[0].confirmations, 2, "two independent runs = two");
    }

    #[tokio::test]
    async fn a_self_primed_confirmation_does_not_count() {
        // A row whose plan was corpus-primed from this very mapping ("p1") is
        // circular and must not back its own confidence.
        let corpus = LocalCorpus::new();
        let base = contribution(OutcomeLabel::ResolvedConfirmed, SignOff::HumanConfirmed);
        // One genuine de-novo confirmation seeds the mapping...
        corpus
            .submit(
                &base
                    .clone()
                    .with_provenance(provenance("run-A", false, &[])),
            )
            .await
            .expect("de-novo");
        // ...a later run primed FROM p1 adds no independent support.
        corpus
            .submit(
                &base
                    .clone()
                    .with_provenance(provenance("run-B", true, &["p1"])),
            )
            .await
            .expect("self-primed");
        let hits = corpus
            .query(&base.outcome.signature.to_signature(), &config_class())
            .await
            .expect("query");
        assert_eq!(
            hits[0].confirmations, 1,
            "self-primed confirmation is circular and excluded"
        );
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
    async fn an_attestation_cannot_be_replayed_onto_a_fabricated_run_id() {
        // The attestation binds the provenance pin, so one valid attestation
        // cannot be cloned onto rows with fresh run ids to inflate confirmations.
        let authority = provenance::SignOffAuthority::generate();
        let corpus = LocalCorpus::new().with_authority(authority.public_key());
        let genuine = contribution(OutcomeLabel::ResolvedConfirmed, SignOff::HumanConfirmed)
            .with_provenance(provenance("run-A", false, &[]))
            .attested_by(&authority);
        corpus.submit(&genuine).await.expect("genuine run-A");
        // Reuse the same attestation but claim a different run.
        let mut replayed = genuine.clone();
        replayed.provenance = Some(provenance("run-B", false, &[]));
        let error = corpus.submit(&replayed).await.expect_err("replay");
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

    // --- MH-4 tamper-evidence, EI-06 revocation, MH-8 reopened-demotion -------

    #[tokio::test]
    async fn file_corpus_refuses_a_hand_edited_row() {
        let path = TempPath::new("tamper");
        {
            let corpus = FileCorpus::open(&path.0).expect("open");
            corpus
                .submit(&contribution(
                    OutcomeLabel::ResolvedConfirmed,
                    SignOff::HumanConfirmed,
                ))
                .await
                .expect("write a chained row");
        }
        // Hand-edit the persisted row's evidence to ANOTHER grammar-valid symptom
        // (so it still deserializes) — the chain no longer verifies.
        let text = std::fs::read_to_string(&path.0).expect("read");
        let tampered = text.replace("event_41", "xid_79");
        assert_ne!(text, tampered, "the edit must actually change the file");
        std::fs::write(&path.0, tampered).expect("write");
        assert!(
            matches!(FileCorpus::open(&path.0), Err(CorpusError::Storage(_))),
            "a hand-edited row must be caught as a storage (integrity) error"
        );
    }

    // --- The disk boundary (`FileCorpus::open`) re-de-ids leaf tokens (C4) -----
    //     A row on disk was never through the constructor, and open() alone (no
    //     authority) runs no per-row content gate — so the leaf `#[serde(try_from)]`
    //     guards are what refuse an out-of-vocab action / non-grammar symptom that
    //     an editor with file-write access (the keyless chain is recomputable)
    //     could craft. The refusal is a parse (Storage) error at open.

    #[tokio::test]
    async fn file_corpus_open_refuses_an_out_of_vocab_action_on_disk() {
        let path = TempPath::new("disk-poison-action");
        let row = r#"{"outcome":{"signature":{"fingerprint":"x","symptoms":["event_41"]},"plan":{"id":"p1","title":"cim_query","steps":[{"description":"cim_query","action":"powershell -c whoami on DESKTOP-NATHAN01","risk":"read_only"}]},"label":"resolved_confirmed","verification":{"result":"pass"}},"config_class":{"derived_hash":"x"},"sign_off":"human_confirmed"}"#;
        std::fs::write(&path.0, format!("{row}\n")).expect("write crafted row");
        assert!(
            matches!(FileCorpus::open(&path.0), Err(CorpusError::Storage(_))),
            "an out-of-vocab action on disk must fail to deserialize at open"
        );
    }

    #[tokio::test]
    async fn file_corpus_open_refuses_a_non_grammar_symptom_on_disk() {
        let path = TempPath::new("disk-poison-symptom");
        let row = r#"{"outcome":{"signature":{"fingerprint":"x","symptoms":["desktop-nathan01"]},"plan":{"id":"p1","title":"cim_query","steps":[{"description":"cim_query","action":"cim_query","risk":"read_only"}]},"label":"resolved_confirmed","verification":{"result":"pass"}},"config_class":{"derived_hash":"x"},"sign_off":"human_confirmed"}"#;
        std::fs::write(&path.0, format!("{row}\n")).expect("write crafted row");
        assert!(
            matches!(FileCorpus::open(&path.0), Err(CorpusError::Storage(_))),
            "a non-grammar symptom on disk must fail to deserialize at open"
        );
    }

    #[tokio::test]
    async fn file_corpus_open_refuses_a_poisoned_part_class_on_disk() {
        // #4: the label's part_class rides onto the row unmodified and egresses to
        // the API wire via `wire_label`. A hand-edited hardware label whose
        // part_class carries identity must fail to DESERIALIZE at open, matching
        // the other stored leaves.
        let path = TempPath::new("disk-poison-part-class");
        let row = r#"{"outcome":{"signature":{"fingerprint":"x","symptoms":["event_41"]},"plan":{"id":"p1","title":"cim_query","steps":[{"description":"cim_query","action":"cim_query","risk":"read_only"}]},"label":{"escalated_hardware":{"part_class":"psu on DESKTOP-NATHAN01"}}},"config_class":{"derived_hash":"x"},"sign_off":"human_confirmed"}"#;
        std::fs::write(&path.0, format!("{row}\n")).expect("write crafted row");
        assert!(
            matches!(FileCorpus::open(&path.0), Err(CorpusError::Storage(_))),
            "an identity-bearing part_class on disk must fail to deserialize at open"
        );
    }

    #[tokio::test]
    async fn file_corpus_open_refuses_a_poisoned_run_id_on_disk() {
        // #4: the provenance run_id rides onto the row unmodified. A hand-edited
        // row whose run_id is a smuggled path/email must fail to deserialize.
        let path = TempPath::new("disk-poison-run-id");
        let row = r#"{"outcome":{"signature":{"fingerprint":"x","symptoms":["event_41"]},"plan":{"id":"p1","title":"cim_query","steps":[{"description":"cim_query","action":"cim_query","risk":"read_only"}]},"label":"escalated_human_unresolved"},"config_class":{"derived_hash":"x"},"sign_off":"human_confirmed","provenance":{"run_id":"nathan@cec.direct","retrieval_first":false}}"#;
        std::fs::write(&path.0, format!("{row}\n")).expect("write crafted row");
        assert!(
            matches!(FileCorpus::open(&path.0), Err(CorpusError::Storage(_))),
            "an identity-bearing run_id on disk must fail to deserialize at open"
        );
    }

    #[tokio::test]
    async fn file_corpus_open_refuses_an_identity_bearing_title_on_disk() {
        // #5: StoredPlan.title is the one stored leaf with no deserialize-time
        // try_from. Every LEAF here is admissible (valid id, vocabulary
        // action/symptom) so the row deserializes clean, but the title carries
        // identity the mint would never produce (the mint reconstructs it as the
        // joined actions, "cim_query"). The de_identify_plan image check at open
        // must refuse it — a cold-start open (no authority) must not serve an
        // identity-bearing title into the human trace or consent screen.
        let path = TempPath::new("disk-poison-title");
        let row = r#"{"outcome":{"signature":{"fingerprint":"x","symptoms":["event_41"]},"plan":{"id":"p1","title":"DESKTOP-NATHAN01 nathan 192.168.1.20","steps":[{"description":"cim_query","action":"cim_query","risk":"read_only"}]},"label":"resolved_confirmed","verification":{"result":"pass"}},"config_class":{"derived_hash":"x"},"sign_off":"human_confirmed"}"#;
        std::fs::write(&path.0, format!("{row}\n")).expect("write crafted row");
        assert!(
            matches!(FileCorpus::open(&path.0), Err(CorpusError::Storage(_))),
            "an identity-bearing plan title on disk must be refused at cold-start open"
        );
    }

    #[tokio::test]
    async fn file_corpus_chain_survives_a_clean_reopen() {
        let path = TempPath::new("chain-ok");
        {
            let corpus = FileCorpus::open(&path.0).expect("open");
            for _ in 0..3 {
                corpus
                    .submit(&contribution(
                        OutcomeLabel::ResolvedProvisional,
                        SignOff::VerifierConfirmed,
                    ))
                    .await
                    .expect("write");
            }
        }
        // An untouched file reopens cleanly (the chain verifies).
        let reopened = FileCorpus::open(&path.0).expect("clean reopen");
        assert_eq!(reopened.len(), 3);
    }

    #[tokio::test]
    async fn a_revoked_plan_is_never_offered() {
        let corpus = LocalCorpus::new().with_revoked(["p1".to_string()]);
        corpus
            .submit(&contribution(
                OutcomeLabel::ResolvedConfirmed,
                SignOff::HumanConfirmed,
            ))
            .await
            .expect("row admitted");
        let hits = corpus
            .query(
                &FaultSignature::from_symptoms(vec![Symptom("event_41".into())]),
                &config_class(),
            )
            .await
            .expect("query");
        assert!(hits.is_empty(), "an owner-revoked plan is not retrievable");
    }

    #[tokio::test]
    async fn a_reopened_outcome_demotes_the_fix() {
        let corpus = LocalCorpus::new();
        let sig = FaultSignature::from_symptoms(vec![Symptom("event_41".into())]);
        // One confirmation, then a reopen for the same plan: net zero → not offered.
        corpus
            .submit(
                &contribution(OutcomeLabel::ResolvedConfirmed, SignOff::HumanConfirmed)
                    .with_provenance(provenance("run-A", false, &[])),
            )
            .await
            .expect("confirm");
        corpus
            .submit(
                &contribution(OutcomeLabel::Reopened, SignOff::HumanConfirmed)
                    .with_provenance(provenance("run-B", false, &[])),
            )
            .await
            .expect("reopen");
        let demoted = corpus.query(&sig, &config_class()).await.expect("query");
        assert!(
            demoted.is_empty(),
            "a reopened fix is demoted out of retrieval"
        );

        // A second independent confirmation outweighs the one reopen → offered (1).
        corpus
            .submit(
                &contribution(OutcomeLabel::ResolvedConfirmed, SignOff::HumanConfirmed)
                    .with_provenance(provenance("run-C", false, &[])),
            )
            .await
            .expect("reconfirm");
        let hits = corpus.query(&sig, &config_class()).await.expect("query");
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].confirmations, 1, "2 confirmations net 1 reopen");
    }

    #[tokio::test]
    async fn retrieval_is_scoped_to_the_config_class() {
        let corpus = LocalCorpus::new();
        let row = contribution(OutcomeLabel::ResolvedConfirmed, SignOff::HumanConfirmed);
        corpus.submit(&row).await.expect("accepted");
        let other_class = ConfigClass::from_inventory(["os:windows 10"]);
        let hits = corpus
            .query(&row.outcome.signature.to_signature(), &other_class)
            .await
            .expect("query");
        assert!(hits.is_empty(), "a ticket matches only like configs");
    }

    // --- Attestation binds the variant and the verdict (C2, C3) ---------------

    #[tokio::test]
    async fn the_config_class_variant_is_bound_not_just_its_key() {
        // `BomRevision("x")` and `DerivedHash("x")` share the key "x" but are
        // distinct comparability classes. Binding only the key would let one valid
        // attestation be replayed across classes; the variant tag prevents it.
        let authority = provenance::SignOffAuthority::generate();
        let corpus = LocalCorpus::new().with_authority(authority.public_key());
        let mut row = Contribution::new(
            Outcome {
                signature: FaultSignature::from_symptoms(vec![Symptom("event_41".into())]),
                plan: Plan::new("p1", "restart service"),
                label: OutcomeLabel::ResolvedConfirmed,
                verification: Some(Verification::pass()),
            },
            ConfigClass::BomRevision("x".into()),
            SignOff::HumanConfirmed,
        )
        .expect("test contribution de-identifies")
        .attested_by(&authority);
        // Flip to the other variant with the same inner key.
        row.config_class = ConfigClass::DerivedHash("x".into());
        let error = corpus.submit(&row).await.expect_err("variant rebind");
        assert!(matches!(
            error,
            CorpusError::Gate(GateError::AttestationInvalid)
        ));
    }

    #[tokio::test]
    async fn the_verification_verdict_is_bound_to_the_attestation() {
        // The gate keys a resolved label on the verdict, so the authority must
        // sign the verdict it approved. Swapping in a *different* passing verdict
        // (still a Pass, so the label check would accept it) must break the
        // attestation — otherwise the evidence is unauthenticated.
        let authority = provenance::SignOffAuthority::generate();
        let corpus = LocalCorpus::new().with_authority(authority.public_key());
        let row = contribution(OutcomeLabel::ResolvedConfirmed, SignOff::HumanConfirmed)
            .attested_by(&authority);
        let mut tampered = row.clone();
        tampered.outcome.verification = Some(Verification {
            result: VerificationResult::Pass,
            class: Some(VerificationClass::Deterministic),
            recurring: Vec::new(),
            cleared: Vec::new(),
            introduced: Vec::new(),
        });
        let error = corpus.submit(&tampered).await.expect_err("swapped verdict");
        assert!(matches!(
            error,
            CorpusError::Gate(GateError::AttestationInvalid)
        ));
    }

    // --- Open-time re-admission: at-rest rows face the same boundary (C4/5/6) --

    #[tokio::test]
    async fn an_attested_file_reopens_under_its_authority() {
        let authority = provenance::SignOffAuthority::generate();
        let path = TempPath::new("attested-reopen");
        {
            let corpus = FileCorpus::open(&path.0)
                .expect("open")
                .with_authority(authority.public_key())
                .expect("empty file opens");
            corpus
                .submit(
                    &contribution(OutcomeLabel::ResolvedConfirmed, SignOff::HumanConfirmed)
                        .attested_by(&authority),
                )
                .await
                .expect("attested row admitted");
        }
        // Reopening under the same authority re-verifies every at-rest row.
        let reopened = FileCorpus::open(&path.0)
            .expect("chain verifies")
            .with_authority(authority.public_key())
            .expect("attested history opens");
        assert_eq!(reopened.len(), 1);
    }

    #[tokio::test]
    async fn a_cold_start_file_is_refused_under_an_authority() {
        // A row written at cold start carries no attestation. The keyless chain
        // still verifies, but opening UNDER an authority must refuse it rather
        // than serve unattested history.
        let authority = provenance::SignOffAuthority::generate();
        let path = TempPath::new("coldstart-then-authority");
        {
            let corpus = FileCorpus::open(&path.0).expect("open");
            corpus
                .submit(&contribution(
                    OutcomeLabel::ResolvedConfirmed,
                    SignOff::HumanConfirmed,
                ))
                .await
                .expect("cold-start write");
        }
        let opened = FileCorpus::open(&path.0).expect("keyless chain verifies");
        assert!(
            matches!(
                opened.with_authority(authority.public_key()),
                Err(CorpusError::Storage(_))
            ),
            "an unattested at-rest row must be refused once an authority is configured"
        );
    }

    #[tokio::test]
    async fn a_rechained_forged_file_is_refused_under_an_authority() {
        // The C6 attack: an editor fabricates a confirmed row with NO attestation
        // and recomputes the (keyless) chain. open() alone is fooled — the chain
        // verifies — but an authority re-checks the attestation and refuses it.
        let authority = provenance::SignOffAuthority::generate();
        let path = TempPath::new("rechained-forge");
        let mut forged = contribution(OutcomeLabel::ResolvedConfirmed, SignOff::HumanConfirmed);
        let hash = chain_hash("", &forged);
        forged.integrity = Some(RowIntegrity {
            prev: String::new(),
            hash,
        });
        let line = serde_json::to_string(&forged).expect("serialize");
        std::fs::write(&path.0, format!("{line}\n")).expect("write forged file");

        let opened = FileCorpus::open(&path.0).expect("keyless chain verifies the forgery");
        assert_eq!(opened.len(), 1, "open() alone serves the forged row");
        assert!(
            matches!(
                opened.with_authority(authority.public_key()),
                Err(CorpusError::Storage(_))
            ),
            "the attestation re-check refuses a forged at-rest row"
        );
    }

    // --- Reopen demotion is run-deduped, symmetric with confirmations (C7/C14) -

    #[tokio::test]
    async fn a_replayed_reopen_does_not_over_demote_a_confirmed_fix() {
        // Two genuinely independent confirmations, then the SAME reopen run
        // submitted twice. A replayed reopen must cancel ONE confirmation, not
        // two — otherwise duplicating a single Reopened line buries the fix.
        let corpus = LocalCorpus::new();
        let sig = FaultSignature::from_symptoms(vec![Symptom("event_41".into())]);
        for run in ["run-A", "run-C"] {
            corpus
                .submit(
                    &contribution(OutcomeLabel::ResolvedConfirmed, SignOff::HumanConfirmed)
                        .with_provenance(provenance(run, false, &[])),
                )
                .await
                .expect("confirm");
        }
        for _ in 0..2 {
            corpus
                .submit(
                    &contribution(OutcomeLabel::Reopened, SignOff::HumanConfirmed)
                        .with_provenance(provenance("run-B", false, &[])),
                )
                .await
                .expect("reopen replay");
        }
        let hits = corpus.query(&sig, &config_class()).await.expect("query");
        assert_eq!(
            hits.len(),
            1,
            "two confirmations survive one replayed reopen"
        );
        assert_eq!(
            hits[0].confirmations, 1,
            "2 confirmations net exactly 1 distinct reopen run"
        );
    }

    // --- HttpCorpus enforces the gate before the network (C10) ----------------

    #[tokio::test]
    async fn http_corpus_enforces_the_gate_before_the_network() {
        // An unroutable base url: if the gate did not fire first, we would get a
        // Transport error (or hang), not a Gate error.
        let corpus = HttpCorpus::new("http://127.0.0.1:0");
        let error = corpus
            .submit(&contribution(
                OutcomeLabel::ResolvedConfirmed,
                SignOff::Unconfirmed,
            ))
            .await
            .expect_err("gate rejects before the network");
        assert!(
            matches!(error, CorpusError::Gate(GateError::Unconfirmed)),
            "an unconfirmed row must fail at the gate, never reach the transport"
        );
    }

    #[tokio::test]
    async fn http_corpus_with_authority_refuses_an_unattested_row_before_the_network() {
        let authority = provenance::SignOffAuthority::generate();
        let corpus = HttpCorpus::new("http://127.0.0.1:0").with_authority(authority.public_key());
        let error = corpus
            .submit(&contribution(
                OutcomeLabel::ResolvedConfirmed,
                SignOff::HumanConfirmed,
            ))
            .await
            .expect_err("missing attestation");
        assert!(matches!(
            error,
            CorpusError::Gate(GateError::AttestationMissing)
        ));
    }

    // --- A partial (legacy + chained) file is a tamper (C13) ------------------

    #[tokio::test]
    async fn a_mixed_legacy_and_chained_file_is_refused() {
        // Row 0 legacy (no integrity), row 1 chained: the "some rows have no
        // chain link" splice path — a tamper, not silent acceptance.
        let path = TempPath::new("mixed-chain");
        let legacy = contribution(OutcomeLabel::ResolvedConfirmed, SignOff::HumanConfirmed);
        let mut chained = contribution(
            OutcomeLabel::ResolvedProvisional,
            SignOff::VerifierConfirmed,
        );
        let hash = chain_hash("", &chained);
        chained.integrity = Some(RowIntegrity {
            prev: String::new(),
            hash,
        });
        let body = format!(
            "{}\n{}\n",
            serde_json::to_string(&legacy).expect("serialize legacy"),
            serde_json::to_string(&chained).expect("serialize chained"),
        );
        std::fs::write(&path.0, body).expect("write");
        assert!(
            matches!(FileCorpus::open(&path.0), Err(CorpusError::Storage(_))),
            "a file mixing chained and unchained rows is refused"
        );
    }

    // --- The v4 attestation: provenance bound as a COMMITMENT (Q6) ------------

    #[test]
    fn the_v4_message_binds_provenance_only_through_its_commitment() {
        // Q6 minimization: the signed bytes must no longer embed the raw run
        // id or priming graph — only the commitment — so a served row can
        // carry (attested outcome + commitment) and still verify.
        let row = contribution(OutcomeLabel::ResolvedConfirmed, SignOff::HumanConfirmed)
            .with_provenance(provenance("run-secret-7781", true, &["primed-plan-a"]));
        let message = String::from_utf8(crate::schema::attestation_message(&row))
            .expect("canonical bytes are utf8");
        assert!(
            message.starts_with("cec-signoff-attestation-v4\n"),
            "the v4 domain tag must lead the message"
        );
        assert!(
            !message.contains("run-secret-7781") && !message.contains("primed-plan-a"),
            "raw provenance fields must not appear in the signed bytes: {message}"
        );
        let commitment = row.provenance().expect("present").commitment();
        assert!(
            message.contains(&commitment),
            "the provenance commitment must be bound into the signed bytes"
        );
    }

    #[test]
    fn provenance_commitment_binds_every_field_and_sorts_primed() {
        let base = provenance("run-A", true, &["p1", "p2"]);
        let c = base.commitment();
        assert_eq!(c.len(), 64);
        assert_ne!(provenance("run-B", true, &["p1", "p2"]).commitment(), c);
        assert_ne!(provenance("run-A", false, &["p1", "p2"]).commitment(), c);
        assert_ne!(provenance("run-A", true, &["p1"]).commitment(), c);
        assert_ne!(provenance("run-A", true, &["p1", "p3"]).commitment(), c);
        assert_eq!(
            provenance("run-A", true, &["p2", "p1"]).commitment(),
            c,
            "primed_from is a set: order must not move the commitment"
        );
    }

    // --- The v2 chain canonical encoding (serde-independent, F2) --------------

    /// A fully-populated contribution with FIXED values everywhere (a literal
    /// fingerprint, a BOM config class, a hand-set attestation) so the chain
    /// canonical bytes are reproducible below without serde or the encoder.
    fn populated_row() -> Contribution {
        let mut plan = Plan::new("heuristic-1", "free text title");
        plan.steps.push(common::PlanStep {
            description: "d1".into(),
            action: "cim_query".into(),
            risk: common::Risk::ReadOnly,
        });
        plan.steps.push(common::PlanStep {
            description: "d2".into(),
            action: "registry_set".into(),
            risk: common::Risk::Reversible,
        });
        let mut row = Contribution::new(
            Outcome {
                signature: FaultSignature {
                    fingerprint: "00ffaa".into(),
                    symptoms: vec![Symptom("event_41".into()), Symptom("0x1234".into())],
                },
                plan,
                label: OutcomeLabel::EscalatedHardware {
                    part_class: "psu".into(),
                },
                verification: Some(Verification {
                    result: VerificationResult::Fail,
                    class: Some(VerificationClass::Intermittent),
                    recurring: vec![Symptom("event_41".into()), Symptom("0x1234".into())],
                    cleared: Vec::new(),
                    introduced: Vec::new(),
                }),
            },
            ConfigClass::from_bom("rev-a"),
            SignOff::HumanConfirmed,
        )
        .expect("populated row de-identifies")
        .with_provenance(provenance("run-77", true, &["plan-a", "plan-b"]));
        row.attestation = Some(SignOffAttestation {
            authority_id: "authid01".into(),
            signature: "deadbeef".into(),
        });
        row.integrity = Some(RowIntegrity {
            prev: "ignored-prev".into(),
            hash: "ignored-hash".into(),
        });
        row
    }

    #[test]
    fn chain_hash_matches_the_documented_canonical_encoding() {
        use sha2::{Digest, Sha256};
        // The expected bytes are assembled here BY HAND from the declared
        // `cec-corpus-chain-v2` encoding — no serde, no call into the encoder —
        // so this test is the serde-independence pin: a struct-layout or serde-
        // attribute change cannot move the hash, and any change to the encoding
        // itself must consciously edit this literal. Lists are in STORED order
        // (symptoms deliberately unsorted here: "event_41" before "0x1234").
        let expected_canonical = "cec-corpus-chain-v2\n\
            prev[4]=prev\n\
            fp[6]=00ffaa\n\
            syms:2\n\
            sym[8]=event_41\n\
            sym[6]=0x1234\n\
            plan[11]=heuristic-1\n\
            title[25]=cim_query -> registry_set\n\
            steps:2\n\
            desc[9]=cim_query\n\
            act[9]=cim_query\n\
            risk:ReadOnly\n\
            desc[12]=registry_set\n\
            act[12]=registry_set\n\
            risk:Reversible\n\
            label[22]=escalated_hardware:psu\n\
            ver:Fail;class=Some(Intermittent);rec:2\n\
            rec[8]=event_41\n\
            rec[6]=0x1234\n\
            signoff:HumanConfirmed\n\
            class:bom\n\
            classkey[5]=rev-a\n\
            attid[8]=authid01\n\
            attsig[8]=deadbeef\n\
            run[6]=run-77\n\
            rf:true\n\
            primed:2\n\
            primed[6]=plan-a\n\
            primed[6]=plan-b\n";
        let mut hasher = Sha256::new();
        hasher.update(expected_canonical.as_bytes());
        let expected: String = hasher
            .finalize()
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect();
        assert_eq!(
            chain_hash("prev", &populated_row()),
            expected,
            "chain_hash must be sha256 over exactly the documented v2 canonical bytes"
        );
    }

    #[test]
    fn chain_hash_binds_every_field_and_ignores_integrity() {
        let base = populated_row();
        let base_hash = chain_hash("prev", &base);

        // The `integrity` field is the ONE exclusion (it holds this hash).
        let mut no_integrity = base.clone();
        no_integrity.integrity = None;
        assert_eq!(
            chain_hash("prev", &no_integrity),
            base_hash,
            "the integrity field must never feed its own hash"
        );

        // Every other field, mutated alone, must change the hash — and no two
        // mutations may collide with each other (pairwise-distinct encodings).
        type Mutation = (&'static str, Box<dyn Fn(&mut Contribution)>);
        let mutations: Vec<Mutation> = vec![
            (
                "fingerprint",
                Box::new(|c| c.outcome.signature.fingerprint = "beefed".into()),
            ),
            (
                "symptom value",
                Box::new(|c| c.outcome.signature.symptoms[0] = StoredSymptom("0x9999".into())),
            ),
            (
                "symptom stored order",
                Box::new(|c| c.outcome.signature.symptoms.swap(0, 1)),
            ),
            (
                "plan id",
                Box::new(|c| c.outcome.plan.id = StoredPlanId::from("other-plan")),
            ),
            (
                "plan title",
                Box::new(|c| c.outcome.plan.title = "forged title".into()),
            ),
            (
                "step description",
                Box::new(|c| c.outcome.plan.steps[0].description = StoredAction::from("forged")),
            ),
            (
                "step action",
                Box::new(|c| c.outcome.plan.steps[0].action = StoredAction::from("forged2")),
            ),
            (
                "step risk",
                Box::new(|c| c.outcome.plan.steps[0].risk = common::Risk::Destructive),
            ),
            (
                "step count",
                Box::new(|c| {
                    c.outcome.plan.steps.pop();
                }),
            ),
            (
                "label",
                Box::new(|c| c.outcome.label = OutcomeLabel::Withdrawn),
            ),
            (
                "label part_class",
                Box::new(|c| {
                    c.outcome.label = OutcomeLabel::EscalatedHardware {
                        part_class: "gpu".into(),
                    }
                }),
            ),
            (
                "verification dropped",
                Box::new(|c| c.outcome.verification = None),
            ),
            (
                "verification result",
                Box::new(|c| {
                    c.outcome.verification.as_mut().expect("present").result =
                        VerificationResult::Pass
                }),
            ),
            (
                "verification class",
                Box::new(|c| c.outcome.verification.as_mut().expect("present").class = None),
            ),
            (
                "verification recurring",
                Box::new(|c| {
                    c.outcome
                        .verification
                        .as_mut()
                        .expect("present")
                        .recurring
                        .pop();
                }),
            ),
            (
                "sign_off",
                Box::new(|c| c.sign_off = SignOff::VerifierConfirmed),
            ),
            (
                "config-class variant (same key)",
                Box::new(|c| c.config_class = ConfigClass::DerivedHash("rev-a".into())),
            ),
            (
                "config-class key",
                Box::new(|c| c.config_class = ConfigClass::from_bom("rev-b")),
            ),
            ("attestation dropped", Box::new(|c| c.attestation = None)),
            (
                "attestation authority id",
                Box::new(|c| {
                    c.attestation.as_mut().expect("present").authority_id = "other".into()
                }),
            ),
            (
                "attestation signature",
                Box::new(|c| {
                    c.attestation.as_mut().expect("present").signature = "feedface".into()
                }),
            ),
            ("provenance dropped", Box::new(|c| c.provenance = None)),
            (
                "provenance run id",
                Box::new(|c| c.provenance.as_mut().expect("present").run_id = "run-88".into()),
            ),
            (
                "provenance retrieval_first",
                Box::new(|c| c.provenance.as_mut().expect("present").retrieval_first = false),
            ),
            (
                "provenance primed_from",
                Box::new(|c| {
                    c.provenance.as_mut().expect("present").primed_from.pop();
                }),
            ),
        ];
        let mut seen = std::collections::HashSet::new();
        seen.insert(base_hash.clone());
        seen.insert(chain_hash("other-prev", &base));
        for (name, mutate) in &mutations {
            let mut mutated = base.clone();
            mutate(&mut mutated);
            let hash = chain_hash("prev", &mutated);
            assert_ne!(
                hash, base_hash,
                "mutating {name} must change the chain hash"
            );
            assert!(
                seen.insert(hash),
                "mutation {name} collided with another mutation's encoding"
            );
        }
    }

    #[test]
    fn chain_canonical_is_unambiguous_across_field_boundaries() {
        // A value carrying a plausible encoded LINE must not collide with the
        // genuinely-structured row: length-prefixing keeps "one symptom whose
        // bytes spell two" byte-distinct from two symptoms.
        let mut smuggled = populated_row();
        smuggled.outcome.signature.symptoms = vec![StoredSymptom("a\nsym[1]=b".into())];
        let mut genuine = populated_row();
        genuine.outcome.signature.symptoms =
            vec![StoredSymptom("a".into()), StoredSymptom("b".into())];
        assert_ne!(
            chain_hash("prev", &smuggled),
            chain_hash("prev", &genuine),
            "a symptom embedding an encoded line must not forge a second symptom"
        );
    }

    // --- The query READ path re-validates served plans ------------------------

    /// One-shot HTTP responder: accepts a single loopback connection and
    /// answers any request with the given JSON body.
    fn serve_one_json(body: String) -> String {
        let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("bind");
        let addr = listener.local_addr().expect("addr");
        std::thread::spawn(move || {
            use std::io::{Read, Write};
            if let Ok((mut stream, _)) = listener.accept() {
                let mut buf = [0u8; 4096];
                let _ = stream.read(&mut buf);
                let response = format!(
                    "HTTP/1.1 200 OK\r\ncontent-type: application/json\r\n\
                     content-length: {}\r\nconnection: close\r\n\r\n{}",
                    body.len(),
                    body
                );
                let _ = stream.write_all(response.as_bytes());
            }
        });
        format!("http://{addr}")
    }

    #[tokio::test]
    async fn http_query_refuses_a_served_out_of_vocab_action_at_the_wire() {
        // Layer 1 (leaf validation): a compromised (or buggy) corpus server feeds
        // a mapping whose stored step action is request prose the mint would never
        // emit. Built as a struct literal (only possible in-crate) to simulate
        // wire bytes; on the READ path `StoredAction`'s `try_from` makes the body
        // FAIL to deserialize, so the poisoned row never reaches retrieval-first.
        let plan = StoredPlan {
            id: "p1".into(),
            title: "cim_query".into(),
            steps: vec![StoredStep {
                description: "run powershell on DESKTOP-NATHAN01".into(),
                action: "powershell -c Get-CimInstance on DESKTOP-NATHAN01".into(),
                risk: common::Risk::ReadOnly,
            }],
        };
        let mapping = FixMapping {
            signature: StoredSignature::from_signature(&FaultSignature::from_symptoms(vec![
                Symptom("event_41".into()),
            ])),
            plan,
            confirmations: 3,
        };
        let base = serve_one_json(serde_json::to_string(&vec![mapping]).expect("serialize"));
        let corpus = HttpCorpus::new(base);
        let error = corpus
            .query(
                &FaultSignature::from_symptoms(vec![Symptom("event_41".into())]),
                &config_class(),
            )
            .await
            .expect_err("a served out-of-vocab action must be refused");
        assert!(matches!(
            error,
            CorpusError::Gate(GateError::ServedPlanInadmissible)
        ));
    }

    #[tokio::test]
    async fn http_query_refuses_a_hand_edited_title_via_the_image_check() {
        // Layer 2 (de-identified image): every LEAF is admissible — valid id,
        // vocabulary action/description — so the body deserializes cleanly, but
        // the derived `title` (a plain string, not leaf-typed) carries identity
        // the mint would never produce. The `de_identify_plan` equality check
        // reconstructs the title as the joined actions and refuses the mismatch.
        let plan = StoredPlan {
            id: "p1".into(),
            title: "Fix DESKTOP-NATHAN01 for nathan".into(), // != "cim_query"
            steps: vec![StoredStep {
                description: "cim_query".into(),
                action: "cim_query".into(),
                risk: common::Risk::ReadOnly,
            }],
        };
        let mapping = FixMapping {
            signature: StoredSignature::from_signature(&FaultSignature::from_symptoms(vec![
                Symptom("event_41".into()),
            ])),
            plan,
            confirmations: 3,
        };
        let base = serve_one_json(serde_json::to_string(&vec![mapping]).expect("serialize"));
        let corpus = HttpCorpus::new(base);
        let error = corpus
            .query(
                &FaultSignature::from_symptoms(vec![Symptom("event_41".into())]),
                &config_class(),
            )
            .await
            .expect_err("a hand-edited title must be refused");
        assert!(matches!(
            error,
            CorpusError::Gate(GateError::ServedPlanInadmissible)
        ));
    }

    #[tokio::test]
    async fn http_query_refuses_an_adversary_seeded_served_symptom() {
        // 2d — the read-path poison harness. The adversary controls the SERVED
        // bytes: every canonical `leakguard::POISON` token is planted into a
        // served signature symptom. A clean action/id keeps the plan admissible,
        // so ONLY the symptom guard can catch it — `StoredSymptom`'s `try_from`
        // makes each poisoned body fail to deserialize (the closed grammar
        // refuses a hostname/asset/MAC/serial shape), so the poisoned mapping is
        // never handed to retrieval-first.
        let mut raw = Plan::new("heuristic-1", "title");
        raw.steps.push(common::PlanStep {
            description: "d".into(),
            action: "cim_query".into(),
            risk: common::Risk::ReadOnly,
        });
        let clean_plan = de_identify_plan(&raw).expect("clean plan de-identifies");
        for poison in leakguard::POISON {
            // Build the served signature with the poison token as a "symptom".
            // `from_symptom` wraps without validating (the construction path);
            // the leak is caught on the READ, at deserialize.
            let signature = StoredSignature::from_signature(&FaultSignature::from_symptoms(vec![
                Symptom::from(*poison),
            ]));
            let mapping = FixMapping {
                signature,
                plan: clean_plan.clone(),
                confirmations: 5,
            };
            let base = serve_one_json(serde_json::to_string(&vec![mapping]).expect("serialize"));
            let corpus = HttpCorpus::new(base);
            let result = corpus
                .query(
                    &FaultSignature::from_symptoms(vec![Symptom::from(*poison)]),
                    &config_class(),
                )
                .await;
            assert!(
                matches!(
                    result,
                    Err(CorpusError::Gate(GateError::ServedPlanInadmissible))
                ),
                "adversary-seeded symptom {poison:?} must be refused on read, got {result:?}"
            );
        }
    }

    #[tokio::test]
    async fn http_query_admits_a_clean_de_identified_mapping() {
        let mut raw = Plan::new("heuristic-1", "free text title");
        raw.steps.push(common::PlanStep {
            description: "prose description".into(),
            action: "cim_query".into(),
            risk: common::Risk::ReadOnly,
        });
        let clean = de_identify_plan(&raw).expect("a clean-action plan de-identifies");
        let mapping = FixMapping {
            signature: StoredSignature::from_signature(&FaultSignature::from_symptoms(vec![
                Symptom("event_41".into()),
            ])),
            plan: clean.clone(),
            confirmations: 1,
        };
        let base = serve_one_json(serde_json::to_string(&vec![mapping]).expect("serialize"));
        let corpus = HttpCorpus::new(base);
        let got = corpus
            .query(
                &FaultSignature::from_symptoms(vec![Symptom("event_41".into())]),
                &config_class(),
            )
            .await
            .expect("a clean de-identified mapping is admitted");
        assert_eq!(got.len(), 1);
        assert_eq!(got[0].plan, clean);
    }

    // --- On-disk/wire compatibility + the v2 hard cutover ---------------------
    //     The StoredPlan/StoredSymptom type split pinned the wire SHAPE (field
    //     names, order, serde attributes) — that pin stays: the canned row must
    //     (a) deserialize, (b) round-trip byte-identically, (c) verify its
    //     tamper-evidence chain at open, and (d) pass the evidence-integrity
    //     gate. The 2026-07 migration bundle (chain `cec-corpus-chain-v2`)
    //     deliberately invalidated stored hash VALUES: a v1-era file must now be
    //     REFUSED at open (hard cutover — the private corpus re-ingests once).
    //
    //     The canned rows are assembled from split literals so this file's TEXT
    //     never carries a contiguous serialized corpus-row shape: the invariant
    //     hook's corpus-row backstop (SECURITY.md) scans file text, a synthetic
    //     fixture is its one sanctioned exception, and splitting keeps the
    //     backstop meaningful for real rows — the same fragment-assembly trick
    //     the hook uses for its own key-block markers.

    /// The shared row head, split BEFORE the fingerprint value.
    const CANNED_ROW_HEAD: &str = r#"{"outcome":{"signature":{"fingerprint":"#;

    /// The current row, captured verbatim from `FileCorpus::submit` (minus the
    /// head): fingerprint value onward, chained under `cec-corpus-chain-v2`. If
    /// the round-trip assertion breaks, the wire shape drifted and existing
    /// JSONL corpora + hash chains would fail to load — a hard-constraint
    /// regression.
    const CANNED_ROW_REST: &str = r#""fff691c0bd22a2f56518c192a32c08d963fcce3c268d3bdbfc4585793f772db6","symptoms":["0x1234","event_41","explorer.exe"]},"plan":{"id":"heuristic-1","title":"cim_query -> registry_set","steps":[{"description":"cim_query","action":"cim_query","risk":"read_only"},{"description":"registry_set","action":"registry_set","risk":"reversible"}]},"label":"resolved_confirmed","verification":{"result":"pass"}},"config_class":{"derived_hash":"e78fadbf556168e39c6b6b82fa7ed43f5b13882969324c184437de943605b45d"},"sign_off":"human_confirmed","provenance":{"run_id":"run-fixture","retrieval_first":false},"integrity":{"prev":"","hash":"3643fc60d7a248068b8b8b94006ef3b61c7dbbfbdcce3d46db0205d3cf3af522"}}"#;

    /// The SAME row as chained by the retired v1 (serde-image) encoding —
    /// frozen history, kept to pin the hard cutover.
    const V1_ERA_ROW_REST: &str = r#""acfebcbe984c7cd1","symptoms":["0x1234","event_41","explorer.exe"]},"plan":{"id":"heuristic-1","title":"cim_query -> registry_set","steps":[{"description":"cim_query","action":"cim_query","risk":"read_only"},{"description":"registry_set","action":"registry_set","risk":"reversible"}]},"label":"resolved_confirmed","verification":{"result":"pass"}},"config_class":{"derived_hash":"edc9373418556993"},"sign_off":"human_confirmed","provenance":{"run_id":"run-fixture","retrieval_first":false},"integrity":{"prev":"","hash":"74415a1e1785a230eb1eaa159607812e45c616fe08956a821b2092c7bde0fc2d"}}"#;

    fn canned_row() -> String {
        format!("{CANNED_ROW_HEAD}{CANNED_ROW_REST}")
    }

    #[test]
    fn the_canned_row_deserializes_and_round_trips_byte_identically() {
        let canned = canned_row();
        let row: Contribution =
            serde_json::from_str(&canned).expect("canned row still deserializes");
        let reserialized = serde_json::to_string(&row).expect("serializes");
        assert_eq!(
            reserialized, canned,
            "the corpus-row wire shape changed — existing JSONL corpora would break"
        );
        // The row is still admissible truth (resolved+confirmed+passing verdict).
        assert!(ensure_evidence_integrity(&row).is_ok());
    }

    #[tokio::test]
    async fn the_canned_file_opens_verifies_and_serves() {
        let canned = canned_row();
        let path = TempPath::new("canned-load");
        std::fs::write(&path.0, format!("{canned}\n")).expect("write canned row");
        // open() runs `verify_chain`: the stored hash must match the recomputed
        // v2 canonical chain.
        let corpus = FileCorpus::open(&path.0).expect("canned v2 chain verifies at open");
        assert_eq!(corpus.len(), 1);
        // And it is retrievable as a fix at its own signature + config class.
        let row: Contribution = serde_json::from_str(&canned).expect("deserialize");
        let hits = corpus
            .query(
                &row.outcome.signature.to_signature(),
                &row.config_class.clone(),
            )
            .await
            .expect("query");
        assert_eq!(hits.len(), 1, "the canned row still backs a fix mapping");
        assert_eq!(hits[0].plan.id(), "heuristic-1");
    }

    #[test]
    fn a_v1_era_file_is_refused_at_open() {
        // The hard cutover, pinned: a file chained under the retired v1
        // (serde-image) encoding fails `verify_chain` at open — it must be
        // re-ingested, never silently accepted alongside v2 rows.
        let path = TempPath::new("v1-era-refused");
        std::fs::write(&path.0, format!("{CANNED_ROW_HEAD}{V1_ERA_ROW_REST}\n"))
            .expect("write v1-era row");
        assert!(
            matches!(FileCorpus::open(&path.0), Err(CorpusError::Storage(_))),
            "a v1-era chain must be refused at open (hard cutover)"
        );
    }
}
