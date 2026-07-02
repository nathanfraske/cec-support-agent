# Consolidated work plan — full repo + branch scope (2026-07-02)

**Scope:** every branch of `nathanfraske/cec-support-agent`, both open PRs, the orphan
tracking branch, and all in-repo docs/checklists — reconciled against the actual code at
each branch tip. Where a document and the code disagreed, the code was checked directly
(builds run in disposable worktrees; no branch was mutated).

**One architecture decision is folded in throughout (owner steer, 2026-07-02):** the
engine **presents as an API consumed by AllMyStuff / MyOwnMesh**, rather than being
driven as a spawn-per-diagnosis CLI sidecar. This supersedes RFC decision **D1**
(single-shot CLI) in `docs/integration-rfc-for-chris.md` and promotes several
deliberately-deferred items (daemon mode, the post-execution envelope, W8, the
`HttpCorpus` read-path hardening) into the critical path. Details in §3.

---

## 1. Verified state of the world

### 1.1 Branch / PR topology

```
main (77c6dbd) ── merged PR #1: 8-stage support pipeline, 119 tests
 └─ feat/agent-ops-evidence-integrity (b7ad864) ── OPEN PR #2 → main · CI GREEN · 159 tests
     └─ feat/myown-integration-p0 (673a381) ── OPEN PR #3 → PR #2's branch · CI GREEN · 165 tests
         └─ feat/corpus-leak-prevention (cf95d1c) ── NO PR · ⚠ DOES NOT COMPILE

ops/agent-handoff (ee1379b) ── orphan branch, 24 session-end-hook commits (2026-06-14/15);
                               canonical TODOS.md / FOLLOWUPS.md / HANDOFFS.md / .claude/memory state
                               lives ONLY here (never committed to any feature branch)
```

Stack note: PR #3 does not contain PR #2's tip hashes (`951ae82`/`b7ad864`) — it carries
**patch-identical** clones (`53dd992`/`673a381`, verified via `git patch-id`; the
`.github` diff between the two tips is empty). Merge order PR #2 → PR #3 is safe; at
worst PR #3 shows two redundant no-op commits. No content divergence exists.

Last activity on all branches: **2026-06-15** (~2.5 weeks idle). Both PRs were green and
explicitly awaiting the owner's merge action when the last session ended.

### 1.2 Done (implemented + tested, sitting in green PRs or merged)

