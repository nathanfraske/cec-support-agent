# HANDOFFS

The cross-agent baton. An agent picking up this repo should be able to read **only this file** and know
exactly where things stand, what to do next, and what was learned — without hunting. Times are **UTC**.

Keep three things current, in the same turn as the work:
1. **Current state** — where things stand right now (branch, what's done, what's in flight).
2. **Pick up here** — the exact next step(s), concrete enough to start immediately (file, command, decision).
3. **Lessons learned** — durable, append-only. Anything you discovered the hard way (a gotcha, a non-obvious
   constraint, a tool quirk, a dead end and why) so the next agent does not relearn it. Never delete a lesson.

Below "Pick up here", keep a reverse-chronological **handoff log** of dated entries so the trail is auditable.

---

## Current state

**As of 2026-06-15 ~01:15 UTC.** Done this session: (1) the engine audit-fix (PR #2), (2) the **private
corpus** structure + format + the `corpus-ingest` compiler + the seedless validation gate. Active next:
(3) a clean **MyOwn-family integration** plan (AllMyStuff / MyOwnMesh) — see "Pick up here".

- **(1) Engine audit-fix:** 14 findings FIXED + verified; **PR #2**
  (https://github.com/nathanfraske/cec-support-agent/pull/2), branch `feat/agent-ops-evidence-integrity` at
  `11f0609`; 159 tests green.
- **(2) Private corpus:** a SEPARATE off-tree private git repo at **`/mnt/e/cec-corpus-private`** (HEAD
  `5c5d15c`) holds the YAML ground-truth fix-flow format (`cec-fix-flow/v1`), templates, lint, vocabulary,
  no-leak rails, AND the **`corpus-ingest` compiler (W4–W7, BUILT + verified end-to-end)**: `keygen` (seed
  age-encrypted at rest), `compile` (de-id → attest → gate → hash-chained JSONL), `verify`. Proven loop:
  author YAML → compile → the engine retrieves it retrieval-first (`CorpusPrimed`). An adversarial code review
  found + fixed one CRITICAL symptom-leak. **The seedless `corpus-ingest check` validation gate** (private
  `5c5d15c`) + a CI merge-gate (`.github/workflows/validate.yml`) now mechanize **propose-then-authorize**: a
  bot may push but cannot merge an inadmissible/leaky entry; paper-ready checklist item **A10** (public
  `271db03`, local) records it. The PUBLIC repo's matching rails (`BOUNDARY.md`, hardened `.gitignore`/
  pre-commit) are on **PR #2** (`920e22a`). Still deferred: W1 (gitleaks+activate hooks + branch protection),
  W2 (private remote), W8 (HTTP/mesh service), W9 (rotation). **Operator's first step:** `make keygen` with a
  real `CEC_SEED_PASSPHRASE`. **Secrets note (LOW, accepted):** `/mnt/e/secrets` perms recalibrated — the bot
  PAT is a deliberate push-only control, the seed is encrypted at rest (FOLLOWUPS).

- **Audit:** re-ran the `autodiagnoser-engine-audit` workflow (`wf_5c1c16b9-613`) — the previous agent's run
  had not persisted results and no live task survived. 23 agents, ~1M tokens: 18 findings verified →
  **14 confirmed, 0 uncertain, 4 refuted**. Full detail in `.claude/audit/confirmed-findings.txt`; the fix
  diff in `.claude/audit/fix.diff`.
- **Fixed (14 confirmed → 7 distinct defects), each independently re-verified CLOSED:** A (CRITICAL) at-rest
  rows re-admitted on the keyless chain alone → `FileCorpus::with_authority` re-admits every loaded row,
  fails closed; B (HIGH) attestation_message field-injection → length-prefix + count-frame, v2→v3; C (HIGH)
  reopen demotion run-dedup; D (MED) bind config_class variant; E (MED) bind outcome.verification; F (LOW)
  seed-without-pubkey derives the enforcing key; G (LOW) versioned chain_hash. +11 tests.

--- previous (superseded) ---

**As of 2026-06-14 ~20:12 UTC.** The agent-ops + evidence-integrity work is COMPLETE and verified; ready to
commit on branch `feat/agent-ops-evidence-integrity`.

- **What this repo is.** `cec-support-agent` — the open Rust engine (Cargo workspace, 10 crates + the
  `support-agent` CLI). Pipeline: intake interview → collect diagnostics → candidate plans (swarm
  hypothesis fan-out) → judge panel (route/score/escalate) → provenance-signed plan → consent-gated
  execution → verification (diff re-collected signature) → sign-off-gated, de-identified corpus write-back.
  The **corpus is private and lives elsewhere**; only the corpus *client* + schema are here. Its truth is
  the **inverted corpus**: signed-off `(FaultSignature, Plan, OutcomeLabel)` triples earned at the gate.
- **Important.** The GitHub repo literally named `CEC_AutoDiagnoser` is EMPTY. The real work is the
  `cec-support-agent` repo, cloned into the local `/home/nathan/CEC_AutoDiagnoser` working dir. Remote:
  `https://github.com/nathanfraske/cec-support-agent.git`, default branch `main`.
- **Delivered this session (all verified):**
  - Tracking layer: `.claude/hooks/{followups,todos,handoffs}-context.sh` + `FOLLOWUPS.md` / `TODOS.md` /
    `HANDOFFS.md` (append-only with tombstones, UTC date+time), wired in `.claude/settings.json`.
  - WSL-ephemeral durability: `.claude/hooks/session-start.sh` + `session-end.sh` + `.claude/memory/` mirror.
    **Verified live** — Stop hook pushed `ops/agent-handoff` to the remote, `main` untouched.
  - `docs/evidence-integrity-and-research-checklist.md` — EI-01..08 + research PP analogs adapted to the
    inverted corpus; the runnable checklist; the unified `ensure_evidence_integrity()` design; attack→defense.
  - `docs/local-agent-infrastructure.md` — current cec-llm-broker (:8080) hybrid stack.
  - `docs/wsl-ephemeral-state-policy.md` — the durability contract as implemented.
  - `docs/research/` tree (README + negative-results [populated] + claims/prereg [scaffolds] + instrumentation).
  - 14 deferred engine GAP items in `FOLLOWUPS.md`; agent-ops pointer in `AGENTS.md`.
  - Recon + design panel artifacts under `.claude/recon/*.json` and `.claude/wf-*.js`.

## Pick up here

Branch `feat/agent-ops-evidence-integrity` is **presented for review as PR #2**
(https://github.com/nathanfraske/cec-support-agent/pull/2).

**ACTIVE NEXT TASK (2026-06-15): design the clean MyOwn-family integration.** cec-support-agent (AGPL) is "the
engine behind the AllMyStuff brain." The family (org `mrjeeves`, dev `nathanfraske`): **AllMyStuff** (MIT,
Tauri+Svelte+Rust device-inventory + mesh-wiring "brain"; crates incl. `allmystuff-inventory`,
`allmystuff-bridge`, `allmystuff-graph`), **MyOwnMesh** (MIT, pure-Rust private mesh — `myownmesh-core` with
`identity.rs` / `protocol/rpc.rs` / `protocol/governance.rs`, + STUN/TURN + Nostr signaling), **MyOwnLLM**
(local inference). Integration seams identified: **(a)** AllMyStuff `allmystuff-inventory` → the engine's
`host_inventory()`/`config_class` (closes the A7/MH-6 honest-config-class gap); **(b)** AllMyStuff embeds the
engine as its diagnostic brain — but **AGPL→MIT forces a process/RPC boundary, NOT static linking** (license +
clean-arch); **(c)** the private corpus served over a **MyOwnMesh RPC service** (W8 realized privately — no
public endpoint) with mesh **device-identity → config_class/provenance attestation** and **owner-authorization
→ the sign-off gate** (HumanConfirmed; "authorization not authentication" maps cleanly); **(d)** inference →
MyOwnLLM. Clean-integration invariant: the engine stays standalone (cold-start, no mesh required) and exposes
**trait seams** (it already has `host_inventory()`, `CorpusStore`, `SandboxValidator`); the MyOwn crates
provide **adapters** — deps point app→engine→mesh, never a cycle. A design workflow is mapping the real APIs +
synthesizing a phased plan (will land in `docs/`). **Reactive:** PR #2 review comments; the deferred
`FOLLOWUPS.md` residuals (keyless-chain anchor, chain_hash canonical encoding, rotation registry, Windows-CIM,
real-VM-backend, research-tree). Build loop: `. "$HOME/.cargo/env"` then `cargo build/test/clippy/fmt --workspace` (run
cargo with `dangerouslyDisableSandbox`). Per-fix status is in `TODOS.md`; confirmed findings in
`.claude/audit/confirmed-findings.txt`; the reusable audit is `.claude/wf-audit.js`.

Increments delivered: (1) structured gate + bound verdict; (2) MH-1 ed25519 attestation; (3) MH-2 class +
run-provenance + EI-03 independent confirmations; (4) MH-4/8/EI-06 hash-chain tamper-evidence + revocation +
reopened-demotion; (5) deterministic plan canonicalization; (6) MH-5 risk reconciliation; (7) MH-3/NR-1
Unverified verdict; (8) MH-6 config_class host_inventory point; (9) MH-1 operator CLI wiring (keygen + env);
(10) sandbox-validation evidence wiring. Residuals (Windows CIM, real VM backend, key rotation, tail-anchor,
inference cert-pinning, research-tree fill) remain in `FOLLOWUPS.md`.

--- previous (superseded) ---

Branch `feat/agent-ops-evidence-integrity`, pushed to origin (durable). **Increments 1 & 2 of the engine work
are DONE** (committed + pushed): (1) structured `ensure_evidence_integrity` gate + the verification verdict
bound into the row + destructive-resolved-fix⇒human enforced in corpus-client; (2) **MH-1 keystone** — ed25519
sign-off attestation (`provenance::SignOffAuthority`/`SignOffPublicKey`; engine holds only the public key;
stores `.with_authority(pubkey)`; a self-asserted `HumanConfirmed` is refused). 136 tests green; fmt/clippy/
license-checks clean. Build loop: `. "$HOME/.cargo/env"` then `cargo build/test/clippy/fmt --workspace`
(run cargo with `dangerouslyDisableSandbox` — it needs the registry network).

**Next options (all in `FOLLOWUPS.md`, none blocked):**
- **MH-1 operator/CLI wiring** — generate+persist the authority keypair, configure the store from
  `CEC_SIGNOFF_PUBKEY`, and produce attestations at human sign-off time (NOT both keys in the engine process).
  Until then MH-1 is library-only (embedders/tests use it).
- **MH-2 remainder** — carry `VerificationClass` + a provenance/lane pin onto the row (dep-free; unblocks
  EI-03/A5 independent-confirmation guard).
- **MH-3 / NR-1** — real post-fix re-collection (replaces the bootstrap echo at `main.rs:558-559`).
- **MH-4/8/EI-06** — hash-chained tamper-evidence + owner-only revocation for `FileCorpus` (sha2 only).
- Canonical-JSON plan encoding; MH-6 honest config_class (Windows); fill `docs/research/` (ordering discipline).
See `docs/evidence-integrity-and-research-checklist.md` §9 for the implementation status.

## Lessons learned (append-only)

- [2026-06-14 19:46 UTC] The local `CEC_AutoDiagnoser` working dir was an empty, non-git folder; the GitHub
  repo of that exact name is also empty. The actual engine is the **`cec-support-agent`** repo. If a CEC
  working dir looks empty, the code is in a differently-named GitHub repo — check `gh repo list` before
  assuming greenfield.
- [2026-06-14 19:50 UTC] This repo's pre-commit guard + `.gitignore` only block corpus/weights *data*
  formats (`corpus/`, `weights/`, `*.gguf|safetensors|bin|sqlite|duckdb`). Markdown, shell, and JSON under
  `.claude/` are not blocked — but `core.hooksPath` is NOT set here yet, so the guard is dormant until
  `git config core.hooksPath scripts/githooks` is run.
- [2026-06-14 19:50 UTC] CEC-Platform's FOLLOWUPS.md uses date-only and *deletes* resolved items; the owner
  wants the STRICTER variant here — date+time and append-only tombstones. TODOS.md mirrors CEC-Platform's
  TODO.md (already tombstoned). Don't copy CEC-Platform's followups policy verbatim.
- [2026-06-14 19:56 UTC] **WSL gotcha (verified):** a pristine post-wipe `git clone` has NO git identity
  AND the memory-dir name sanitizes `_`→`-` (`tr '/._' '---'`, not CEC-Platform's `tr '/.' '--'`). Without an
  identity, `git commit-tree` fails "empty ident name" and the durable handoff push dies SILENTLY (the hook
  fail-softs). `session-end.sh` now exports a `GIT_*_NAME/EMAIL` fallback (`cec-agent-handoff[bot]`). The gh
  credential helper (`!/usr/bin/gh auth git-credential`) is already wired, so auth was never the problem.
  Verified: the Stop hook pushed branch `ops/agent-handoff` (commit by the bot) carrying the tracking files +
  memory mirror, with `main` untouched (no checkout / no HEAD move).

- [2026-06-14 20:12 UTC] **Load-bearing integrity insight (for the engine work):** in the inverted corpus,
  the sign-off gate `ensure_signed_off` (`crates/corpus-client/src/gate.rs:15`) is the single truth-admission
  boundary but it is HOLLOW — it checks only `sign_off.is_confirmed()` over a caller-set enum. A library
  embedder can submit `Contribution{ sign_off: HumanConfirmed }` with no human. So "zero unsigned rows" is a
  *discipline*, not a guarantee, until MH-1 (owner-key attestation over `(signature, plan, label, sign_off,
  config_class)`) lands. Build MH-1 before any other integrity gap — they all hang off it. Full design +
  the 11 gaps in `docs/evidence-integrity-and-research-checklist.md`.

- [2026-06-14 20:40 UTC] **No Rust toolchain in WSL** — the engine is normally built on Windows (cargo.exe is
  on the Windows PATH; CI builds in GH Actions). Installed rustup/stable 1.96 in WSL for a local loop:
  `. "$HOME/.cargo/env"` then `cargo build/test/clippy/fmt --workspace`. `/target` is gitignored. Use
  `dangerouslyDisableSandbox` for cargo (it needs the network for the registry on first build).
- [2026-06-14 20:40 UTC] **Gate semantics (Increment 1):** `ensure_evidence_integrity` admits hard negatives
  (non-resolved labels) freely (a failure is truth too) but a RESOLVED label needs a matching passing verdict
  AND, if the plan is destructive, human sign-off. The verdict is bound via `Outcome.verification:
  Option<common::Verification>` (None for never-executed outcomes). On non-Windows the bootstrap labels
  EscalatedHumanUnresolved (tools unsupported) so the resolved-accept path can only be exercised live on
  Windows — it's covered by unit tests in `crates/corpus-client/src/gate.rs`.

- [2026-06-14 21:05 UTC] **MH-1 design (ed25519, owner-chosen):** sign-off attestation is ASYMMETRIC, unlike
  plan-signing (which stays HMAC because judge+executor are one process). `provenance::SignOffAuthority` holds
  the private key; the engine embeds only `SignOffPublicKey` and verifies. corpus-client now depends on
  provenance (verify side) — no cycle (provenance only deps `common`). The attestation covers a canonical,
  serde-independent tuple string (`schema::attestation_message`), so it survives the known serde-field-order
  fragility. A store enforces attestation ONLY when `.with_authority()` is set (cold start has none → unchanged).

- [2026-06-14 23:15 UTC] **Workflow results are NOT auto-persisted across a session boundary.** The previous
  agent launched the `autodiagnoser-engine-audit` Workflow but its result never landed in a file and no live
  task survived into this session — so "an audit is running" in a handoff is not resumable state. If you
  launch a Workflow whose output the next agent needs, WRITE the returned `result` JSON to a file (e.g.
  `.claude/audit/<name>-result.json`) in the same turn. Re-running the audit was cheap here (read-only, ~1M
  tokens) but not free. The script (`.claude/wf-audit.js`) and the scoped diff (`.claude/audit/engine.diff`)
  DO survive on disk, so a re-run is one `Workflow({scriptPath})` call.
- [2026-06-14 23:15 UTC] **The tamper-evidence chain is KEYLESS — it is not an integrity boundary by itself.**
  `chain_hash` is sha256 over public inputs, so anyone with file-write access recomputes it; `verify_chain`
  proves internal consistency, NOT authenticity. The real at-rest boundary is the ed25519 attestation, and it
  was only ever checked on `submit`, never on rows loaded at `open` — so a file-rewrite of forged "confirmed"
  rows was served whole (the audit's CRITICAL C6). The fix: `FileCorpus::with_authority` re-admits every
  at-rest row. CONSEQUENCE for the next agent: a corpus accreted at cold start (no authority) CANNOT be opened
  under an authority later — every unattested row is refused. That is intended fail-closed, but it means
  turning on enforcement requires a corpus built under that authority, and key ROTATION now needs a key-id →
  key registry (filed in FOLLOWUPS) before a rotated key can open old rows.

- [2026-06-14 23:40 UTC] **`chmod` is a no-op on the `/mnt/e` DrvFs mount (verified).** `chmod 700/600`
  silently "succeeds" but perms stay `0o777` — so a secret on `/mnt/e` is world-readable and POSIX perms
  cannot fix it. `/mnt/e/secrets` already holds a real GitHub PAT + sudo password world-readable. For a secret
  that must be BOTH durable (survive a WSL wipe → off-tree on `/mnt/e`) AND protected, use encryption-at-rest
  (`age`/`gpg`) or Windows ACLs (`icacls`), not `chmod`. This is why the corpus ed25519 seed custody (WIRING
  W5) is encrypt-at-rest, not a `chmod 600`.
- [2026-06-14 23:40 UTC] **The private corpus is `/mnt/e/cec-corpus-private` — a SEPARATE git repo, never
  touched by public-repo git.** The boundary is mechanical: the public `.gitignore` + pre-commit refuse
  `*.flow.y{a,}ml`/`*.jsonl`/`*.ndjson`/`*.seed`/`*.env`/`cec-corpus*`; the dependency arrow is private→public
  only (the deferred `corpus-ingest` crate git-deps the engine at `schema/PIN`, the public workspace gains
  nothing). The YAML→row mapping is in `/mnt/e/cec-corpus-private/spec/fix-flow.schema.md`: every DERIVED field
  (fingerprint, plan.title/description, attestation, integrity) is compiler-only and FORBIDDEN in YAML, and the
  gate's coupling rules are encoded in the JSON Schema so an inadmissible flow fails the lint, not the gate.

- [2026-06-15 01:15 UTC] **The "MyOwn family" ecosystem (org `mrjeeves`, dev `nathanfraske`) — load-bearing
  context for integration.** cec-support-agent (this engine, **AGPL-3.0**) is the diagnostic brain behind
  **AllMyStuff** (`github.com/mrjeeves/AllMyStuff`, **MIT** — a Tauri+Svelte device-inventory + mesh-wiring app;
  `allmystuff-inventory` is cross-platform hardware/device inventory = the real source the engine's
  `host_inventory()` wants). Both run on **MyOwnMesh** (`github.com/mrjeeves/MyOwnMesh`, **MIT** — pure-Rust
  private mesh: `myownmesh-core` = identity + RPC + protocol/governance, plus STUN/TURN + Nostr signaling).
  **MyOwnLLM** is local inference. **License watch-out:** AGPL (engine) embedded into MIT apps makes the
  combined work AGPL — so the clean pattern is the app driving the engine over a **process/RPC boundary**, not
  static linking. The engine already exposes the right trait seams (`host_inventory`, `CorpusStore`,
  `SandboxValidator`) for adapter-based integration without a dep cycle.

## Handoff log (reverse-chronological)

- **2026-06-15 01:15 UTC** — **Seedless validation gate + MyOwn-family integration recon.** Added
  `corpus-ingest check` (full admissibility + de-id validation, no seed — split `flow::compile` into
  `validate`+`compile`), `make check`, a CI merge-gate (`.github/workflows/validate.yml`), and a local
  pre-commit best-effort gate — mechanizing propose-then-authorize (bot pushes, can't merge an inadmissible/
  leaky entry). Public checklist item **A10** records it as paper-ready (reproducible). Private `5c5d15c`,
  public `271db03` (local, belongs on PR #2 — push pending owner OK). Then reconned the MyOwn family
  (AllMyStuff/MyOwnMesh/MyOwnLLM) and identified the integration seams (see "Pick up here"); a design workflow
  is producing the integration plan. **Lesson:** `cargo test` does not rebuild the `bin` — `cargo build`
  before re-testing a CLI fix.
- **2026-06-15 00:50 UTC** — **Built the `corpus-ingest` compiler (private repo W4–W7).** A pinned-git-dep
  Rust crate that compiles authored YAML flows → de-identified, ed25519-attested, gate-validated, hash-chained
  corpus rows; seed custody is **age passphrase encryption-at-rest** (the owner's choice; `chmod` is dead on
  `/mnt/e`). Verified end-to-end: keygen→compile→verify on the worked example (zero identity strings); 4
  negative tests reject (tamper, destructive+verifier, non-vocab symptom, wrong passphrase); and the engine
  retrieves the compiled row **retrieval-first**. An adversarial review caught one **CRITICAL**: a spaced
  multi-token symptom (`"DESKTOP-NATHAN01 jsmith.exe"`) could masquerade as a module name and leak identity
  into the attested signature (the plan de-id is no backstop for the signature) — FIXED by enforcing the
  extractor's `[a-z0-9._]` single-token charset, + the crate's first 4 tests. Private HEAD `400351d`. Next
  (operator): `make keygen` with the real passphrase; then W1/W2/W8/W9. **Lesson:** `cargo test` does NOT
  rebuild the `bin` — re-run `cargo build` before re-testing a CLI fix or you test the stale binary.
- **2026-06-14 23:40 UTC** — **Private corpus structure + ground-truth format.** Built the off-tree private
  repo `/mnt/e/cec-corpus-private` (HEAD `c636168`): the `cec-fix-flow/v1` YAML format (`spec/`), 4 templates +
  a worked example, the JSON-Schema lint (validated to accept all templates and reject every inadmissible
  flow), the `vocabulary.yaml` snapshot (faithful to the real `extract.rs`), the no-leak rails on both repos,
  and the W0–W9 deferred-wiring plan. Ran a design panel (5 agents) then 2 adversarial auditors (no-leak +
  format↔gate); fixed all actionable findings. Public-side rails (`BOUNDARY.md`, `.gitignore`, pre-commit) are
  in the working tree, **push pending owner OK**. Next: present; decide whether the public rails ride PR #2 or
  a separate branch; then the deferred ingest pipeline (WIRING W4–W9). **HIGH owner item:** `/mnt/e/secrets`
  world-readable (FOLLOWUPS).
- **2026-06-14 23:15 UTC** — **Audit + fix pass.** Re-ran the adversarial audit workflow (the prior run's
  results were lost): 14 confirmed findings → 7 distinct fixes, all in `crates/corpus-client` + `support-agent`:
  **(A, CRITICAL)** open-time attestation re-admission (`FileCorpus::with_authority` → `Result`, re-runs the
  full gate over every at-rest row; main.rs wires `?`); **(B, HIGH)** `attestation_message` v3 — length-prefix
  every attacker-controlled value + count-frame every repeated section (kills the signed-byte collision);
  **(C, HIGH)** reopen demotion run-deduped via a `HashSet` keyed by `confirmation_key`; **(D, MED)** bind the
  `ConfigClass` variant; **(E, MED)** bind `outcome.verification`; **(F, LOW)** seed-without-pubkey derives the
  enforcing key; **(G, LOW)** versioned `chain_hash`. +11 tests (159 total), clippy/fmt clean, CLI smoke OK.
  Two independent adversarial reviewers re-verified all 7 CLOSED with no regression. FOLLOWUPS got the 3 deeper
  residuals. **Committed locally; push + PR pending owner OK.** Next: present for review / open the PR.
- **2026-06-14 21:05 UTC** — Implemented Increment 2 (MH-1 keystone): ed25519 sign-off attestation in
  provenance + corpus-client; stores `.with_authority`; +12 tests incl. the forgery test; ed25519-dalek
  license-clean. Updated the checklist doc (§9 changelog), research inventory, FOLLOWUPS. All gates green.
- **2026-06-14 20:41 UTC** — Implemented Increment 1 of the engine work (structured evidence-integrity gate +
  verdict binding + destructive-fix-needs-human in corpus-client; +6 tests; SECURITY.md updated). Installed
  the WSL Rust toolchain. All gates green. Next: MH-1 attestation (needs the owner's key-custody decision).
- **2026-06-14 20:12 UTC** — Ran the recon fan-out (5 agents) + a 4-lens design panel (7 agents) via the
  Workflow tool. Wrote the three docs, scaffolded `docs/research/`, populated `FOLLOWUPS.md` with 14 engine
  GAP items, added the `AGENTS.md` pointer. Verified all hooks/settings/files. Everything ready to commit on
  `feat/agent-ops-evidence-integrity`. Next: commit; then engine work (MH-1 first).
- **2026-06-14 19:51 UTC** — Cloned `cec-support-agent` into the working dir; authored the three tracking
  hooks + seed files; launched the recon fan-out. Next: WSL parity hooks, settings.json, evidence-integrity
  checklist (design panel), local-agent infra doc.
