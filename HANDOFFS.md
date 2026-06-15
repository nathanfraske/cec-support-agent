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

**As of 2026-06-15 ~03:55 UTC.** Two workstreams in flight; main working dir is on `feat/myown-integration-p0`.

**(A) Corpus leak-prevention methodology — ACTIVE (owner: implement Phases 0–2).** On branch
**`feat/corpus-leak-prevention`** (rebased onto the P0 tip `673a381` so it has the envelope + all de-id code;
force-pushed; worked in a worktree at `/tmp/cec-leak`). `docs/corpus-leak-prevention.md` = the methodology
(57 vectors, 4 layers, red-teamed, honest §6). **Phase 0 DONE + verified** (`cf95d1c`): `crates/deid` validating
mints (`action`=frozen-vocabulary membership = the keystone C1 fix; `plan_id`=slug charset; `symptom`=extractor
round-trip; each `Result`); `de_identify_plan`+`Contribution::new`→`Result` (out-of-vocab action/id REFUSES the
row); `crates/leakguard` canonical POISON; the leakage suite now BITES (seeds action/id, asserts refusal — proven
red-on-revert); drift guard. 180 tests, clippy+fmt clean. **Pick up here: Phase 1** (type split + `Prose` leaf
typing + sealed `Debug` + private `Contribution` fields + `trybuild` + write-gate idempotence) then **Phase 2**
(read-side `from_served` + frozen dictionaries + ban `serde_json::Value`) — see `docs/corpus-leak-prevention.md`
§4 and FOLLOWUPS. NOTE: the private `corpus-ingest` will need to adapt to `Contribution::new -> Result` on its
next engine-pin bump (FOLLOWUPS).

**(B) MyOwn-family integration P0 — DONE, both PRs green, awaiting merge.** **PR #2** (`b7ad864`) and stacked
**PR #3** (`673a381`) are FULLY GREEN on CI (check ×3 OSes, audit, secrets — zero failures). 170 tests. Merge
order: **PR #2 first**, then PR #3 (auto-retargets to `main`). RFC Q1–Q5 await Chris.