| Work | Where | Evidence |
|---|---|---|
| 8-stage pipeline: intake interview, verification classes, routing taxonomy, HMAC plan provenance, outcome labels + flywheel, Windows tools, CLI, model tiering | `main` | PR #1 merged, 119 tests |
| Evidence-integrity gate (`ensure_evidence_integrity`, `GateError`), verification verdict bound to rows | PR #2 | `corpus-client/src/gate.rs` |
| **MH-1/EI-08 keystone:** ed25519 sign-off attestation — authority holds the private key, engine embeds/verifies the pubkey, self-asserted `HumanConfirmed` refused; operator wiring (`gen-signoff-key`, `CEC_SIGNOFF_PUBKEY`, `CEC_SIGNOFF_SEED`) | PR #2 | `provenance/src/lib.rs`, `gate.rs:104-119`, `main.rs:299-360` |
| MH-2/EI-01/EI-02/EI-03: `VerificationClass`, `RowProvenance{run_id, retrieval_first, primed_from}`, independent-confirmation guard keyed on `run_id` | PR #2 | `schema.rs:132-148`, `store.rs:40-54` |
| MH-4/MH-8/EI-05/EI-06: sha256 hash-chain tamper-evidence, owner-only revocation, run-deduped `Reopened` demotion, corroboration budget | PR #2 | `schema.rs:269-306`, `store.rs` |
| MH-5 risk reconciliation; MH-3/NR-1 `Unverified` verdict (safety-net half); MH-6 `host_inventory()` extension point; serde-independent canonicalization; sandbox-evidence wiring | PR #2 | Increments 5–10 |
| **14 adversarial-audit findings (audit-C1..C14) all fixed & re-verified** — incl. the CRITICAL at-rest re-admission bypass (`with_authority` re-gates every loaded row, fails closed) | PR #2 | `.claude/audit/confirmed-findings.txt`, commit `11f0609` |
| CI hardening: concurrency group, SHA-pinned actions, cargo-deny-action, gitleaks v3 + token fix, `dependabot.yml` (github-actions only), hardened pre-commit corpus blocklist | PR #2 tip (inherited by #3) | `.github/workflows/ci.yml` |
| **MyOwn P0 seams:** `InventoryProvider`/`CoarseHostInventory`/`ExternalInventory`; `--inventory-keys`; `--json` → `cec-diagnose/v1` envelope (de-identified by construction — free-text `title`/`rationale`/`description` deliberately omitted after the D1-leak review fix); stdout purity (one JSON line on stdout, trace to stderr); process-level contract tests | PR #3 | `common/src/inventory.rs`, `main.rs`, `tests/cli_contract.rs` |
| Leak-prevention methodology doc: 9 threat classes (leak-C1..C9), 4 defense layers, 5-phase plan, honest §3.1 not-closable declarations | `feat/corpus-leak-prevention` | `docs/corpus-leak-prevention.md` (494 lines, `f609454`) |
| `crates/deid` (validating mints: frozen `ACTION_VOCABULARY`, `plan_id` charset, `symptom` round-trip) + `crates/leakguard` (canonical POISON set) — both compile and pass their own unit tests in isolation | `feat/corpus-leak-prevention` | `cf95d1c` |
| Off-tree private corpus repo (`/mnt/e/cec-corpus-private`): YAML fix-flow format, `corpus-ingest` compiler W4–W7, seedless validation CI gate (checklist A10) | private repo | narrative only — not verifiable from this repo |

### 1.3 ⚠ Critical finding: the newest tip is broken

`feat/corpus-leak-prevention@cf95d1c` (**"closes C1"**) **does not compile** —
independently verified twice (`cargo check --workspace --tests`): 7×E0599 in
`corpus-client`, 2×E0308 in `support-agent`.

Root cause: every *call site* (`lib.rs`, `store.rs`, `gate.rs`, `main.rs:886-897`, all
test files) was rewritten to treat `Contribution::new` / `de_identify_plan` as returning
`Result<_, deid::Reject>`, but `crates/corpus-client/src/schema.rs` **was never edited**
(zero diff vs. its base). The mints exist but are wired into no production path, so
**leak-C1 is NOT actually closed** and the commit message is false. The likely mechanism:
the work happened in an ephemeral worktree (`/tmp/cec-leak`, removed at session end per
the handoff) and the keystone `schema.rs` edit was lost before commit — exactly the WSL
loss-mode the repo's own durability policy exists to prevent. No PR exists for the
branch, so CI never caught it.

### 1.4 In progress (started, explicitly unfinished)

- **PR #2 and PR #3: green, unmerged.** The last handoff's explicit next action was
  "merge PR #2 first, then PR #3 (auto-retargets to main)". Never taken.
- **Leak-prevention Phase 0: half-landed** (§1.3) — the `schema.rs` wiring + `Result`
  propagation is the missing half. The branch also violates the repo's own tracking
  discipline: neither of its commits updated TODOS/FOLLOWUPS/HANDOFFS.
- **Leak-prevention Phases 1–2: owner-approved, zero code.** Phase 1 (type split
  `StoredPlan`/`StoredSymptom`, strip `Serialize` from raw `Plan`/`Candidate`/`Outcome`/
  `DiagnosticEvent`/`ToolOutcome`, `Prose(String)` leaf typing, sealed `Debug`, private
  `Contribution` fields, `trybuild` compile-fail tests, write-gate idempotence) is a
  LARGE workspace-wide serde refactor; Phase 2 (read-side `from_served` re-validation,
  frozen stop-code/module dictionaries, ban `serde_json::Value` on boundary types)
  follows. HANDOFFS.md documents concrete resume gotchas (e.g. `FileCorpus::open` /
  `HttpCorpus::query` deserialize the in-flight `Plan` today; `render_consent` copies
  `plan.title`; the strict symptom mint rejects legitimate `<id-prefix>_<digits>` tokens).
