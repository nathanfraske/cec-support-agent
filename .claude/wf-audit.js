export const meta = {
  name: 'autodiagnoser-engine-audit',
  description: 'Adversarial audit of the evidence-integrity engine diff: 5 review dimensions → independently verify each finding → confirmed findings to fix',
  phases: [{ title: 'Review' }, { title: 'Verify' }],
}

const REPO = '/home/nathan/CEC_AutoDiagnoser'
const DIFF = `${REPO}/.claude/audit/engine.diff`
const CTX = `You are auditing the evidence-integrity engine changes in the cec-support-agent Rust workspace (branch feat/agent-ops-evidence-integrity). The full diff vs main is at ${DIFF}; READ IT, and read the actual current source under ${REPO}/crates/*/src/*.rs to confirm context (the diff is the source of truth for what changed).

What the changes implement (so you judge against intent):
- A structured truth-admission gate (corpus-client/src/gate.rs): ensure_evidence_integrity (sign-off confirmed; a RESOLVED label needs a matching passing Verification; a resolved DESTRUCTIVE plan needs HumanConfirmed) + ensure_attested (ed25519 attestation by an authority the engine doesn't hold).
- ed25519 sign-off attestation (provenance/src/lib.rs: SignOffAuthority/SignOffPublicKey/SignOffSignature) over a canonical tuple (corpus-client/src/schema.rs: attestation_message, which also binds RowProvenance).
- RowProvenance (run_id/retrieval_first/primed_from) + EI-03 independent-confirmation counting + Reopened-demotion + owner revocation + a sha256 hash-chain tamper-evidence (RowIntegrity) on FileCorpus (corpus-client/src/store.rs).
- Verdict::Unverified for no-real-re-collection (agent-core/src/verify.rs), deterministic plan canonicalization (provenance), risk reconciliation (agent-core/src/dispatch.rs), sandbox-validation wiring + CLI operator wiring for the keys (support-agent/src/main.rs).

Hunt for REAL defects: correctness bugs, security/integrity bypasses, broken invariants, math errors (off-by-one, saturating arithmetic, key collisions), non-determinism in signed/hashed bytes, unhandled edge cases, panics, and tests that are tautological or that assert the wrong thing. Be concrete: cite file:line and explain the exact failure path. Do not report style nits.`

const FINDINGS_SCHEMA = {
  type: 'object',
  additionalProperties: true,
  properties: {
    dimension: { type: 'string' },
    findings: {
      type: 'array',
      items: {
        type: 'object',
        additionalProperties: true,
        properties: {
          title: { type: 'string' },
          severity: { type: 'string', enum: ['critical', 'high', 'medium', 'low'] },
          location: { type: 'string' },
          description: { type: 'string' },
          failure_path: { type: 'string' },
          suggested_fix: { type: 'string' },
          confidence: { type: 'string', enum: ['high', 'medium', 'low'] },
        },
        required: ['title', 'severity', 'location', 'description', 'suggested_fix'],
      },
    },
  },
  required: ['dimension', 'findings'],
}

const VERDICT_SCHEMA = {
  type: 'object',
  additionalProperties: true,
  properties: {
    title: { type: 'string' },
    verdict: { type: 'string', enum: ['confirmed', 'refuted', 'uncertain'] },
    reasoning: { type: 'string' },
    corrected_severity: { type: 'string', enum: ['critical', 'high', 'medium', 'low'] },
    exact_location: { type: 'string' },
    fix: { type: 'string' },
  },
  required: ['title', 'verdict', 'reasoning', 'corrected_severity'],
}