This session's later arc (all owner-approved): fixed PR #2's red CI, triaged the secrets job, then ran the
owner's 3 cleanup tracks:
- **fmt regression** (PR #2 `11f0609`) → fixed `538cd43`.
- **`secrets`/gitleaks job** triaged (`wf_60234519-881`): root cause = missing `GITHUB_TOKEN` env (a
  gitleaks-action breaking change), NOT a leak — gitleaks full-history + an independent 10-method cross-check
  both `all_clear`. Fixed (token env + `permissions` + `checkout@v5`/`gitleaks-action@v3`).
- **Track 1 — P0 adversarial review (`wf_923ec5a0-84d`, 13 confirmed):** found + fixed **2 CRITICAL** P0 bugs
  (`ddd1145`): **D1** the `cec-diagnose/v1` envelope leaked raw `--describe` via `candidates[].rationale`/`title`
  (hostname/user/IP/serial in cleartext) → now ships only `{plan_id, source, max_risk, actions[]}`; **D2** the
  stdout-purity hole — `record_outcome`/`sandbox_validated_for` (free fns) used bare `println!` so `--json
  --sign-off` emitted 2 lines → fixed via a module-scoped `tprintln!`. **D4** the de-id test was vacuous →
  rewritten to bite. +5 tests incl. `tests/cli_contract.rs`.
- **Track 2 — FOLLOWUPS reconciliation:** tombstoned 8 engine-gap items implemented by PR #2's increments
  (verified against the live code), re-filed 4 residuals; the section went ~12 open → 6 open / 11 closed.
- **Track 3 — CI hardening (`673a381`/`b7ad864`):** `concurrency` block (already cutting duplicate runs),
  `cargo-deny-action` (prebuilt, honors `deny.toml`), SHA-pinned all third-party actions + `.github/dependabot.yml`.

- **P0 (DONE, this branch):** the engine's dependency-free machine-output + inventory seams.
  - `crates/common/src/inventory.rs` — `InventoryProvider` trait + `CoarseHostInventory` (today's
    os/arch/family default, **byte-identical cold start**) + `ExternalInventory` (caller-supplied, re-hashed,
    never stored). Exported from `common`.
  - CLI `--inventory-keys <file|->` (external identity-free config keys → honest `config_class`, closes the
    **A7/MH-6** gap) and `--json` (the **`cec-diagnose/v1`** envelope).
  - **Wire contract** (what AllMyStuff codes against): under `--json`, **stdout = exactly one JSON line**, the
    envelope; the human trace goes to **stderr** (robust under `--sign-off`, not "parse the last line"). Done
    with local `human!`/`hprint!` macros in `run()`. Envelope is de-identified by construction (vocab symptoms,
    hashed config class, action vocab).
  - **Versioning (owner left to agent):** `cec-diagnose/v1`, **additive-only within a major**; a breaking
    change bumps the major and the consumer errors on an unknown one.
  - De-id regression tests on the inventory path. **165 tests green, clippy clean, fmt CLEAN.** Smoke-verified
    end-to-end (stdout = 1 valid `cec-diagnose/v1` object; non-json mode unchanged).
- **RFC for Chris:** `docs/integration-rfc-for-chris.md` — the frame, D1 (single-shot) + D2 (versioning)
  decided, **Q1–Q5 open** for Chris, the wire contract, P0 = built. `docs/integration-myown-family.md` P0
  section updated to DONE with verified accept-criteria.
- **PR #2 red CI — FIXED + pushed.** `11f0609` had shipped **4 rustfmt-1.9 wrapping violations** in
  `corpus-client/{schema.rs,store.rs}` (CI runs `cargo fmt --all -- --check`; the prior "fmt clean" predated
  rustfmt 1.9.0). Fixed as the portable commit `538cd43`, fast-forwarded onto `feat/agent-ops-evidence-integrity`
  (`920a..538cd43`, also bringing the 2 pending doc commits). **PR #2 `check` is now green on all 3 platforms.**
- **Pushed:** `feat/agent-ops-evidence-integrity` → `538cd43` (PR #2 green); `feat/myown-integration-p0` →
  `d61b962` as **PR #3** (stacked on PR #2's branch, so it shows only the P0 delta; auto-retargets to `main`
  when PR #2 merges).

--- previous (superseded) ---

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

### PRIMARY — Corpus leak-prevention Phases 1–2 (owner chose Phases 0–2; Phase 0 DONE)

**Branch:** `feat/corpus-leak-prevention` on origin at `cf95d1c` (= the P0 tip `673a381` + the methodology
doc + Phase 0). The `/tmp/cec-leak` worktree was removed (ephemeral); **resume by re-adding it:**
`git worktree add /tmp/cec-leak feat/corpus-leak-prevention` then build there. The design + the precise step
list is `docs/corpus-leak-prevention.md` §4 (Phases 1–2) and §2 Layers 1–2 (the *how*). **Read it first.**

**Phase 1 — type split + leaf `Prose` typing + sealed `Debug` (the C1/C3 compile-error hard stops). LARGE,
high-ripple, workspace-wide serde refactor — do it in green sub-steps:**
1. Introduce `StoredPlan` / `StoredSymptom` (in `crates/deid`, or a `de-id::stored` module) as the ONLY
   `Serialize + Deserialize` corpus-bound payload. `FileCorpus::open`/`HttpCorpus::query` (`store.rs`) and
   `chain_hash` route through a `StoredContribution`. **GOTCHA (red-team-confirmed): you CANNOT just strip
   `Deserialize` from `Plan` — `FileCorpus::open` (`store.rs:~286`) and `HttpCorpus::query` (`store.rs:~450`)
   deserialize the in-flight types today.** That is exactly why the split is needed: they deserialize
   `StoredPlan`, and raw `Plan` then genuinely has no serde path.
2. Remove `Serialize` from raw `Plan`/`Candidate`/`Outcome`/`DiagnosticEvent`/`ToolOutcome`/`AgentRun`. This is
   what makes `serde_json::to_string(&candidate)` a hard `E0277`. Ripples to: the `--json` envelope
   (`diagnose_envelope` already only emits primitives — good), panel, every domain-type serde consumer.
3. `Prose(String)` — private field, **no `Serialize`, no `Display`** — for `Plan.title`/`PlanStep.description`/
   `Candidate.rationale`/`DiagnosticEvent.message`/`StepResult.summary`, exposed only via an `into_inner()` the
   egress lint denylists. (Closes the red-team "String-laundering" bypass: removing `Serialize` from the struct
   does nothing for `json!({"x": c.rationale})` while `rationale` is a plain `String`.) Fix `render_consent`
   (`main.rs:~794`) which copies `plan.title` out into a printable `String`.
4. Seal/redact `Debug` on the raw types (don't punt to a lint — `format!("{outcome:?}")` leaks).
5. Make `Contribution` fields **private**, `Contribution::new` the only constructor; add `trybuild` compile-fail
   tests pinning (a) `to_string(&candidate)`, (b) struct-literal `Contribution{..}`, (c) `format!("{:?}", outcome)`.
6. **Write gate (closes the runtime `/mnt/e` path no git/CI sees):** `gate::ensure_evidence_integrity` re-runs
   `de_identify_plan` and asserts idempotence, and re-runs a symptom check. **GOTCHA:** the strict symptom
   round-trip mint rejects a legitimately-extracted `<id-prefix>_<digits>` symptom (produced from two input
   tokens, doesn't round-trip as one) — handle the prefixed-id grammar here (that's why Phase 0 did NOT yet wire
   `deid::symptom` into the write path; only action/id are validated so far).

**Phase 2 — read-side re-de-id + closed dictionaries:** `from_served` re-validates every served `FixMapping`
(`#[serde(try_from)]` on `StoredSymptom`/`ActionToken` so a bad wire value fails to *deserialize*); replace the
`is_stop_code_name`/`module_name` SHAPE heuristics (`extract.rs`) with FROZEN dictionaries (they currently keep
any ALL-CAPS_UNDERSCORE token + any `*.exe/.dll/.sys` — so asset tags / in-house binaries pass as "symptoms");
ban `serde_json::Value` on boundary types (`ToolOutcome.data`, `AgentStep.args` → typed summaries).

**Verification discipline (non-negotiable — the old suite was vacuous):** for every guard you add, PROVE it
fails on a reverted fix (revert→red→`git checkout`→green), as done for Phase 0's C1 guards.

**Build loop:** `. "$HOME/.cargo/env"` then `cargo build/test/clippy/fmt --workspace` (cargo with
`dangerouslyDisableSandbox`). **DOWNSTREAM:** the private `corpus-ingest` (`/mnt/e/cec-corpus-private`) calls
`Contribution::new` (now `Result`) — it breaks on its next engine-pin bump (FOLLOWUPS).

### SECONDARY — MyOwn integration: merge the green PRs

**PR #2** (`b7ad864`) + stacked **PR #3** (`673a381`) are FULLY green on CI. Merge **PR #2 first**, then PR #3
(auto-retargets to `main`). The leak-prevention branch is based on PR #3's tip, so it should rebase onto `main`
after both merge (or merge PR #3 first into it). RFC Q1–Q5 await Chris. P1/P2 of the INTEGRATION (AllMyStuff-side
`inventory_to_config_keys()` + serde-only `diagnose` contract, the MIT half in the AllMyStuff repo) is the next
integration build step and does not need Q1–Q5.

Per-item status in `TODOS.md`; deferred backlog in `FOLLOWUPS.md`.

--- previous (superseded) ---

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
provide **adapters** — deps point app→engine→mesh, never a cycle. **The plan is written:
`docs/integration-myown-family.md`** (process boundary keeps AllMyStuff MIT; P0–P4 phased; 3 load-bearing
claims verified; surfaced a real engine finding — `HttpCorpus::query` is unverified, in FOLLOWUPS). NEXT:
owner decides the 7 open questions + whether to start P0 (the engine `--json`/`--inventory-keys` seams — the
dependency-free first step). **Reactive:** PR #2 review comments; the deferred
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

- [2026-06-15 02:00 UTC] **A `--json` machine contract must own stdout, not share it.** The first P0 cut
  printed the human trace AND the JSON envelope to stdout (envelope as "the last line"). That contract BREAKS
  under `--json --sign-off`, where execution output prints AFTER the envelope, and is brittle even without it.
  The fix is the Unix norm: under `--json`, route ALL human output to **stderr** so **stdout is pure machine
  output** (one envelope line). Implemented with `run()`-local `human!`/`hprint!` macros that switch on
  `args.json`; the envelope emitter (`emit_diagnose_envelope`, a separate fn) keeps real `println!`. Verify a
  machine contract by asserting `stdout | wc -l == 1` and that it `json.load`s — not by eyeballing the run.
- [2026-06-15 02:00 UTC] **rustfmt version skew silently red-lit CI.** PR #2's `11f0609` was committed with
  "fmt clean" under an older rustfmt; **rustfmt 1.9.0** (the WSL/CI toolchain) wraps several `writeln!`/`use`
  lines differently, so `cargo fmt --all -- --check` (which CI runs) failed on all platforms and PR #2 could
  not merge — a green local `cargo build/test` does NOT imply green CI. ALWAYS run `cargo fmt --all --check`
  (not just `cargo fmt` then trust it) and, when a PR is "presented," confirm with `gh pr checks <N>` rather
  than assuming. The fix is mechanical (`cargo fmt --all`); keep it as its own commit so it's cherry-pickable.

- [2026-06-15 03:10 UTC] **A de-id guarantee dies on the FIRST un-audited serialization path.** The corpus
  write path was carefully de-identified (`de_identify_plan` strips free text to the action vocabulary), but the
  P0 `--json` envelope was a SEPARATE serialization that emitted `candidate.rationale`/`plan.title` verbatim —
  and the heuristic rationale is `format!("...: {describe}")`, so the raw request text (hostname/user/IP/serial)
  shipped in cleartext. The hashed fields right next to it (config_class, fingerprint) made it look safe. Lesson:
  every NEW path that serializes domain objects to an external boundary must be independently de-id-audited;
  don't assume a guarantee enforced elsewhere covers it. Emit only allowlisted, de-identified fields (here: the
  tool-name `actions` vocabulary), never free text — and write a test that PLANTS identity and greps the output.
- [2026-06-15 03:10 UTC] **`run()`-local macros don't cover the functions `run()` calls.** The `--json` stdout-
  purity fix used a `macro_rules!` defined inside `run()`, which silently left `record_outcome`/`sandbox_validated_for`
  (free functions) writing to stdout via bare `println!` — so `--json --sign-off` broke the one-line contract. A
  contract that must hold across a call graph needs a MODULE-scoped router (`tprintln!(json, …)`), and a
  PROCESS-level test (`tests/cli_contract.rs`, `wc -l == 1`) — a unit test of the envelope function can't see it.

- [2026-06-15 03:55 UTC] **A de-id test that avoids the leaky field is worse than no test — it manufactures
  false confidence.** The "adversarial" leakage suite seeded identity into describe/title/description but used a
  clean `action:"driver_rollback"` / `id:"model-1"`, and even *asserted the action was preserved* — so it passed
  precisely because it never touched the two fields `de_identify_plan` copied verbatim. A "proof of no leak" must
  plant into EVERY field the sink keeps, and you must PROVE it fails on a reverted fix (do the revert→red→restore
  check) — otherwise you have a vault around a sieve. Fixed in Phase 0: the mints validate action/id (refuse,
  not copy), the suite seeds them, and `leakguard::POISON` is the single source so a future test can't re-narrow it.
- [2026-06-15 03:55 UTC] **Provenance ≠ content: a "came-from-the-de-id-function" wrapper certifies whatever the
  function copied through.** The red-team's keystone point. `de_identify_plan` was the trusted chokepoint, but it
  trusted `action`/`id`. The fix is a VALIDATING mint (a positive allowlist + a round-trip property), not just a
  newtype proving origin. When building the Phase-1 `DeIdentified<T>`, the security boundary is the mint
  PREDICATE, not the type tag.

## Handoff log (reverse-chronological)

- **2026-06-15 03:55 UTC** — **Corpus leak-prevention: methodology designed + Phase 0 implemented + verified.**
  Owner asked to codify prevention of all corpus leaks incl. agent-accidental ones. Ran a 15-agent workflow
  (`wf_148ceb35-f02`, 57 vectors, 11 critical): wrote `docs/corpus-leak-prevention.md` (4 layers, red-teamed,
  honest §6 on guarantee-vs-accepted-risk). Owner chose Phases 0–2. Implemented **Phase 0** on
  `feat/corpus-leak-prevention` (`cf95d1c`): `crates/deid` validating mints + `crates/leakguard` poison set;
  `de_identify_plan`/`Contribution::new`→`Result` (closes the CRITICAL C1 action/id pass-through); the leakage
  suite now BITES (proven red-on-revert). 180 tests, gates clean. Verified the discipline the old suite lacked.
  Phases 1–2 (type split + leaf `Prose` + read-side + dictionaries) remain — a large workspace-wide serde
  refactor (FOLLOWUPS). **Lessons:** a de-id test that avoids the leaky field manufactures false confidence;
  provenance ≠ content (validate the mint predicate, not just the type tag).
- **2026-06-15 03:10 UTC** — **Cleanup while Chris drafts: 3 owner-chosen tracks, both PRs still green.**
  **Track 1 (P0 adversarial review, `wf_923ec5a0-84d`, 18 agents):** 13 confirmed findings → fixed 2 CRITICAL
  (D1 envelope de-id leak via `candidates[].rationale`; D2 stdout-purity hole in free fns under `--json
  --sign-off`) + the vacuous de-id test (D4); refactored `emit_diagnose_envelope`→`diagnose_envelope()->Value`;
  +5 tests incl. process-level `tests/cli_contract.rs`. 170 tests green. Pushed `ddd1145` to PR #3 (P0-only code,
  nothing to port to PR #2). **Track 2 (FOLLOWUPS reconciliation):** verified each engine-gap item vs the live
  code; tombstoned 8 (PR #2 increments) + re-filed 4 residuals (~12 open → 6/11). **Track 3 (CI hardening,
  `673a381`/`b7ad864`):** concurrency block, `cargo-deny-action`, SHA-pinned actions + dependabot. CI re-verified
  fully green on both PRs (the concurrency block is already cancelling duplicate runs). **Lessons:** a de-id
  guarantee dies on the first un-audited serialization path; a `run()`-local macro doesn't cover called functions.
- **2026-06-15 02:30 UTC** — **Triaged + fixed the `secrets`/gitleaks CI job → both PRs fully green.** Scouted
  the failure inline (the PR-event run errored "GITHUB_TOKEN is now required to scan pull requests"; the
  push-event run passed clean), downloaded gitleaks 8.24.3 and scanned the FULL history (36 commits) + working
  tree → `no leaks found`. Ran workflow `wf_60234519-881` (4 agents) to adversarially verify the exact fix
  (permissions/fork-PR nuance), audit adjacent CI issues, and independently cross-check for real secrets
  (10 methods, `all_clear`). Applied to `.github/workflows/ci.yml`: `GITHUB_TOKEN` env + `permissions` block +
  `checkout@v4→v5` + `gitleaks-action@v2→v3` (the Node-20→24 cutover is 2026-06-16). Landed on both branches
  no-force (PR #3 `53dd992`; PR #2 cherry-pick `951ae82` via a throwaway worktree, leaving the dirty tracking
  files untouched). CI settled fully green on both PRs (check×3 + audit + secrets, 0 failures). Deferred CI
  hygiene → FOLLOWUPS. **Lesson:** gitleaks-action@v2+ needs `GITHUB_TOKEN` in `env` for `pull_request` events
  — a missing-token fail looks identical to a "secret found" red X; always read the job log before assuming a leak.
- **2026-06-15 02:00 UTC** — **MyOwn integration P0 BUILT + fixed PR #2's red CI.** Owner greenlit (single-shot
  CLI, versioning = agent's call, the rest → an RFC for Chris). Implemented P0 on `feat/myown-integration-p0`:
  `common::InventoryProvider`/`CoarseHostInventory`/`ExternalInventory` (`inventory.rs`), CLI `--inventory-keys`
  + `--json` (`cec-diagnose/v1`), and — for a robust machine contract — routed the human trace to **stderr** so
  `--json` stdout is **one pure JSON line** (`human!`/`hprint!` macros). Spec'd the versioning policy
  (additive-within-major). Wrote `docs/integration-rfc-for-chris.md` (D1/D2 decided, Q1–Q5 for Chris) and
  updated the integration doc's P0 → DONE. **165 tests green, clippy + fmt CLEAN**, smoke-verified. Discovered
  via `gh pr checks 2` that **PR #2 is RED on CI** — a rustfmt-1.9 regression from `11f0609`; fixed in-tree
  (portable fmt-only commit `538cd43`). Owner approved **"push both"**: fast-forwarded the fmt fix + 2 doc
  commits onto `feat/agent-ops-evidence-integrity` (PR #2 `check` now green on all 3 platforms), and pushed P0
  as **stacked PR #3** (`d61b962`, base = PR #2's branch). Only red left is the pre-existing `secrets`/gitleaks
  job. TODOS/FOLLOWUPS updated. **Lessons:** a `--json` contract must own stdout; rustfmt version skew can
  silently red-light CI — always `gh pr checks` a "presented" PR.
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
