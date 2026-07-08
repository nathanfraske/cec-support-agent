# TODOS

The LIVE checklist of everything the agent is **getting done**, in checkbox form, checked off as work
completes. **APPEND-ONLY** — items are never deleted; completed / obsolete ones stay as timestamped
history with a tombstone. Times are **UTC**.

Format:
- `- [ ] [added YYYY-MM-DD HH:MM UTC] <task>` — active / in progress
- `- [x] [added YYYY-MM-DD HH:MM UTC · done YYYY-MM-DD HH:MM UTC] <task>` — completed (left in place)
- `- [~] [added YYYY-MM-DD HH:MM UTC · obsolete YYYY-MM-DD HH:MM UTC → <tombstone>] <task>` — obsolete,
  with a tombstone pointing where it went (e.g. `FOLLOWUPS.md "<entry>"`, `PR #N`, another line)

Conventions:
- Every entry carries the exact date **and time** it was added, and the time it was checked off.
- Never delete a line. Flip `- [ ]` to `- [x]` (done) or `- [~]` (obsolete) and append the tombstone.
- Distinct from `FOLLOWUPS.md` (deferred / not-now backlog) and `HANDOFFS.md` (resume state).

## Active

### Session 2026-06-14 — evidence-integrity checklist + agent infra hardening

