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
- [ ] [added 2026-06-14 20:50 UTC] Increment 9 — MH-1 operator wiring: keygen + `CEC_SIGNOFF_PUBKEY`/seed env, attest at sign-off
- [ ] [added 2026-06-14 20:50 UTC] Increment 10 — Sandbox-validation evidence: structured `SandboxValidator` wiring
- [ ] [added 2026-06-14 20:50 UTC] AUDIT: adversarial multi-agent review of the full diff → fix findings
- [ ] [added 2026-06-14 20:50 UTC] Present for review (PR-ready summary)

## Done / obsolete (history)

_(completed items stay above, in place, with their `· done` tombstone)_
