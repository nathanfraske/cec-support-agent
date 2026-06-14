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

Branch `feat/agent-ops-evidence-integrity`, pushed to origin (durable). **Increment 1 of the engine work is
DONE** (committed): the evidence-integrity gate `ensure_evidence_integrity` is now structured + binds the
verification verdict into the row + enforces destructive-resolved-fix⇒human IN corpus-client. Build/test loop
works locally (the WSL Rust toolchain had to be installed — `. "$HOME/.cargo/env"` then
`cargo build/test/clippy/fmt --workspace`; 125 tests green).

**Next (Increment 2): MH-1, the keystone.** The gate still trusts a caller-set `SignOff` enum — an embedder
can construct `Contribution{ sign_off: HumanConfirmed }` and pass. Implement owner-key **attestation** over
`(signature, plan, label, sign_off, config_class)` (extend `provenance::SignedPlan` → `SignedContribution`),
verified in the gate for `HumanConfirmed`. **This needs an owner decision first** (in `FOLLOWUPS.md`
"[Custody]"): the key-custody / trust model — asymmetric (ed25519, engine holds only the public key) vs HMAC,
and where the human-authority key lives. Surface that decision before coding MH-1.
Other queued engine work is in `FOLLOWUPS.md`. The original full design: build **MH-1 (EI-08
owner-key attestation over the contribution tuple)** FIRST — every other integrity gap degrades to a
forgeable annotation without it (`crates/corpus-client/src/gate.rs:15` + `crates/provenance/src/lib.rs:63-80`).
Then verdict-binding (MH-2), real post-fix re-collection (MH-3/NR-1), tamper-evidence + revocation (MH-4/8),
honest `config_class` (MH-6). To start the research track, fill `docs/research/` following the commit-ordering
discipline (negative-results before claims; prereg before any lane field). See `docs/evidence-integrity-and-research-checklist.md` §6–§8.

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

## Handoff log (reverse-chronological)

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