- **RFC Q1–Q5 awaiting Chris/owner** (`docs/integration-rfc-for-chris.md`): Q1 identity
  unification (gates P3/P4 — the single-pubkey gate can't represent a multi-owner mesh),
  Q2 mesh-inference privacy, Q3 `myownmesh-core` pin source-of-truth, Q4
  `MeshSandboxValidator`, Q5 tail-truncation anchor distribution.
- **Tracking-state stranding:** the final session's TODOS/FOLLOWUPS/HANDOFFS updates and
  all four `.claude/memory/*.md` files exist **only** on `ops/agent-handoff`.
- **MH-3/NR-1 real post-fix re-collection:** `recollect_post_signature()` returns `None`
  unconditionally (`main.rs:982-984`) — verdicts are honest (`Unverified` ⇒ escalate) but
  no genuine post-state diff exists yet; needs the Windows backend. Blocks research M1.
- **MH-6/A7 engine-native inventory:** the seam exists; engine-native `cfg(windows)` CIM
  enrichment (board/BIOS/chipset/GPU/driver — never serials) still pending a Windows host.
- **Private-repo operator steps:** W0 (real keygen passphrase), W1 (gitleaks + activate
  hooks + branch protection), W2 (private remote), W9 (key rotation). W8 (corpus service)
  is reshaped by the API steer (§3).

### 1.5 Documented-but-stale (fix the docs, not the code)

Direct code inspection confirms these documents **understate** what's built:

- `docs/evidence-integrity-and-research-checklist.md` §3/§6 still mark MH-2, MH-4, MH-6,
  MH-8, EI-01..03, EI-05, EI-06 as GAP/PARTIAL; its §9 changelog stops at Increment 2.
  Increments 3–10 landed afterwards.
- `FOLLOWUPS.md` (handoff-branch copy) leaves "MH-2/EI-01 remainder", "EI-03/A5",
  "MH-4/MH-8/EI-06" open — all are implemented and tested.
- `SECURITY.md:32` still calls sign-off attestation "tracked as the keystone follow-up" —
  it is implemented and enforced under `.with_authority`.
- `docs/research/negative-results.md` NR-3/NR-4 describe conditions since fixed.
- Stale handoff notes say "PR #2 is RED (fmt)" — the fix (`538cd43`) is already an
  ancestor of both PR tips; **do not re-apply it**.
- ID-namespace hazard: three unrelated `C1..Cn` schemes coexist (leak-prevention C1–C9,
  audit findings C1–C14, research-draft C1/C2). Always cite the source file.

---

## 2. Immediate housekeeping (Phase A) — land what's green, stop the bleeding

Ordered; everything here is small.

- **A1. Merge PR #2** ("Rebase and merge" or a merge commit preferred; squash also safe —
  patch-id identity makes PR #3's CI clones no-op either way).
- **A2. Merge PR #3** after it auto-retargets to `main` (use "Update branch" first if
  GitHub asks). Result: main carries the evidence-integrity layer + MyOwn P0. ~165 tests.
- **A3. Rescue the tracking state:** copy the final-session `TODOS.md`, `FOLLOWUPS.md`,
  `HANDOFFS.md`, and `.claude/memory/*.md` from `ops/agent-handoff` onto `main` (one
  `docs(tracking)` commit), so the canonical resume state survives independently of the
  orphan branch.
- **A4. Doc-truth reconciliation** (one commit, no code): update the checklist §3/§6/§9,
  tombstone the stale FOLLOWUPS entries, fix `SECURITY.md:32` to name the enforced gates,
  annotate `negative-results.md` NR-3/NR-4 as fixed-since (per its own honesty rule),
  and add a "C-namespace disambiguation" note.
