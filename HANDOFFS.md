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

**As of 2026-07-04 ~19:10 UTC.** **The migration bundle (items 4+6, "PR-2") is BUILT and green on branch
`claude/workflow-model-optimization-e1y1sx`** (= `main` @ `44aaa88` + the owner-decisions docs commit
`c73afde` + 3 code commits). 235 tests, clippy `-D warnings` clean, fmt clean. Not yet pushed/PR'd at the
time of this block — a §7 blind-audit panel (3 auditors) is running on the two new kernels first.

- **F2 (`92df52d`):** `chain_hash` = sha256 over an explicit field-by-field length-prefixed
  `chain_canonical` (domain `cec-corpus-chain-v2`) — serde-independent, binds EVERY field incl.
  `plan.title`/step `description`/the attestation (which `attestation_message` leaves derived/excluded),
  never `integrity`; lists bind in STORED order (tamper-evidence, not set-canonicalization). Guards: a
  hand-assembled canonical-bytes pin, a 25-mutation binding sweep (pairwise-distinct), an ambiguity case, a
  v1-era-file-refused-at-open pin. All proven red on a v1 revert.
- **leak-C7 (`e17f38f`):** `fingerprint_of` = HMAC-SHA256 (`cec-fingerprint-v2`, 64-hex) under a
  per-deployment salt: `CEC_FINGERPRINT_SALT` read at startup on both CLI paths (mirrors the sign-off key,
  owner decision 2026-07-03), <16 bytes refuses startup with a fixed no-echo message, unset → documented
  PUBLIC cold-start default (`common::COLD_START_FINGERPRINT_SALT` — domain separation only, NOT secret).
  Salt is write-once (`common::set_fingerprint_salt`). Proven red against the silent salt-ignore regression.
- **Control E logging half (`90ff2c2`):** retrieval keys moved from the GET URL path into a
  `POST /v1/mappings/query` JSON body (no live corpus service exists; the service contract lands with Q6/B4).
- **Hard cutover shipped as decided:** stored fingerprints, config-class derived hashes, and chain hashes
  all changed; a v1-era corpus file now FAILS at open. **OPERATOR NOTE: the private corpus
  (`/mnt/e/cec-corpus-private`) must re-ingest once** on its next engine-pin bump, and any deployment
  should set `CEC_FINGERPRINT_SALT` (e.g. `openssl rand -hex 32`) — the cold-start default gives no
  unlinkability. Canned fixture regenerated (fragment-split so the invariant hook's corpus-row backstop
  stays meaningful; hook pattern widened to 16-64 hex).
- **Docs updated in the same pass:** leak doc §3.1(2) + §4 item 15 BUILT/DONE notes; cartography control E
  BUILT note; addendum §7 substrate/reserved-values list now names the keyed fingerprint +
  `cec-corpus-chain-v2`; FOLLOWUPS closed the `chain_hash canonical encoding` + control-E items.

--- previous (superseded by the migration-bundle state above) ---