const DIMENSIONS = [
  { key: 'crypto-attestation', prompt: `Focus: the ed25519 sign-off ATTESTATION and its canonical message. Check provenance/src/lib.rs (SignOffAuthority/PublicKey/Signature, from_hex/from_seed_hex/unhex_array bounds) and corpus-client/src/schema.rs attestation_message. Is the message DETERMINISTIC and UNAMBIGUOUS (could two different contributions produce the same message — field-injection via unescaped separators in plan id/title/action, symptoms, config_class.key(), run_id)? Does it cover everything that must be bound (signature, plan incl. risk, label incl. EscalatedHardware part_class, sign_off, config_class, provenance)? Can a valid attestation be replayed onto a different row? Any panic in hex/byte parsing? Is ensure_attested's verification correct?` },
  { key: 'gate-bypass', prompt: `Focus: the admission gate. corpus-client/src/gate.rs (ensure_evidence_integrity + ensure_attested) and store.rs admit(). Can any row reach disk/memory/network that should be refused? Check: the resolved⇔verdict matching (LabelVerdictMismatch vs ResolvedWithoutPass branches — is every case covered correctly?), the destructive-resolved⇒human rule (does it use the de-identified plan's real max risk?), confirmed-but-unattested when an authority IS set (across LocalCorpus/FileCorpus/HttpCorpus — does HttpCorpus enforce attestation? it does not call admit before the gate?), and whether a non-resolved (hard-negative) row with a destructive plan should also need human. Is the cold-start (no authority) fallback safe?` },
  { key: 'chain-and-math', prompt: `Focus: the tamper-evidence chain and the confirmation math in corpus-client/src/store.rs. chain_hash/verify_chain: is the chain sound (prev-binding, ordering, the all-or-nothing rule), and what EXACTLY is the tail-truncation gap — can a row be appended/removed at the tail undetected, or a whole file replaced? FileCorpus::submit attaches integrity under the chain_head mutex — is there a TOCTOU or a mismatch between the in-memory rows and the file (e.g. on a write error after the head moves)? fix_mappings: confirmation_key legacy "row:{index}" — can indices collide or double-count across queries? Is the reopened saturating_sub correct (could a reopen for a DIFFERENT plan/run wrongly demote)? Does revocation key on plan_id correctly?` },
  { key: 'cli-dataflow', prompt: `Focus: support-agent/src/main.rs data flow. Is run_provenance correctly populated (primed_from = the precedent plan ids; does it match what EI-03 expects)? Is the attestation attached AFTER provenance in record_outcome (so the run_id is bound)? The env key handling (parse_env_pubkey/authority): set-but-invalid is a hard error — is there a path where enforcement is silently skipped? recollect_post_signature returns None → Unverified → is the label/flow correct, and is the original signature still derived honestly? risk reconciliation runs on all candidates incl. CorpusPrimed — correct? Any ordering bug where escalation is computed before sandbox validation? Does to_verification(class) bind the right class?` },
  { key: 'tests-and-gaps', prompt: `Focus: test adequacy and silent-regression risk across the diff. For each new invariant (attestation-required, replay-defeated, resolved-needs-verdict, destructive-needs-human, independent-confirmations, reopened-demotes, tamper-detected, risk-raised, unverified-not-resolved), is there a test that would FAIL if the invariant were removed — or is any test tautological / asserting the wrong thing? What important cases are NOT tested (e.g. HttpCorpus attestation, the gate's LabelVerdictMismatch path, a tampered tail, provenance bound into attestation end-to-end, mixed legacy+chained file)? List the highest-value missing tests. Also flag any #[allow(dead_code)] or TODO that hides a real gap.` },
]

phase('Review')

const reviewed = await pipeline(
  DIMENSIONS,
  (d) => agent(`${CTX}\n\nYOUR DIMENSION: ${d.prompt}\n\nReturn ONLY real, concrete findings (empty array if none). Cite file:line and the exact failure path.`,
    { label: `review:${d.key}`, phase: 'Review', schema: FINDINGS_SCHEMA }),
  // For each dimension's findings, independently verify each one in parallel.
  (review) => parallel((review.findings || []).map((f) => () =>
    agent(`${CTX}\n\nAnother reviewer reported this finding — VERIFY it adversarially against the actual current code. Try to REFUTE it; only confirm if you can trace the concrete failure path in the real source. Set verdict=refuted if the code actually handles it, verdict=uncertain if you cannot tell. Give the exact_location and a minimal fix if confirmed.\n\nFINDING (dimension ${review.dimension}):\ntitle: ${f.title}\nseverity: ${f.severity}\nlocation: ${f.location}\ndescription: ${f.description}\nfailure_path: ${f.failure_path || '(none given)'}\nsuggested_fix: ${f.suggested_fix}`,
      { label: `verify:${(f.title || 'finding').slice(0, 40)}`, phase: 'Verify', schema: VERDICT_SCHEMA })
      .then((v) => ({ ...v, dimension: review.dimension, original_severity: f.severity, suggested_fix: f.suggested_fix }))
  )),
)

const verified = reviewed.flat().filter(Boolean)
const confirmed = verified.filter((v) => v.verdict === 'confirmed')
const uncertain = verified.filter((v) => v.verdict === 'uncertain')
log(`Audit: ${verified.length} findings verified — ${confirmed.length} confirmed, ${uncertain.length} uncertain, ${verified.filter(v => v.verdict === 'refuted').length} refuted`)

return { confirmed, uncertain, refuted_count: verified.filter((v) => v.verdict === 'refuted').length, all: verified }