- **A5. Rebase `feat/corpus-leak-prevention` onto the new `main`** (it will then carry
  only `f609454` + `cf95d1c`).

## 3. The architecture steer: engine presents as an API (Phase B)

**Decision (owner, 2026-07-02):** AllMyStuff and MyOwnMesh consume the engine **as an
API**, not by embedding it or driving a spawn-per-diagnosis CLI. This supersedes RFC
**D1**. Record the supersession in `integration-rfc-for-chris.md` and
`integration-myown-family.md` (B1 below).

What survives unchanged — these were designed for exactly this move:

- **The license firewall.** The boundary was already a *process* boundary; an API is
  still one. The AGPL §13 network clause is the README's stated engine-protection lever —
  a network service is the case it was chosen for. AllMyStuff stays MIT: HTTP client +
  serde-only schema mirror, zero `cec-*` cargo edge.
- **`cec-diagnose/v1`** becomes the diagnose response schema verbatim (it was built
  de-identified-by-construction and versioned additive-only for this).
- Cold start, the sign-off gate, consent semantics, de-id discipline — all unchanged.
- The single-shot CLI **stays** (runtime invariants #2/#4: self-host parity, no mandatory
  service); the API is additive.

Work items:

- **B1 (S). Revise the two integration docs:** D1 → superseded (API service); reshape
  P1/P2 (below); Q1–Q5 stand. Note for Chris that Q2 sharpens: the engine itself is now a
  network surface, so inference egress and API exposure are separate knobs.
- **B2 (S). Pin the envelope's enum wire values** before any app codes against them:
  `route`/`source`/`max_risk`/`consent_required`/`escalation` are currently emitted via
  `Debug` formatting — a variant rename would silently break the wire without a schema
  bump. Pin explicit serde string values (byte-identical to today's output) + a
  regression test. Cheap now, expensive after P1'.
- **B3 (M). `cec-support-agent serve`** — the API v1:
  - `POST /v1/diagnose` → body `{describe, inventory_keys?, options?}` → **exactly the
    `cec-diagnose/v1` envelope** (reuse `diagnose_envelope()`; no new serialization path).
  - Two-phase execution preserved as API semantics: `POST /v1/execute` presents the
    signed winning plan reference + an explicit consent assertion + sign-off level; the
    engine re-verifies the judge's HMAC plan signature, the consent gate and required
    escalation exactly as the CLI does; response is the **post-execution envelope
    (`cec-execute/v1`: outcome label + verification verdict)** — this promotes the
    previously-deferred FOLLOWUPS item into scope.
  - `GET /v1/health` (+ schema version advertisement).
  - **Bind 127.0.0.1 by default; refuse non-loopback bind without an explicit
    `--allow-remote` flag** — mirrors the leak-C2 accepted-risk posture and the Q2 lean.
  - Server deps (axum or hyper) must pass `cargo deny check licenses`.
  - The API is a **new egress sink** — the exact class of bug that caused the P0 D1 leak.
    Mitigation in the same PR: responses restricted to the existing envelope types, plus
    poison-token contract tests (port `cli_contract.rs` to the HTTP surface).
- **B4 (S). Harden `HttpCorpus::query`** (`store.rs:425-453`): re-verify the ed25519
  attestation + `admit()` on every received row. Was "should be considered" for P3's
  `MeshCorpus`; with HTTP now a primary transport it is no longer optional.
- **B5 (S). Lifecycle guidance for embedders:** AllMyStuff may still bundle the engine
  binary and manage `serve` as a child process (reusing its sidecar pattern) or connect
  to an already-running service — either way it *talks HTTP*, never stdio, never links.

## 4. Repair and land leak-prevention (Phases C, D)

- **C1 (S). Fix the broken Phase 0** on the rebased branch: make
  `de_identify_plan → Result` + `Contribution::new → Result<Self, deid::Reject>` in
  `schema.rs`, calling `deid::action`/`deid::plan_id` per the doc's Phase-0 exit
  criterion #3; propagate; run the full suite (expect ~180 tests incl. the two C1
  regression guards and the vocabulary drift test); update TODOS/FOLLOWUPS in the same
  commit (the discipline the tip skipped). Commit honestly ("actually closes leak-C1").
