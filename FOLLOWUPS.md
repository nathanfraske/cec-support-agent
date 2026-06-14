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

- [ ] [added 2026-06-14 20:09 UTC] **[EI-08 / MH-1 — keystone]** Implement owner-key attestation over the contribution tuple: extend `provenance::SignedPlan` → `SignedContribution` over `(signature, plan, label, sign_off, config_class)` with a judge/verifier identity, and verify it in `ensure_signed_off`/`ensure_evidence_integrity` for `HumanConfirmed`. — today a caller can construct `Contribution{ sign_off: HumanConfirmed }` and the gate passes; where to resume: `crates/corpus-client/src/gate.rs:15` + `crates/provenance/src/lib.rs:63-80`
- [ ] [added 2026-06-14 20:09 UTC] **[Custody]** Decide the non-ephemeral judge key custody / rotation / audit-log retention path — `SigningKey::generate()` at `crates/support-agent/src/main.rs:485` mints a fresh per-run key with no identity, so the EI-08 attestation cannot be real without an identified, persistent judge — where to resume: `crates/provenance/src/lib.rs:11-16`
- [ ] [added 2026-06-14 20:09 UTC] **[Canonicalization]** Replace serde field-order canonicalization with a sorted/canonical-JSON encoder before signatures are cross-version/cross-language verified — fragile integrity assumption today; where to resume: `crates/provenance/src/lib.rs:88-91`
- [ ] [added 2026-06-14 20:09 UTC] **[MH-3 / NR-1]** Wire a real post-fix re-collection to replace the bootstrap echo `signature_of(&collect_diagnostics(&args.describe))` so the verdict reflects a genuine post-state diff and `ResolvedConfirmed` cannot be trivially self-minted — where to resume: `crates/support-agent/src/main.rs:558-559`
- [ ] [added 2026-06-14 20:09 UTC] **[MH-2 / EI-01]** Add a provenance/lane pin and bind the `verify.rs` Verdict + `VerificationClass` + recurring-symptom diff into `Contribution`/`Outcome` so resolved rows are auditable against their own evidence — where to resume: `crates/corpus-client/src/schema.rs:76-101`
- [ ] [added 2026-06-14 20:09 UTC] **[EI-03 / A5]** Add a run-independence guard to confirmation aggregation keyed on `run_id`/lane, with a test that a duplicate row does not inflate the count (today `confirmations_aggregate_per_plan` submits the identical row twice and asserts `confirmations==2`) — where to resume: `crates/corpus-client/src/store.rs:39-50,411-423`
- [ ] [added 2026-06-14 20:09 UTC] **[MH-4 / MH-8 / EI-06]** Add per-row tamper-evidence (signature or hash chain) + an owner-only revocation/retraction list to `FileCorpus`; re-verify on `FileCorpus::open`; have `fix_mappings` honor revocation and let `OutcomeLabel::Reopened` demote a prior resolved mapping (the T-104 "retracted claim must not become truth" case) — where to resume: `crates/corpus-client/src/store.rs:26-53,136-157,181-197`
- [ ] [added 2026-06-14 20:09 UTC] **[MH-6 / A7]** Derive `config_class` from real CIM hardware/driver inventory (or BOM revision) instead of OS+ARCH, attested to the producing machine — retrieval scoping is weaker than documented until then; where to resume: `crates/support-agent/src/main.rs:742-747`
- [ ] [added 2026-06-14 20:09 UTC] **[MH-5]** Validate model-generated steps (claimed-risk-vs-actual-action reconciliation) and de-identify at generation; add inference-channel provenance (no cert pinning / endpoint / model attestation today) so a swapped endpoint is visible on the row — where to resume: `crates/support-agent/src/main.rs:878-886`
- [ ] [added 2026-06-14 20:09 UTC] **[Sandbox evidence]** Provide a production `SandboxValidator` impl (the `swarm` trait has none; the CLI hardcodes `sandbox_validated=false`, `main.rs:376`) and decide whether sandbox evidence is bound into the row — so "unvalidated equals escalate" is backed by positive validation evidence
- [ ] [added 2026-06-14 20:09 UTC] **[Research tree — fill]** Fill `docs/research/{claims,prereg-control-lane,instrumentation-inventory}.md` (scaffolded this session) following the commit-ordering discipline: `negative-results.md` must be committed before `claims.md`, and `prereg-control-lane.md` before any corpus row carries a `lane` field (else VOID) — where to resume: `docs/research/`
- [ ] [added 2026-06-14 20:09 UTC] **[Custody activation]** Decide whether to run `git config core.hooksPath scripts/githooks` to activate the corpus/weights pre-commit exfil guard (dormant in fresh clones — `core.hooksPath` is unset), and extend `SECURITY.md`'s invariant list to name each new evidence-integrity gate so a bypass is a reportable security issue — why deferred: changes git behavior for every future commit; owner's call (safe to enable — fmt-check only touches Rust, gitleaks only staged)

### WSL-ephemeral / agent-ops optional hardening (from `docs/wsl-ephemeral-state-policy.md`)

The durability contract is complete as-is; these are optional tightenings.

- [ ] [added 2026-06-14 20:09 UTC] Provision a bot PAT scoped to `cec-support-agent` `contents:write`, placed in `/mnt/e/secrets/cec-bot.env` (survives-WSL) and consumed via `ops/secrets/load-secrets.sh` — so the `ops/agent-handoff` push authenticates as the bot instead of the owner `gh` login — why deferred: gh credential fallback works today; PAT is a least-privilege/auditability upgrade, not a blocker
- [ ] [added 2026-06-14 20:09 UTC] Add a cargo-shaped `ops/provision.sh` (cargo build/test/clippy + githook install) so disaster recovery is one idempotent script — why deferred: do NOT copy CEC-Platform's KiCad/CUDA/broker provisioner; this repo is a Rust workspace and needs its own
- [ ] [added 2026-06-14 20:09 UTC] Add claude-rc survivability units (tmux + `systemd --user claude-rc@.service` with `Restart=always`, `rc-recover.sh`) repointed to the AutoDiagnoser ops path, so a dropped WSL console never orphans the agent — why deferred: nice-to-have resilience layer, independent of the durability contract
