# FOLLOWUPS

Standing, **append-only** backlog of everything **deferred to the future / not being implemented now** —
any non-blocking item the agent chose not to do this turn but that must be revisited. Maintained by the
agent per the SessionStart followups policy. Times are **UTC**.

**Append-only with tombstones — items are NEVER deleted.** When a follow-up is done, promoted, or dropped,
it is tombstoned in place so the deferral history stays fully auditable.

Format:
- `- [ ] [added YYYY-MM-DD HH:MM UTC] <item> — <why deferred / context / where to resume>` — open
- `- [x] [added YYYY-MM-DD HH:MM UTC · closed YYYY-MM-DD HH:MM UTC → <where it went>] <item>` — done/promoted
  (tombstone points where it went: a `PR #N`, a `TODOS.md` line, another doc, or "dropped: <reason>")

Conventions:
- NON-BLOCKING items only. Blocking work is finished in-turn, not parked here.
- Every entry carries the exact date **and time** it was added.
- Never delete a line. Flip `- [ ]` to `- [x]` and append the `· closed …` tombstone instead.
- Distinct from `TODOS.md` (the live checklist of work being done now) and `HANDOFFS.md` (resume state).

## Open

### Evidence-integrity engine work (from `docs/evidence-integrity-and-research-checklist.md` §8)

These are the GAP items the checklist identifies — real engine changes that are out of scope for this
documentation/governance pass. Build **MH-1 (EI-08 attestation) first**: the others degrade to forgeable
annotations without it.