- **C2 (S). Open PR #4** for the branch → main. CI finally runs against it.
- **C3.** Downstream note: private `corpus-ingest` adapts to the `Result` API on its next
  engine-pin bump (hard compile failure is the designed behavior).
- **D1 (L). Phase 1 — the type-invariant refactor** (owner-approved): `StoredPlan`/
  `StoredSymptom` as the only serde corpus types; strip `Serialize` from raw types;
  `Prose(String)` (no `Serialize`/`Display`) for title/description/rationale/message;
  sealed `Debug`; private `Contribution` fields; `trybuild` compile-fail tests;
  write-gate idempotence. Use HANDOFFS.md's recorded gotchas. **New requirement from the
  API steer:** the API/envelope module joins the egress-sink inventory, and `Prose` must
  be unrepresentable in API responses by construction.
- **D2 (M). Phase 2** — `from_served` re-validation via `#[serde(try_from)]` (closes the
  read-side for Http/Mesh corpora, compounding B4), frozen stop-code/module dictionaries
  (leak-C5), ban `serde_json::Value` on boundary types.
- **D3 (M, later). Phases 3–4** as scoped in `docs/corpus-leak-prevention.md`: egress
  lint + `xtask scan-content` + hook activation (subsumes the open "custody activation"
  item); `PromptPayload` / `--allow-remote-inference` (leak-C2); keyed-HMAC fingerprints
  (leak-C7 — also de-correlates the envelope's `config_class`); CODEOWNERS + copy the §5
  Agent Contract into AGENTS.md (the doc explicitly instructs this and it was never done).

## 5. MyOwn-family integration, reshaped for the API (Phase E)

P-numbering kept, primes mark the reshape. P1'/P2' live in the AllMyStuff repo.

- **E1 = P1' (M).** App-side de-id allowlist `inventory_to_config_keys()` (unchanged) +
  serde-only mirror of the **HTTP API** request/response schemas (instead of CLI stdio).
  Accept: no AGPL package in `cargo metadata`; seeded hostname/mac/ip/serial yield zero
  emitted config keys.
- **E2 = P2' (M).** Engine-service lifecycle in AllMyStuff (bundle + manage `serve`, or
  discover a running instance) + `diagnose_run` two-phase consent UI driving
  `/v1/diagnose` → human consent → `/v1/execute`. Accept: no execution before consent;
  graceful degrade with no engine; CI guard fails if AGPL enters the app graph.