**As of 2026-07-03 ~03:15 UTC.** **Execution-zone trio MERGED (PR #16 → `main` @ `44aaa88`); 4 owner
decisions locked; B4 re-scoped; migration bundle is the next code PR.** Branch
`claude/repo-scope-work-plan-h93qx5` restarted from `main` @ `44aaa88`.

- **Owner decisions (2026-07-03), all recorded:** **Q7** = ed25519 custodied judge key (RFC Q7 DECIDED note;
  pairs with F3 — a second custodied ed25519 key alongside sign-off); **Q1 volunteer-half** = volunteer is a
  pure execution target, a central authority attests (RFC Q1 DECIDED-partial; operator single-key unification
  still open); **leak-C7 salt** = per-deployment secret loaded like the sign-off key + documented cold-start
  default; **migration** = hard cutover (public repo has no rows; private corpus re-ingests once).
- **B4 (item 5) RE-SCOPED and deferred → FOLLOWUPS.** `HttpCorpus::query` serves bare `FixMapping`s with **no
  attestation** to re-verify (`store.rs:470`, `schema.rs:86`); attested-read-path re-verification is **RFC
  Q6-gated** (served-row provenance minimization, still open) + needs the corpus service. Not the small item
  I'd planned.
- **Next code = items 4+6, the migration bundle (NOW UNBLOCKED).** F2 serde-independent `chain_hash`
  (`schema.rs:461`, replace `serde_json::to_vec` with explicit length-prefixed encoding; bump
  `cec-corpus-chain-v1`→`v2`) + leak-C7 keyed/salted `fingerprint_of` (`common/src/hash.rs`, per-deployment
  secret salt + cold-start default). One hard cutover: update in-tree fixtures + operator re-ingest note.
  Precision-critical crypto — do it as its own focused PR-2. Plan: `scratchpad/lane2-implementation-plan.md`.

--- previous (superseded by the trio-merged state above) ---

**As of 2026-07-03 ~02:17 UTC.** **Lane ② underway — items 2 and 3 done on-branch (not yet PR'd).** Branch
`claude/repo-scope-work-plan-h93qx5` (restarted from `main` @ `ac14edf`). **Item 3 (exec audit-log skeleton)
just landed:** new `crates/support-agent/src/audit.rs` — `ExecutionRecord` (closed de-identified field set:
minted plan_id, opaque run_id, unix ts, outcome-label token, `caller_key: None` until rung-2), `to_line()`,
`AuditSink` trait + `NullSink` default. Emitted at the single `record_outcome` funnel (every outcome incl.
declines), using the MINTED id read back from the contribution (never the raw pre-mint id) + reused
`serve::wire_label`. Injection seam: `AppState.audit` (serve) / `&NullSink` (CLI). 3 new tests; workspace
green (fmt/clippy -D/tests; support-agent unit 36→39, 223 pass total). Rung-2 wiring (persistent sink,
caller_key, CLI seam, refuse-path marker) → FOLLOWUPS. **Remaining Lane ②:** item 1 (MCP-wrapper spec doc)
→ then open **PR-1** for the execution-zone trio (items 1/2/3); item 5/B4 (HttpCorpus read attestation);
items 4+6 bundled migration (F2 `chain_hash` + leak-C7 keyed fingerprint). Plan:
`scratchpad/lane2-implementation-plan.md`.

--- previous (superseded by the item-3 state above) ---

**As of 2026-07-03 ~01:47 UTC.** **Lane ② (pure-engine work) is UNDERWAY.** PR #15 (panels + fleet-design
docs) merged to `main` (`ac14edf`); branch **`claude/repo-scope-work-plan-h93qx5`** restarted from the new
`main`; babysitter cron `69d7ae77` retired. Owner greenlit the whole of Lane ② ("both, together") from
`docs/test-validation-fleet-design.md` §5 — 3 fleet contracts + 3 corpus-hardening items — to land as green
PRs. **Plan of record:** `scratchpad/lane2-implementation-plan.md` (6 items, PR split, the two migration
risks). **Item 2 DONE (committed on-branch, not yet PR'd):** the `SandboxValidator` "lowers-an-escalation,
never mints truth" contract is now normative in `crates/swarm/src/lib.rs` docs, pinned by a new
`support-agent` test (`a_clean_sandbox_can_never_mint_a_resolved_row`) proving clean-apply + `None`
re-collection → `Unverified` → `EscalatedHumanUnresolved`. Workspace green (clippy -D, all tests; unit 35→36).
**Remaining Lane ② (see TODOS + plan):** item 1 (MCP-wrapper spec doc), item 3 (exec audit-log skeleton),
item 5/B4 (HttpCorpus read-path attestation re-verify), items 4+6 bundled (F2 canonical `chain_hash` +
leak-C7 keyed fingerprint — one corpus-migration moment, needs a salt-custody micro-decision). **PR strategy:**
accumulate the execution-zone trio (items 1/2/3) then open PR-1; keep the migration bundle its own PR. Do NOT
build the off-box/distributed access-MCP parts — those are Q7/Q1-gated.

--- previous (superseded by the Lane ② state above) ---

**As of 2026-07-03 ~01:22 UTC.** **The test-and-validation-fleet model is now DESIGNED (decision-ready,
no code)** on branch **`claude/repo-scope-work-plan-h93qx5`**. This scoped the two highest-risk *runtime*
surfaces the owner asked to stand up: **(a)** the target-environment access MCP (how a diagnosis agent
reaches a client/volunteer PC) and **(b)** the sandbox test-harness MCP. Per the owner's design-first steer,
this is a threat model + contract, **not** an implementation — these are the on-machine execution zone and
must be modeled before built.

- **`docs/test-validation-fleet-design.md` landed.** The cardinal rule: **both MCP surfaces WRAP the
  existing gates (two-phase consent, ed25519 sign-off, HMAC plan-provenance, risk reconciliation,
  escalation-recompute, advisory-only rail) — they never expose a raw on-machine tool.** The verb is a gated
  `diagnose`/`execute` over `/v1/execute`, never `registry_set`/`download_file`/shell/the internal
  `{tool,args}` loop. Contents: the 6-stage loop mapped onto existing code (§1); the T-1..T-7 execution-boundary
  threat model, each mapped to the gate that stops it + the NEW wire-boundary guard (§2); the `SandboxValidator`
  "lowers-an-escalation-never-raises-trust-without-a-signature" contract (§3); **§3.1 the Windows-reproduction
  mechanism** (see below); the volunteer fleet as a de-identified execution *target* with no volunteer-id on the
  row (§4); greenlight-now vs infra-gated vs Chris-gated sequencing (§5); anti-scope (§6); honest gaps (§7).
- **§3.1 answers the owner's live question** ("a golden image per Windows update?"): **No.** `ConfigClass`
  (`common/src/config_class.rs`) is the image key and keys on `{release branch} × {hardware/driver inventory}`
  (shipped test uses `"os:windows 11 23h2"`, not a monthly build) — a cumulative update mints no new class.
  When a patched image *is* needed, the KB is injected **offline via DISM from the Update Catalog** (no box
  "downloads" it); images are demand-driven. The hard boundary: a VM reproduces *software-state* cheaply/offline
  but **cannot synthesize real silicon** (OEM driver stack/firmware) — that dimension is exactly why the
  volunteer fleet exists (sandbox = software-state classes; volunteers = hardware classes).
- **RFC Q7 filed** (`docs/integration-rfc-for-chris.md`): plan-provenance signing across the *execution*
  boundary — the current per-run symmetric HMAC assumes judge==executor in one process; off-box that breaks.
  Fork: **(a)** judge-runs-on-target (HMAC stays in-process) vs **(b)** ed25519 with a persistent custodied
  judge key. Pairs with Q1 (is a volunteer a rostered identity that holds an authority, or a pure target?).
- **The three hard gates the design surfaces (all pre-existing, now named as fleet-blocking):** **F4** real
  post-fix re-collection (`recollect_post_signature() -> None` stub, NR-1) — *gates the value of the entire
  fleet*, since every run is `Unverified` until it lands; **F5** a production `SandboxValidator` VM backend
  (seam wired `None` in both callers); the **volunteer enrollment + scoped/revocable consent framework**
  (no volunteer concept exists in code at all — the largest greenfield, mostly policy/legal not engine).

**No `.rs` code changed this pass** — three docs only (`test-validation-fleet-design.md` new; `integration-rfc-
for-chris.md` +Q7; the design doc's §5 points at Q7). 210 tests unchanged.

--- previous (superseded by the fleet-design state above) ---

**As of 2026-07-02 ~18:54 UTC.** **Corpus cartography (leak-C10) is now the fourth enforced corpus
property** — after **admissibility** (the sign-off/attestation gate), **authenticity** (de-id + the leak
Phase 0-2 type barrier), and **access** (the auth ladder + loopback/mesh posture) — **non-mappability /
query-oracle resistance** is now a stated, tracked property of the corpus surface, on branch
**`claude/repo-scope-work-plan-h93qx5`** (on top of the API-posture state below).

- **The threat model landed:** `docs/corpus-cartography-threat.md` (owner-raised: "can a surface expose the
  internal corpus by mapping it out through trusted calls?") — the honest limit (§0: a rostered caller *is*
  permitted to learn its own answers; the goal is minimize-not-eliminate), 7 concrete verified vectors
  V1-V7, a lettered control set A-G, the NON-MAPPABILITY rule set (§3b), and the phased sequence onto
  F2→F3→B4→F1→E3.
- **leak-C10 defined in the taxonomy:** `docs/corpus-leak-prevention.md` §1.2 gained a C10 row (ranked below
  C9 as a distinct orthogonal axis — it needs no identity to survive de-id, so no `DeIdentified<T>` closes
  it) + a §3.1(4) cross-reference.
- **The `source` membership label drop SHIPPED** — `4cf9d8f` (code, committed prior to this docs/policy
  pass): `cec-diagnose/v1` candidates now carry only `{plan_id, max_risk, actions[]}`; `source`
  (cold_model/corpus_primed/human) and the dead `wire_source` are removed, with a negative pin so it cannot
  be silently re-added. This closes control D's label half (vector V1) only.
- **Non-mappability policy is now BINDING in `AGENTS.md`** — the threat doc's §3b 7-rule set, copied in as a
  sibling block to the existing §2.5 egress-sink checklist, same short imperative voice.
- **Wire-contract docs corrected to match the shipped code:** `docs/integration-rfc-for-chris.md`'s candidate
  body is now `{plan_id, max_risk, actions[]}` with a removal note + the enum-grammar note corrected; real
  question **Q6** ("how much provenance does a served row expose?") is now filed in the RFC's open-questions
  section, gated on B4. `docs/api-extension-design.md` §5 decision log gained the dated `source`-drop entry.
- **Deferred to `FOLLOWUPS.md`, each attributed to the threat doc:** control D's remainder (retrieval-first
  latency + slate-count differential equalization — a genuine owner trade-off, costs the retrieval-first
  speed win, only bites once a non-owner is served, decide before/at E3), control A (per-identity query
  budget, E3/rung-2), control B (per-identity query audit log, the query-side twin of the deferred MH-1
  audit-log item, E3/rung-2), control E (keyed/salted HMAC fingerprint — greenlightable, pulls forward the
  existing leak-C7 residual), and control C (B4 served-row provenance-graph minimization — decide as a B4
  precondition, cheap now / expensive after B4 ships).

**No `.rs` code changed this pass** — the code drop was already committed (`4cf9d8f`); this was the
docs/policy/tracking pass to match it. 210 tests (unchanged from the API-posture state below — no engine
code touched).

--- previous (superseded by the corpus-cartography state above) ---

**As of 2026-07-02 ~22:40 UTC.** **The owner's 2026-07-02 API-posture decisions are IMPLEMENTED** on branch
**`claude/repo-scope-work-plan-h93qx5`** (on top of leak Phase 2; +5 commits, **committed locally, NOT
pushed** — the orchestrator opens the PR). Responding to `docs/api-extension-design.md` (§1.6/§2.5/§3), in
green sub-step commits:

- **Trusted calls only (leak C2)** (`697e16d`): `validate_inference_endpoints` refuses a non-loopback
  `--endpoint`/`--fast-endpoint` at startup on BOTH the diagnose and serve paths unless
  `--allow-remote-inference` is passed (loopback = localhost / 127.0.0.0/8 / [::1]); fixed error, never echoes
  the URL; fails closed on an unparseable host. Builds leak-doc §3.1(b). Proven red-on-revert; live-smoked.
- **Route-surface pinning** (`588f1ec`): the frozen `route_surface` list (GET /v1/health, POST /v1/diagnose,
  POST /v1/execute) is folded into the router by `build_router`, and `router_surface_is_frozen` pins the exact
  (method, path) set — adding ANY route is a deliberate test edit. Never-routable invariant (attest, keygen,
  corpus WRITE) in serve.rs module docs + SECURITY.md (a violation is a reportable security issue). Proven
  red-on-revert (a rogue /v1/attest route fails the pin).
- **AGPL §13 notice** (`64ffa48`): `--allow-remote` prints a one-line stderr network-service / §13
  Corresponding-Source notice at startup; same note in SECURITY.md. Auth ladder resolved: hard-loopback by
  default, remote = mesh-only, NO bearer-token tier will be built.
- **Docs** (`878fd4d`): DECISION LOG (§5) in api-extension-design.md (corpus-over-API = mesh-rostered/loopback
  only, never token-auth public HTTP; attested rows, encrypted transport; doc-level, route-pin is the guard);
  the 6-rule §2.5 egress-sink checklist copied into AGENTS.md as binding policy.

**210 tests** (was 205: +4 endpoint-loopback, +1 route-pin), clippy `-D warnings` clean, fmt clean (pinned
1.96.1). The §13 notice and both C2 refusals were live-smoked on the real binary. No corpus endpoint was
built — decision 2 is doc-level plus the mechanical route-pin guard. Remaining: the `PromptPayload` chokepoint
(leak §3.1(a)) and the corpus-endpoint build (gated on B4 + Q1) — both FOLLOWUPS, none blocking.

--- previous (superseded by the API-posture state above) ---

**As of 2026-07-02 ~21:30 UTC.** **Corpus leak-prevention Phase 2 is DONE** on branch
**`claude/repo-scope-work-plan-h93qx5`** (started == origin/main @ `86e24cf`, which already carried Phase 1;
now +3 commits, **committed locally, NOT pushed** — the orchestrator opens the PR). Phase 2 is the C4/C5
hard stops, landed in 3 green sub-steps, each proven and committed:

- **C5 frozen dictionaries** (`a0818bc`): `common::extract` replaces the `is_stop_code_name`/`module_name`
  SHAPE heuristics with FROZEN `STOP_CODE_NAMES` (Microsoft bugchecks) + `MODULE_NAMES` (OS/driver
  allowlist), sorted for binary search. New public closed-grammar `is_symptom_token` (`VOCABULARY ∪ 0x-hex
  ∪ <prefix>_<digits> ∪ STOP_CODE_NAMES ∪ MODULE_NAMES`); `deid::symptom` wired to it — closes the Phase-1
  `<prefix>_<digits>` blocker (`event_41` is admitted directly, no round-trip). Asset tags / custom binaries
  / hostnames now refused; explorer.exe/event_41/xid_79/0x1234/real bugchecks stay admissible.
- **C4 read-side re-de-id** (`a759afd`): `#[serde(try_from = "String")]` validating deserialization on
  `StoredSymptom` (grammar), new `StoredAction` (frozen ACTION_VOCABULARY; on step action + description) and
  `StoredPlanId` (slug), plus `common::Symptom` (for `verification.recurring`). An out-of-vocab action /
  inadmissible id / non-grammar symptom now FAILS TO DESERIALIZE at `HttpCorpus::query` (transport/admission
  split → `ServedPlanInadmissible`) and `FileCorpus::open` (Storage parse error). Symptom mint wired into the
  1f gate (`GateError::SymptomNotDeIdentified`). Adversary-seeded read-path poison harness (leakguard::POISON
  in a served symptom) refused; proven red-on-revert.
- **2c serde_json::Value scoping** (doc commit): the only `Value` fields (`ToolOutcome.data`,
  `AgentStep.args`) have no serialize sink post-Phase-1 → documented the invariant, not re-typed (typing =
  C2/Phase-4). Scoped honestly per the doc.

**Wire/on-disk shape byte-identical for admissible rows** (canned pre-split fixture + chain stable; envelope
pins green). **205 tests** (was 198 after Phase 1), clippy `-D warnings` clean, fmt clean, CLI e2e smoke green
(the real binary keeps `whea_uncorrectable_error`/`event_41`/`explorer.exe`, drops asset tag `RIG_NATHAN_DESK`).
**Phases 0–2 of the leak methodology are now complete.** Remaining leak work is Phases 3–4 (egress-allowlist
dylint + content gate + live hook; PromptPayload/`--allow-remote-inference`; keyed HMAC; CODEOWNERS) — all
FOLLOWUPS/accepted-risk, none blocking. See `docs/corpus-leak-prevention.md` §4 Phases 3–4.

--- previous (Phase 1, superseded by the Phase-2 state above) ---

**As of 2026-07-02 ~19:15 UTC.** **Corpus leak-prevention Phase 1 is DONE** on branch
**`claude/repo-scope-work-plan-h93qx5`** (which started == origin/main @ `a31198e`; now +5 commits,
**committed locally, NOT pushed** — the orchestrator opens the PR). Phase 1 is the C1/C3 compile-error
hard stops, landed in 4 green sub-steps, each proven and committed:

- **1a type split** (`a347878`): `crates/corpus-client/src/stored.rs` holds `StoredPlan`/`StoredStep`/
  `StoredSymptom`/`StoredSignature`/`StoredOutcome` — the ONLY corpus-serializable payload. `de_identify_plan`
  mints a `StoredPlan`; `Contribution`.outcome + `FixMapping` carry stored types. Removed `Serialize`
  (+ dead `Deserialize`) from raw `Plan`/`PlanStep`/`Candidate`/`Outcome`/`DiagnosticEvent`/`StepResult`/
  `ExecutionResult`/`ToolOutcome`/`AgentRun`/`AgentStep`/`provenance::SignedPlan`. `Contribution` fields →
  `pub(crate)` + accessors; retrieval-first rehydrates via `StoredPlan::to_plan`.
- **1b Prose + 1d sealed Debug** (`3790dbd`): `common::Prose` (private field, no Serialize/Display,
  redacting Debug) for `title`/`description`/`rationale`/`message`/`summary`; containers keep a derived
  Debug that is auto-sealed. `render_consent`/human-trace/`provenance::canonical` read via `as_str()`.
- **1f write-gate idempotence** (`22ec564`): `ensure_evidence_integrity` re-mints the stored plan and
  refuses a non-de-identified row — `GateError::RowNotDeIdentified`.
- **trybuild** (`9a9cd5b`): `to_string(&candidate)`, struct-literal `Contribution{..}`, `format!("{}",prose)`
  all fail to compile (pinned `.stderr` @ 1.96.1). Plus a runtime Debug-no-leak test.

**Wire/on-disk shape is byte-identical** — a canned pre-split corpus row still deserializes, round-trips
identically (so `chain_hash` is stable), verifies its chain at open, and gate-passes. `cec-diagnose/v1` +
`cec-execute/v1` envelopes unchanged (pinning tests green). **198 tests** (was 189), clippy `-D warnings`
clean, fmt clean, CLI e2e smoke green. Both write-gate (1f) and one trybuild case proven red-on-revert.
**Next: Phase 2** (read-side `from_served`/`#[serde(try_from)]` + frozen stop-code/module dictionaries +
ban `serde_json::Value`) — see FOLLOWUPS + `docs/corpus-leak-prevention.md` §4.

--- previous (superseded) ---

**As of 2026-07-02 ~15:50 UTC.** The 2.5-week-stalled handoff was picked up (remote Claude session,
owner-directed): the full repo/branch scope is consolidated in **`docs/consolidated-work-plan.md`** (read it
first — it is the plan of record), and the first wave is executed:

- **PR #2 and PR #3 are MERGED to `main`** (`2d9620a`, `3b269f8`) — main now carries the evidence-integrity
  layer + MyOwn P0 (171 tests after the wire-pinning below).
- **`feat/corpus-leak-prevention` was BROKEN as pushed and is now repaired:** the pushed `cf95d1c` did not
  compile — the keystone `schema.rs` edit (`de_identify_plan`/`Contribution::new` → `Result` calling the
  `deid` mints) was lost with the ephemeral `/tmp/cec-leak` worktree, so "closes C1" was false on origin.
  Rebased onto main, edit restored inside the Phase 0 commit (`0855884`), re-verified (180 tests, clippy,
  fmt), force-pushed, **PR #5 open** → main. Phases 1–2 remain the next big increment.
- **Owner architecture decision (2026-07-02): the engine presents as an API** consumed by
  AllMyStuff/MyOwnMesh (supersedes RFC D1 — recorded in `docs/integration-rfc-for-chris.md` +
  `docs/integration-myown-family.md`). `cec-support-agent serve`: `POST /v1/diagnose` → the
  `cec-diagnose/v1` envelope; `POST /v1/execute` → a post-execution `cec-execute/v1` envelope (un-defers
  that FOLLOWUPS item); loopback-bound by default. P1/P2 reshape into API client + service lifecycle;
  `HttpCorpus::query` read-path attestation hardening is promoted (B4). Not yet implemented — B3/B4 in the
  plan.
- **Envelope enum wire values are PINNED** (`ec1e388`, on the housekeeping PR): snake_case tokens via
  exhaustive matches + a pinning test replace `Debug` formatting; `part_class` hoisted to an additive
  sibling field. Done now because the envelope still has zero consumers.
- **Tracking state rescued + docs de-staled** (this branch): the final-session TODOS/FOLLOWUPS/HANDOFFS +
  `.claude/memory/*` now live on a real branch; checklist §3/§6/§9, SECURITY.md, and negative-results
  NR-2/3/4 no longer understate the code.

--- previous (superseded) ---

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

> **Update 2026-07-04 ~19:10 UTC (the live front):** the migration bundle is BUILT (see Current state).
> **Update 2026-07-04 ~21:20 UTC:** PR #17 is MERGED (`e16fd35`); leak Phase-3 3b/3c is BUILT on the
> restarted branch (PR-3 next: push → open → babysit → merge). After PR-3, the remaining pure-engine
> menu: **L3a dylint egress lint** (FOLLOWUPS, large), **PromptPayload** (FOLLOWUPS — needs the owner's
> strict-vs-explicit call first), or start the **corpus service + B4** build (Q6 is decided; design the
> served-row provenance COMMITMENT with it). The infra gates (F4 re-collection, F5 VM backend) remain the
> real prod blockers — see the 2026-07-04 status report. Older text below:
> Steps (1)+(2) are DONE (panel returned, fixes landed `8626f23`, **PR #17 open** —
> https://github.com/nathanfraske/cec-support-agent/pull/17): **(3)** merge PR #17 when CI is green, then
> the next PURE-ENGINE greenlightable work
> (no owner/Chris gate) is **leak Phase 3** (egress-allowlist lint + `xtask scan-content` + hooks/CI
> boundary job — leak doc §4 items 12-13) and/or the **`PromptPayload` chokepoint** (§3.1(1a)/item 14);
> everything else on the list is gated (B4 → RFC Q6; F4 re-collection + F5 VM backend → infra; volunteer
> framework → policy). **Do NOT attempt B4 before Q6 is decided.**

--- superseded live-front notes below ---

> **Update 2026-07-03 ~03:15 UTC (the live front):** execution-zone trio (items 1/2/3) is MERGED to `main`
> (PR #16, `44aaa88`); the 4 owner decisions are locked (see Current state). **NEXT CODE = the F2 + leak-C7
> migration bundle (items 4+6), now unblocked** — do it as its own focused PR-2, one hard cutover:
> (a) F2: rewrite `chain_hash` (`corpus-client/src/schema.rs:461`) from `serde_json::to_vec(&bare)` to an
> explicit field-by-field length-prefixed canonical encoding (mirror `provenance::canonical` /
> `attestation_message`), bump the domain tag `cec-corpus-chain-v1`→`-v2`; (b) leak-C7: make
> `fingerprint_of` (`common/src/hash.rs`) a keyed HMAC with a **per-deployment secret salt loaded like the
> sign-off key + a documented cold-start default** (owner-decided), bumping its domain likewise; (c) update
> every in-tree test fixture that pins a chain hash or a fingerprint; (d) add an operator note (PR body +
> HANDOFFS) that the PRIVATE corpus must re-ingest once. Red-on-revert tests: reordered field → different v2
> hash but a struct-layout change → SAME hash (serde-independence); two salts → different fingerprints,
> identity never in the fingerprint. Plan: `scratchpad/lane2-implementation-plan.md` item 6. **B4 is deferred
> (Q6-gated) — do NOT attempt it before Q6 is decided.** The fleet-design / decision docs below are landed.

--- superseded live-front notes below ---

> **Update 2026-07-03 ~01:22 UTC (the live front):** the **test-and-validation-fleet design is landed and
> decision-ready** (`docs/test-validation-fleet-design.md`; RFC gained **Q7**). It is DESIGN ONLY — do **not**
> start building the MCP surfaces; the owner's confirmed steer is design/threat-model-first for the
> execution zone. **Next step: present the doc + await the owner's call** on two owner-gated forks — **Q7**
> (plan-signing topology across the execution boundary: judge-on-target vs ed25519 custodied key) and **Q1**
> (is a volunteer a rostered identity or a pure execution target). Three items are **greenlightable now,
> pure-engine, no infra/no Chris** if the owner says go (design doc §5): (1) the gated-MCP-wrapper spec over
> `/v1/execute` (frozen `{diagnose,execute}` verb contract + egress-sink inheritance) — hardens serve
> regardless of any volunteer; (2) the `SandboxValidator` production *contract* + a "clean report cannot mint
> a resolved row" test; (3) an execution audit-log skeleton (hashed key + plan-id + timestamp — the twin of
> the cartography V7 gap). The hard data gate under everything is **F4** (real post-fix re-collection; until
> it lands every run is `Unverified`). The corpus-leak-prevention / API-posture / cartography work below is
> all SHIPPED (see the log) — it is history, not the next step.

--- superseded corpus-leak-prevention resume notes below (Phase 2 is DONE — see the handoff log) ---

> **Update 2026-07-02:** the branch state below has moved — `feat/corpus-leak-prevention` is rebased onto
> main (PRs #2/#3 merged), the missing Phase-0 `schema.rs` edit is restored (`0855884`), and **PR #5** is
> open. The worktree advice stands. Phases 1–2 below remain the primary engine increment; sequence them
> against the **engine-as-API work (B3/B4)** per `docs/consolidated-work-plan.md` §9 — B3/B4 first is the
> recommended order (small, unblocks app-side work; Phase 1 must then include the API module in its
> egress-sink inventory).

### PRIMARY — Corpus leak-prevention Phase 2 (Phase 0 + Phase 1 DONE)

**Update 2026-07-02 ~19:15 UTC:** **Phase 1 is DONE** (the type barrier — see Current state) on
`claude/repo-scope-work-plan-h93qx5`, committed locally (`a347878`/`3790dbd`/`22ec564`/`9a9cd5b`), not
pushed. Only **Phase 2** remains. Phase-2 scope + the precise design is `docs/corpus-leak-prevention.md`
§4 Phase 2 + §2 Layer 1 (1c/1e) + Layer 2 (2c/2d). **Read it first.** Concretely:
1. **Read-side re-de-id (1e):** a `from_served(FixMapping) -> Result` that re-validates every served row, and
   `#[serde(try_from = "String")]` on `StoredSymptom` (and a new `ActionToken`) so a non-vocabulary WIRE value
   fails to *deserialize*. Today `HttpCorpus::query` already re-validates the served PLAN via `de_identify_plan`
   equality (good), but `StoredSymptom`/`StoredStep` still deserialize any string — that is what Phase 2 closes.
   The poison harness must drive an **adversary-seeded** served row (2d).
2. **Wire the strict symptom mint (the Phase-1 residual):** `deid::symptom` rejects a legit `<prefix>_<digits>`
   symptom, so it was NOT wired into the write path. Phase 2 replaces the `is_stop_code_name`/`module_name`
   SHAPE heuristics in `common/src/extract.rs` with FROZEN dictionaries and a closed grammar
   (`VOCABULARY ∪ 0x-hex ∪ <known-prefix>_<digits> ∪ stop-code dict ∪ module allowlist`), then wires the
   symptom mint into `de_identify_plan` + the 1f gate. Migrate `Verification.recurring` to `StoredSymptom`.
3. **Ban `serde_json::Value` on boundary types** (`ToolOutcome.data`, `AgentStep.args`) → typed summaries (2c).

**GOTCHA carried forward:** the strict symptom round-trip mint rejects a two-token-derived `<prefix>_<digits>`
symptom — the closed-grammar mint (step 2) is what makes it admissible. Don't wire the naive round-trip mint
into the write path.

--- superseded (Phase 1, now DONE) ---

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

- **(2026-07-04) `mv file.bak file` after a revert-proof restores an OLD mtime, and cargo will happily
  re-run the STALE test binary** — the restored source looked green-checked but the "test result" came from
  the still-reverted build; only a `touch` (or editing the file) forces the rebuild. After any
  backup/restore dance, `touch` the restored file (or verify with a `grep` on the source) before trusting
  the re-run. Cost one confusing "restored but still FAILED" cycle this session.
- **(2026-07-04) The invariant hook's corpus-row backstop fires on the sanctioned canned FIXTURE too — the
  fix is fragment-assembly, not weakening the pattern.** Any edit to `store.rs` re-flagged the pre-existing
  fixture (16-hex fingerprint shape). Splitting the fixture string at the fingerprint boundary (the same
  trick the hook uses for its own key-block markers) makes the file text non-contiguous while the runtime
  value is unchanged — so the hook pattern could be WIDENED (16-64 hex, catching v2-era real rows) instead
  of allowlisted.

- **(2026-07-03) Never put backticks in a `git commit -m "…"` double-quoted message — bash runs them as
  command substitution and silently DELETES the wrapped words from the commit.** A projectops commit lost
  every `` `verify` ``/`` `invariants` ``/`` `source` `` token this way (the `command not found` noise was the
  tell). Fix: write the message to a file and `git commit -F <file>` (or `--amend -F`), or use a `$'...'`/
  heredoc — a file authored via the Write tool never touches the shell. Earlier session commits with
  backticks in `-m` were likely mangled too (they are merged; not worth rewriting). Going forward: `-F` for
  any message with backticks.
- **(2026-07-02) A content-matching hook must not contain the patterns it matches, and must key on a
  distinctive VALUE, not structural keywords.** When the Tier-1 PostToolUse guard (`invariant-check.sh`)
  went live (the harness picks up `.claude/settings.json` mid-session), it self-flagged twice: once because
  the addendum documented the corpus-row shape `"outcome":{"signature":{"fingerprint"…}` as prose, and once
  because the hook's own source held the literal `-----BEGIN AGE ENCRYPTED FILE-----` marker. Fixes: (1) key
  the corpus-row check on the 16-hex fingerprint VALUE (`"fingerprint":"[0-9a-f]{16}"`) so a real row matches
  but a prose/ellipsis description does not; (2) assemble the seed/key markers from concatenated fragments so
  the contiguous literal never appears in the hook's source. General rule: a grep-heuristic guard is only safe
  if its pattern cannot occur in legitimate documentation OR in the guard's own text — prefer a distinctive
  value over a structural keyword, and hard-block only on the near-zero-false-positive signal (here, the
  exfil PATH), leaving fuzzier structural checks to a typed tool (`projectops invariants`), not a grep.
- **(2026-07-02) Pin a route surface by making the router and the pin read ONE list.** axum's `Router`
  does not expose its routes for inspection, so a "freeze the endpoints" test cannot introspect the built
  router. The working pattern: a `route_surface() -> Vec<(method, path, MethodRouter<Arc<AppState>>)>` is the
  single source of truth; `build_router` folds it into the axum `Router`, and the pinning test maps it to the
  `(method, path)` set and asserts equality. Because both the live router and the test read the same list they
  cannot drift, and adding a route changes the set the test checks → a deliberate test edit is forced. The
  handler in each tuple must be generic over the state (`get(health)`/`post(diagnose)` both unify to
  `MethodRouter<Arc<AppState>>`), and the test only sorts/compares the `(&str,&str)` pair, never the
  MethodRouter.

- **(2026-07-02) Parse an endpoint's host by hand for the loopback check — no `url` crate is a dep, and
  fail closed.** `endpoint_is_loopback` strips the scheme, takes the authority up to the first `/`/`?`/`#`,
  drops userinfo before `@`, and splits host from port — special-casing a bracketed IPv6 literal (`[::1]`)
  whose colons are part of the host. `localhost` matches directly; everything else goes through
  `IpAddr::parse().is_loopback()` (covers 127.0.0.0/8 and ::1). An unparseable host is treated as
  NON-loopback (refused) — the C2 egress guard must fail closed, never admit on a parse miss.

- **(2026-07-02) `#[serde(try_from)]` read-side validation only bites where the type is DESERIALIZED —
  and the corpus row's symptoms are two DIFFERENT types.** The signature symptoms are `StoredSymptom`
  (in corpus-client); the verification `recurring` symptoms are `common::Symptom` (in common). You cannot
  make `Verification.recurring: Vec<StoredSymptom>` — that needs `common` → `corpus-client`, the wrong
  dependency direction (a cycle). The fix is to put the grammar predicate in `common` (`is_symptom_token`)
  and add `try_from` to BOTH symptom types, each calling it. A single `StoredVerification` newtype would
  also work but is more churn for no extra safety. Lesson: when a validated leaf appears on a row via two
  crates, validate it in the lowest crate that owns the predicate, not by moving the field.

- **(2026-07-02) Read-side leaf validation forces test fixtures to use REAL grammar tokens — the
  synthetic `boot_loop` was never extractable.** Once `StoredSymptom`/`Symptom` validate on deserialize,
  every serialize→deserialize round-trip in a test (FileCorpus reopen, the gate) must carry a grammar
  member. ~10 corpus-client tests seeded `Symptom("boot_loop")`, which `extract_symptoms` never produces
  (it yields `["boot","loop"]` from "boot loop"), so it fails the closed grammar. Swept to `event_41`
  (a real `<prefix>_<digits>` token). Lesson: a test symptom must be something the extractor can emit;
  `is_symptom_token(tok)` is the check. Watch the tamper test that does `text.replace("<sym>", ...)` —
  replace with ANOTHER valid token (`xid_79`) so it still tests the CHAIN, not the new deserialize guard.

- **(2026-07-02) A closed-grammar mint unblocks the `<prefix>_<digits>` symptom the round-trip mint
  couldn't.** Phase 1 left `deid::symptom` on `extract_symptoms(s) == [s]`, which rejects `event_41`
  (built from two input tokens "event"+"41", so it never round-trips as one token) — that is why the
  symptom mint could not be wired into the write gate. Replacing the round-trip with direct closed-grammar
  membership (`is_symptom_token`) admits `event_41` directly and makes the mint sound for BOTH the gate and
  the `try_from` read guards. Lesson: when a "round-trip through the producer" predicate rejects a
  legitimate producer output, the producer is many-to-one; encode the grammar as membership, not round-trip.

- **(2026-07-02) `#[serde(try_from)]` moves the deserialize error inside `response.json()` — split
  transport from admission or the error type is wrong.** With validating leaves, a poisoned served row
  fails DURING `reqwest::Response::json()`, which maps to a transport error — but semantically it is an
  admission refusal, not a network fault. Read the body as text (transport) THEN
  `serde_json::from_str::<Vec<FixMapping>>` (admission), mapping the parse error to
  `GateError::ServedPlanInadmissible`. This also keeps the `de_identify_plan` equality check as a clean
  Layer-2 guard for the derived `title` (a plain String, not leaf-typed).

- **(2026-07-02) Typing the leaf prose makes sealed `Debug` (1d) fall out for free — no manual impls.** The
  methodology's 1d said "manual redacting `Debug` on each raw type." But once the prose is a `Prose` newtype
  (1b) whose OWN `Debug` redacts, every containing struct keeps a *derived* `Debug` that is automatically
  sealed — `format!("{:?}", outcome)` can't spill request text because the text lives only in `Prose` leaves.
  This is strictly better than manual impls (a manual impl can forget a field; a new `Prose` field is sealed
  by construction). Do 1b before 1d and 1d is a no-op. Proven by a runtime test that plants identity in a
  plan title/description and greps the Outcome's Debug.

- **(2026-07-02) Put the STORED types where their non-`common` deps already live (corpus-client), not the
  `deid` crate the doc named.** `StoredOutcome` needs `OutcomeLabel`/`SignOff`, which live in
  `corpus-client::schema`, and `de_identify_plan`/`Contribution` are there too. Placing `StoredPlan`/… in
  `deid` would have forced `OutcomeLabel` to move or split the stored types across two crates. `deid` stays
  the home of the validating MINTS (the doc's actual security point); the stored DATA types belong next to
  the row. Make the fields `pub(crate)` (+ `pub` accessors + `to_plan`/`to_signature` rehydration for
  embedders): in-crate de-id/gate code reads them with unchanged syntax, an external struct-literal fails,
  and there's no cross-crate accessor plumbing.

- **(2026-07-02) A byte-identical wire split needs a canned pre-change fixture test, and provenance/self-
  priming can make a "should retrieve" fixture return 0.** The type split must not change the serde image
  (existing `chain_hash` + at-rest JSONL). Capture a real row from the pre-split code (run it through
  `FileCorpus::submit`, read the file), hard-code it, and assert: deserializes + re-serializes
  byte-identically + `FileCorpus::open` re-verifies the chain + gate-passes. Gotcha: my first fixture set
  `retrieval_first:true, primed_from:[<its own plan id>]` — that is the CIRCULAR/self-primed case, so it
  legitimately contributes 0 confirmations and `query` returned empty. Regenerate the fixture with a
  de-novo provenance (`retrieval_first:false, primed_from:[]`) so it actually backs a mapping.

- **(2026-07-02) `.into()` that adapts across a same-turn field-type change trips clippy::useless_conversion
  in the interim step.** `StoredPlan::to_plan` does `title: self.title.clone().into()` — String→Prose once
  1b lands, but String→String (a no-op `.into()`) in the split-only step before it. Since each sub-step is
  committed and clippy-gated separately, write the split step with a direct assignment and switch to `.into()`
  in the Prose step, or clippy `-D warnings` fails the intermediate commit.

- **(2026-07-02) A "DONE + verified" claim is about a LOCAL tree until the pushed tip is rebuilt.** The
  session-end mirror captured the tracking files but not the code worktree: the Phase-0 keystone edit died
  with `/tmp/cec-leak`, origin got a non-compiling tip whose commit message claimed a CRITICAL leak class
  closed, and no PR existed so CI never looked. Rules: (1) ephemeral worktrees push (or at least
  `cargo check` the pushed sha in a fresh worktree) before session end; (2) open the PR immediately — an
  unreviewed "done" branch with no CI is where false claims survive; (3) when resuming any handoff, re-verify
  its central claims against origin before building on them.

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

- **2026-07-04 21:20 UTC** — **PR #17 MERGED (`main` @ `e16fd35`); leak Phase 3's 3b/3c BUILT on the
  restarted branch.** The migration bundle + blind-audit fixes + all 3 decision records are on main; the
  babysit trigger was deleted on merge. Then built the Layer-3 content gate: `tools/xtask` boundary tool
  (scan-content / allowlist-freeze / install-hooks), frozen `.boundary-allow.txt` (seeded from a real
  sweep; every entry sanctioned), content-gated pre-commit hook (gitleaks now warn-and-skip; CI is the
  backstop), CI `boundary` job, gitleaks seed/salt/row rules. Red-proven on planted violations; 245 tests.
  3a (type-aware dylint) filed in FOLLOWUPS as the remaining Layer-3 half; PromptPayload RE-SCOPED in
  FOLLOWUPS (Q2/D3 changed its premise — needs an owner call between strict-de-identified prompts vs
  explicit-but-raw channels; nothing blocks meanwhile). NOTE: the xtask poison sweep deliberately excludes
  the bare author name (public identity ≠ corpus leak; compound tokens all stay) — documented in the tool.

- **2026-07-04 20:35 UTC** — **Owner decided Q1 (separate keys — Q1 now FULLY decided) and D3 (the
  integration posture): the engine is an independent authenticated API; MyOwnMesh is transport only (no
  `myownmesh-core` link, daemon-gateway pattern); no MyOwnLLM for now.** Recorded in the RFC (new D3 +
  DECIDED/MOOT/DEFERRED/REFRAMED notes on Q1–Q5) + integration-doc supersession banner; the 2026-06-15
  "awaiting Chris" FOLLOWUPS item is CLOSED — nothing hard-blocks on Chris now (Q5's anchor moved into our
  own B4/corpus-service wire contract; his side just writes an AllMyStuff API client someday). Grounding:
  live review of MyOwnMesh v0.2.28 (generic RPC call/serve/call_stream, typed pub/sub, per-device ed25519
  roster identity, closed-network role tiers; its own GUI is a daemon CLIENT over local sockets — the
  exact pattern D3 adopts) and AllMyStuff v0.2.17 (remote desktop/shell/files over the daemon; headless
  `allmystuff serve`). Note: cross-tier `add_repo` (mrjeeves/*) is blocked in nathanfraske-scoped sessions
  — reviewed via the public GitHub pages instead. **PR #17: subscribed, CI 10/10 green at `72985c6`,
  hourly check-in armed.**

- **2026-07-04 20:05 UTC** — **Owner DECIDED RFC Q6** (agreed with the minimal-attested-unit
  recommendation): recorded in the RFC with a design wrinkle flagged (the attestation binds the provenance
  pin, so the minimized served row needs a provenance commitment — solve inside the B4 wire-type design).
  Control-C + Q6-filed FOLLOWUPS closed. **Q1 operator-half recommendation delivered (not yet decided):
  SEPARATE keys** — the 2026-07-03 decisions (central attestation authority; custodied judge key + F3
  registry) made authority keys ROLE keys (portable, slow rotation, tight custody) while a mesh `DeviceId`
  is a DEVICE key (per-machine, fast rotation, broad surface); unifying welds a long-lived role to a
  disposable device and makes one compromised laptop a corpus-truth-minting event. The RFC's old
  "lean: unified" predates those decisions. Also delivered the Chris-blockers rundown (Q2-Q5 all have safe
  engine-side defaults; they gate only the mesh-serving tier).

- **2026-07-04 19:40 UTC** — **Blind panel returned; all confirmed findings fixed (`8626f23`); 237
  green.** 3/3 auditors: the chain-v2 canonical encoding is CLEAN (independent concrete collision attempts
  all failed on the count/length guards — strong convergence). Real finds, verified against source then
  fixed: CRITICAL non-UTF-8 `CEC_FINGERPRINT_SALT` silently treated as unset (fail-open to the public
  salt) → startup refusal + cfg(unix) e2e; MEDIUM missing fault/config fingerprint domain separation →
  `domain:` line in the MAC message (fixture regen #3); HIGH silent cold-start at the serve boundary →
  one-line NOTICE (live-smoked) + `fingerprint_salt_is_configured()`. The §7 method earned its keep again:
  the NotUnicode fail-open was invisible to the sighted tests because they were written against the same
  `Err(_) => unset` premise. Two FOLLOWUPS filed (corpus-ingest salt-loader parity; strip-downgrade
  re-flag). Pushed; **PR #17 is open** (https://github.com/nathanfraske/cec-support-agent/pull/17) with the
  operator re-ingest + salt note in the body. Next: watch CI → merge → restart the branch from main.

- **2026-07-04 19:10 UTC** — **Migration bundle (items 4+6) built green in 3 commits + docs/tracking; §7
  blind panel launched.** Picked up the 03:15 baton on the designated branch
  `claude/workflow-model-optimization-e1y1sx`. F2 chain-v2 (`92df52d`), leak-C7 keyed fingerprint + salt
  custody (`e17f38f`), POST-body query (`90ff2c2`); 235 tests; every new guard proven red-on-revert
  (including the silent salt-ignore regression, the one that matters). Fixture regenerated twice (once per
  kernel) and fragment-split to keep the invariant hook's corpus-row backstop alive (pattern widened
  16→16-64 hex). Lesson recorded below (mtime/mv). Owner asked for a full project status report — delivered
  in-session; the standing/gates summary lives in this file's Current state + Pick up here.

- **2026-07-03 03:15 UTC** — **Execution-zone trio merged (PR #16); 4 owner decisions locked; B4 re-scoped.**
  Merged PR #16 (all 10 checks green, no review threads) via merge-commit (`44aaa88`) — items 1/2/3 now on
  `main`; restarted the branch; set up then, on the auto-merge cron being blocked by the safety classifier,
  replaced it with a watch-and-fix cron (`83c3af29`), which I deleted after merging. Collected 4 owner
  decisions via AskUserQuestion and recorded them: RFC Q7 DECIDED (ed25519 custodied judge key — owner
  diverged from my judge-on-target rec), RFC Q1 DECIDED-partial (volunteer = pure target, central authority
  attests), leak-C7 salt (per-deployment secret + cold-start default), migration (hard cutover). **Discovered
  B4 (item 5) is mis-scoped:** `HttpCorpus::query` serves attestation-less `FixMapping`s, so attested-read
  re-verification is RFC-Q6-gated + corpus-service-gated, not a small item → deferred to FOLLOWUPS. **Next:**
  the F2 + leak-C7 migration bundle (items 4+6), now unblocked by the decisions. **Lessons:** (1) the auto-mode
  classifier BLOCKS an autonomous cron that auto-merges an agent-authored PR without explicit review-auth —
  an interactive merge on the owner's explicit "merge on green" instruction is fine, but a cron that self-
  merges is not; use a watch-and-fix cron (fix CI + ping on green) and let a human merge. (2) Verify a
  planned item against the CODE before committing to its size — B4 looked like "add a verify call" but the
  served type carries nothing to verify; the plan inherited a stale assumption. (3) When editing a doc,
  match on text UNIQUE to the target — I replaced a section header with a duplicate `Q1` line because my
  old_string's tail (`## What's already built`) wasn't anchored to unique surrounding content; re-read and
  fixed.

- **2026-07-03 02:17 UTC** — **Lane ② item 3: execution audit-log skeleton landed on-branch.** New
  `crates/support-agent/src/audit.rs` (`ExecutionRecord` + `to_line` + `AuditSink`/`NullSink`), wired at the
  `record_outcome` funnel so every outcome (incl. declined-consent Withdrawn) emits one de-identified record
  built from the MINTED plan id (read back via `contribution.outcome().plan().id()`), the opaque run id, and
  the reused `serve::wire_label` token — no prose/tool-output/raw id can cross (closed field set). Injection
  seam added: `AppState.audit: Arc<dyn AuditSink>` (serve) defaulting to `NullSink`; CLI passes `&NullSink`.
  3 tests incl. a capturing-sink proof that one record per outcome carries the minted id and no title prose.
  Green (fmt/clippy -D/tests). Rung-2 (persistent sink, caller_key, CLI seam, refuse-path marker) → FOLLOWUPS.
  **Next:** item 1 (MCP-wrapper spec doc) → open PR-1 (items 1/2/3). **Lessons:** (1) a `\| tail` on
  `cargo fmt -- --check` MASKS fmt's non-zero exit (pipeline exit = tail's 0), so `&& echo OK` runs even when
  fmt is DIRTY — check fmt with `> /dev/null; echo $?`, never through a pipe. (2) a `pub fn` used only by
  tests in a BIN crate still trips `dead_code` under clippy `-D warnings` (bin crates don't treat `pub` as a
  public API) — mark it `#[allow(dead_code)]` with a "used by tests + <future caller>" note, the existing
  `signature_of` convention. (3) naming a param `audit` shadows the `audit` MODULE — reference the module as
  `crate::audit::` inside that fn.

- **2026-07-03 01:47 UTC** — **PR #15 merged; Lane ② started; item 2 (sandbox contract) landed on-branch.**
  Merged PR #15 (all 10 checks green, no review threads) via merge-commit (`ac14edf`), restarted the branch
  from the new `main`, retired babysitter cron `69d7ae77`. Owner picked "both, together" for Lane ②, so I
  wrote the plan (`scratchpad/lane2-implementation-plan.md`) and started with the safety pin: the
  `SandboxValidator` contract now normatively states "a sandbox LOWERS an escalation, never MINTS truth" (in
  `crates/swarm/src/lib.rs` trait + `ValidationReport` docs), pinned by a new red-on-revert test in
  `support-agent` that proves a clean sandbox apply + `None` re-collection yields `Verdict::Unverified` →
  `OutcomeLabel::EscalatedHumanUnresolved` (never resolved). Full workspace green. **Next:** item 3 (exec
  audit-log skeleton), item 1 (MCP-wrapper spec doc), then open PR-1 for the execution-zone trio; the
  hash-migration bundle (F2 + leak-C7) stays a separate final PR. **Lesson:** when writing an
  invariant-pinning test, do NOT couple it to a judge score you don't control (`heuristic_candidate`'s score
  could dip below the Reversible→VerifierConfirm 0.6 threshold and flake the assertion) — the escalation
  *lowering* is already proven in `panel`; the new test asserts only the sandbox≠verdict separation, which
  has no score dependency.

- **2026-07-03 01:22 UTC** — **Test-and-validation-fleet model designed (decision-ready, no code).** Scoped
  the two highest-risk runtime surfaces the owner asked to stand up — (a) the target-environment access MCP,
  (b) the sandbox test-harness MCP — via a sonnet ground-truth surface-map (read-only, every fact `file:line`)
  + an opus threat-model/design pass, cross-checked against each other. Landed `docs/test-validation-fleet-
  design.md`: cardinal rule (both surfaces WRAP the existing gates, never expose a raw tool; verb is gated
  `diagnose`/`execute` over `/v1/execute`); the T-1..T-7 execution-boundary threat model; the SandboxValidator
  "lowers-an-escalation, never raises-trust-without-a-signature" contract; **§3.1 the Windows-reproduction
  mechanism** (config_class = image key at `{release branch}×{hw/driver inventory}` granularity → a monthly
  update mints no new class; offline DISM injection so no box "downloads" an update; the VM-can't-synthesize-
  silicon boundary that splits sandbox=software-state from volunteers=hardware); volunteer machine as a
  de-identified execution target with no volunteer-id on the row (leak-C10 extended); greenlight-now vs
  infra-gated vs Chris-gated sequencing. Filed **RFC Q7** (plan-signing across the execution boundary:
  judge-on-target vs ed25519 custodied key; pairs with Q1). Named F4 (real re-collection) as the hard gate on
  the whole fleet's value. Docs-only; 210 tests unchanged. **Next: present + await owner on Q7/Q1 and whether
  to greenlight the three pure-engine items (MCP wrapper spec, SandboxValidator contract, exec audit-log
  skeleton).** **Lesson:** a design agent's "no Q6 exists" style claim can go stale against a doc that gained
  the item later — always cross-check a design doc's cross-references against the *current* file content (the
  surface-map caught exactly this: the cartography doc still says "no Q6 defined anywhere" but the RFC now has
  Q6). Filed as a FOLLOWUPS reconcile.

- **2026-07-03 01:01 UTC** — **Review panels (Tier 3) built; the addendum standup Tiers 0-3 are complete.**
  Merged PR #14 (projectops) to `main`, restarted the branch, then built `tools/projectops_panel.py` (PR
  #15): it runs the projectops checks and renders one self-contained, theme-aware HTML dashboard
  (verification / security-invariants / backlog / blind-audit; summary tiles; status pills + a severity
  stripe on failing rows; both light/dark themes via tokens; static snapshot since the CSP forbids a
  rendered page calling MCP). A live instance was rendered as a claude.ai Artifact for the owner. Dogfooding
  the panel surfaced + fixed a real `projectops verify` bug: a missing cargo SUBCOMMAND (`cargo deny` not
  installed) exits non-127, so it read `fail`; `verify` now treats "no such command" as `skipped` (matching
  gitleaks). **Standup status:** Tier 0 (provision.sh), Tier 1 (invariant guards + freshness), Tier 2
  (projectops server), Tier 3 (panels) all DONE. **Pick up here:** the remaining refinements (FOLLOWUPS) —
  a Stop verify-gate via `projectops verify --checks`, scheduled/Stop panel regen, and deeper `invariants`.
  ALSO NEW (owner question 2026-07-03): whether to stand up (a) the client-PC-access MCP the diagnosis
  agents drive and (b) the sandboxed-environment MCP test harness — both are the ENGINE's runtime surface,
  a different track from the agentic dev-tooling above; scoping pending.
- **2026-07-03 00:07 UTC** — **projectops server (Tier 2) built; PR #13 merged first.** Merged PR #13
  (addendum spec + Tier-1 guards, all green) to `main`, restarted the branch, then built the `projectops`
  keystone as a fresh PR #14: `tools/projectops.py` (pure-stdlib CLI — `verify` the cargo/gitleaks suite as
  structured JSON, `invariants` six fast git/grep security guards, `backlog` the tracking parse, `leak_scan`
  the de-id slice) + `tools/projectops_server.py` (a minimal MCP stdio server, raw JSON-RPC 2.0, NO
  third-party SDK — so it is self-contained and testable) + `.mcp.json`. Validated end-to-end: the server
  handshakes (initialize/tools-list/tools-call), `invariants` passes on the real tree AND provably bites on
  a re-added `source` oracle / rogue `/v1/attest` route / unsorted vocabulary; `backlog` parses; `verify
  --checks fmt` runs. **Pick up here:** the review panels (§4) are now unblocked (they render `projectops`
  JSON) — build the verification/backlog/invariants/blind-audit panels; wire a Stop verify-gate via
  `projectops verify --checks`; and deepen `projectops invariants` (a real no-raw-Serialize check + the full
  vocab/registry drift). This rides PR #14; merge when green. Lesson from the prior turn still applies: a
  content-matching guard must not contain the patterns it matches.
- **2026-07-02 23:56 UTC** — **AGENTIC ADDENDUM Tier-1 enforcement built (on PR #13's branch).** Turned the
  addendum's proposed guards into working hooks: `invariant-guard.sh` (PreToolUse — hard-blocks a
  corpus/weights/seed PATH write, the near-zero-false-positive signal; content-level oracle/Serialize checks
  deliberately left to the future `projectops invariants` tool, not a fragile grep-block), `invariant-check.sh`
  (PostToolUse — surfaces conflict markers / serialized-corpus-rows / seed-key blocks; made self-safe after it
  dogfood-flagged the addendum's own prose and its own source — see the new Lessons entry), `tracking-freshness.sh`
  (Stop — nudges if crates/ changed without a HANDOFFS/TODOS update; the full cargo suite stays out of Stop as
  impractical), and `ops/provision.sh` (Tier-0 activator: `core.hooksPath`, chmod, suite). All wired in
  `.claude/settings.json` (now PreToolUse+PostToolUse+Stop) and validated (block/allow/surface/self-reference).
  Addendum §2 updated to built-not-proposed. **Pick up here:** the `projectops` MCP server (§3) is the next
  keystone — it gives the panels and a Stop verify-gate their structured data, and is the proper structural
  home for the re-added-`source`/`Serialize`/route checks the PreToolUse guard intentionally skips. This rides
  PR #13 (now "spec + Tier-1 enforcement"); merge when green.
- **2026-07-02 22:42 UTC** — **PR #12 merged (all session work now on `main`); AGENTIC ADDENDUM authored +
  wired.** Merged PR #12 after resolving a `ci.yml` conflict (PR #4's `actions/checkout` bump vs the gitleaks
  OSS-binary rewrite — kept the OSS binary, took main's newer checkout SHA); the merge carried the
  full-stack audit fixes (escalation bypass, confirmation replay, registry-backup clobber, part_class/run_id
  validators, 3 LOW), the CI gitleaks fix, and the dependabot-deferral note. Branch restarted from the new
  main; babysitter crons retired. Then, per the owner's request, authored **`docs/AGENTIC_ADDENDUM.md`** — a
  cec-support-agent agentic-infrastructure spec modeled on `AGENTIC_ADDENDUM_1.md`: §1 the four tracking
  files + memory mirror, §2 the real hooks (SessionStart 4-script chain, `session-end.sh` durability push,
  the user-level Stop git-check) + the proposed PreToolUse invariant guard / Stop verify-gate, §3-§6 the
  `projectops`/panels surface + lifecycle, and **§7 the fully-blind audit** adapted to our crypto/de-id
  kernels with the frozen constants as "reserved values," grounded in this session's audit catching the
  escalation bypass + replay hole. Grounded in a 4-agent read-only ground-truth extract (`wf_44941c16-6d7`).
  Made it "reachable + hooked": referenced from AGENTS.md, and surfaced by a new `addendum-context.sh`
  SessionStart hook (wired in settings.json; validated JSON + 5 SessionStart hooks). **Pick up here:** the
  addendum's §2b/§2d proposals (a PreToolUse exfil/oracle guard; folding `verify` + a tracking-freshness
  check into the Stop gate) and §3 (`projectops` server) are not yet implemented — build them if the owner
  wants the mechanical backstop, not just the spec.
- **2026-07-02 18:54 UTC** — **Corpus cartography (leak-C10) threat model + non-mappability policy landed
  (docs/policy only — no `.rs` touched).** Owner-raised threat ("can a surface expose the internal corpus by
  mapping it out through trusted calls?") analyzed by a 2-agent check; produced `docs/corpus-cartography-
  threat.md` (honest limit §0, vectors V1-V7 §2, control set A-G §3, the NON-MAPPABILITY rule set §3b,
  accepted residuals §4, phased sequence §5). This session made the docs/policy/tracking match the code drop
  already committed (`4cf9d8f`, dropping `source` from `cec-diagnose/v1`): added **leak-C10** to
  `docs/corpus-leak-prevention.md` §1.2 (+ §3.1(4) cross-reference); copied the 7-rule non-mappability set
  into `AGENTS.md` as a sibling block to the §2.5 egress-sink checklist; corrected the wire-contract body in
  `docs/integration-rfc-for-chris.md` to `{plan_id, max_risk, actions[]}` + filed real question **Q6**
  (served-row provenance exposure, gated on B4); added the dated decision-log entry to
  `docs/api-extension-design.md` §5. Filed 6 deferred controls to `FOLLOWUPS.md` (D-remainder
  latency/slate equalization, A per-identity budget, B per-identity audit log, E keyed HMAC, C B4
  provenance-minimization, Q6 pointer), each attributed to the threat doc. Non-mappability is now the
  **fourth** enforced corpus property (admissibility, authenticity, access, non-mappability). **Next:**
  the deferred controls above — E (keyed HMAC) is greenlightable now / pull it forward; A/B full form and C
  are gated on B4/E3 per the threat doc's §5 sequence.

- **2026-07-02 21:30 UTC** — **Corpus leak-prevention Phase 2 built (the C4/C5 read-side + dictionary
  stops).** On `claude/repo-scope-work-plan-h93qx5` (started == origin/main `86e24cf`, Phase 1 already in),
  3 green sub-steps, committed + proven, NOT pushed: **C5** (`a0818bc`) frozen `STOP_CODE_NAMES`/`MODULE_NAMES`
  dictionaries + closed-grammar `is_symptom_token` replace the shape heuristics; `deid::symptom` wired to the
  grammar (admits `event_41` — the Phase-1 blocker). **C4** (`a759afd`) `#[serde(try_from)]` validating
  deserialization on `StoredSymptom`/`StoredAction`/`StoredPlanId`/`common::Symptom` — an out-of-vocab action,
  inadmissible id, or non-grammar symptom fails to deserialize at `HttpCorpus::query` (transport/admission
  split → `ServedPlanInadmissible`) and `FileCorpus::open` (Storage error); symptom mint wired into the 1f
  gate (`GateError::SymptomNotDeIdentified`); adversary-seeded read-path poison harness. **2c** (doc commit)
  scoped honestly — the `Value` fields have no serialize sink post-Phase-1, documented not re-typed
  (typing = C2/Phase-4). Wire byte-identical (canned fixture + chain stable); envelope pins green. 198→205
  tests, clippy/fmt clean, CLI e2e smoke green (real binary keeps `whea_uncorrectable_error`/`event_41`/
  `explorer.exe`, drops asset tag `RIG_NATHAN_DESK`). Both a dictionary case and the read-side symptom guard
  proven red-on-revert. **Phases 0–2 of the methodology are complete;** Phases 3–4 remain (FOLLOWUPS/
  accepted-risk). Design decisions (see lessons): symptom validated in-place on `common::Symptom` (no
  `StoredVerification` — a cycle); test symptom fixtures swept `boot_loop`→`event_41`.

- **2026-07-02 19:15 UTC** — **Corpus leak-prevention Phase 1 built (the C1/C3 type barrier).** On
  `claude/repo-scope-work-plan-h93qx5` (started == origin/main `a31198e`), 4 green sub-steps, committed +
  proven, NOT pushed: **1a** type split (`a347878`) — `crates/corpus-client/src/stored.rs` stored payload
  types are the only serde corpus types; `de_identify_plan → StoredPlan`; `Serialize` removed from all raw
  domain types (Plan/PlanStep/Candidate/Outcome/DiagnosticEvent/StepResult/ExecutionResult/ToolOutcome/
  AgentRun/AgentStep/SignedPlan); `Contribution` fields `pub(crate)` + accessors + rehydration. **1b/1d**
  (`3790dbd`) — `common::Prose` (no Serialize/Display, redacting Debug) for the 5 prose leaves; containers
  keep an auto-sealed derived Debug. **1f** (`22ec564`) — write gate re-mints the stored plan
  (`GateError::RowNotDeIdentified`), red-on-revert proven. **trybuild** (`9a9cd5b`) — 3 compile-fail cases
  + runtime Debug-no-leak, red-on-revert proven. Wire shape byte-identical (canned pre-split row fixture);
  `cec-diagnose/v1`+`cec-execute/v1` unchanged. 189→198 tests, clippy/fmt clean, CLI e2e smoke green.
  Design decisions (justified, see lessons): stored types live in **corpus-client** not the `deid` crate
  (OutcomeLabel/SignOff live there); 1d achieved via **Prose's redacting Debug + derived container Debug**
  (not per-struct manual impls); `Contribution` fields **pub(crate)** (external struct-literal still fails);
  symptoms kept **structurally typed** (strict mint deferred to Phase 2 per the `<prefix>_<digits>` gotcha).
  **Next: Phase 2** (read-side `from_served`/`try_from` + frozen dictionaries + ban `serde_json::Value`).

- **2026-07-02 16:20 UTC** — **Wave 2: PRs #5/#6 merged; the engine's API face is built (B3/B4/H4).**
  Both PRs went red on a NEW upstream RustSec advisory (anyhow 1.0.102 downcast_mut UB) — lockfile-bumped on
  both branches, green, merged (owner's go-ahead). Then on the restarted work branch: **B4** `HttpCorpus::query`
  read-side re-validation (a served plan must equal its own de-identified image; fails closed;
  `GateError::ServedPlanInadmissible`; attestation-on-the-wire residual → FOLLOWUPS); **H4** exact toolchain pin
  (1.96.1, lockstep note in ci.yml) + dependabot cargo ecosystem; **B3** `cec-support-agent serve` — loopback
  HTTP API (`/v1/health`, `/v1/diagnose` → `cec-diagnose/v1` + additive `session_id`, `/v1/execute` two-phase
  consent → **`cec-execute/v1`** with pinned label/verdict values), one-shot TTL'd sessions, escalation
  re-checked server-side, declined consent recorded as Withdrawn, `plan_id` app-side retry, axum 0.8.
  189 tests/clippy/fmt clean + live e2e smoke (health → diagnose → 409 under-escalated → honest escalated
  execute). **Next:** leak-prevention Phases 1–2 (include the serve module in the egress-sink inventory), then
  P1' (AllMyStuff API client) once Chris weighs in.
- **2026-07-02 15:50 UTC** — **Stalled handoff resumed: repo-wide consolidation + first wave executed.**
  Scoped every branch/PR/doc (9-agent analysis + direct verification) → `docs/consolidated-work-plan.md`.
  Merged PR #2 (`2d9620a`) then PR #3 (`3b269f8`). Found the pushed leak-prevention tip `cf95d1c` did NOT
  compile (9 errors; `schema.rs` keystone edit never committed — lost with the ephemeral worktree; "closes
  C1" was false on origin): rebased, restored the edit in-commit (`0855884`), re-verified 180 tests/clippy/
  fmt, opened **PR #5**. Recorded the owner's **engine-as-API** supersession of RFC D1 in both integration
  docs. Pinned the `cec-diagnose/v1` enum wire grammar (snake_case, exhaustive matches, pinning test,
  `part_class` sibling field) while the envelope has zero consumers (`ec1e388`). Rescued the final-session
  tracking files from `ops/agent-handoff` onto a real branch; de-staled checklist/SECURITY/negative-results.
  Housekeeping PR opened from `claude/repo-scope-work-plan-h93qx5`. **Next:** B3/B4 (serve API v1 +
  `HttpCorpus` read hardening), then leak Phases 1–2.
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
