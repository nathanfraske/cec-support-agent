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
- [x] [added 2026-06-14 21:05 UTC · closed 2026-06-15 02:40 UTC → DONE in PR #2 Increment 9 (commit `7c5d9b3`): `gen-signoff-key` generates+persists the keypair; `CEC_SIGNOFF_PUBKEY` configures the store (`.with_authority`) to ENFORCE; `CEC_SIGNOFF_SEED` self-attests; enforce-without-seed refuses the write] **[MH-1 — operator/CLI wiring]** Wire the sign-off authority into the `support-agent` CLI / embedders: a way to (a) generate + persist an authority key pair (seed on the Windows/durable side, public key embedded), (b) configure the corpus store with `.with_authority(pubkey)` from config/env (e.g. `CEC_SIGNOFF_PUBKEY`), and (c) actually PRODUCE an attestation at sign-off time via a real human/verifier action that holds the seed — NOT both keys in the engine process (that would defeat the asymmetric guarantee). Until wired, the mechanism is library-only (used by embedders/tests). Where to resume: `crates/support-agent/src/main.rs` (store construction + record_outcome) and `Contribution::attested_by`.
- [ ] [added 2026-06-14 21:05 UTC] **[MH-1 — key rotation + audit log]** Decide authority key rotation (support multiple trusted public keys / a key id → key registry so a rotated key still verifies historical rows) and an audit log of attestations (which authority signed which row, when). Today a store trusts exactly one authority public key. Where to resume: `crates/provenance/src/lib.rs` (SignOffPublicKey set) + `crates/corpus-client/src/gate.rs` (ensure_attested).
- [ ] [added 2026-06-14 21:05 UTC] **[MH-1 — verifier vs human authorities]** Currently one authority key attests any level (the tuple includes `sign_off`, so the level is bound, but a single key signs both VerifierConfirmed and HumanConfirmed). Consider distinct verifier-authority vs human-authority keys so the two trust tiers are cryptographically separable. Where to resume: `crates/corpus-client/src/gate.rs` ensure_attested.
- [x] [added 2026-06-14 20:09 UTC · closed 2026-06-15 02:40 UTC → DONE in PR #2 Increment 5 (commit `02635e3`): `provenance::canonical()` is now deterministic + serde-independent (explicit field names + length-prefixed free text) and the `serde_json` dep was dropped, so plan/attestation signatures no longer depend on struct field order. A cross-LANGUAGE canonical-JSON form remains, tracked as `[chain_hash canonical encoding]` below] **[Canonicalization]** Replace serde field-order canonicalization with a sorted/canonical-JSON encoder before signatures are cross-version/cross-language verified — fragile integrity assumption today; where to resume: `crates/provenance/src/lib.rs:88-91`
- [ ] [added 2026-06-14 20:09 UTC] **[MH-3 / NR-1]** Wire a real post-fix re-collection to replace the bootstrap echo `signature_of(&collect_diagnostics(&args.describe))` so the verdict reflects a genuine post-state diff and `ResolvedConfirmed` cannot be trivially self-minted — where to resume: `crates/support-agent/src/main.rs:558-559`
- [x] [added 2026-06-14 20:09 UTC · closed 2026-06-14 20:41 UTC → partially done in `feat/agent-ops-evidence-integrity` (Increment 1); remainder re-filed below] **[MH-2 / EI-01]** Add a provenance/lane pin and bind the `verify.rs` Verdict + `VerificationClass` + recurring-symptom diff into `Contribution`/`Outcome` so resolved rows are auditable against their own evidence — where to resume: `crates/corpus-client/src/schema.rs:76-101`
- [x] [added 2026-06-14 20:41 UTC · closed 2026-06-15 02:40 UTC → DONE in PR #2 Increment 3 (commit `9efaa20`): (a) `VerificationClass` is carried on the row via `Verification.class`; (b) `RowProvenance {run_id, retrieval_first, primed_from}` is bound on the row and into the attestation] **[MH-2 / EI-01 — remainder]** The `verify.rs` Verdict + recurring-symptom diff are now bound into the row (`Outcome.verification: Option<common::Verification>`), and the gate rejects a resolved label without a matching passing verdict. STILL TODO [superseded by the closure note at the head of this entry — (a) and (b) are both verified in code]: (a) carry the `VerificationClass` (deterministic/intermittent/hardware) onto the row too; (b) add the provenance/lane pin (`retrieval_first`, `primed_from_plan_ids`, `run_id`) derived from observable facts so a confirmation's origin is auditable. — where to resume: `crates/common/src/verification.rs` + `crates/corpus-client/src/schema.rs`
- [x] [added 2026-06-14 20:09 UTC · closed 2026-06-15 02:40 UTC → DONE in PR #2 Increment 3 (commit `9efaa20`): confirmation aggregation now keys on `run_id` (`store.rs` `run:{}`), so a re-submitted or self-primed row cannot inflate the count] **[EI-03 / A5]** Add a run-independence guard to confirmation aggregation keyed on `run_id`/lane, with a test that a duplicate row does not inflate the count (today `confirmations_aggregate_per_plan` submits the identical row twice and asserts `confirmations==2`) — where to resume: `crates/corpus-client/src/store.rs:39-50,411-423`
- [x] [added 2026-06-14 20:09 UTC · closed 2026-06-15 02:40 UTC → DONE in PR #2 Increment 4 (commit `8cc57a8`), audit-hardened in `11f0609`: `RowIntegrity` sha256 hash-chain on `FileCorpus`, re-verified on open; owner-only `.with_revoked`; `OutcomeLabel::Reopened` demotes a prior resolved mapping (T-104). Keyless-chain anchor residual tracked as `[Chain integrity]` below] **[MH-4 / MH-8 / EI-06]** Add per-row tamper-evidence (signature or hash chain) + an owner-only revocation/retraction list to `FileCorpus`; re-verify on `FileCorpus::open`; have `fix_mappings` honor revocation and let `OutcomeLabel::Reopened` demote a prior resolved mapping (the T-104 "retracted claim must not become truth" case) — where to resume: `crates/corpus-client/src/store.rs:26-53,136-157,181-197`
- [x] [added 2026-06-14 20:09 UTC · closed 2026-06-15 02:40 UTC → ADDRESSED via the seam: Increment 8 (`cd2db18`) added the `host_inventory()` point; P0 (`d61b962`) added `InventoryProvider` + `--inventory-keys` so an external inventory tool (AllMyStuff) supplies real, identity-free config keys → honest `config_class`. Residual: engine-NATIVE Windows CIM enrichment, re-filed below] **[MH-6 / A7]** Derive `config_class` from real CIM hardware/driver inventory (or BOM revision) instead of OS+ARCH, attested to the producing machine — retrieval scoping is weaker than documented until then; where to resume: `crates/support-agent/src/main.rs:742-747`
- [x] [added 2026-06-14 20:09 UTC · closed 2026-06-15 02:40 UTC → core DONE in PR #2 Increment 6 (`32ccb20`): `Dispatcher::reconcile_risk` raises any step whose model-claimed risk understates its tool's real risk, wired before the consent gate. Residual: de-id-at-generation + inference-channel provenance, re-filed below] **[MH-5]** Validate model-generated steps (claimed-risk-vs-actual-action reconciliation) and de-identify at generation; add inference-channel provenance (no cert pinning / endpoint / model attestation today) so a swapped endpoint is visible on the row — where to resume: `crates/support-agent/src/main.rs:878-886`
- [x] [added 2026-06-14 20:09 UTC · closed 2026-06-15 02:40 UTC → core DONE in PR #2 Increment 10 (`2d48299`): `sandbox_validated_for` feeds a real disposable-VM validator's result into escalation (clean apply = positive evidence); the CLI wires `None` (conservative default). Residual: a production SandboxValidator impl, re-filed below] **[Sandbox evidence]** Provide a production `SandboxValidator` impl (the `swarm` trait has none; the CLI hardcodes `sandbox_validated=false`, `main.rs:376`) and decide whether sandbox evidence is bound into the row — so "unvalidated equals escalate" is backed by positive validation evidence
- [ ] [added 2026-06-14 20:09 UTC] **[Research tree — fill]** Fill `docs/research/{claims,prereg-control-lane,instrumentation-inventory}.md` (scaffolded this session) following the commit-ordering discipline: `negative-results.md` must be committed before `claims.md`, and `prereg-control-lane.md` before any corpus row carries a `lane` field (else VOID) — where to resume: `docs/research/`
- [ ] [added 2026-06-14 20:09 UTC] **[Custody activation]** Decide whether to run `git config core.hooksPath scripts/githooks` to activate the corpus/weights pre-commit exfil guard (dormant in fresh clones — `core.hooksPath` is unset), and extend `SECURITY.md`'s invariant list to name each new evidence-integrity gate so a bypass is a reportable security issue — why deferred: changes git behavior for every future commit; owner's call (safe to enable — fmt-check only touches Rust, gitleaks only staged)

#### Residuals re-filed from the 2026-06-15 reconciliation (the partial items above whose CORE landed in PR #2)

- [ ] [added 2026-06-15 02:40 UTC] **[MH-6 / A7 — engine-native CIM enrichment]** The seam exists (an external tool can supply real inventory keys via `--inventory-keys`), but the engine's OWN `CoarseHostInventory` still derives only os/arch/family. A Windows build SHOULD enrich it under `cfg(windows)` with CIM **configuration** fields (board vendor/model, BIOS version/date, chipset, GPU model, driver versions) — never serials/service tags — so a standalone engine run scopes retrieval to genuinely-like hardware. — why deferred: needs a Windows host to build+verify `cfg(windows)` code; the config_class is already attestation-bound so its derivation is tamper-evident regardless. Resume: `crates/common/src/inventory.rs` (a `cfg(windows)` provider) + `crates/support-agent/src/main.rs` (`host_config_class`).
- [ ] [added 2026-06-15 02:40 UTC] **[MH-5 — de-id-at-generation + inference-channel provenance]** Risk reconciliation landed; still open: (a) de-identify model-generated step prose at generation time (so identity in a model's free text never reaches a row); (b) inference-channel provenance — cert-pin / attest the endpoint + model so a swapped inference endpoint is visible on the row. — why deferred: needs the inference integration (MyOwnLLM, RFC Q2) to be designed first. Resume: `crates/support-agent/src/main.rs` (generation path).
- [ ] [added 2026-06-15 02:40 UTC] **[Sandbox evidence — production validator]** A production disposable-VM `SandboxValidator` impl (the `swarm` trait has none; the CLI wires `None`). Once present, a clean apply in a real sandbox becomes positive evidence that can lower an escalation. — why deferred: infrastructure (a VM/snapshot backend); RFC Q4 asks whether a MyOwnMesh peer could BE the sandbox. Resume: implement `SandboxValidator` for a real backend + wire it in `main.rs`.
- [ ] [added 2026-06-15 02:40 UTC] **[MH-3 / NR-1 — real post-fix re-collection]** (left OPEN above, noted here for completeness) The self-mint risk is mitigated (Increment 7's `Unverified` verdict escalates an unobserved outcome), but the genuine post-fix re-collection (`recollect_post_signature` returns `None` in the bootstrap) still needs the Windows backend so the verdict reflects a real post-state diff. Resume: `crates/support-agent/src/main.rs` `recollect_post_signature`.

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
- [ ] [added 2026-06-15 01:25 UTC] **[HttpCorpus read-path is unverified]** `HttpCorpus::query`
  (`crates/corpus-client/src/store.rs:425-453`) returns the server's `FixMapping`s with NO `admit()` and NO
  attestation check — the submit path is gated but the **read** path trusts the corpus server entirely. A
  malicious/compromised server could feed forged precedents the engine uses retrieval-first. Surfaced by the
  MyOwn-integration design. Fix: the new `MeshCorpus` MUST re-verify the ed25519 attestation on every received
  row (planned, P3 acceptance (d) in `docs/integration-myown-family.md`); apply the same hardening to
  `HttpCorpus` (carry attestation-bearing rows on the query path, or re-verify). — where to resume:
  `crates/corpus-client/src/store.rs` (HttpCorpus::query) + the mesh adapter.
- [x] [added 2026-06-14 23:05 UTC] **[chain_hash canonical encoding]** `chain_hash` now carries a version
  prefix (`cec-corpus-chain-v1`) but still hashes the `serde_json` image of the row — coupled to struct
  field order, fine for same-code recompute but not cross-language. If the chain ever needs external
  verification, switch it to the serde-independent canonical encoder used for the attestation/plan
  signatures. — where to resume: `crates/corpus-client/src/schema.rs` (chain_hash). · closed 2026-07-04
  19:05 UTC → F2 landed on the migration-bundle PR (`92df52d`): `chain_canonical` explicit field-by-field
  length-prefixed encoding, domain tag `cec-corpus-chain-v2`, serde-independence pinned by a hand-assembled
  canonical-bytes test + a 25-mutation binding sweep; v1-era files refused at open (hard cutover).
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

### MyOwn-family integration (P0 landed in-tree; P1+ and cross-repo decisions deferred)

- [x] [added 2026-06-15 02:00 UTC · closed 2026-06-15 02:10 UTC → owner chose "push both"; fmt fix `538cd43` + the 2 doc commits fast-forwarded to `origin/feat/agent-ops-evidence-integrity` (`920a..538cd43`); PR #2 CI re-running. Verify green via `gh pr checks 2`] **[PR #2 is RED on CI — fmt regression]** `11f0609` (on PR #2's branch `feat/agent-ops-evidence-integrity`) introduced 4 rustfmt-1.9 wrapping violations in `crates/corpus-client/src/{schema.rs,store.rs}`; CI runs `cargo fmt --all -- --check`, so all `check` jobs (ubuntu/macos/windows) fail and PR #2 cannot merge. FIXED in the working tree and isolated as a **portable fmt-only commit** on `feat/myown-integration-p0` so it can be cherry-picked onto PR #2's branch.
- [x] [added 2026-06-15 02:10 UTC · closed 2026-06-15 02:22 UTC → TRIAGED (workflow `wf_60234519-881`): root cause = missing `GITHUB_TOKEN` env (gitleaks-action@v2 breaking change for PR-event scans), NOT a leak — gitleaks full-history + an independent 10-method cross-check both `all_clear`. Fix identified (token env + permissions block + `checkout@v5`/`gitleaks-action@v3` for the Node deadline). Apply pending owner's branch/push routing — see the new CI-fix items below.] **[CI `secrets` job fails fast — pre-existing, not P0]** The `secrets` job (`gitleaks/gitleaks-action@v2`, `.github/workflows/ci.yml`) fails in ~4s on PRs #2 and #3 — and was ALREADY failing on PR #2 before any P0/fmt change, so it is NOT a regression from this work.
- [x] [added 2026-06-15 02:22 UTC · closed 2026-06-15 03:10 UTC → DONE in commits `673a381` (PR #3) / `b7ad864` (PR #2): added the `concurrency` block, swapped the audit job to `EmbarkStudios/cargo-deny-action@v2` (honors `deny.toml`), SHA-pinned all third-party actions (checkout/gitleaks/cargo-deny/rust-toolchain), and added `.github/dependabot.yml`. NOT done (intentional, has a trade-off and was outside the chosen scope): scoping `on: push` to `branches:[main]` — push+PR still both trigger; the concurrency block only cancels SUPERSEDED runs on the same ref, not push-vs-PR. Re-open that sub-item if the duplicate push/PR runs become annoying.] **[CI hygiene — nice-to-haves, no deadline]** (1) concurrency block; (2) cargo-deny-action; (3) SHA-pin + dependabot.
- [ ] [added 2026-06-15 02:00 UTC] **[`--json --sign-off` post-execution envelope]** The `cec-diagnose/v1` envelope is emitted pre-execution with `"executed": false`; under `--sign-off` the run then executes/records but does NOT emit a second, post-execution envelope (recorded outcome label, verification verdict). — why deferred: P0's use case is the **diagnose-only** path (`diagnose --json`, no `--sign-off`), per RFC D1 single-shot; a post-exec envelope is P2+ when AllMyStuff drives the execute phase. Resume: emit a terminal envelope after `record_outcome` carrying the label/verdict, or define a separate `cec-execute/v1` schema.
- [ ] [added 2026-06-15 02:00 UTC] **[Daemon mode — later]** RFC D1 chose a single-shot CLI (AllMyStuff spawns `cec-support-agent diagnose --json` per diagnosis). A persistent daemon (warm corpus, lower per-call latency) is explicitly deferred until latency demands it. — why deferred: single-shot is simplest and has nothing to orphan; revisit only if profiling shows spawn/corpus-load cost matters.
- [x] [added 2026-06-15 02:00 UTC] **[RFC Q1–Q5 awaiting Chris/owner]** `docs/integration-rfc-for-chris.md` poses 5 open questions that gate P3/P4: Q1 identity unification (one ed25519 seed for mesh DeviceId + corpus sign-off, or split), Q2 mesh inference exposing raw (non-de-identified) prose to a peer, Q3 the `myownmesh-core` pin single-source-of-truth, Q4 `MeshSandboxValidator` now-or-later, Q5 tail-truncation anchor distribution to mesh peers. — why deferred: pending the owner's review + Chris's input ("implementing correctly the first time"); P1/P2 (app-side de-id allowlist + serde-only contract) do not need them and can start first.
  · closed 2026-07-04 20:35 UTC → resolved by the owner's D3 integration-posture decision + Q1
  operator-half decision (RFC, 2026-07-04): Q1 fully DECIDED (separate keys), Q2 decided-for-now
  (local inference; no MyOwnLLM), Q3 MOOT (no myownmesh-core link), Q4 DEFERRED, Q5 REFRAMED into
  the engine's B4/corpus-service wire contract. Nothing hard-blocks on Chris anymore; his side
  only adds an API client in AllMyStuff whenever tier-2 lands.

### Corpus leak-prevention (methodology Phases 0–2 in flight on `feat/corpus-leak-prevention`)

- [ ] [added 2026-06-15 03:55 UTC] **[Private `corpus-ingest` adapts to `Contribution::new -> Result`]** Phase 0 changed `corpus-client::Contribution::new` and `de_identify_plan` to return `Result<_, deid::Reject>`. The PRIVATE repo `/mnt/e/cec-corpus-private` `corpus-ingest` crate calls these (it git-deps the engine). It will fail to build when it next bumps the engine pin past `cf95d1c`. — why deferred: the private repo is a separate workstream and pins a fixed engine rev; it adapts on the next deliberate bump. Resume: in `corpus-ingest`, propagate the `Result` (the YAML→row compile already validates symptoms via a single-token charset, so a clean flow will mint OK; an inadmissible action/id now hard-fails the compile, which is the desired behavior).
- [x] [added 2026-06-15 03:55 UTC · closed 2026-07-02 19:15 UTC → Phase 1 DONE on `claude/repo-scope-work-plan-h93qx5` (`a347878`/`3790dbd`/`22ec564`/`9a9cd5b`, see TODOS "Session 2026-07-02 — leak Phase 1"); Phase 2 residuals re-filed below] **[Phase 1–2 of the leak methodology remain]** Phase 0 DONE (`cf95d1c`, now on main). Phase 1 (type split + `Prose` leaf typing + sealed `Debug` + private `Contribution` fields + `trybuild` + write-gate idempotence) is now DONE. Phase 2 (read-side `from_served` + frozen dictionaries + ban `serde_json::Value`) remains — re-filed below.

#### Corpus leak-prevention Phase 1 DONE — Phase 2 residuals + Phase-1 design deferrals (2026-07-02 19:15 UTC)

- [x] [added 2026-07-02 19:15 UTC · closed 2026-07-02 21:30 UTC → Phase 2 DONE on `claude/repo-scope-work-plan-h93qx5` (`a0818bc`/`a759afd` + the 2c doc commit; see TODOS "Session 2026-07-02 — leak Phase 2"); dictionary-curation + 2c-typing residuals re-filed below] **[Phase 2 — read-side re-de-id + closed dictionaries]** The C4/C5 hard stops. DONE: (a) `#[serde(try_from = "String")]` on `StoredSymptom`/`StoredAction`/`StoredPlanId`/`common::Symptom` — an out-of-vocab action, inadmissible id, or non-grammar symptom fails to *deserialize* at `HttpCorpus::query` (transport/admission split → `ServedPlanInadmissible`) and `FileCorpus::open` (Storage parse error); the `de_identify_plan` equality check stays as a Layer-2 guard for the derived (non-leaf-typed) `title`; adversary-seeded read-path poison harness added; (b) FROZEN `STOP_CODE_NAMES`/`MODULE_NAMES` dictionaries + closed-grammar `is_symptom_token` replace the shape heuristics; (c) `serde_json::Value` scoped honestly — see the 2c residual below.
- [x] [added 2026-07-02 19:15 UTC · closed 2026-07-02 21:30 UTC → Phase 2 (`a0818bc`/`a759afd`)] **[Phase 1 residual — strict symptom mint on the write path]** DONE. The closed-grammar `is_symptom_token` admits a legitimate `<prefix>_<digits>` token (`event_41`) directly — the blocker for the old round-trip mint — so `deid::symptom` is now sound and the 1f gate (`ensure_evidence_integrity`) re-validates every stored symptom (signature + `verification.recurring`) via `common::is_symptom_token`, refusing an embedder-built or hand-edited identity-shaped symptom with `GateError::SymptomNotDeIdentified`.
- [x] [added 2026-07-02 19:15 UTC · closed 2026-07-02 21:30 UTC → Phase 2 (`a759afd`); resolved in place, NOT via StoredVerification] **[Phase 1 residual — Verification.recurring still uses raw Symptom]** RESOLVED by validating `common::Symptom` in place rather than migrating to `StoredSymptom`. `common::Verification.recurring: Vec<Symptom>` cannot use `StoredSymptom` (that would need `common` → `corpus-client`, the wrong dependency direction / a cycle). Instead `common::Symptom` gained `#[serde(try_from = "String")]` validating `is_symptom_token`, so `recurring` is validated on read AND the 1f gate re-checks it. `Symptom` keeps `Serialize` (it is the in-flight type; the wire stays a bare string, byte-identical). No `StoredVerification` needed.
- [ ] [added 2026-07-02 19:15 UTC] **[Phase 1 note — Contribution fields are pub(crate), not fully private]** Deliberate divergence from the doc's "private fields": the fields are `pub(crate)` so the de-id/gate code and the in-crate adversarial tamper tests read/forge them directly, while an EXTERNAL struct-literal still fails to compile (pinned by the `contribution_struct_literal` trybuild case → E0451). `Contribution::new` is the sole external constructor — the threat model (an embedder bypassing the mint) is fully covered. If a strict "no in-crate struct literal either" posture is wanted later, make them private + add `#[cfg(test)]` mutators. No action needed unless that posture changes.
- [ ] [added 2026-07-02 19:15 UTC] **[Phase 1 note — agent-loop/model prose leaves stay String]** `ToolOutcome.summary` and `AgentRun.answer`/`AgentStep` are NOT in the 1b `Prose` list, so they stay `String` (their Debug still shows them). These are the model-inference / agent-loop egress (leak class C2), an accepted-risk boundary handled by §3.1 / Phase 4 (`PromptPayload` chokepoint + `--allow-remote-inference`), not the corpus/print sinks Phase 1 seals. `ToolOutcome`/`AgentRun` did lose `Serialize` this phase. Resume: methodology §3.1 / Phase 4.
- [ ] [added 2026-07-02 19:15 UTC] **[Phase 1 downstream — corpus-ingest additional breakages]** Beyond the `-> Result` change (item above), Phase 1 also: `de_identify_plan` now returns `StoredPlan` (not `Plan`); `Contribution` fields are `pub(crate)` (no struct literal); `Outcome`/`Plan` lost `Serialize`. The private `/mnt/e/cec-corpus-private` `corpus-ingest` will need these adaptations on its next engine-pin bump — it builds a raw `Outcome` and calls `Contribution::new` (that path is unchanged), but if it reads `de_identify_plan`'s result as a `Plan` or serializes a raw domain type, it must switch to the stored types/accessors. Resume: `corpus-ingest` on the next pin bump.

#### Corpus leak-prevention Phase 2 DONE — new residuals (2026-07-02 21:30 UTC)

- [ ] [added 2026-07-02 21:30 UTC] **[Phase 2 residual — the frozen dictionaries are conservative and CODEOWNERS-curated]** `STOP_CODE_NAMES` (~42 Microsoft bugcheck names) and `MODULE_NAMES` (~68 OS/driver modules) in `crates/common/src/extract.rs` are a decision-ready but INCOMPLETE curated set. Per methodology §6, C5 is "strong but dictionary-dependent": a missing real bugcheck/module is a false-negative (a genuine symptom silently dropped from the signature → weaker retrieval), an over-broad entry is a leak. They need CODEOWNERS-gated curation as real crash evidence surfaces new legitimate names — expand deliberately, never widen to a shape. The `is_symptom_token`/`extract_symptoms` self-consistency test guards the grammar; a snapshot test against the private `vocabulary.yaml` would catch engine↔corpus drift (Phase-4/L4 CODEOWNERS territory). Resume: `crates/common/src/extract.rs` dictionaries + `docs/corpus-leak-prevention.md` §6.
- [ ] [added 2026-07-02 21:30 UTC] **[Phase 2 residual (2c) — type ToolOutcome.data / AgentStep.args into allowlisted summaries]** DEFERRED (not skipped-and-forgotten): the two untyped `serde_json::Value` fields have NO serialize/print sink post-Phase-1 (their types lost `Serialize`), so the 2c *serialization* boundary is already closed and typing them buys nothing for the corpus/print surface. Their residual exposure is the agent-loop / model-prompt egress (leak class C2) — the accepted-risk boundary handled by §3.1 / Phase 4 (`PromptPayload` chokepoint + `--allow-remote-inference`). Type them into allowlisted summaries WHEN the PromptPayload chokepoint lands (same C2 surface, one change). The invariant is documented on both fields so a re-add of `Serialize` is a visible leak. Resume: methodology §3.1 / Phase 4; `crates/agent-core/src/{tool.rs,agent.rs}`.
- [ ] [added 2026-07-02 21:30 UTC] **[Phase 2 downstream — corpus-ingest stored-type field changes]** Phase 2 changed the stored payload field TYPES: `StoredStep.action`/`.description` are now `StoredAction`, `StoredPlan.id` is `StoredPlanId`, and `StoredSymptom`/`common::Symptom` validate on deserialize. `de_identify_plan`/`Contribution::new` signatures are unchanged and `StoredPlan::from_minted` is `pub(crate)`, so the private `corpus-ingest` (which compiles YAML → `Outcome` → `Contribution::new`) is unaffected UNLESS it deserializes a stored row containing a symptom/action/id its own YAML lint would not itself produce (in which case the stricter engine guard is the desired behavior — a leaky flow now hard-fails at load, not just at compile). Resume: `corpus-ingest` on the next engine-pin bump.

- [ ] [added 2026-07-02 16:20 UTC] **[HttpCorpus/MeshCorpus — attestation on the READ wire]** `HttpCorpus::query` now re-validates served plans (de-id mints + idempotence, fails closed), but cryptographic re-verification of row attestations is impossible on this path: the `FixMapping` aggregate carries no attestation. The corpus-service wire contract (P3 `MeshCorpus` / any future corpus service) must serve attested rows (or mappings + their backing attested rows) so the client can run `ensure_attested` per row — P3 acceptance (d) already requires this for the mesh. Resume: `crates/corpus-client/src/store.rs` (query), the P3 adapter design in `docs/integration-myown-family.md`.
- [ ] [added 2026-07-02 16:20 UTC] **[serve ↔ run pipeline glue unification]** `crates/support-agent/src/serve.rs` re-composes the diagnose pipeline from the same building blocks as `run()` (headless), so ~80 lines of orchestration glue exist twice. The security invariants live in the shared functions (gate, reconcile_risk, required_escalation, execute_signed_plan, record_outcome), not the glue — but unify when the leak-Phase-1 type refactor reshapes these paths anyway. Resume: extract a shared `diagnose_core()` both callers use.
- [ ] [added 2026-07-02 16:20 UTC] **[serve hardening — later]** Per-request endpoint/model overrides (currently server-start flags); request body size limits + timeouts; structured logs; graceful shutdown. None block P1' (the AllMyStuff API client) — the wire contract is stable additive-only.

#### API-posture decisions (owner, 2026-07-02 22:30 UTC) — see TODOS "Session 2026-07-02 — API-posture decisions"

- [x] [added 2026-07-02 22:05 UTC · closed 2026-07-02 22:15 UTC → BUILT on `claude/repo-scope-work-plan-h93qx5` (`697e16d`); leak-doc §3.1 item 1(b) + Phase 4 item 14 annotated] **[leak §3.1(b) — `--endpoint` localhost-allowlist + `--allow-remote-inference`]** The pragmatic-minimum half of the C2 architectural decision is built: `validate_inference_endpoints` refuses a non-loopback `--endpoint`/`--fast-endpoint` on both the diagnose and serve paths unless `--allow-remote-inference` is passed; the refusal never echoes the URL and fails closed on an unparseable host. The `PromptPayload` chokepoint (§3.1 item 1(a)) remains the type-level follow-on — see the accepted-risk C2 items above and Phase 4.
- [ ] [added 2026-07-02 22:30 UTC] **[leak §3.1(a) — `PromptPayload` chokepoint]** The type-level half of C2: a sealed `PromptPayload` whose constructor builds `ChatMessage` user/system content only from de-identified fields (vocabulary symptoms, enum tags, `case_brief()`), so raw `describe`/`event.message` cannot reach a prompt by construction. §3.1(b) (the endpoint allowlist, now built) makes remote egress an audited act; (a) is what makes the payload itself de-identified. — why deferred: needs the inference integration (MyOwnLLM, RFC Q2) designed first; ride it with the ToolOutcome.data / AgentStep.args typing (same C2 surface). Resume: `crates/inference` (`ChatMessage`), `crates/support-agent/src/main.rs` (ModelGenerator), leak methodology §3.1 / Phase 4.
- [ ] [added 2026-07-02 22:30 UTC] **[corpus-over-API — the bar is now recorded, not the code]** `docs/api-extension-design.md` §5 fixes the posture for a future corpus endpoint: mesh rostered identity or loopback only, never token-auth public HTTP; per-row attestation on the served wire (the `HttpCorpus::query` FixMapping read-wire gap above / B4 must close FIRST); encrypted transport (mesh / TLS). No endpoint exists; the `router_surface_is_frozen` pin is the mechanical guard that one is not added without meeting this bar. Resume: gate on B4 + Q1 (mesh key registry) per api-extension-design §1.1.

### Corpus cartography (leak-C10) — deferred controls (threat doc landed 2026-07-02)

The owner-raised "can a surface expose the internal corpus by mapping it out through trusted calls?"
question was analyzed in `docs/corpus-cartography-threat.md` (companion to `docs/corpus-leak-prevention.md`
§1.2 leak-C10 and the `AGENTS.md` non-mappability rule set). The `source` membership label was dropped from
`cec-diagnose/v1` the same session (control D, partial — see TODOS). These are the controls the threat doc
recommends that are NOT yet built, each attributed to the threat doc's §3 control lettering.

- [ ] [added 2026-07-02 18:53 UTC] **[Corpus cartography — control D remainder, latency/slate-count
  equalization]** Retrieval-first hit/miss **latency** and **slate-count** differentials (vector V3) survive
  the `source`-label removal: a hit skips model generation (faster) and the `corpus_primed` candidate count
  discloses fix coverage even without the label. Equalizing timing costs the retrieval-first speed win — a
  genuine owner trade-off, not a bug. — why deferred: only bites once the surface serves a non-owner (rung-0
  loopback self-use is not a threat); decide before/at E3 (mesh serving). Resume:
  `docs/corpus-cartography-threat.md` §2 V3, §3 control D; `crates/support-agent/src/serve.rs`
  retrieval-first branch (`serve.rs:360,365`).
- [ ] [added 2026-07-02 18:53 UTC] **[Corpus cartography — control A, per-identity query budget/rate-limit]**
  No per-caller query budget exists today (`MAX_SESSIONS=256` is a pending-session memory bound, not a
  throughput knob — `serve.rs:90-92`); a caller issuing sequential diagnoses (or letting sessions TTL-expire)
  can enumerate the corpus for free. — why deferred: the full per-identity form needs the mesh `Identity`
  (rung-2/E3); a coarse per-process cap is greenlightable sooner if wanted. Resume:
  `docs/corpus-cartography-threat.md` §3 control A; `serve` middleware + `serve_corpus`; make it an E3
  acceptance criterion alongside attested-rows-only + encrypted transport.
- [ ] [added 2026-07-02 18:53 UTC] **[Corpus cartography — control B, per-identity query audit log]** No
  audit log of diagnose queries exists — `handle_diagnose` logs nothing about the query itself, so bulk
  enumeration is neither attributable nor detectable after the fact. Log the hashed key + caller id +
  timestamp only, never `describe`. This is the query-side twin of the deferred MH-1 audit-log item
  (FOLLOWUPS "[MH-1 — key rotation + audit log]" above). — why deferred: a real identity to attribute to
  needs rung-2/E3; a hashed-key+timestamp skeleton is greenlightable now. Resume:
  `docs/corpus-cartography-threat.md` §3 control B; `serve`/`serve_corpus`.
- [x] [added 2026-07-02 18:53 UTC] **[Corpus cartography — control E, keyed/salted HMAC fingerprint]** The
  fingerprint and config-class key are unsalted FNV-1a — dictionary-reversible, and (per leak-C7) the exact
  reason the cartography probe space is enumerable rather than opaque (a caller can compute-then-probe keys
  offline against the planned `POST /v1/corpus/query`). This is the SAME item as the existing leak-C7
  residual (`docs/corpus-leak-prevention.md` §3.1(2)) / Phase-4 F-track — greenlightable, pull it forward; it
  is the non-mappability prerequisite that makes the probe space opaque. — why deferred: not yet scheduled;
  no blocker. Resume: `crates/common/src/hash.rs` (`fingerprint_of`/`from_inventory` → keyed HMAC,
  per-deployment salt); move retrieval keys out of logged/GET URLs into request bodies (`store.rs:434-439`).
  · closed 2026-07-04 19:05 UTC → BOTH halves landed on the migration-bundle PR (`e17f38f` keyed
  HMAC-SHA256 `cec-fingerprint-v2` + `CEC_FINGERPRINT_SALT` custody per the owner's 2026-07-03
  decision; `90ff2c2` retrieval keys into the `POST /v1/mappings/query` body). Docs: leak doc
  §3.1(2) BUILT note; cartography control E BUILT note.
- [x] [added 2026-07-02 18:53 UTC] **[Corpus cartography — control C, B4 provenance-graph minimization]** B4
  proposes serving essentially the whole `Contribution` minus `integrity`, including `RowProvenance`
  (`primed_from`, the priming graph) and possibly `confirmations` — both disclose corpus derivation/
  confirmation structure beyond a single answer (leak-C10 vectors V5/V6). Resolve by shipping only the
  minimal attested unit (attested `StoredOutcome` + attestation), never `primed_from` or raw `confirmations`,
  unless a decision log entry explicitly authorizes it. — why deferred: B4 hasn't shipped; the fix is a field
  choice, cheap NOW and expensive after B4 ships — decide it as a B4 wire-contract precondition, not after.
  Resume: `docs/corpus-cartography-threat.md` §2 V5/V6, §3 control C; the B4 served-row type
  (`docs/trusted-corpus-access-trajectory.md` §2.1). · closed 2026-07-04 20:05 UTC → DECIDED by the owner
  (RFC Q6 DECIDED note, 2026-07-04): served rows ship ONLY the minimal attested unit (attested
  `StoredOutcome` + attestation), never `primed_from`/`run_id`/`retrieval_first`/raw `confirmations`.
  The build half lives in the open "[B4 — attested read path]" item (now service-gated only).
- [x] [added 2026-07-02 18:53 UTC] **[Corpus cartography — Q6 filed against B4]** Filed the real question Q6
  ("how much provenance does a served row expose?") in `docs/integration-rfc-for-chris.md`'s open-questions
  section — the threat doc noted no Q6 existed anywhere in the tree (RFC had Q1-Q5 only). Resolution is
  control C above (provenance-graph minimization); Q6 stays open until the B4 wire contract makes that call
  explicit. — why deferred: gated on B4 shipping; owner/Chris decision. Resume:
  `docs/integration-rfc-for-chris.md` Q6. · closed 2026-07-04 20:05 UTC → Q6 DECIDED by the owner
  (provenance-graph minimization; RFC DECIDED note). One design wrinkle recorded with the decision:
  the attestation message binds the provenance pin, so a minimized served row needs a provenance
  COMMITMENT the consumer can verify against — design that with the B4 wire type.
- [ ] [added 2026-07-02 19:28 UTC] **[Crypto-dep major bumps — dependabot #8/#9/#10, deferred for one
  deliberate coordinated upgrade]** getrandom 0.2→0.3 (#8), hmac 0.12→0.13 (#9), sha2 0.10→0.11 (#10) each
  break the `provenance` crate at build: getrandom 0.3 renamed `getrandom::getrandom(&mut buf)` →
  `getrandom::fill(&mut buf)` (`crates/provenance/src/lib.rs:57,167`); sha2 0.11 bumps the `digest` trait
  version so `hmac::HmacCore<sha2::Sha256>: hmac::Mac` no longer holds (9 errors), and hmac 0.13 is the
  matching RustCrypto major (27 errors) — **#9 and #10 are COUPLED and must bump together** with `digest`.
  — why deferred: these are the signing/hashing/entropy primitives (HMAC-SHA256 plan provenance, sha256
  `chain_hash`/`content_hash`/attestation, OS entropy for keys and run_ids); a 36-error auto-patch by the
  merge babysitter is not safe. Do a deliberate, tested upgrade that bumps hmac+sha2+digest together,
  migrates the `Mac` API, and verifies `attestation_message` / `chain_hash` / `content_hash` / plan-signature
  outputs are byte-unchanged (the wire-compat + attestation tests must stay green). Resume:
  `crates/provenance/src/lib.rs` + the workspace `Cargo.toml` dep versions; dependabot PRs #8/#9/#10 left open.
- [x] [added 2026-07-02 22:42 UTC · closed 2026-07-02 23:56 UTC → PR #13: Tier-1 guards built] **[AGENTIC
  ADDENDUM — mechanical backstops]** BUILT this turn: (a) the `PreToolUse` invariant guard
  (`invariant-guard.sh`) hard-blocking a corpus/weights/seed **path** write (the re-added-oracle/Serialize
  content checks were intentionally NOT hard-blocked — a grep heuristic false-positives on our own prose;
  they moved to the PostToolUse surface + the future `projectops invariants` tool); (b) the PostToolUse
  reaction (`invariant-check.sh`, conflict markers + serialized-corpus-row + seed/key blocks, self-safe);
  the Stop tracking-freshness nudge (`tracking-freshness.sh`); and `ops/provision.sh` (Tier-0 activator).
  All validated (block/allow/surface/self-reference) and wired in `.claude/settings.json`. STILL OPEN →
  re-filed below.
- [x] [added 2026-07-02 23:56 UTC · closed 2026-07-03 00:07 UTC → PR #14: projectops built] **[AGENTIC
  ADDENDUM — projectops server (§3)]** BUILT: `tools/projectops.py` (pure-stdlib CLI keystone — `verify`
  the cargo/gitleaks suite as JSON, `invariants` the fast git/grep guards [no-exfil-tracked, source-label-
  absent (leak-C10), frozen /v1 route surface, ACTION_VOCABULARY sorted, wire-pins present, hooks
  executable], `backlog` the TODOS/FOLLOWUPS parse, `leak_scan` the de-id/poison slice) + `tools/
  projectops_server.py` (minimal MCP stdio server, raw JSON-RPC 2.0, no SDK dep) wired in `.mcp.json`.
  Validated: server initialize/tools-list/tools-call e2e; invariants pass on the real tree and provably
  bite on a re-added `source`/rogue route/unsorted vocab. STILL OPEN → re-filed below.
- [x] [added 2026-07-03 00:07 UTC · closed 2026-07-03 01:01 UTC → PR #15: panels built] **[AGENTIC ADDENDUM
  — review panels (§4)]** BUILT: `tools/projectops_panel.py` renders the `projectops` JSON into one
  self-contained, theme-aware HTML dashboard (verification / security-invariants / backlog / blind-audit
  sections; summary tiles; status pills + severity stripe; both themes; static snapshot per the CSP). Live
  instance rendered as an Artifact. Dogfooding it fixed a `verify` bug (a missing cargo SUBCOMMAND like
  `cargo deny` exits non-127 → was read `fail`; now treated `skipped`). STILL OPEN → re-filed below.
- [ ] [added 2026-07-03 01:01 UTC] **[AGENTIC ADDENDUM — remaining: Stop verify-gate, scheduled panel
  regen, deeper invariants]** (a) fold the cargo `verify` suite into the Stop gate via `projectops verify
  --checks <fast subset>` (the fast tracking-freshness half is built; the full-suite half stays out of every
  turn-end as impractical); (b) nothing yet regenerates the panel on a schedule or Stop — it is a manual
  snapshot; wire a regen (a cron, or a Stop hook writing `panel.html`); (c) deepen `projectops invariants`
  with a real `no raw type derives Serialize` check and the full `ACTION_VOCABULARY` == dispatcher-registry
  drift (today the type system + the in-tree drift test cover them). — why deferred: the panels/server are
  done; these are refinements. Resume: `docs/AGENTIC_ADDENDUM.md` §2d/§4; `tools/projectops.py`.
- [ ] [added 2026-07-03 01:22 UTC] **[doc reconcile — stale cross-reference]** `docs/corpus-cartography-
  threat.md` §2 V6 still asserts *"there is no 'Q6' defined anywhere in the tree"* and recommends filing one,
  but `docs/integration-rfc-for-chris.md` now HAS a **Q6** ("how much provenance does a served row expose?")
  — the recommendation is already actioned, so the cartography claim is stale. Reconcile: update the V6 note
  to point at the now-existing RFC Q6 (and Q7). — why deferred: caught by the fleet-design surface-map as a
  file-content disagreement, not blocking the design landing; a 2-line doc edit. Resume:
  `docs/corpus-cartography-threat.md` §2 V6 ~`:149-151`.
- [ ] [added 2026-07-03 01:22 UTC] **[fleet — the three greenlightable pure-engine items]** From
  `docs/test-validation-fleet-design.md` §5, greenlightable now (no infra, no Chris) once the owner says go:
  (1) the gated-MCP-wrapper spec over `/v1/execute` (frozen `{diagnose,execute}` verb contract + egress-sink
  inheritance); (2) the `SandboxValidator` production *contract* + a "clean report cannot mint a resolved row"
  test; (3) an execution audit-log skeleton (hashed key + plan-id + timestamp — the exec twin of the
  cartography V7 / MH-1 query-log gap). — why deferred: design-first steer; awaiting the owner's greenlight
  before any code. Resume: `docs/test-validation-fleet-design.md` §5.
- [ ] [added 2026-07-03 01:22 UTC] **[fleet — owner-gated forks Q7 + Q1]** **Q7** (plan-signing topology
  across the execution boundary: judge-on-target keeps HMAC in-process vs ed25519 with a persistent custodied
  judge key) and **Q1** (is a volunteer a rostered identity that can hold a sign-off authority, or a pure
  execution target a central authority attests) both block the access-MCP topology. — why deferred:
  owner/Chris decisions, no code depends on them yet. Resume: `docs/integration-rfc-for-chris.md` Q1/Q7;
  `docs/test-validation-fleet-design.md` §2.1 T-6 / §5.
- [ ] [added 2026-07-03 02:17 UTC] **[fleet item 3 — audit-log rung-2 wiring]** The execution audit skeleton
  (`crates/support-agent/src/audit.rs`) is in place with a `NullSink` default. Rung-2 wiring, deferred:
  (a) a **persistent, access-controlled sink** (the `NullSink` records nothing today) — the query-side twin
  is the MH-1 audit-log FOLLOWUPS item, keep them symmetric; (b) **`caller_key`** — fill it once a caller
  identity layer exists (today the loopback trust boundary is the OS user, so the field is `None`), pairs
  with Q1; (c) a **CLI seam** — the CLI passes `&NullSink` inline; add a `--audit-log`/config seam like serve's
  `AppState.audit`; (d) the **refuse path** — when `Contribution::new` rejects (leak-guard hit) no audit
  record is emitted (no admissible id to log); consider an id-less "execution refused" marker so a refusal is
  still attributable. — why deferred: skeleton is the greenlit scope; these are rung-2/identity work. Resume:
  `crates/support-agent/src/audit.rs`; `record_outcome` in `main.rs`; `docs/test-validation-fleet-design.md` §2.2.
- [ ] [added 2026-07-03 03:15 UTC] **[B4 — attested read path, Q6-gated]** `HttpCorpus::query`
  (`corpus-client/src/store.rs:470`) serves bare `FixMapping`s that carry **no attestation** (`schema.rs:86`;
  attestation is on `Contribution:198`), and the read path only re-validates the plan's de-id image. To
  re-verify the ed25519 attestation on served rows, the wire contract must serve an **attested row type**
  (attested `StoredOutcome` + attestation), which is **RFC Q6** (served-row provenance minimization — still
  OPEN) and also needs the corpus service to exist. So B4 is part of the larger corpus-over-HTTP/mesh service
  build, not a small independent item. — why deferred: on inspection it is Q6-gated + service-gated, not the
  add-a-verify-call I'd scoped. Resume: decide RFC Q6, define the served-row type, then
  `HttpCorpus::query` calls `ensure_attested`/`admit` against `self.authority`. Ref: `consolidated-work-plan.md`
  B4; `corpus-cartography-threat.md` V6; RFC Q6.
- [ ] [added 2026-07-03 01:22 UTC] **[fleet — the hard data/infra gates F4/F5 + volunteer framework]** **F4**
  real post-fix re-collection (`recollect_post_signature() -> None` stub, `main.rs`; NR-1) — until it lands
  every run is `Verdict::Unverified` and can back no resolved row, so it gates the *value* of the entire
  fleet; needs a Windows host. **F5** a production `SandboxValidator` VM backend (the seam is wired `None` in
  both callers). The **volunteer enrollment + scoped/revocable consent + legal framework** — no volunteer
  concept exists in code at all; largest greenfield, mostly policy/legal not engine. — why deferred: infra +
  legal + owner sequencing, all downstream of the greenlight items and the Q7/Q1 forks. Resume:
  `docs/test-validation-fleet-design.md` §5; `docs/consolidated-work-plan.md` F4/F5.

- [ ] [added 2026-07-04 19:40 UTC] **[Private corpus-ingest — salt-loader parity]** The private
  `corpus-ingest` (and the future corpus service) must load `CEC_FINGERPRINT_SALT` with EXACTLY the
  engine's semantics or the two sides silently disagree on every fingerprint: trim() the UTF-8 value,
  refuse <16 bytes, refuse set-but-not-UTF-8 (never treat as unset), set before the first fingerprint
  (`common::set_fingerprint_salt`; probe `fingerprint_salt_is_configured()`). Flagged by the 2026-07-04
  blind panel (auditor 2, trim-normalization divergence). — why deferred: private-repo change, lands with
  the one-time v2 re-ingest. Resume: `/mnt/e/cec-corpus-private` compile path; engine loader
  `crates/support-agent/src/main.rs::load_fingerprint_salt`.
- [ ] [added 2026-07-04 19:40 UTC] **[Keyless-chain strip-downgrade — re-flagged by the blind panel]**
  Auditor 2 independently re-derived the known keyless-chain residual in a sharper form: stripping
  `integrity` from EVERY row demotes a chained file to accepted-unchained legacy (`verify_chain` with==0
  path) — strictly easier than rechaining. Same class as the existing "[Chain integrity — key or anchor
  the head]" item (attestation still guards confirmed rows under an authority; unattested cold-start
  files remain soft). No new code action beyond that item; recorded so the panel's convergence is
  auditable. — why deferred: duplicate of the tracked head-anchor item, listed for the audit trail.