- [x] [added 2026-06-14 20:09 UTC · closed 2026-06-14 21:05 UTC → DONE (library) in `feat/agent-ops-evidence-integrity` Increment 2; operator/CLI UX re-filed below] **[EI-08 / MH-1 — keystone]** Implement owner-key attestation over the contribution tuple. — IMPLEMENTED as ed25519 (owner chose asymmetric): `provenance::SignOffAuthority` signs, the engine holds only `SignOffPublicKey`, the gate (`ensure_attested`) refuses a confirmed row without a valid attestation when a store has `.with_authority(...)`. A constructed `Contribution{ sign_off: HumanConfirmed }` is now refused (test `authority_store_rejects_an_unattested_confirmed_row`).
- [x] [added 2026-06-14 20:09 UTC · closed 2026-06-14 21:05 UTC → partially addressed (mechanism exists); operator UX/rotation/audit re-filed below] **[Custody]** Decide the non-ephemeral judge key custody / rotation / audit-log retention path. — The attestation now uses a persistent ed25519 authority key (seed via `SignOffAuthority::from_seed_hex`), distinct from the ephemeral per-run plan-signing HMAC key. Custody UX (where the seed lives, rotation, audit log) remains, re-filed below.
- [ ] [added 2026-06-14 21:05 UTC] **[MH-1 — operator/CLI wiring]** Wire the sign-off authority into the `support-agent` CLI / embedders: a way to (a) generate + persist an authority key pair (seed on the Windows/durable side, public key embedded), (b) configure the corpus store with `.with_authority(pubkey)` from config/env (e.g. `CEC_SIGNOFF_PUBKEY`), and (c) actually PRODUCE an attestation at sign-off time via a real human/verifier action that holds the seed — NOT both keys in the engine process (that would defeat the asymmetric guarantee). Until wired, the mechanism is library-only (used by embedders/tests). Where to resume: `crates/support-agent/src/main.rs` (store construction + record_outcome) and `Contribution::attested_by`.
- [ ] [added 2026-06-14 21:05 UTC] **[MH-1 — key rotation + audit log]** Decide authority key rotation (support multiple trusted public keys / a key id → key registry so a rotated key still verifies historical rows) and an audit log of attestations (which authority signed which row, when). Today a store trusts exactly one authority public key. Where to resume: `crates/provenance/src/lib.rs` (SignOffPublicKey set) + `crates/corpus-client/src/gate.rs` (ensure_attested).
- [ ] [added 2026-06-14 21:05 UTC] **[MH-1 — verifier vs human authorities]** Currently one authority key attests any level (the tuple includes `sign_off`, so the level is bound, but a single key signs both VerifierConfirmed and HumanConfirmed). Consider distinct verifier-authority vs human-authority keys so the two trust tiers are cryptographically separable. Where to resume: `crates/corpus-client/src/gate.rs` ensure_attested.
- [ ] [added 2026-06-14 20:09 UTC] **[Canonicalization]** Replace serde field-order canonicalization with a sorted/canonical-JSON encoder before signatures are cross-version/cross-language verified — fragile integrity assumption today; where to resume: `crates/provenance/src/lib.rs:88-91`
- [ ] [added 2026-06-14 20:09 UTC] **[MH-3 / NR-1]** Wire a real post-fix re-collection to replace the bootstrap echo `signature_of(&collect_diagnostics(&args.describe))` so the verdict reflects a genuine post-state diff and `ResolvedConfirmed` cannot be trivially self-minted — where to resume: `crates/support-agent/src/main.rs:558-559`
- [x] [added 2026-06-14 20:09 UTC · closed 2026-06-14 20:41 UTC → partially done in `feat/agent-ops-evidence-integrity` (Increment 1); remainder re-filed below] **[MH-2 / EI-01]** Add a provenance/lane pin and bind the `verify.rs` Verdict + `VerificationClass` + recurring-symptom diff into `Contribution`/`Outcome` so resolved rows are auditable against their own evidence — where to resume: `crates/corpus-client/src/schema.rs:76-101`
- [ ] [added 2026-06-14 20:41 UTC] **[MH-2 / EI-01 — remainder]** The `verify.rs` Verdict + recurring-symptom diff are now bound into the row (`Outcome.verification: Option<common::Verification>`), and the gate rejects a resolved label without a matching passing verdict. STILL TODO: (a) carry the `VerificationClass` (deterministic/intermittent/hardware) onto the row too; (b) add the provenance/lane pin (`retrieval_first`, `primed_from_plan_ids`, `run_id`) derived from observable facts so a confirmation's origin is auditable. — where to resume: `crates/common/src/verification.rs` + `crates/corpus-client/src/schema.rs`
- [ ] [added 2026-06-14 20:09 UTC] **[EI-03 / A5]** Add a run-independence guard to confirmation aggregation keyed on `run_id`/lane, with a test that a duplicate row does not inflate the count (today `confirmations_aggregate_per_plan` submits the identical row twice and asserts `confirmations==2`) — where to resume: `crates/corpus-client/src/store.rs:39-50,411-423`
- [ ] [added 2026-06-14 20:09 UTC] **[MH-4 / MH-8 / EI-06]** Add per-row tamper-evidence (signature or hash chain) + an owner-only revocation/retraction list to `FileCorpus`; re-verify on `FileCorpus::open`; have `fix_mappings` honor revocation and let `OutcomeLabel::Reopened` demote a prior resolved mapping (the T-104 "retracted claim must not become truth" case) — where to resume: `crates/corpus-client/src/store.rs:26-53,136-157,181-197`
- [ ] [added 2026-06-14 20:09 UTC] **[MH-6 / A7]** Derive `config_class` from real CIM hardware/driver inventory (or BOM revision) instead of OS+ARCH, attested to the producing machine — retrieval scoping is weaker than documented until then; where to resume: `crates/support-agent/src/main.rs:742-747`
- [ ] [added 2026-06-14 20:09 UTC] **[MH-5]** Validate model-generated steps (claimed-risk-vs-actual-action reconciliation) and de-identify at generation; add inference-channel provenance (no cert pinning / endpoint / model attestation today) so a swapped endpoint is visible on the row — where to resume: `crates/support-agent/src/main.rs:878-886`
- [ ] [added 2026-06-14 20:09 UTC] **[Sandbox evidence]** Provide a production `SandboxValidator` impl (the `swarm` trait has none; the CLI hardcodes `sandbox_validated=false`, `main.rs:376`) and decide whether sandbox evidence is bound into the row — so "unvalidated equals escalate" is backed by positive validation evidence
- [ ] [added 2026-06-14 20:09 UTC] **[Research tree — fill]** Fill `docs/research/{claims,prereg-control-lane,instrumentation-inventory}.md` (scaffolded this session) following the commit-ordering discipline: `negative-results.md` must be committed before `claims.md`, and `prereg-control-lane.md` before any corpus row carries a `lane` field (else VOID) — where to resume: `docs/research/`
- [ ] [added 2026-06-14 20:09 UTC] **[Custody activation]** Decide whether to run `git config core.hooksPath scripts/githooks` to activate the corpus/weights pre-commit exfil guard (dormant in fresh clones — `core.hooksPath` is unset), and extend `SECURITY.md`'s invariant list to name each new evidence-integrity gate so a bypass is a reportable security issue — why deferred: changes git behavior for every future commit; owner's call (safe to enable — fmt-check only touches Rust, gitleaks only staged)