- [x] [added 2026-06-14 19:45 UTC · done 2026-06-14 19:46 UTC] Locate the real project — clone `cec-support-agent` into the `CEC_AutoDiagnoser` working dir (the GitHub `CEC_AutoDiagnoser` repo is empty; work lives in `cec-support-agent`)
- [x] [added 2026-06-14 19:48 UTC · done 2026-06-14 20:00 UTC] Recon fan-out (workflow `autodiagnoser-recon`, 5 agents): mapped current pipeline + 11 evidence-integrity points + 8 checklist hook points + 11 gaps; CEC-Platform EI-01..08 + research paper-track PP-01..13; the inverted-ground-truth-corpus model; the WSL-ephemeral state policy (parity spec); and the current local-agent infra (cec-llm-broker :8080, hybrid WSL-docker + Windows-native seats). Findings saved to `.claude/recon/*.json`
- [x] [added 2026-06-14 19:48 UTC · done 2026-06-14 19:51 UTC] FOLLOWUPS.md SessionStart hook (date+time, append-only with tombstones — stricter than CEC-Platform) + seed file
- [x] [added 2026-06-14 19:48 UTC · done 2026-06-14 19:51 UTC] TODOS.md SessionStart hook (checklist, append-only with tombstones) + seed file
- [x] [added 2026-06-14 19:48 UTC · done 2026-06-14 19:51 UTC] HANDOFFS.md SessionStart hook (baton: current state + pick-up-here + lessons; injects contents at session start) + seed file
- [x] [added 2026-06-14 19:48 UTC · done 2026-06-14 19:57 UTC] WSL-ephemeral state parity: session-start + session-end durability hooks (durable in-tree memory mirror + off-tree handoff branch push), adapted to this git repo — VERIFIED end-to-end: Stop hook pushed `ops/agent-handoff` to the remote with `main` untouched; hardened against the no-git-identity post-wipe gotcha
- [x] [added 2026-06-14 19:48 UTC · done 2026-06-14 19:57 UTC] `.claude/settings.json` wiring all SessionStart (session-start + followups + todos + handoffs) + Stop (session-end) hooks — valid JSON, all 5 hooks present + executable
- [x] [added 2026-06-14 19:48 UTC · done 2026-06-14 20:08 UTC] Design panel (4 lenses: runtime-code / research-track / governance / red-team, workflow `autodiagnoser-evidence-checklist`) + write the **evidence-integrity & research checklist** adapted to the inverted-ground-truth-corpus approach → `docs/evidence-integrity-and-research-checklist.md` (the engine's truth is accreted from signed-off outcomes; sign-off gate = truth-admission boundary; unified `ensure_evidence_integrity()` design + runnable checklist + attack→defense table)
- [x] [added 2026-06-14 19:48 UTC · done 2026-06-14 20:08 UTC] Document the **current local-agent infrastructure** (changed) → `docs/local-agent-infrastructure.md` (cec-llm-broker :8080 front door, hybrid WSL-docker + Windows-native seats, what changed)
- [x] [added 2026-06-14 20:00 UTC · done 2026-06-14 20:08 UTC] Document the **WSL-ephemeral state policy** as implemented → `docs/wsl-ephemeral-state-policy.md`
- [x] [added 2026-06-14 20:09 UTC · done 2026-06-14 20:10 UTC] Copy the checklist's deferred GAP items + WSL optional-hardening into `FOLLOWUPS.md` (14 items, dated/timed)
- [x] [added 2026-06-14 20:09 UTC · done 2026-06-14 20:11 UTC] Scaffold the `docs/research/` tree (README + negative-results [populated] + claims/prereg [disciplined scaffolds] + instrumentation-inventory), commit-ordering discipline intact
- [x] [added 2026-06-14 20:11 UTC · done 2026-06-14 20:11 UTC] Add agent-ops/durability/evidence-integrity pointer section to `AGENTS.md`
- [x] [added 2026-06-14 19:48 UTC · done 2026-06-14 20:12 UTC] Final verification: all 5 hooks executable + syntax-clean + valid JSON, settings.json valid, all 14 required files present, infra doc well-formed
- [x] [added 2026-06-14 20:12 UTC · done 2026-06-14 20:15 UTC] Update HANDOFFS.md final state; commit everything on branch `feat/agent-ops-evidence-integrity` (commit `69d659d`; set repo-local git identity nathanfraske@cec.direct — the same no-identity gotcha the WSL doc warns about)
- [x] [added 2026-06-14 20:15 UTC · done 2026-06-14 20:19 UTC] **Push `feat/agent-ops-evidence-integrity` to origin** — pushed `c508970` (owner approved); work now durable on the remote

### Engine work — evidence-integrity gaps (owner: implement, 2026-06-14)

- [x] [added 2026-06-14 20:19 UTC · done 2026-06-14 20:24 UTC] Read the corpus-client / provenance / panel / verify / main.rs code to ground the first increment design
- [x] [added 2026-06-14 20:19 UTC · done 2026-06-14 20:40 UTC] Increment 1 — the structured evidence-integrity gate foundation: `ensure_evidence_integrity` with a structured `GateError` (Unconfirmed / ResolvedWithoutPass / LabelVerdictMismatch / DestructiveFixNeedsHuman); bound the verification verdict into the row (`common::Verification` on `Outcome`, `Verdict::to_verification()`); destructive-resolved-fix-needs-human enforced IN corpus-client (not just the CLI); resolved label must carry a matching passing verdict. +6 gate tests; 125 tests green; fmt + clippy -D warnings clean; live CLI smoke OK. Installed the WSL Rust toolchain (none was present) for the build/test loop.
- [x] [added 2026-06-14 20:40 UTC · done 2026-06-14 20:41 UTC] Extend `SECURITY.md` invariant list to name the strengthened evidence-integrity gate
- [x] [added 2026-06-14 20:45 UTC · done 2026-06-14 21:05 UTC] Increment 2 — **MH-1 keystone: ed25519 sign-off attestation** (owner chose asymmetric). `provenance::SignOffAuthority`/`SignOffPublicKey`/`SignOffSignature` (engine holds only the public key); `Contribution.attestation` + `attested_by()` + canonical `attestation_message()` over (signature, plan, label, sign_off, config_class); gate `ensure_attested()` + `GateError::AttestationMissing/Invalid`; stores gain `.with_authority(pubkey)` and enforce attestation when configured (cold start unchanged). +12 tests incl. the forgery test (self-asserted HumanConfirmed refused). 136 tests green; fmt + clippy clean; ed25519-dalek dep tree license-clean for cargo-deny; CLI smoke OK.

### Full engine implementation sweep (owner: "continue down the list until full, then audit + fix, then present", 2026-06-14 20:49 UTC)

- [x] [added 2026-06-14 20:50 UTC · done 2026-06-14 21:20 UTC] Increment 3 — MH-2 remainder (`VerificationClass` moved to common + bound on the row via `Verification.class`; `RowProvenance` {run_id, retrieval_first, primed_from} on the row, populated in main.rs) + EI-03/A5 (independent-confirmation guard keyed on run_id; self-primed rows excluded; provenance bound into the attestation so a run_id can't be forged to inflate counts). +5 tests (140 total); gates green.
- [x] [added 2026-06-14 20:50 UTC · done 2026-06-14 21:35 UTC] Increment 4 — MH-4/MH-8/EI-06: `RowIntegrity` sha256 hash-chain on `FileCorpus` (attached on write, full chain re-verified on open — a hand-edited/reordered/removed row is a Storage error); owner-only `.with_revoked(plan_ids)` (revoked plans never offered); `Reopened` events net against confirmations (a recurred fix is demoted; T-104). +4 tests (144 total); gates green. Residual: tail-truncation needs an external anchor (FOLLOWUPS).
- [x] [added 2026-06-14 20:50 UTC · done 2026-06-14 21:40 UTC] Increment 5 — Canonicalization: provenance `canonical()` now a deterministic, serde-independent encoder (explicit field names + length-prefixed free text), so signatures don't depend on struct field order / the JSON serializer. Dropped the now-unused serde_json dep from provenance. +1 test, title/desc tamper assertions added (145 total); gates green.
- [x] [added 2026-06-14 20:50 UTC · done 2026-06-14 21:50 UTC] Increment 6 — MH-5 core: `Dispatcher::reconcile_risk` raises any step whose model-claimed risk under-states its registered tool's real risk (never lowers), wired into main.rs before judge/consent/execute so a mislabeled state-changing action can't slip the consent gate. +2 tests (147 total); gates green. Residual (FOLLOWUPS): inference-channel provenance (cert pinning / endpoint+model attestation) and de-id of model prose at generation time.
- [x] [added 2026-06-14 20:50 UTC · done 2026-06-14 22:00 UTC] Increment 7 — MH-3/NR-1: `verify_outcome` now takes an `Option` post-signature; `None` (no live re-collection — the bootstrap echo) yields the new `Verdict::Unverified` (→ `EscalatedHumanUnresolved`), so a run that never re-observed the machine escalates instead of being recorded resolved. `recollect_post_signature()` is the Windows wiring point (returns None in the bootstrap). +1 test (148 total); gates green; smoke shows the honest NR-1 escalation.
- [x] [added 2026-06-14 20:50 UTC · done 2026-06-14 22:05 UTC] Increment 8 — MH-6/A7 (PARTIAL, honest): `host_config_class` now derives from a `host_inventory()` extension point (cross-platform os/arch/family). The substantive Windows-CIM hardware/driver enrichment is the documented extension point but needs a Windows host to build+verify `cfg(windows)` code — kept in FOLLOWUPS rather than ship unverifiable code. The config_class is already attestation-bound (Increment 1), so it's tamper-evident regardless of derivation. Gates green.
- [x] [added 2026-06-14 20:50 UTC · done 2026-06-14 22:15 UTC] Increment 9 — MH-1 operator wiring: `gen-signoff-key` verb; `CEC_SIGNOFF_PUBKEY` configures the store to ENFORCE attestation; `CEC_SIGNOFF_SEED` makes the run self-attest (single-operator); set-but-invalid key = hard error (no silent unprotected run); enforce-without-seed refuses the write. Help/ENVIRONMENT updated. Verified end-to-end (keygen→enforce→attest admits; enforce-only refuses; bad key errors). Gates green (148 tests).
- [x] [added 2026-06-14 20:50 UTC · done 2026-06-14 22:25 UTC] Increment 10 — Sandbox-validation evidence: `sandbox_validated_for(Option<&dyn SandboxValidator>, winner)` validates the JUDGE WINNER through an optional disposable-VM validator and feeds the REAL result into `required_escalation` (clean apply = positive evidence; dirty/failed = stays unvalidated→escalate). CLI wires `None` (no VM backend) → conservative default unchanged. +1 test with a fake validator (149 total). Real VM backend = infrastructure, FOLLOWUPS. Gates green.
- [x] [added 2026-06-14 20:50 UTC · done 2026-06-14 22:55 UTC] AUDIT: adversarial multi-agent review of the full diff → fix findings — re-launched the `autodiagnoser-engine-audit` workflow `wf_5c1c16b9-613` (the previous agent's run never persisted results / no live task remained). 23 agents, ~1M tokens: 18 findings verified → **14 confirmed, 0 uncertain, 4 refuted**. Full detail in `.claude/audit/confirmed-findings.txt`. Collapse to 7 distinct defects (A–H below).
- [x] [added 2026-06-14 22:55 UTC · done 2026-06-14 23:10 UTC] FIX A (CRITICAL, C4/C5/C6): at-rest rows re-admitted on the keyless hash chain alone — `FileCorpus::with_authority` now returns `Result` and re-runs `admit` (incl. `ensure_attested`) over every loaded row, failing closed; main.rs wires `with_authority(...)?`. Independently verified CLOSED (no false-refusal of an authority-accreted corpus).
- [x] [added 2026-06-14 22:55 UTC · done 2026-06-14 23:10 UTC] FIX B (HIGH, C1): `attestation_message` field-injection — length-prefix every attacker-controlled value + count-frame every repeated section (mirror `provenance::canonical`), domain tag v2→v3. Adversarial re-collision attempt failed → CLOSED.
- [x] [added 2026-06-14 22:55 UTC · done 2026-06-14 23:10 UTC] FIX C (HIGH, C7/C14): reopen demotion run-dedup — `reopened: u32` → `reopens: HashSet` keyed by `confirmation_key` (symmetric with confirmations, excludes circular). CLOSED.
- [x] [added 2026-06-14 22:55 UTC · done 2026-06-14 23:10 UTC] FIX D (MED, C2): bind the `ConfigClass` variant discriminant (`class:bom`/`class:hash`) before the key. CLOSED.
- [x] [added 2026-06-14 22:55 UTC · done 2026-06-14 23:10 UTC] FIX E (MED, C3): bind `outcome.verification` (result+class+recurring) into the attestation. CLOSED.
- [x] [added 2026-06-14 22:55 UTC · done 2026-06-14 23:10 UTC] FIX F (LOW, C9): `CEC_SIGNOFF_SEED` without `PUBKEY` — derive the enforcing pubkey from the seed. Verified live (banner now shows derived enforcement). CLOSED.
- [x] [added 2026-06-14 22:55 UTC · done 2026-06-14 23:10 UTC] FIX G (LOW, C8): `chain_hash` versioned domain prefix `cec-corpus-chain-v1`.
- [x] [added 2026-06-14 22:55 UTC · done 2026-06-14 23:10 UTC] FIX H (tests, C10–C13 + regressions A–E): +11 tests (gate: provisional-over-pass mismatch, Unverified/OffMachine-not-resolved; store: variant-bind, verdict-bind, attested-reopen, cold-start-refused, rechained-forge-refused, replayed-reopen, HttpCorpus gate×2, mixed-chain). 159 tests green, clippy/fmt clean.
- [x] [added 2026-06-14 22:55 UTC · done 2026-06-14 23:12 UTC] Reconcile FOLLOWUPS.md — appended the 3 deeper residuals the fixes leave open (keyless-chain key/anchor, chain_hash canonical encoding, rotation×at-rest-readmission interaction)
- [x] [added 2026-06-14 22:55 UTC · done 2026-06-14 23:12 UTC] Independent adversarial verification of the fix diff (2 parallel reviewers: crypto-encoding lens + gate/store lens) → all 7 fixes CLOSED, no regression, no new bug
- [x] [added 2026-06-14 20:50 UTC · done 2026-06-14 23:25 UTC] Present for review — committed `11f0609`, pushed `feat/agent-ops-evidence-integrity`, opened **PR #2** (owner approved push+PR) → https://github.com/nathanfraske/cec-support-agent/pull/2

### Private corpus: structure + ground-truth YAML format (owner, 2026-06-14 22:25 UTC)

Decisions: off-tree separate private git repo at `/mnt/e/cec-corpus-private`; YAML-authored fix flows compiled
to de-identified + ed25519-attested + gate-validated JSONL; BUILD NOW = structure + format spec + templates +
no-leak rails on both repos; DEFER (as precise wiring TODOs) = the Rust ingest compiler, query/verify tools,
the corpus service.

- [x] [added 2026-06-14 22:30 UTC · done 2026-06-14 22:50 UTC] Design panel (workflow `private-corpus-design` `wf_b61fe974-59a`): 4 lenses + synthesis → canonical design (structure, YAML spec, rails, W0–W9 wiring). Provenance in the private repo `.design/`.
- [x] [added 2026-06-14 22:30 UTC · done 2026-06-14 23:00 UTC] Create the off-tree private repo `/mnt/e/cec-corpus-private` (git init, repo-local identity) with README + BOUNDARY + .gitignore + .githooks/pre-commit + the full directory tree; committed (`4520667`)
- [x] [added 2026-06-14 22:30 UTC · done 2026-06-14 23:10 UTC] Write the YAML fix-flow format SPEC `spec/fix-flow.schema.md` (field-by-field; authored/derived/forbidden table; gate-coupling rules; 12 validation rules) + the machine lint `spec/fix-flow.schema.json` + `spec/vocabulary.yaml` (faithful to the real extract.rs)
- [x] [added 2026-06-14 22:30 UTC · done 2026-06-14 23:10 UTC] Write 4 ready-to-fill YAML templates + a worked example — de-identified vocabulary only; all PASS the JSON-Schema lint, which REJECTS every inadmissible flow (unconfirmed, label/verdict mismatch, destructive-without-human, verdict-on-hard-negative, forbidden derived keys, IPv4 bom)
- [x] [added 2026-06-14 22:30 UTC · done 2026-06-14 23:20 UTC] Harden the PUBLIC repo no-leak rails (.gitignore + scripts/githooks/pre-commit broadened) + create `BOUNDARY.md`; rails verified to bite (ignore + hook grep) and to allow source
- [x] [added 2026-06-14 22:30 UTC · done 2026-06-14 23:35 UTC] Adversarial verification pass (2 parallel auditors): no-leak — no corpus data/keys in either tree or history, one-way coupling holds; format↔gate — COMPLETE/CORRECT (all enums match, worked example admittable). Fixed all actionable findings (reopened⇒provenance, *.env/*.ndjson/.yml ignores, IPv4 bom, spec honesty on the bom residual, W0 chmod-no-op)
- [x] [added 2026-06-14 22:30 UTC · done 2026-06-14 23:35 UTC] Record the deferred ingest-pipeline wiring plan (W0–W9, ordered, with acceptance checks) in `/mnt/e/cec-corpus-private/WIRING.md` + public FOLLOWUPS pointers (secrets exposure, guard activation, private remote, corpus-ingest, ignore residual)
- [x] [added 2026-06-14 23:35 UTC · done 2026-06-14 23:45 UTC] Present: public no-leak rails (`.gitignore`/pre-commit/`BOUNDARY.md`) + tracking committed `920e22a` and PUSHED onto **PR #2** (owner chose to extend PR #2). Private repo committed locally at `c636168`; its remote is deferred (WIRING W2).

### Private corpus: ingest pipeline W4–W7 (owner: "build it now", seed custody = age encryption-at-rest, 2026-06-15 00:13 UTC)

- [x] [added 2026-06-15 00:20 UTC · done 2026-06-15 00:40 UTC] Build `corpus-ingest` (private repo `crates/corpus-ingest`, pinned git-dep on the engine `11f0609`): `keygen` (ed25519 authority, seed AGE-encrypted at rest via `CEC_SEED_PASSPHRASE`), `compile` (de_identify → from_symptoms → with_provenance → attested_by LAST → gate dry-run → FileCorpus rebuilt-from-empty + hash chain; symptom/action/coupling validation), `verify` (chain + per-row re-admission + tail anchor). Committed `b34b916`.
- [x] [added 2026-06-15 00:20 UTC · done 2026-06-15 00:40 UTC] Verify end-to-end: keygen→compile→verify on the worked example (attested, chained, zero identity strings); 4 negative tests (tamper, destructive+verifier, non-vocab symptom, wrong passphrase) all reject; the ENGINE retrieves the compiled row **retrieval-first** (`CorpusPrimed`, 1 confirmation) — the full author→compile→attest→retrieve loop proven.
- [x] [added 2026-06-15 00:40 UTC · done 2026-06-15 00:50 UTC] Adversarial code review of `corpus-ingest` → found 1 CRITICAL (a spaced multi-token symptom could masquerade as a module name and leak identity into the signature; no plan-de-id backstop). FIXED: enforce the extractor's `[a-z0-9._]` single-token charset + added the crate's first 4 tests. Re-verified the leak flow is rejected. Committed `400351d`.
- [x] [added 2026-06-15 01:01 UTC · done 2026-06-15 01:15 UTC] Validation gate (entries-independent, owner: "go ahead"): `corpus-ingest check` (seedless full admissibility + de-id validation; `flow::compile` split into `validate`+`compile`); `make check`; CI merge-gate `.github/workflows/validate.yml` (no secret → gates untrusted branches; bot pushes, can't merge); local pre-commit best-effort gate. Verified: worked example valid no-seed, inadmissible/leaky flow rejected. Private `5c5d15c`.
- [x] [added 2026-06-15 01:01 UTC · done 2026-06-15 01:15 UTC] Paper-ready checklist tie-in: `docs/evidence-integrity-and-research-checklist.md` **A10** — curated ground truth is admissibility/de-id-gated reproducibly (seedless `corpus-ingest check` + CI) → every paper-backing entry is admissible + de-identified by construction. Public `271db03` (local; push to PR #2 pending owner OK).
- [ ] [added 2026-06-15 00:50 UTC] Remaining (operator/deferred): W0 run `make keygen` (real passphrase); W1 install gitleaks + activate hooks + enable branch protection on the private remote (requires the validate-corpus check + a review); W2 private remote; W8 the HTTP service; W9 key rotation. Plus the LOW/accepted `/mnt/e/secrets` perms note.

### MyOwn-family integration plan (owner, 2026-06-15 01:01 UTC)

Wire cec-support-agent (AGPL engine) cleanly into the MyOwn family: **AllMyStuff** (MIT device-inventory +
mesh-wiring "brain"), **MyOwnMesh** (MIT private mesh: identity/RPC/governance), **MyOwnLLM** (local inference).

- [x] [added 2026-06-15 01:01 UTC · done 2026-06-15 01:25 UTC → folded into the design workflow below, which cloned + mapped both repos] Recon both repos' real APIs (AllMyStuff `allmystuff-inventory`/`-bridge`; MyOwnMesh `myownmesh-core` identity/rpc/governance) + the engine's trait seams + the AGPL↔MIT boundary
- [x] [added 2026-06-15 01:01 UTC · done 2026-06-15 01:25 UTC] Design workflow (`myown-integration-design` `wf_4462fe7a-a37`, 8 agents): mapped the real APIs (cloned both repos) + synthesized the **integration plan** → `docs/integration-myown-family.md`. Verified the 3 load-bearing claims (AllMyStuff's myownmesh Tauri-sidecar pattern is real → reuse for the engine; `allmystuff-protocol` is serde-only → license firewall; `HttpCorpus::query` is unverified → a real finding). Plan: process boundary (no link) keeps AllMyStuff MIT; P0–P4 phased; seams = inventory→config_class, sidecar brain, corpus-over-mesh, identity unification, inference→MyOwnLLM.
- [x] [added 2026-06-15 01:01 UTC · done 2026-06-15 01:25 UTC] Surfaced engine finding → FOLLOWUPS: `HttpCorpus::query` returns server rows unverified (no admit/attestation on the read path); MeshCorpus must re-verify (P3).
- [x] [added 2026-06-15 01:25 UTC · done 2026-06-15 01:30 UTC → owner greenlit: single-shot CLI, versioning = agent's call, the rest → an RFC for Chris] Present the integration plan + 7 open questions for the owner's call; decide P0 start (the engine `--json`/`--inventory-keys` seams are the dependency-free first step)

### MyOwn integration P0 — implement (owner greenlit single-shot, 2026-06-15 01:30 UTC)

- [x] [added 2026-06-15 01:30 UTC · done 2026-06-15 01:45 UTC] P0 implementation: `common/src/inventory.rs` — `InventoryProvider` trait + `CoarseHostInventory` (today's os/arch/family default, byte-identical cold start) + `ExternalInventory`; CLI `--inventory-keys <file|->` (external identity-free config keys → honest `config_class`, closes A7/MH-6) and `--json` (the `cec-diagnose/v1` envelope). De-id regression tests on the inventory path. Builds + 165 tests green; clippy clean.
- [x] [added 2026-06-15 01:45 UTC · done 2026-06-15 02:00 UTC] **Wire-contract correctness:** under `--json`, route the human trace to **stderr** so **stdout is exactly one `cec-diagnose/v1` JSON line** (robust under `--sign-off`, not "parse the last line"). Local `human!`/`hprint!` macros in `run()`. Smoke-verified: stdout = 1 valid JSON line, trace on stderr, non-json mode unchanged.
- [x] [added 2026-06-15 01:45 UTC · done 2026-06-15 01:50 UTC] Versioning policy (owner left to agent): `cec-diagnose/v1` — additive-only within a major (consumers ignore unknown fields); a breaking change (remove/rename/retype) bumps the major and the consumer errors on an unknown one. Spec'd in the RFC (D2) + integration doc.
- [x] [added 2026-06-15 01:45 UTC · done 2026-06-15 01:55 UTC] RFC for Chris → `docs/integration-rfc-for-chris.md` (the frame; D1 single-shot + D2 versioning decided; Q1–Q5 open for Chris; the wire contract AllMyStuff codes against; P0 = built). Integration doc P0 section updated to **DONE** with the verified accept-criteria.
- [x] [added 2026-06-15 01:55 UTC · done 2026-06-15 02:00 UTC] **Found + fixed: PR #2 is RED on CI** — `11f0609` shipped 4 rustfmt-1.9 violations in `corpus-client/{schema.rs,store.rs}`; CI runs `fmt --all --check`. Fixed in-tree (isolated as a portable fmt-only commit for cherry-pick onto PR #2). Whole branch now fmt CLEAN. The push to make PR #2 green is owner-gated → FOLLOWUPS.
- [x] [added 2026-06-15 02:00 UTC · done 2026-06-15 02:10 UTC] Commit P0 + RFC + docs + fmt fix locally (durable); present to owner; await the push-routing call (PR #2 green) + Chris's Q1–Q5 — owner chose "push both": fmt fix + 2 doc commits fast-forwarded onto `feat/agent-ops-evidence-integrity` (`920a..538cd43`, PR #2 CI re-running), P0 pushed and opened as **stacked PR #3** (base = PR #2's branch). Commits `538cd43` (fmt), `d61b962` (P0).
- [x] [added 2026-06-15 02:10 UTC · done 2026-06-15 02:15 UTC] Confirm PR #2 CI is GREEN after the fmt fix (`gh pr checks 2`) — all `check` jobs (ubuntu/macos/windows) + audit pass; only the pre-existing `secrets` job red (now under triage). PR #3 likewise.

### Triage: the failing `secrets`/gitleaks CI job (owner, 2026-06-15 02:15 UTC)

- [x] [added 2026-06-15 02:15 UTC · done 2026-06-15 02:18 UTC] Root-cause the `secrets` job failure — CONFIRMED via the job logs: gitleaks-action@v2 requires `GITHUB_TOKEN` for `pull_request`-event scans (breaking change); the workflow's step has none, so the PR-event run fails fast (~4s) while the push-event run passes. NOT a license issue (personal repo), NOT a regression from P0.
- [x] [added 2026-06-15 02:15 UTC · done 2026-06-15 02:18 UTC] Rule out a real leak — ran gitleaks 8.24.3 locally over the FULL git history (36 commits) AND the working tree with the repo's `.gitleaks.toml`: both "no leaks found", exit 0. The de-id rails held.
- [x] [added 2026-06-15 02:18 UTC · done 2026-06-15 02:22 UTC] Workflow `triage-secrets-ci` (`wf_60234519-881`, 4 agents, ~92k tok): VERDICT — root cause = missing `GITHUB_TOKEN` env (gitleaks-action@v2 breaking change for PR-event scans); fix = add the token env (+ a `permissions: {contents:read, pull-requests:write}` block); **both gitleaks AND the independent 10-method cross-check agree `all_clear` — NO real secret in history or tree** (the 4 hits are templates/prose, e.g. `main.rs:159` is the Rust placeholder `CEC_SIGNOFF_SEED={}`). Adjacent: **Node 20→24 deprecation (forced June 16)** → bump `checkout@v4→@v5` + `gitleaks-action@v2→@v3` (both pure runtime migrations, drop-in). Fork-PR caveat: token forced read-only → never use `pull_request_target` (pwn-request RCE); if comments ever needed, `GITLEAKS_ENABLE_COMMENTS: 'false'`. Nice-to-haves (no deadline): double-trigger+concurrency, cargo-deny-action, SHA-pin + dependabot → FOLLOWUPS.
- [x] [added 2026-06-15 02:22 UTC · done 2026-06-15 02:30 UTC] Apply the verified `ci.yml` fix (token env + `permissions` + `checkout@v5` + `gitleaks-action@v3`) — owner chose "both branches now (no force-push)": committed on PR #3 (`53dd992`), cherry-picked onto PR #2's branch via a clean worktree (`951ae82`), both pushed (identical ci.yml). **Both `secrets` jobs now PASS on push AND pull_request triggers.** `check`/`audit` re-verifying with `checkout@v5` (background poll). Deferred CI nice-to-haves → FOLLOWUPS.

### Cleanup while Chris drafts (owner picked 3 tracks, 2026-06-15 02:35 UTC)

- [x] [added 2026-06-15 02:35 UTC · done 2026-06-15 03:00 UTC] Track 1 — adversarial review of the P0 diff (`d61b962`) before merge. Workflow `wf_923ec5a0-84d` (18 agents, ~696k tok): **13 confirmed findings → 2 CRITICAL fixed.** **D1 (de-id leak):** the `cec-diagnose/v1` envelope emitted `candidates[].rationale`/`title` verbatim → raw `--describe` (hostname/user/IP/serial) crossed to AllMyStuff in cleartext; fixed — envelope now ships only `{plan_id, source, max_risk, actions[]}`. **D2 (stdout purity):** `record_outcome`/`sandbox_validated_for` (free fns) used bare `println!` → `--json --sign-off` emitted 2 stdout lines; fixed via a module-scoped `tprintln!`. **D4:** the de-id test was vacuous → rewritten to bite. `emit_diagnose_envelope`→`diagnose_envelope()->Value`; +5 tests incl. `tests/cli_contract.rs`. 170 tests green, clippy+fmt clean. Committed `ddd1145`, pushed to PR #3 (P0-only code — nothing to cherry-pick to PR #2). RFC wire-contract made explicit.
- [x] [added 2026-06-15 02:35 UTC · done 2026-06-15 02:42 UTC] Track 2 — reconcile the stale FOLLOWUPS: verified each engine-gap item against the current code, tombstoned 5 fully-done (MH-1 wiring, canonicalization, MH-2 remainder, EI-03, MH-4/8/EI-06) + 3 partials (MH-6/A7, MH-5, sandbox-evidence) with the implementing PR #2 increment/commit; re-filed the 4 residuals. Section went ~12 open → 6 open / 11 closed.
- [x] [added 2026-06-15 02:35 UTC · done 2026-06-15 03:10 UTC] Track 3 — finish CI hardening: added `concurrency` block, swapped audit → `EmbarkStudios/cargo-deny-action@v2` (honors `deny.toml`), SHA-pinned all third-party actions + `.github/dependabot.yml` (github-actions, weekly). Committed `673a381` (PR #3) / cherry-picked `b7ad864` (PR #2); CI re-verifying (background poll). Did NOT scope `on: push` to main (trade-off, outside chosen scope) → noted in FOLLOWUPS.

### Corpus leak-prevention methodology (owner, 2026-06-15 03:11 UTC)

Codify + enforce prevention of ALL corpus identity-leak vectors, incl. ones an agent could accidentally
introduce (the D1 envelope leak this session was exactly that). Bar: a leak must be a compile-error / CI-fail /
blocked commit — NOT discipline.

- [x] [added 2026-06-15 03:12 UTC · done 2026-06-15 03:12 UTC] Recon existing de-id model + rails: "de-id by structured EXTRACTION not scrubbing" (extract_symptoms charset; de_identify_plan at the `Contribution::new` chokepoint); boundary rails (gitignore/pre-commit/gitleaks/BOUNDARY.md, hook DORMANT in fresh clones); authored-corpus `corpus-ingest` validate gate. Structural weakness: de-id is a chokepoint discipline, raw domain objects flow freely → any new sink bypasses it (D1).
- [x] [added 2026-06-15 03:12 UTC · done 2026-06-15 03:25 UTC] Design workflow `wf_148ceb35-f02` (15 agents, ~1.3M tok): **57 vectors mapped (11 critical, 22 high)**, 4 layers designed, 3-way red-team, synthesized `docs/corpus-leak-prevention.md` (494 lines, all 6 sections). VERIFIED already-real: the de-id "proof" test (corpus-client/src/lib.rs) seeds only describe/title/description with clean `action`/`id` → can't fail on the 2 fields `de_identify_plan` passes verbatim (CRITICAL `plan-action-verbatim`); `.claude/recon|audit` artifacts tracked+unignored. Doc landed on **branch `feat/corpus-leak-prevention` `9f1e057`** (off main), pushed.
- [x] [added 2026-06-15 03:12 UTC · done 2026-06-15 03:30 UTC] Present the methodology + phased plan; owner chose **Phases 0–2 (hard-guarantee tier)**. Work on branch `feat/corpus-leak-prevention` (rebased onto the P0 tip `673a381` so it has the envelope + all de-id code; force-pushed).
- [x] [added 2026-06-15 03:30 UTC · done 2026-06-15 03:55 UTC] **Phase 0 — DONE + verified** (commit `cf95d1c`): `crates/deid` validating mints (`action`=frozen-vocabulary membership — the keystone C1 fix; `plan_id`=clean-slug charset; `symptom`=extractor round-trip, each `Result`); `de_identify_plan`+`Contribution::new`→`Result` (an out-of-vocab action/id REFUSES the row, not copied through); `crates/leakguard` canonical POISON set; the leakage suite now BITES (seeds action/id, asserts refusal) — **PROVEN red on revert, green on fix**; drift guard (registered tools ⊆ ACTION_VOCABULARY); record_outcome surfaces refusals. 180 tests, clippy+fmt clean.
- [x] [added 2026-06-15 03:30 UTC · done 2026-07-02 19:15 UTC] **Phase 1 — type split + leaf Prose typing + sealed Debug** (the C1/C3 hard stops): DONE on `claude/repo-scope-work-plan-h93qx5` in 4 green sub-steps — see the "Session 2026-07-02 — leak Phase 1" block below. `StoredPlan`/`StoredStep`/`StoredSymptom`/`StoredSignature`/`StoredOutcome` are the only serde corpus payload; `Serialize`+dead-`Deserialize` removed from raw `Plan`/`PlanStep`/`Candidate`/`Outcome`/`DiagnosticEvent`/`StepResult`/`ExecutionResult`/`ToolOutcome`/`AgentRun`/`AgentStep`/`SignedPlan`; `Prose` (no Serialize/Display, redacting Debug) for title/description/rationale/message/summary; private `Contribution` fields + `trybuild` compile-fail tests; write-gate re-mints the stored plan (`GateError::RowNotDeIdentified`). Wire shape byte-identical (canned-row fixture). 198 tests, clippy/fmt clean.
- [x] [added 2026-06-15 03:30 UTC · done 2026-07-02 21:30 UTC] **Phase 2 — read-side re-de-id + closed dictionaries** (the C4/C5 hard stops): DONE on `claude/repo-scope-work-plan-h93qx5` in 3 green sub-steps — see the "Session 2026-07-02 — leak Phase 2" block below. Frozen `STOP_CODE_NAMES`/`MODULE_NAMES` dictionaries + closed-grammar `is_symptom_token` replace the shape heuristics; `#[serde(try_from)]` validating deserialization on `StoredSymptom`/`StoredAction`/`StoredPlanId`/`common::Symptom` refuses an out-of-vocab action, inadmissible id, or non-grammar symptom at the wire (`HttpCorpus::query`) and disk (`FileCorpus::open`); the closed-grammar symptom mint is wired into the 1f gate (`GateError::SymptomNotDeIdentified`); adversary-seeded read-path poison harness added. `serde_json::Value` 2c scoped honestly (no serialize boundary post-Phase-1 → documented, not re-typed). Wire byte-identical; 205 tests, clippy/fmt clean.
- [ ] [added 2026-06-15 03:30 UTC] **[downstream]** The private repo `corpus-ingest` calls `Contribution::new` (now `Result`) — it must adapt when it next bumps the engine pin → FOLLOWUPS.

### Session 2026-07-02 — consolidation, merges, leak-fix, API steer (remote session)

- [x] [added 2026-07-02 15:50 UTC · done 2026-07-02 15:50 UTC] Repo/branch-wide scope + consolidated plan of record → `docs/consolidated-work-plan.md` (9-agent analysis + direct verification; owner's engine-as-API steer folded in, supersedes RFC D1)
- [x] [added 2026-07-02 15:50 UTC · done 2026-07-02 15:50 UTC] Merge PR #2 (`2d9620a`) then PR #3 (`3b269f8`) into `main` (patch-identical CI clones merged clean, as predicted)
- [x] [added 2026-07-02 15:50 UTC · done 2026-07-02 15:50 UTC] **Repair `feat/corpus-leak-prevention`:** pushed tip `cf95d1c` did NOT compile (missing `schema.rs` keystone edit — verified twice in fresh worktrees); rebased onto main, restored `de_identify_plan`/`Contribution::new` → `Result` wiring the `deid` mints, re-verified (180 tests, clippy, fmt), force-pushed `0855884`, opened **PR #5**
- [x] [added 2026-07-02 15:50 UTC · done 2026-07-02 15:50 UTC] Rescue final-session tracking state (TODOS/FOLLOWUPS/HANDOFFS + `.claude/memory/*`) from `ops/agent-handoff` onto a real branch
- [x] [added 2026-07-02 15:50 UTC · done 2026-07-02 15:50 UTC] De-stale governance docs: checklist §3/§6 flips + §9 Increments 3–10 changelog, SECURITY.md attestation-is-enforced, negative-results NR-2/3/4 fixed-since notes, C-namespace disambiguation
- [x] [added 2026-07-02 15:50 UTC · done 2026-07-02 15:50 UTC] Record the **engine-as-API** supersession of RFC D1 in `docs/integration-rfc-for-chris.md` + `docs/integration-myown-family.md` (open question #1 resolved; Q2 sharpened; Q1–Q5 still awaiting Chris)
- [x] [added 2026-07-02 15:50 UTC · done 2026-07-02 15:50 UTC] Pin `cec-diagnose/v1` enum wire grammar (snake_case tokens, exhaustive matches, pinning test, `part_class` additive sibling) — zero consumers yet, last cheap moment (`ec1e388`); 171 tests green
- [x] [added 2026-07-02 15:50 UTC · done 2026-07-02 16:20 UTC] **B3/B4 — engine API v1:** `cec-support-agent serve` (loopback-bound; `POST /v1/diagnose` → `cec-diagnose/v1`; `POST /v1/execute` two-phase consent → `cec-execute/v1` post-execution envelope; poison-token contract tests on the HTTP surface) + `HttpCorpus::query` re-verifies attestation/`admit()` on every received row — see `docs/consolidated-work-plan.md` §3
- [x] [added 2026-07-02 15:50 UTC · done 2026-07-02 16:05 UTC] Merge PR #5 (leak Phase 0, repaired) + the housekeeping PR once CI is green; then rebase any in-flight work (both merged after the anyhow-advisory lockfile fix; the audit-job failure root-caused to a NEW upstream RustSec advisory, not the diffs)
- [x] [added 2026-07-02 15:50 UTC · done 2026-07-02 16:20 UTC] H4 — pin an exact Rust toolchain (or a tested-version CI job) + extend dependabot to the `cargo` ecosystem (the `channel = "stable"` drift already broke CI once, `538cd43`)

- [x] [added 2026-07-02 16:20 UTC · done 2026-07-02 16:20 UTC] **B4 (achievable half) — HttpCorpus read-side re-validation:** `query` refuses any served mapping whose plan is not exactly its own de-identified image (validating mints + equality; fails closed; `GateError::ServedPlanInadmissible`); one-shot-listener tests. Cryptographic attestation re-verification on this path needs attested rows on the wire → FOLLOWUPS
- [x] [added 2026-07-02 16:20 UTC · done 2026-07-02 16:20 UTC] **B3 — `cec-support-agent serve` (the API face):** GET /v1/health, POST /v1/diagnose (headless pipeline → `cec-diagnose/v1` + additive `session_id`), POST /v1/execute (two-phase consent; one-shot TTL'd sessions; escalation re-checked; declined consent = recorded Withdrawn; `plan_id` for app-side retry) → post-execution **`cec-execute/v1`** envelope (pinned label/verdict wire values — un-defers the post-exec-envelope FOLLOWUPS item). Loopback-bound; non-loopback refused without `--allow-remote`. axum 0.8 (license-clean). 189 tests, clippy/fmt clean; live-smoked e2e (health → diagnose → 409 on under-escalated sign-off → honest escalated execute)

### Session 2026-07-02 — corpus leak-prevention Phase 2 (read-side + dictionaries)

Owner-directed Phase 2 (the C4/C5 hard stops) on branch
`claude/repo-scope-work-plan-h93qx5`, on top of Phase 1. 3 green sub-steps, each
committed after compiling + tested; guards proven red-on-revert. Not pushed
(orchestrator opens the PR). Read-side spec: `docs/corpus-leak-prevention.md` §2
Layer 1e/2c + §4 Phase 2.

- [x] [added 2026-07-02 20:00 UTC · done 2026-07-02 20:40 UTC] **C5 frozen dictionaries** (`a0818bc`): `common::extract` — `STOP_CODE_NAMES` (Microsoft bugcheck names) + `MODULE_NAMES` (OS/driver allowlist), sorted for binary search, replace the `is_stop_code_name`/`module_name` SHAPE heuristics (which kept any ALL_CAPS_UNDERSCORE token / any `stem.exe`). New public closed-grammar predicate `is_symptom_token` (`VOCABULARY ∪ 0x-hex ∪ <prefix>_<digits> ∪ STOP_CODE_NAMES ∪ MODULE_NAMES`); `deid::symptom` wired to it — closes the Phase-1 blocker (the round-trip mint rejected `event_41`; the grammar admits it directly). explorer.exe/event_41/xid_79/0x1234/real bugchecks stay admissible; asset tags (RIG_NATHAN_DESK), custom binaries (acmecorp_agent.dll, app.exe), hostnames refused. Guard bites red-on-revert (shape heuristic leaks john_smith). 201 tests.
- [x] [added 2026-07-02 20:40 UTC · done 2026-07-02 21:15 UTC] **C4 read-side re-de-id** (`a759afd`): `#[serde(try_from = "String")]` validating deserialization on `StoredSymptom` (closed grammar), new `StoredAction` (frozen ACTION_VOCABULARY; carries step action AND description) and `StoredPlanId` (clean slug), plus `common::Symptom` (for `verification.recurring`). A served/at-rest out-of-vocab action, inadmissible id, or non-grammar symptom now FAILS TO DESERIALIZE — `HttpCorpus::query` (transport/admission split → `ServedPlanInadmissible`) and `FileCorpus::open` (Storage parse error). Symptom mint wired into the 1f gate (`GateError::SymptomNotDeIdentified`) over signature + recurring. Adversary-seeded read-path poison harness (leakguard::POISON in a served symptom) refused; proven red-on-revert (dropping the StoredSymptom guard serves `desktop-nathan01` into retrieval-first). Wire byte-identical (canned fixture + chain stable); test symptom fixtures moved off synthetic `boot_loop` → `event_41`. 205 tests.
- [x] [added 2026-07-02 21:15 UTC · done 2026-07-02 21:30 UTC] **2c serde_json::Value scoping** (`5-th commit`): the only `Value` fields are `ToolOutcome.data` and `AgentStep.args`; Phase 1 removed `Serialize` from both types, so neither has any path to a serialize/print sink — the 2c serialization boundary is already closed. Documented the invariant on both fields (a future re-add of Serialize is a visible leak); typing into an allowlisted summary is C2/Phase-4 (the agent-loop/model-prompt egress), not a Phase-2 corpus/print sink. No re-typing — scoped honestly per the doc.

### Session 2026-07-02 — corpus leak-prevention Phase 1 (type barrier)

Owner-directed Phase 1 (the C1/C3 compile-error hard stops) on branch
`claude/repo-scope-work-plan-h93qx5`. Worked in 4 green sub-steps, each committed
after compiling + tested; guards proven red-on-revert. Not pushed (orchestrator
opens the PR).

- [x] [added 2026-07-02 17:30 UTC · done 2026-07-02 18:05 UTC] **1a type split** (`a347878`): `crates/corpus-client/src/stored.rs` — `StoredPlan`/`StoredStep`/`StoredSymptom`/`StoredSignature`/`StoredOutcome` as the ONLY corpus-serializable payload; `de_identify_plan` mints a `StoredPlan`; `Contribution`.outcome + `FixMapping` carry stored types; removed `Serialize`(+dead `Deserialize`) from raw `Plan`/`PlanStep`/`Candidate`/`Outcome`/`DiagnosticEvent`/`StepResult`/`ExecutionResult`/`ToolOutcome`/`AgentRun`/`AgentStep`/`provenance::SignedPlan`. `Contribution` fields → `pub(crate)` + accessors; retrieval-first rehydrates via `StoredPlan::to_plan`. Canned pre-split row fixture proves the wire shape is byte-identical (deserializes, round-trips, chain verifies at open, gate-passes). 191 tests.
- [x] [added 2026-07-02 18:05 UTC · done 2026-07-02 18:30 UTC] **1b Prose + 1d sealed Debug** (`3790dbd`): `common::Prose` (private field; no Serialize/Deserialize/Display; redacting Debug; `as_str()`/`into_inner()`/`From`) for `Plan.title`/`PlanStep.description`/`Candidate.rationale`/`DiagnosticEvent.message`/`StepResult.summary`. Because the prose is Prose-typed, containers keep a derived Debug that is auto-sealed — runtime test proves `format!("{:?}", outcome)` never spills planted prose (diverged from the doc's "manual Debug impls" — Prose's own Debug is stronger + can't forget a field). `render_consent`, the human trace, and `provenance::canonical` read via the sanctioned `as_str()` accessor. 194 tests.
- [x] [added 2026-07-02 18:30 UTC · done 2026-07-02 18:50 UTC] **1f write-gate idempotence** (`22ec564`): `ensure_evidence_integrity` re-mints the stored plan (rehydrate → `de_identify_plan`) and refuses a row that is not its own de-id image — `GateError::RowNotDeIdentified`. Catches an out-of-vocab action, an inadmissible id, or a hand-edited title/description on any row (incl. off-constructor rows from disk / an embedder). Symptoms kept structurally typed (strict mint deferred to Phase 2 per the `<prefix>_<digits>` gotcha). Proven red-on-revert. 197 tests.
- [x] [added 2026-07-02 18:50 UTC · done 2026-07-02 19:15 UTC] **trybuild compile-fail guards** (`9a9cd5b`): `trybuild` dev-dep (MIT OR Apache-2.0; subtree deny.toml-clean) + 3 pinned cases — `to_string(&candidate)` (E0277), struct-literal `Contribution {..}` (E0451 private fields), `format!("{}", prose)` (E0277 no Display) — `.stderr` pinned to 1.96.1 (trybuild-normalized paths → CI-portable). Proven red-on-revert (re-adding `Display for Prose` fails the harness). 198 tests total; clippy `-D warnings` + fmt clean; e2e CLI smoke green (clean `cec-diagnose/v1`, human trace renders).

### Session 2026-07-02 — API-posture decisions (owner, remote session)

Owner's 2026-07-02 API-posture decisions responding to `docs/api-extension-design.md`, on branch
`claude/repo-scope-work-plan-h93qx5` (on top of leak Phase 2). Green sub-step commits, each compiled +
tested; the two guards proven red-on-revert. Not pushed (orchestrator opens the PR).

- [x] [added 2026-07-02 22:05 UTC · done 2026-07-02 22:15 UTC] **Trusted calls only (leak C2)** (`697e16d`):
  `validate_inference_endpoints` refuses a non-loopback `--endpoint`/`--fast-endpoint` at startup on BOTH the
  diagnose and serve paths unless `--allow-remote-inference` is passed (loopback = localhost / 127.0.0.0/8 /
  [::1]); the refusal is a fixed message that never echoes the URL; `endpoint_is_loopback` fails closed on an
  unparseable host. Builds leak-doc §3.1(b) (annotated there + Phase 4 item 14). +4 tests (loopback admitted /
  non-loopback refused on both flags / flag admits); guard proven red-on-revert (neutering the guard fails the
  refusal test). Live-smoked on both paths.
- [x] [added 2026-07-02 22:05 UTC · done 2026-07-02 22:20 UTC] **Route-surface pinning** (`588f1ec`): the
  frozen `route_surface` list (GET /v1/health, POST /v1/diagnose, POST /v1/execute) is folded into the router
  by `build_router`, and `router_surface_is_frozen` pins the exact (method, path) set — adding ANY route is a
  deliberate test edit. The never-routable invariant (attest, keygen, corpus WRITE) is stated in serve.rs's
  module docs and added to SECURITY.md's invariant list (a violation is a reportable security issue). +1 test;
  proven red-on-revert (a rogue /v1/attest route fails the pin).
- [x] [added 2026-07-02 22:05 UTC · done 2026-07-02 22:25 UTC] **AGPL §13 notice + auth-ladder resolution**
  (`64ffa48`): `--allow-remote` prints a one-line stderr network-service / §13 Corresponding-Source notice at
  startup (live-smoked); same note in SECURITY.md (Network exposure and AGPL §13). Auth ladder resolved:
  hard-loopback by default, remote = mesh-only, no bearer-token tier will be built. README has no serve section
  → skipped.
- [x] [added 2026-07-02 22:05 UTC · done 2026-07-02 22:30 UTC] **Docs — decision log + binding checklist**
  (`878fd4d`): DECISION LOG (§5) in api-extension-design.md — corpus-over-API ships only over mesh rostered
  identity or loopback, never token-auth public HTTP; served rows carry attestation (FixMapping gap closes
  first); encrypted transport (mesh / TLS); no corpus endpoint exists yet, route-pin is the mechanical guard.
  Copied the 6-rule §2.5 egress-sink checklist into AGENTS.md as binding policy (short, imperative).
- [x] [added 2026-07-02 22:05 UTC · done 2026-07-02 22:40 UTC] Tracking: TODOS/FOLLOWUPS/HANDOFFS updated;
  leak §3.1(b) tombstoned as built. 210 tests (was 205), clippy `-D warnings` + fmt clean (pinned 1.96.1).

### Session 2026-07-02 — PR #12 merge + AGENTIC ADDENDUM

- [x] [added 2026-07-02 22:42 UTC · done 2026-07-02 22:42 UTC] Merge PR #12 (all session work) into `main`
  after resolving a `ci.yml` conflict (PR #4 checkout bump vs the gitleaks OSS-binary rewrite → kept the OSS
  binary + main's newer checkout SHA); restart the branch from the new main; retire the babysitter crons.
- [x] [added 2026-07-02 22:42 UTC · done 2026-07-02 22:42 UTC] Ground-truth extract (4-agent read-only
  workflow `wf_44941c16-6d7`) of the real agentic infra, verification suite, security kernels, and frozen
  constants — the accurate basis for the addendum.
- [x] [added 2026-07-02 22:42 UTC · done 2026-07-02 22:42 UTC] Author `docs/AGENTIC_ADDENDUM.md` (§1 four
  tracking files + memory mirror; §2 real hooks + proposed PreToolUse invariant guard / Stop verify-gate;
  §3-§6 projectops/panels/lifecycle; §7 the fully-blind audit adapted to our crypto/de-id kernels with the
  frozen constants as "reserved values").
- [x] [added 2026-07-02 22:42 UTC · done 2026-07-02 22:42 UTC] Make it reachable + hooked: reference from
  AGENTS.md + a new `addendum-context.sh` SessionStart hook wired in `.claude/settings.json` (validated JSON;
  5 SessionStart hooks; hook emits well-formed additionalContext).
- [~] [added 2026-07-02 22:42 UTC · obsolete 2026-07-02 23:56 UTC → split into the two lines below] Implement
  the addendum's mechanical backstops (not yet built): a `PreToolUse` exfil/oracle guard (§2b), a Stop verify
  + tracking-freshness gate (§2d), and the `projectops` MCP server + panels (§3-§4).
- [x] [added 2026-07-02 23:56 UTC · done 2026-07-02 23:56 UTC] **Tier-1 guards built + wired + validated**
  (PR #13): `invariant-guard.sh` (PreToolUse hard-block on corpus/weights/seed PATHS), `invariant-check.sh`
  (PostToolUse surface: conflict markers, serialized-corpus-row, seed/key blocks — self-safe after two
  dogfooded false positives, see HANDOFFS lesson), `tracking-freshness.sh` (Stop nudge if crates/ changed
  without a HANDOFFS/TODOS update), and `ops/provision.sh` (Tier-0 activator). Wired in `.claude/settings.json`
  (PreToolUse/PostToolUse/Stop); validated block/allow/surface/self-reference; addendum §2b/2c/2d/2f updated
  to reflect built-not-proposed.
- [x] [added 2026-07-02 23:56 UTC · done 2026-07-03 00:07 UTC] **projectops server built + validated**
  (PR #14): `tools/projectops.py` (stdlib CLI: verify/invariants/backlog/leak_scan, structured JSON) +
  `tools/projectops_server.py` (minimal MCP stdio server, raw JSON-RPC, no SDK dep) + `.mcp.json`. Server
  handshake + tools/list + tools/call e2e-tested; invariants pass on the tree and bite on a re-added
  `source`/rogue route/unsorted vocab.
- [x] [added 2026-07-03 00:07 UTC · done 2026-07-03 01:01 UTC] **Review panels built** (PR #15):
  `tools/projectops_panel.py` renders projectops JSON into a self-contained theme-aware HTML dashboard
  (verification/invariants/backlog/blind-audit, summary tiles, status pills + severity stripe); live
  instance rendered as an Artifact; dogfooding fixed a `verify` deny/gitleaks skip-vs-fail bug.
- [ ] [added 2026-07-03 01:01 UTC] Remaining addendum refinements (deferred): Stop verify-gate via
  `projectops verify --checks`, scheduled/Stop panel regen (it is a manual snapshot today), deeper
  `projectops invariants` (raw-type-Serialize + full vocab/registry drift). See FOLLOWUPS.

### Session 2026-07-03 — test-and-validation-fleet model (design/threat-model, no code)

Owner asked to stand up the two runtime MCP surfaces of the fleet model: (a) the target-environment access
MCP (agent → client/volunteer PC) and (b) the sandbox test-harness MCP. Owner's steer: design/threat-model
first (highest-risk execution zone). Owner's live question this session: "a golden image per Windows update?"

- [x] [added 2026-07-03 00:40 UTC · done 2026-07-03 01:22 UTC] Map the existing execution/validation/corpus
  surfaces as read-only ground truth (sonnet agent) → `scratchpad/exec-validation-surface-map.md` (every fact
  `file:line`); confirmed the 8 stated gaps (no SandboxValidator impl, `recollect_post_signature -> None`, no
  mesh/roster in code, no volunteer/telemetry/fleet concept, prereg-lane VOID unless filled first,
  `driver_rollback` vocab-without-tool, no serve caller-auth, ephemeral per-run signing key).
- [x] [added 2026-07-03 00:40 UTC · done 2026-07-03 01:22 UTC] Design the two MCP surfaces + threat model
  (opus agent), cross-checked against the ground-truth map, and land `docs/test-validation-fleet-design.md`:
  cardinal WRAP-the-gates rule; T-1..T-7 execution-boundary threat map; SandboxValidator lowers-only contract;
  §3.1 Windows-reproduction mechanism folded in per the owner's question; volunteer = de-identified target,
  no volunteer-id on the row; greenlight/infra/Chris sequencing; F4 named as the fleet's hard data gate.
- [x] [added 2026-07-03 01:22 UTC · done 2026-07-03 01:22 UTC] File **RFC Q7** (plan-signing across the
  execution boundary — judge-on-target vs ed25519 custodied key; pairs with Q1) in
  `docs/integration-rfc-for-chris.md`, and point the design doc §5 at it.

### Session 2026-07-03 — Lane ② pure-engine work (owner: "both, together"; land as green PRs)

Owner greenlit the whole of Lane ② from `docs/test-validation-fleet-design.md` §5: 3 fleet contracts + 3
corpus-hardening items. PR #15 merged first (docs/python-only, kept clean); branch restarted from the new
`main` (`ac14edf`); babysitter cron `69d7ae77` retired. Plan: `scratchpad/lane2-implementation-plan.md`.
Sequencing PR-A(item1) · PR-B(item2) · PR-C(item3) · PR-D(item5) · PR-E(items4+6, bundled migration).

- [x] [added 2026-07-03 01:47 UTC · done 2026-07-03 02:17 UTC] **Item 1** — gated-MCP-wrapper spec landed as
  `docs/execution-mcp-wrapper-spec.md` (normative MUST/MUST NOT: one gated verb pair, wrap-the-gates,
  destructive-floor-unforgeable, risk-reconciled-on-box, out-of-vocab advisory-only, egress-sink inheritance,
  one audit record per execute, off-box = --allow-remote+mesh+TLS, never-routable caps) + a verb contract, an
  anti-scope, a conformance checklist, and the Q7/Q1 forks that gate the distributed variant. Cross-linked
  from the fleet design §5. Completes the execution-zone trio (items 1/2/3) → open PR-1.
- [x] [added 2026-07-03 01:47 UTC · done 2026-07-03 01:47 UTC] **Item 2** — `SandboxValidator` production
  CONTRACT + "can't-mint-truth" test. Strengthened the trait + `ValidationReport` docs in
  `crates/swarm/src/lib.rs` with the normative "a sandbox LOWERS an escalation, never MINTS truth" contract;
  added `a_clean_sandbox_can_never_mint_a_resolved_row` to `support-agent` proving a clean apply + `None`
  re-collection → `Verdict::Unverified` → `EscalatedHumanUnresolved` (not resolved). Workspace green
  (clippy -D, all tests; support-agent unit 35→36).
- [x] [added 2026-07-03 01:47 UTC · done 2026-07-03 02:17 UTC] **Item 3** — execution audit-log skeleton.
  New `crates/support-agent/src/audit.rs`: `ExecutionRecord` (closed de-identified field set — minted
  plan_id, opaque run_id, unix ts, outcome-label token, `caller_key: None` until rung-2), `to_line()`
  (closed-set JSON), `AuditSink` trait + default `NullSink`. Wired at the `record_outcome` funnel (fires for
  every outcome incl. declines, using the MINTED id from the contribution + reused `serve::wire_label`);
  injection seam = `AppState.audit` (serve) / `&NullSink` (CLI). Tests: closed field set, no-op sink, and a
  capturing-sink test proving one record per outcome carries the minted id and no title prose. Green
  (fmt/clippy -D/tests; support-agent unit 36→39). Deferred bits → FOLLOWUPS.
- [~] [added 2026-07-03 01:47 UTC · obsolete 2026-07-03 03:15 UTC → FOLLOWUPS 2026-07-03 03:15] **Item 5 (B4)**
  RE-SCOPED. On inspection `HttpCorpus::query` serves bare `FixMapping`s that carry **no attestation**
  (`schema.rs:86`; attestation lives on `Contribution:198`), and the code comment (`store.rs:514-517`) says
  cryptographic re-verification "lands with the corpus-service wire contract." So B4 is NOT a small
  add-a-verify-call: it needs the served-row type to become an attested row, which is **gated on RFC Q6**
  (served-row provenance minimization, still open) AND on the corpus service existing. Deferred → FOLLOWUPS.
- [x] [added 2026-07-03 03:15 UTC · done 2026-07-03 03:15 UTC] Record owner decisions (2026-07-03): **Q7** =
  ed25519 custodied judge key (RFC Q7 DECIDED note; pairs with F3); **Q1 volunteer-half** = pure execution
  target, central authority attests (RFC Q1 DECIDED-partial note); **leak-C7 salt** = per-deployment secret
  loaded like the sign-off key + cold-start default; **migration** = hard cutover, private corpus re-ingests
  once. Plan updated (`scratchpad/lane2-implementation-plan.md` item 6).
- [x] [added 2026-07-03 01:47 UTC] **Items 4+6 (BUNDLED migration) — NOW UNBLOCKED (decisions in).** F2
  canonical (serde-independent) `chain_hash` (`schema.rs:461`, replace `serde_json::to_vec` with explicit
  length-prefixed encoding, bump `cec-corpus-chain-v1`→`v2`) + leak-C7 keyed/salted HMAC `fingerprint_of`
  (`common/src/hash.rs`, per-deployment secret salt + cold-start default). Both invalidate stored hashes →
  ONE hard cutover: update in-tree fixtures, operator re-ingest note. This is the next code PR (PR-2).
  · done 2026-07-04 19:05 UTC (branch `claude/workflow-model-optimization-e1y1sx`, commits
  `92df52d`/`e17f38f`/`90ff2c2`; 235 tests green; red-on-revert proven for both kernels)

### Session 2026-07-02 — corpus cartography (leak-C10) threat model + non-mappability policy

Owner-raised threat (2026-07-02): "Can a surface expose the internal corpus by mapping it out through
trusted calls?" Analyzed as **corpus cartography** — a fourth corpus property orthogonal to
admissibility/authenticity/access. Docs/policy/tracking pass to match the already-committed code change
(`4cf9d8f`, this branch) that dropped the `source` membership label.

- [x] [added 2026-07-02 18:54 UTC · done 2026-07-02 18:54 UTC] Corpus-cartography check (2-agent design +
  verification pass grounded in the current diagnose/serve code) → `docs/corpus-cartography-threat.md`:
  the honest-limit framing (§0), 7 concrete verified vectors V1-V7 (§2), a lettered control set A-G with
  cost/gate mapping (§3), the NON-MAPPABILITY rule set (§3b), accepted residuals (§4), and the phased
  sequence onto the existing F2→F3→B4→F1→E3 plan (§5).
- [x] [added 2026-07-02 18:54 UTC · done 2026-07-02 18:54 UTC] leak-C10 defined in the taxonomy —
  `docs/corpus-leak-prevention.md` §1.2 gained a C10 row (ranked below C9 as a distinct orthogonal axis: it
  needs no identity to survive de-id, so it is not closable by a type) + a §3.1(4) cross-reference to the
  threat doc's honest-limit + control-set sections.
- [x] [added 2026-07-02 18:54 UTC · done 2026-07-02 18:54 UTC] `source` membership label dropped from the
  `cec-diagnose/v1` candidate body (control D, partial — the label half; the latency/slate-count differential
  half remains a documented residual). CODE already committed prior to this session (`4cf9d8f`
  "feat(serve): drop the `source` membership label from cec-diagnose/v1 (leak-C10)"); this session's scope
  was the docs/policy/tracking to match, not the code.
- [x] [added 2026-07-02 18:54 UTC · done 2026-07-02 18:54 UTC] Non-mappability policy landed as binding
  policy: the 7-rule set from the threat doc's §3b copied into `AGENTS.md` as a sibling block to the existing
  §2.5 egress-sink checklist, same short imperative voice. `docs/corpus-cartography-threat.md` copied
  verbatim into the tree.
- [x] [added 2026-07-02 18:54 UTC · done 2026-07-02 18:54 UTC] Wire-contract docs corrected:
  `docs/integration-rfc-for-chris.md` candidate body updated to `{plan_id, max_risk, actions[]}` + a removal
  note (leak-C10) + the enum-grammar note corrected + real question **Q6** ("how much provenance does a
  served row expose?") filed in the open-questions section, gated on B4;
  `docs/api-extension-design.md` §5 decision log gained the dated `source`-drop entry.
- [x] [added 2026-07-02 18:54 UTC · done 2026-07-02 18:54 UTC] Deferred controls filed to `FOLLOWUPS.md`,
  each attributed to the threat doc: control D remainder (latency/slate equalization, E3-gated), control A
  (per-identity query budget, E3/rung-2), control B (per-identity query audit log, E3/rung-2, MH-1's
  query-side twin), control E (keyed/salted HMAC fingerprint, greenlightable, = existing leak-C7 item pulled
  forward), control C (B4 provenance-graph minimization precondition), and the Q6-filed tombstone pointer.

## Done / obsolete (history)

_(completed items stay above, in place, with their `· done` tombstone)_

### Session 2026-07-04 — migration bundle (PR-2): chain v2 + keyed fingerprint + POST-body query

- [x] [added 2026-07-04 18:20 UTC · done 2026-07-04 18:45 UTC] F2: `chain_hash` → explicit field-by-field
  length-prefixed `chain_canonical` (`cec-corpus-chain-v2`), binds every field incl. title/description/
  attestation, never integrity; hand-assembled canonical pin + 25-mutation binding sweep + ambiguity case +
  v1-era-refused-at-open pin; all proven red on a v1 revert (`92df52d`).
- [x] [added 2026-07-04 18:45 UTC · done 2026-07-04 19:00 UTC] leak-C7: `fingerprint_of` → HMAC-SHA256
  under per-deployment `CEC_FINGERPRINT_SALT` (loaded like the sign-off key, ≥16 bytes fail-closed,
  documented public cold-start default, `cec-fingerprint-v2`, 64-hex); write-once salt lifecycle; unit +
  process-isolated integration + 2 CLI e2e tests; proven red against the silent salt-ignore regression
  (`e17f38f`). Canned fixture regenerated with true v2 values, fragment-split so the invariant hook's
  corpus-row backstop stays meaningful (pattern widened 16→16-64 hex).
- [x] [added 2026-07-04 19:00 UTC · done 2026-07-04 19:02 UTC] Cartography control E logging half:
  retrieval keys out of the GET URL into the `POST /v1/mappings/query` body (`90ff2c2`).
- [x] [added 2026-07-04 19:05 UTC] Blind-audit panel (addendum §7) on the two new kernels — packet in
  scratchpad, 3 blind auditors running; verify any finding against source before trusting it.
  · done 2026-07-04 19:40 UTC — 3/3: chain canonical CLEAN (two independent concrete collision attempts
  failed on the count/length guards); Kernel-2 findings all verified real and FIXED (`8626f23`): CRITICAL
  non-UTF-8 salt fail-open refused at startup; MEDIUM fault/config domain separation (3/3 convergence);
  HIGH silent cold-start → serve NOTICE + `fingerprint_salt_is_configured()` probe; LOW static lp tags +
  honest POST-body scope note. 237 tests green.
- [x] [added 2026-07-04 20:05 UTC · done 2026-07-04 20:05 UTC] Record owner decision **RFC Q6 = DECIDED**
  (provenance-graph minimization: served row = attested `StoredOutcome` + attestation ONLY) in the RFC +
  close the two cartography FOLLOWUPS items (control C; Q6-filed). B4 is now gated only on the corpus
  service existing. Owner also asked for the Q1-operator-half recommendation + the Chris-blockers rundown
  (delivered in-session; recommendation = SEPARATE keys, recorded in the handoff log).
- [x] [added 2026-07-04 20:35 UTC · done 2026-07-04 20:35 UTC] Record owner decisions **Q1 operator-half =
  SEPARATE keys (Q1 fully decided)** + **D3 integration posture** (engine = independent authenticated API,
  loopback-bound; MyOwnMesh daemon = transport only, no `myownmesh-core` link; no MyOwnLLM now) in the RFC;
  Q2 decided-for-now / Q3 moot / Q4 deferred / Q5 reframed→B4 wire contract; integration doc banner;
  closed the "[RFC Q1–Q5 awaiting Chris/owner]" FOLLOWUPS item. Grounded in a live review: MyOwnMesh
  v0.2.28 (generic RPC + roster ed25519 identity + daemon-client embedding pattern), AllMyStuff v0.2.17.
- [x] [added 2026-07-04 20:30 UTC · done 2026-07-04 20:30 UTC] Subscribed to PR #17 activity (babysit to
  merge); CI green 10/10 at head `72985c6`; hourly self check-in armed via send_later.

### Session 2026-07-04 (later) — PR #17 merged; leak Phase 3 (3b/3c) built

- [x] [added 2026-07-04 20:50 UTC · done 2026-07-04 20:52 UTC] Merge PR #17 (all checks green ×2 runs) →
  `main` @ `e16fd35`; babysit trigger deleted; branch restarted from main and force-with-lease pushed.
- [x] [added 2026-07-04 20:55 UTC · done 2026-07-04 21:20 UTC] **Leak Phase 3, 3b+3c BUILT:** `tools/xtask`
  (`scan-content` staged/tree: quoted-key row-shape co-occurrence, POISON-minus-bare-author, one-level
  base64/hex decode-and-rescan, runtime-decode ban in test files; `allowlist-freeze` net-new-fails-in-CI
  with bootstrap exemption; `install-hooks`), frozen `.boundary-allow.txt` seeded from a real tree sweep
  (every entry a sanctioned adversarial-test/doc site), pre-commit hook now content-gated with gitleaks
  warn-and-skip, CI `boundary` job (tree scan + freeze + hook-invokes-gate assert), `.gitleaks.toml`
  seed/salt/row rules + `.claude/**` ignore. All four checks proven red on planted violations
  (poison / encoded-poison / runtime-decode / row-shape). 245 tests green. 3a dylint → FOLLOWUPS.
- [x] [added 2026-07-04 21:45 UTC · done 2026-07-04 21:55 UTC] PR #18 (Phase-3 boundary gate): first CI
  run red on the `secrets` job — corpus-row gitleaks rule matched the shape prefix (8 sanctioned sites);
  tightened to require the 16-64-hex value (`5a7cd81`); rerun 12/12 green (incl. both first `boundary`
  jobs); MERGED → `main` @ `44d623a`; trigger deleted; branch restarted from main.

### Session 2026-07-04 (third) — attestation v4 (provenance commitment) ahead of the re-ingest

- [x] [added 2026-07-04 22:20 UTC · done 2026-07-04 22:45 UTC] **Attestation v4:** `attestation_message`
  binds `RowProvenance::commitment()` (`cec-provenance-commitment-v1` sha256) instead of raw provenance
  fields — resolves the RFC Q6 DECIDED note's wrinkle (a Q6-minimized served row is now verifiable);
  replay protection preserved (existing fabricated-run-id gate test still green); provenance-None message
  bytes unchanged except the version tag (content_hash semantics carry over). 2 new pins (raw fields
  absent from signed bytes + commitment binding/sort completeness); proven red on a raw-binding revert.
  247 tests green. Timed deliberately BEFORE the one-time private-corpus re-ingest so the operator
  re-ingests exactly once. §7 blind panel (2 auditors) running.
- [x] [added 2026-07-04 23:15 UTC · done 2026-07-04 23:20 UTC] PR #19 (attestation v4 + operator
  runbook): 12/12 green, MERGED → `main` @ `0c54578`; branch restarted. Re-ingest window open —
  operator steps handed off via docs/operator-runbook.md.

### Session 2026-07-08 — F4 seam + autonomous loop + repertoire tier + EULA gate (BUILD)

- [x] [added 2026-07-08 03:18 UTC · done 2026-07-08 03:18 UTC] F4 re-collection SEAM (`PostFixCollector`/`NullCollector`/
  `post_fix_collector` swap point), wired CLI+serve; the autonomous VerifierConfirmed→ResolvedProvisional
  loop proven end-to-end with a mock collector (clean→resolved row no human; recurring→hard negative;
  Null→Unverified); red-on-revert. `CandidateSource::Repertoire` (0.7 prior). `6b367a4`.
- [x] [added 2026-07-08 03:18 UTC · done 2026-07-08 03:18 UTC] §7 blind panel on the autonomous path found a real fabrication vector
  (empty `Some(vec![])` → Pass); FIXED (`8bf9047`: empty re-collection fails closed to Unverified) + guard
  test; 2 collector-correctness residuals + Path B/C filed to FOLLOWUPS; playbook hardened.
- [x] [added 2026-07-08 03:18 UTC · done 2026-07-08 03:18 UTC] EULA on-screen acceptance gate: `Tool::requires_eula` +
  `Dispatcher::eula_of` + `EulaAcceptances` + `execute_plan` refusal BEFORE dispatch (installer never runs
  without acceptance); red-on-revert; `docs/eula-acceptance-playbook.md`. 255 tests green.

### Session 2026-07-08 — partial resolution (BUILD + blind audit + gate hardening)

- [x] [added 2026-07-08 04:40 UTC · done 2026-07-08 04:56 UTC] Build partial resolution across common →
  agent-core → corpus-client → support-agent: `VerificationResult::PartialPass`/`Regressed`,
  `Verdict::PartialPass`/`Regressed`, `verify_outcome` three-way reasoning (originals only — no autonomous
  regression detection), `OutcomeLabel::ResolvedPartial`/`Regressed` + `is_beneficial()`, additive
  cleared/introduced binding in attestation + chain (byte-identical pre-change rows), gate block (2b) +
  destructive-needs-human on `is_beneficial`, wire tokens. Commit `661e53e`. 259 tests green.
- [x] [added 2026-07-08 04:56 UTC · done 2026-07-08 05:05 UTC] §7 blind audit (opus, packet-only) of the
  partial-resolution verification + gate + crypto binding — all 6 invariants HOLD, NO defects. Verified
  each against source myself.
- [x] [added 2026-07-08 05:05 UTC · done 2026-07-08 05:10 UTC] Close the audit's one conservative note:
  gate now requires a `PartialPass` to carry a non-empty remainder too (a full clear can't be mislabeled as
  a partial), with a red-on-revert test. Design doc §8 records owner decisions. 260 tests green, clippy clean.
- [x] [added 2026-07-08 05:10 UTC · done 2026-07-08 05:14 UTC] Push branch + open PR for partial resolution
  → **PR #21** (https://github.com/nathanfraske/cec-support-agent/pull/21).
- [x] [added 2026-07-08 05:14 UTC · done 2026-07-08 05:14 UTC] Merge PR #21 on green CI (all 6 checks
  green: check ubuntu/macos/windows, secrets, audit, boundary) → squash-merged, **`main` @ `081ad3d`**.
  Branch reset locally to new main; remote-branch reset needs a force-with-lease (auto-mode declined —
  awaiting owner go-ahead or it lands naturally on the next push).

### Session 2026-07-08 — corpus lifecycle & retrieval (5-feature arc; owner asked all)

- [x] [added 2026-07-08 05:30 UTC · done 2026-07-08 05:30 UTC] Scope all 5 (corpus service, retrieval-as-partial,
  config-transition, workflow retirement, formatting/intent page) → `docs/corpus-lifecycle-design.md`; two
  owner decisions flagged (retirement gating posture; the "page" surface).
- [x] [added 2026-07-08 05:30 UTC · done 2026-07-08 05:40 UTC] **Retrieval-as-partial** — `MappingKind::{Full,
  Partial{cleared}}` (additive, serde-default Full); `fix_mappings` accumulates partial mappings keyed by
  (plan, cleared-set) with the same independence rule; revocation filters both kinds; consumers label
  partials honestly (`mapping_rationale`). 264 tests green (+4), clippy clean, red-on-revert proven.
- [ ] [added 2026-07-08 05:40 UTC] **Config-transition primitive** — structured categorized inventory +
  `ConfigTransition::between` emitting a de-id transition trigger token. (building next)
- [ ] [added 2026-07-08 05:40 UTC] Ask owner: retirement gating posture (Q-retire) + formatting page surface (Q-page).
- [ ] [added 2026-07-08 05:40 UTC] **Workflow retirement** (pending Q-retire; §7 blind audit — new corpus-mutation gate).
- [ ] [added 2026-07-08 05:40 UTC] **Corpus query service** (separate authenticated read API; §7 blind audit — new egress).
- [ ] [added 2026-07-08 05:40 UTC] **Formatting/intent page** (pending Q-page).
