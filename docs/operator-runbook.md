# Operator runbook — standing up the engine after the 2026-07 migrations

Everything the **owner/operator** must do on their end, in order. The engine side is
done and merged: chain `cec-corpus-chain-v2`, keyed `cec-fingerprint-v2`, attestation
`cec-signoff-attestation-v4` (provenance commitment), the leak Phase-3 boundary rails,
and the decision records (Q1/Q2/Q3/Q4/Q5/Q6/Q7/D3). Each step below is something only
the operator can do because it involves a secret, the private repo, GitHub settings,
or a machine the agent does not hold.

## 1. One-time secrets provisioning (do first)

| Env var | Where it must be set | How to generate |
|---|---|---|
| `CEC_FINGERPRINT_SALT` | EVERY process that computes fingerprints for this deployment: `cec-support-agent diagnose`, `cec-support-agent serve`, and the private `corpus-ingest` | `openssl rand -hex 32` |
| `CEC_SIGNOFF_SEED` | ONLY where sign-off is performed (the authority machine / corpus-ingest attest step) | already provisioned (`make keygen` in the private repo); no change unless rotating |
| `CEC_SIGNOFF_PUBKEY` | Every engine that must ENFORCE attestation (diagnose/serve with a corpus) | printed by keygen; already in use |

Hard rules the engine enforces (fail-closed at startup, fixed no-echo messages):
- The salt is **per-deployment, shared**: the corpus writer and every engine that
  queries that corpus MUST use the SAME salt, or query fingerprints will never match
  stored ones. Store it beside the sign-off seed (e.g. `/mnt/e/secrets/`), not in any
  repo.
- Salt under 16 bytes → refused. Set-but-not-UTF-8 → refused (provision as text).
  Unset → the documented PUBLIC cold-start default: everything works, but fingerprints
  are offline-enumerable and `serve` prints a NOTICE. For any box you care about, set
  the salt.

## 2. The one-time private-corpus re-ingest (after PR #19 is on main)

The 2026-07 cutover deliberately invalidated stored hashes ONCE: v2 chain hashes, v2
fingerprints (both retrieval keys), and v4 attestations all changed. A pre-migration
JSONL corpus now **fails at open by design** (pinned by test). In
`/mnt/e/cec-corpus-private`:

1. Bump the engine pin to post-PR-#19 `main`.
2. Adapt `corpus-ingest` (tracked in FOLLOWUPS): `Contribution::new`/`de_identify_plan`
   now return `Result`; stored-type field changes; and load the salt with EXACTLY the
   engine's semantics — `common::set_fingerprint_salt(env.trim().as_bytes())` BEFORE
   the first fingerprint, refuse `NotUnicode` instead of treating it as unset, and
   check `common::fingerprint_salt_is_configured()` after setup.
3. Recompile the corpus from the YAML ground-truth flows (the YAML is the source of
   truth; the JSONL is derived). The recompile re-mints fingerprints under your salt,
   re-chains under v2, and re-attests under v4 — one pass, done.
4. `corpus-ingest verify` must pass; an engine pointed at the new file must open it.

Do NOT re-ingest before PR #19 merges, or you will re-ingest twice.

## 3. Repo/GitHub settings (five minutes, durable)

- **Branch protection on `main`**: require the CI checks `check` (×3 OSes), `audit`,
  `secrets`, and the new `boundary` job; require a PR. The boundary job's
  allowlist-freeze only has teeth if PRs cannot bypass it.
- Optional hardening already filed in FOLLOWUPS: `CODEOWNERS` over the de-id/crypto
  surfaces (leak L4), and the least-privilege bot PAT for the ops branch push.
- On every working clone: `cargo xtask install-hooks` (gitleaks is now optional —
  the hook warns and skips; CI is the backstop).

## 4. Mesh wiring when you want tier-2 (serving your own devices) — per D3

Nothing to build on Chris's side and nothing to link: run `cec-support-agent serve`
loopback-bound on the corpus box, and let YOUR MyOwnMesh daemon (v0.2.28+: generic
RPC / tunnel, roster identity) carry remote traffic to that loopback endpoint —
the same daemon-client pattern MyOwnMesh's own GUI uses. No `myownmesh-core`
dependency, no MyOwnLLM, no token-auth public HTTP ever. The corpus-service endpoint
itself (`POST /v1/mappings/query` server side + B4 attested reads) is the next engine
build and needs nothing from you beyond this runbook's steps 1–3.

## 5. Decisions and ground truth only you can provide (not blocking today)

1. **PromptPayload form** (FOLLOWUPS): strict de-identified-only prompts (changes
   diagnosis quality — the model stops seeing raw prose) vs explicit-but-raw named
   channels (type-visibility only). One call, whenever you next groom the backlog.
2. **F4 — real post-fix re-collection**: the single biggest gap between "demo" and
   "prod that learns." Building the Windows collectors needs runs on your real
   Windows box(es) to validate against; until F4 lands every outcome stays
   `Unverified` and the corpus earns no resolved rows.
3. **Volunteer fleet**: enrollment + scoped/revocable consent framework is
   policy/legal work when you decide to start that tier.

## What you explicitly do NOT need to do

- Nothing is waiting on Chris (D3 dissolved Q2–Q5; his side eventually adds an
  ordinary API client in AllMyStuff, on his own schedule).
- No `myownmesh-core` version pin, no MyOwnLLM setup, no golden image per Windows
  update (ConfigClass keys on release branch × hardware, not monthly builds).