### Engine — residuals surfaced by the adversarial audit (`wf_5c1c16b9-613`, 14 confirmed)

The audit's confirmed findings were fixed this session (commit on `feat/agent-ops-evidence-integrity`;
see `.claude/audit/confirmed-findings.txt` and HANDOFFS). These are the deeper residuals the fixes leave open.

- [ ] [added 2026-06-14 23:05 UTC] **[Chain integrity — key or anchor the head]** The `FileCorpus`
  tamper-evidence chain is KEYLESS (`chain_hash` = sha256 over public inputs), so it proves internal
  consistency but is fully recomputable by anyone with file-write access. The open-time attestation
  re-check added this session (`FileCorpus::with_authority` re-admits every at-rest row) closes the
  forged-row bypass ONLY when an authority is configured; a cold-start corpus (no authority) still relies
  on the keyless chain alone. Defense-in-depth: HMAC the chain with a store-held secret, or anchor the
  chain head with the sign-off authority's signature, so the chain is itself an integrity boundary. — where
  to resume: `crates/corpus-client/src/schema.rs` (chain_hash) + `crates/corpus-client/src/store.rs`
  (verify_chain / with_authority). Subsumes the **tail-truncation** residual (a hash chain cannot detect
  removal of trailing rows without an external/length anchor) noted in `RowIntegrity`'s doc.
- [ ] [added 2026-06-14 23:05 UTC] **[chain_hash canonical encoding]** `chain_hash` now carries a version
  prefix (`cec-corpus-chain-v1`) but still hashes the `serde_json` image of the row — coupled to struct
  field order, fine for same-code recompute but not cross-language. If the chain ever needs external
  verification, switch it to the serde-independent canonical encoder used for the attestation/plan
  signatures. — where to resume: `crates/corpus-client/src/schema.rs` (chain_hash).
- [ ] [added 2026-06-14 23:05 UTC] **[Authority key rotation interacts with at-rest re-admission]** Now
  that `with_authority` re-verifies every at-rest row, rotating the sign-off authority key makes a corpus
  accreted under the OLD key un-openable under the NEW one. The single-key limitation is already filed
  (MH-1 key rotation, above); note here that the at-rest re-admission makes a key-id → key-registry (verify
  historical rows against the key that signed them) a prerequisite for rotation, not just a nicety. — where
  to resume: `crates/provenance/src/lib.rs` (SignOffPublicKey set) + `crates/corpus-client/src/store.rs`.

### Private corpus: structure + format DONE; ingest pipeline deferred (2026-06-14 22:25 UTC)

The off-tree private corpus repo (`/mnt/e/cec-corpus-private`), the YAML ground-truth fix-flow format,
templates, and the no-leak rails (here + there) are built and verified. The full ordered wiring plan with
acceptance checks is in **`/mnt/e/cec-corpus-private/WIRING.md` (W0–W9)** — these are the public-repo-visible
pointers to it. Two independent adversarial audits confirmed: no corpus data/keys in either repo's tree or
history, one-way coupling holds, and the format is complete/correct against the live gate.

- [~] [added 2026-06-14 23:35 UTC · recalibrated 2026-06-15 00:53 UTC → LOW, accepted by owner] **[Secrets
  perms on `/mnt/e/secrets`]** `/mnt/e/secrets` shows `0o777`; `chmod` is a no-op (it's a 9p mount with
  `uid=1000`, no `metadata` option). Originally flagged HIGH, but **recalibrated**: not in git, single-user
  trusted machine, and `chmod 600` wouldn't help anyway (same-user processes read it regardless; "world" in
  WSL is just the one uid). `cec-bot.env` is **NOT dead** — it's a deliberate **least-privilege bot PAT**
  (push-only, cannot merge: a separation-of-duties control mirroring the corpus sign-off gate; consumed by
  `session-end.sh` when `ops/secrets/load-secrets.sh` provides it). The corpus ed25519 seed is now
  **encrypted at rest** (age, `seed.rs`), so the volume perms don't expose it. Owner deems the residual
  acceptable. Re-open only if `E:` is ever backed up/synced off-box, becomes multi-user, or runs untrusted
  code — then move to encrypt-at-rest / Windows ACLs. The gh login token (broad, non-expiring, in ext4
  `~/.config/gh` 0600) is a separate, lower-priority hygiene item.