- **E3 = P3 (L, gated on Q1).** `corpus-mesh-adapter` (AGPL, ships with the engine):
  `MeshCorpus` re-verifying attestation on every received row, `serve_corpus` gated
  read=roster / write=`Role::Owner`. Q1's multi-authority question must be answered
  first (single-pubkey gate can't hold a key set — same registry work as F3).
- **E4 = P4 (M, gated on Q1).** Identity unification seam, inference egress policy
  (loopback default), the three CI guards (no-AGPL-in-app; engine cold-start build;
  no-cycle dep graph).
- **E0 (blocking, zero-code):** get Chris/owner answers to **Q1–Q5**. Q1 gates E3/E4;
  nothing else blocks E1/E2.

## 6. Evidence-integrity residuals (Phase F — engine hardening backlog)

From FOLLOWUPS (the still-genuinely-open subset), PR #2's deferred list, and the audit:

- **F1 (M).** Key/anchor the keyless chain head (HMAC with a store secret, or
  authority-signed head+count anchor) — closes tail-truncation; Q5 decides mesh-peer
  anchor distribution.
- **F2 (S).** `chain_hash` → serde-independent canonical encoding (cross-language
  verifiability; currently coupled to struct field order).
- **F3 (M).** Authority key-id → key registry: rotation currently makes an
  old-key corpus un-openable under `with_authority`; also unlocks multi-owner mesh (Q1)
  and distinct verifier-vs-human authority keys.
- **F4 (M, needs Windows host).** MH-3/NR-1 real post-fix re-collection replacing the
  `None` stub; MH-6 `cfg(windows)` CIM inventory enrichment. Unblocks research M1/M2.
- **F5 (L, infra).** Production `SandboxValidator` (disposable VM; or a mesh peer per Q4).
- **F6 (S).** MH-5 residuals once inference integration lands: de-id at generation time;
  inference-channel provenance on the row.

## 7. Research track (Phase G — blocked mostly on F4)

- **G1.** Revise `negative-results.md` (NR-3/NR-4 fixed-since) *before* any claims cite it.
- **G2.** `--no-retrieval` control-lane toggle (prereg §0 precondition; `corpus.query` is
  currently unconditional at `main.rs:378`).
- **G3.** Fill `claims.md` + `prereg-control-lane.md` honoring the commit-ordering rule
  (prereg before any lane-tagged row exists, else VOID).
- **G4.** Milestones M1–M4 per `docs/research/README.md`; M1 needs F4.
- **G5.** Reconcile the PP-01..13 port: only PP-01/04/06/07 have explicit analogs; map or
  strike the rest.

## 8. Ops / process (Phase H)

- **H1.** Decide: keep the session-end handoff infra (then provision the bot PAT so
  pushes stop riding the owner's `gh` login) or retire it (remote sessions make the WSL
  loss-mode moot — and §1.3 shows the mechanism did not save the one artifact that
  mattered). Either way, A3 removes the single-point-of-truth risk.
- **H2.** `ops/provision.sh` (cargo-shaped disaster recovery); optional claude-rc units.
- **H3.** Private-repo operator steps: W0 real keygen passphrase, W1 gitleaks + hook
  activation + branch protection, W2 private remote, W9 rotation (pairs with F3). W8 is
  superseded by B3/E3 (the corpus is served via the engine API / mesh, never publicly).
- **H4.** Pin an exact Rust toolchain (or a tested-version CI job): `channel = "stable"`
  already broke CI once (rustfmt 1.9, `538cd43`) and is the top re-break risk after 2.5
  idle weeks; note the SHA-pinned toolchain *action* does not fix this. Extend dependabot
  to the `cargo` ecosystem (the ed25519 chain currently gets no advisory PRs).

---

## 9. Recommended execution order

```
A1→A2 (merge PRs)  →  A3/A4 (rescue tracking state, de-stale docs)  →  A5 (rebase leak branch)
→  C1/C2 (fix Phase 0, open PR #4)          ← smallest fix, closes the falsely-claimed leak-C1
→  B1/B2 (RFC supersession + pin wire enums) ← must precede any app coding against the API
→  B3/B4 (serve + cec-execute/v1 + HttpCorpus read hardening)  +  H4 (toolchain pin) in parallel
→  D1→D2 (leak Phases 1–2, the big refactor — API sink included in its scope)
→  E0 (Q1–Q5 answers) → E1/E2 (AllMyStuff API client + consent UI) → E3/E4 (mesh, gated on Q1)
→  F1–F3 opportunistically alongside; F4/F5 when a Windows host / VM backend exists
→  G, remaining H as capacity allows
```

Sizes: S < half a day · M = 1–3 days · L = a week-plus of focused work.

The three highest-leverage moves today: **merge the two green PRs** (everything else
rebases simpler), **fix the non-compiling Phase 0 tip** (a claimed-closed CRITICAL leak
class is in fact open), and **pin the envelope enums before the API ships** (the last
cheap moment to do so).