- [ ] [added 2026-06-14 23:35 UTC] **[Activate the no-leak guard]** Install `gitleaks` on PATH (a hard dep of
  both pre-commit hooks; make it a WSL-durability provisioning step), then `git config core.hooksPath` in BOTH
  repos (`scripts/githooks` here, `.githooks` in the private repo). Today the hooks are DORMANT — only
  `.gitignore` defends, and it cannot stop `git add -f`. — WIRING.md W1. (Supersedes the older `[Custody
  activation]` item above for the corpus-boundary half.)
- [ ] [added 2026-06-14 23:35 UTC] **[Private remote]** Create a PRIVATE GitHub repo for `cec-corpus-private`
  (or reuse `nathanfraske/cec-runs`), add it as `origin`, push. Never mirror to the public org. — WIRING.md W2.
- [x] [added 2026-06-14 23:35 UTC · closed 2026-06-15 00:50 UTC → BUILT (W4–W7) in the private repo, commits `b34b916`+`400351d`; W8/W9 remain below] **[corpus-ingest pipeline]** Build the deferred Rust compiler/tooling in
  the PRIVATE repo (git-deps the public engine at `schema/PIN`; the public workspace gains nothing). DONE:
  keygen (age encryption-at-rest), compile (YAML→de-id→attest→gate→hash-chained JSONL), verify (chain +
  re-admission + tail anchor); verified end-to-end incl. the engine retrieving a compiled row retrieval-first;
  an adversarial review found+fixed a CRITICAL symptom-leak. STILL DEFERRED: W8 the HTTP corpus service,
  W9 key rotation (WIRING.md). The seed-custody decision (W0) = **age passphrase encryption-at-rest**, now
  implemented in `seed.rs`; the operator runs `make keygen` with their real `CEC_SEED_PASSPHRASE`.
- [ ] [added 2026-06-14 23:35 UTC] **[Ignore residual — low]** Path-only ignores can't catch a corpus dump
  renamed to an arbitrary extension (`.txt`/`.csv`) or UPPERCASE `.FLOW.YAML`. The real defense is the hook
  grepping staged *content* for a JSONL-row / `attestation`/`fingerprint` shape — add when the hook is
  activated (W1). Caught today: `*.flow.y{a,}ml`, `*.jsonl`, `*.ndjson`, `*.seed`, `*.env`, `cec-corpus*`.

### WSL-ephemeral / agent-ops optional hardening (from `docs/wsl-ephemeral-state-policy.md`)

The durability contract is complete as-is; these are optional tightenings.

- [ ] [added 2026-06-14 20:09 UTC] Provision a bot PAT scoped to `cec-support-agent` `contents:write`, placed in `/mnt/e/secrets/cec-bot.env` (survives-WSL) and consumed via `ops/secrets/load-secrets.sh` — so the `ops/agent-handoff` push authenticates as the bot instead of the owner `gh` login — why deferred: gh credential fallback works today; PAT is a least-privilege/auditability upgrade, not a blocker
- [ ] [added 2026-06-14 20:09 UTC] Add a cargo-shaped `ops/provision.sh` (cargo build/test/clippy + githook install) so disaster recovery is one idempotent script — why deferred: do NOT copy CEC-Platform's KiCad/CUDA/broker provisioner; this repo is a Rust workspace and needs its own
- [ ] [added 2026-06-14 20:09 UTC] Add claude-rc survivability units (tmux + `systemd --user claude-rc@.service` with `Restart=always`, `rc-recover.sh`) repointed to the AutoDiagnoser ops path, so a dropped WSL console never orphans the agent — why deferred: nice-to-have resilience layer, independent of the durability contract
