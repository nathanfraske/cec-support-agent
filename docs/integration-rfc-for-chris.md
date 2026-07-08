# RFC — MyOwn-family integration: decisions needed

**For:** Chris · **From:** Nathan (+ agent) · **Status:** draft, awaiting input
**Companion:** the full plan is in [`integration-myown-family.md`](integration-myown-family.md).

> **Supersession (2026-07-02, Nathan).** **D1 is reversed: the engine presents as an API**
> consumed by AllMyStuff / MyOwnMesh — a `cec-support-agent serve` service speaking versioned
> HTTP on loopback — instead of a spawn-per-diagnosis CLI sidecar. Everything D1 protected
> survives: the process boundary (and with it the MIT/AGPL firewall — AGPL §13 is the lever
> *designed* for a network service), the `cec-diagnose/v1` envelope (it becomes the
> `POST /v1/diagnose` response verbatim), cold start, and the sign-off/consent gates. The
> single-shot CLI remains for self-host parity. Execution becomes a second endpoint
> (`POST /v1/execute`, two-phase consent preserved) returning a post-execution
> `cec-execute/v1` envelope — which un-defers that FOLLOWUPS item. Q1–Q5 below still stand;
> **Q2 sharpens**: the engine itself is now a network surface, so API exposure
> (loopback-only by default, explicit flag to bind wider) and inference egress are separate
> knobs. Plan details: `docs/consolidated-work-plan.md` §3.

## The frame (30 seconds)

We're wiring the **cec-support-agent** diagnostic engine (AGPL) into the MyOwn family:
**AllMyStuff** (MIT, the device-inventory "brain" app) drives it, **MyOwnMesh** (MIT)
carries identity + a private corpus service. The clean architecture is:

```
AllMyStuff (MIT)  ──spawns as a Tauri sidecar, talks JSON over stdio──▶  engine (AGPL, standalone)  ──links──▶  myownmesh-core (MIT)
```

The engine is reached over a **process boundary, never linked** — so AllMyStuff stays
MIT and the AGPL obligation lands only on the engine binary. (This reuses AllMyStuff's
*existing* `bundle_myownmesh_sidecar` + serde-only-wire-mirror pattern — verified in its
code.) The engine stays standalone/cold-start; integration is additive trait seams.

## Decided — please sanity-check

> **D4. Identity/de-id egress boundary (owner, Nathan, 2026-07-08): de-identify on egress; the release
> trigger is SCHEDULING a session.** A per-machine LOCAL troubleshooting ledger may hold identity while it
> stays on the customer's box. When the customer schedules a session with the shop, that act is the release
> consent, and the DE-IDENTIFIED history (fault signature, plan tried with risk + attested sign_off,
> outcome/verification) is released to the shop and attaches to the identity-bearing ticket there — the
> join of identity ⇄ history happens at the trusted shop tier, never on the wire. The corpus row already
> carries the consent AUTHORITY (attested `sign_off`) + risk + outcome, so the released history is
> "what was tried and that it was authorized," tamper-evident. Consequences: the shop-server tier + the
> reinstall-durable ledger's off-box backup are de-identified (no key custody needed for shop copies).
> ONE sub-fork remains open — whether the diagnosis brain is bound by the same rule (PromptPayload-strict)
> or is a trusted exception; see the RFC/FOLLOWUPS PromptPayload item.


- **D1. ~~Single-shot CLI, not a daemon~~ — SUPERSEDED 2026-07-02 (see banner): the engine
  presents as an API service.** Original rationale kept for the record: AllMyStuff spawns
  `cec-support-agent diagnose --json …` per diagnosis (nothing to orphan, simplest); a
  persistent daemon can come later if latency demands it. *(Nathan's call — and Nathan's
  reversal.)*
- **D2. Result-envelope versioning = `cec-diagnose/v1`.** The `--json` envelope carries
  `"schema_version": "cec-diagnose/v1"`. **Within a major (`v1`), changes are additive
  only** (new optional fields; consumers ignore unknowns). A breaking change (remove /
  rename / retype a field) bumps to `v2`; the consumer checks the major and errors on an
  unknown one. *(Agent's call — flag if you'd version differently.)* Already implemented
  in P0 (below).
- **D3. Integration posture (owner, Nathan, 2026-07-04): the engine is an independent
  service; MyOwnMesh is transport, not a dependency.** The engine serves its own
  authenticated API and stays loopback-bound on its box; remote reach comes from the
  operator's MyOwnMesh daemon carrying/tunneling traffic to that loopback endpoint
  ("comms over his software"). The engine does NOT link `myownmesh-core` — the same
  pattern MyOwnMesh's own GUI uses (a daemon client over local control sockets, never
  embedding core). MyOwnLLM is not used for now (local inference stands). Verified
  against the repos as of 2026-07-04: MyOwnMesh v0.2.28 ships generic RPC
  (call/serve/call_stream) + typed pub/sub with opaque payloads and per-device ed25519
  roster identity; AllMyStuff v0.2.17 already rides the daemon for desktop/shell/files.
  This posture reframes Q2–Q5 below and keeps the AGPL boundary trivially clean (no
  linking anywhere).

## Open — need your call

These are where we're unsure and want to implement correctly the first time.

**Q1. Identity unification — one key or two?** MyOwnMesh gives each device a
cryptographic `Identity` (ed25519). The engine's corpus sign-off authority is *also*
ed25519. Do we **unify** them — one seed is both the device's mesh `DeviceId` and the
corpus sign-off authority — or keep them **separate**?
- *Unified* is clean for a **single operator** who is both the device owner and the
  sign-off authority (one key, set via `CEC_SIGNOFF_SEED`). Domain-tag separation keeps a
  mesh-auth signature from ever being a valid corpus attestation.
- *Separate* is right if sign-off authority should be **portable across devices** or held
  by someone other than the device owner.
- **Lean:** unified for the single-operator case, with the seam left open for a split
  deployment. **Blocks:** the corpus-mesh-adapter's authority wiring (Phase P3/P4).

> **DECIDED 2026-07-03 (owner, Nathan) — the volunteer-role half:** a volunteer is a **pure
> execution target**, NOT an identity that holds a sign-off authority. A **central** human/verifier
> authority holds the ed25519 sign-off seed and attests every outcome; the volunteer's machine runs
> the plan and never holds the seed, so a compromised volunteer cannot mint a resolved row. This
> keeps the sign-off authority centralized (no per-volunteer key), which **narrows Q1 toward the
> single-authority model** for the corpus side. The remaining open Q1 sub-question is unchanged: for
> the OPERATOR, unify the mesh `DeviceId` with the corpus sign-off authority (one key) or keep them
> separate. Pairs with Q7's custodied judge key — both custodied ed25519 keys held centrally.

> **DECIDED 2026-07-04 (owner, Nathan) — the operator half: SEPARATE keys. Q1 is now fully
> decided.** The mesh `DeviceId` is a DEVICE key (per-machine, rotated on reimage/retire, resident
> in the mesh stack on every connect — MyOwnMesh manages it in per-network rosters); the corpus
> sign-off authority is a ROLE key (portable across machines, slow expensive rotation, tight
> custody — the seed lives only where sign-off is performed). Unifying them would weld a long-lived
> role to a disposable device and make one compromised operator box an attacker who can mint
> `HumanConfirmed` rows — the event the attestation layer exists to prevent. The 2026-07-03
> decisions already made authority keys central role keys (volunteer-half; Q7 judge key), and F3's
> registry/rotation machinery must exist anyway, so unification saves nothing meaningful.
> Concretely: the mesh authenticates transport with the `DeviceId`; permission to submit/sign-off
> maps a roster entry (device → allowed roles), never key identity. Supersedes the original
> "lean: unified" above, which predates the 2026-07-03 decisions.

**Q2. Inference over the mesh — loopback only, or fan out to a peer's MyOwnLLM?** Raw
symptom free-text (the user's description) is **NOT** de-identified before it reaches the
model — only the *corpus row* is. So sending inference to a peer's MyOwnLLM over the mesh
would expose raw prose to that peer.
- **Lean:** **loopback-only by default**; any non-loopback/mesh inference endpoint gated
  behind an explicit per-use privacy opt-in. **Decision:** is mesh inference ever in
  scope, or do we hard-require local inference?

> **DECIDED-for-now 2026-07-04 (owner, Nathan, via D3):** MyOwnLLM is not used at present —
> inference stays local (the shipped hard-loopback default + `--allow-remote-inference` escape
> hatch is exactly right). Mesh inference is out of scope until the owner reopens it.

**Q3. `myownmesh-core` pin — single source of truth?** Both AllMyStuff and the engine's
`corpus-mesh-adapter` must build against the **same** `myownmesh-core` version (git tag,
not crates.io). Where does that pin live so the two never drift? **Decision:** a shared
`.myownmesh-rev`-style file, or a documented manual bump?

> **MOOT under D3 (2026-07-04):** the engine never links `myownmesh-core`, so there is no shared
> crate pin to keep in sync. The question only revives if a future decision embeds core (none
> planned). The `corpus-mesh-adapter` concept is superseded by the daemon-gateway posture.

**Q4. `MeshSandboxValidator` — in scope now or later?** The engine has a `SandboxValidator`
trait: a clean apply in a disposable sandbox is positive evidence that can *lower* an
escalation. Over the mesh, a peer node could be that disposable sandbox. **Decision:**
build a `MeshSandboxValidator` in this round, or defer (the conservative default —
"unvalidated ⇒ escalate" — already holds without it)?

> **DEFERRED under D3 (2026-07-04):** out of scope while the mesh is transport-only. The
> conservative default stands; F5 (a local/VM validator backend) is the live sandbox track.

**Q5. Tail-truncation anchor for mesh peers.** The corpus hash chain can't self-detect a
dropped tail; we close that with a committed head+count *anchor* in the corpus repo. When
the corpus is served over the mesh, a **consuming peer** needs that anchor too (else a
serving node could silently truncate). **Decision:** distribute the anchor as part of the
mesh corpus handshake, or is the rostered-owner trust model enough?

> **REFRAMED under D3 (2026-07-04) — ours now, not Chris's:** with the mesh as opaque transport,
> the anchor cannot ride a mesh handshake; it belongs to the ENGINE's corpus-service wire contract
> (serve the head+count anchor alongside query responses, or as a dedicated authenticated
> endpoint). Fold into the B4 / corpus-service design together with the Q6 minimal-attested-unit
> bar. No longer blocked on Chris.

**Q6. How much provenance does a served row expose?** B4 proposes serving essentially the
whole `Contribution` minus `integrity` — including `RowProvenance` (`run_id`,
`retrieval_first`, and `primed_from`, the plan-ids of the precedents that primed a run) —
so the consumer can run `ensure_attested` itself. Shipping `primed_from` on the read wire
exposes the corpus's internal **priming graph** (which fixes were derived from which) —
structure far beyond any single answer, and a corpus-cartography vector (leak-C10; see
`docs/corpus-cartography-threat.md` §2 V6). **Lean:** resolve by **provenance-graph
minimization** — the B4 served-row type ships only the minimal attested unit a consumer
needs to verify and use a row (attested `StoredOutcome` + attestation), never `primed_from`
or raw `confirmations`, unless a decision log entry explicitly authorizes it. **Decision:**
confirm this bar before B4's wire contract ships; gated on B4.

> **DECIDED 2026-07-04 (owner, Nathan): the lean is confirmed — provenance-graph minimization.**
> The B4 served-row wire type ships ONLY the minimal attested unit a consumer needs to verify and
> use a row: the attested `StoredOutcome` plus its `SignOffAttestation`. It never ships
> `primed_from`, `run_id`, `retrieval_first`, or raw `confirmations` unless a future decision-log
> entry explicitly authorizes a named field. This is now the bar the corpus-service wire contract
> builds to (cartography control C is thereby decided; B4 remains gated only on the corpus service
> existing). Note: the attestation covers the provenance pin, so a served row's signature is
> computed over fields the consumer does not receive — the served-row type must carry whatever
> minimal commitment the attestation math needs (e.g. the attestation message may need a
> provenance commitment rather than raw fields); design that with B4.
> **RESOLVED 2026-07-04 (built ahead of the re-ingest): attestation v4** —
> `cec-signoff-attestation-v4` binds `RowProvenance::commitment()` (sha256 over the
> `cec-provenance-commitment-v1` canonical) instead of the raw provenance fields. Replay
> protection is unchanged (a fabricated run id moves the commitment and breaks the signature),
> and a Q6-minimized served row (attested outcome + commitment, no raw provenance) is now
> verifiable by a consumer. Landed with the same hard cutover as the v2 migration, BEFORE the
> one-time private-corpus re-ingest, so the operator re-ingests exactly once.

**Q7. Plan-provenance signing across the *execution* boundary.** Plan signing today is
symmetric HMAC with a **fresh, ephemeral per-run key**, sound *only* because the judge and
executor are the same process (`provenance/src/lib.rs:141-154`; `SignedPlan` is in-process,
never serialized). A distributed access MCP — where the diagnosing agent/judge is off-box and
the executor runs on the target (client or volunteer) — **breaks that same-process
assumption**: a symmetric key shared across the wire is a signing oracle, and an ephemeral
per-run key has no persistent judge custody to attribute a signature to. Two topologies close
it: **(a)** the **judge runs on the target box** — the off-box agent sends diagnostics, the
target judges + signs + executes locally, and HMAC stays in-process; or **(b)** plan signing
goes **ed25519 with a persistent, custodied judge key**, like sign-off attestation. **Decision:**
which topology? It forks the whole access-MCP shape and pairs with Q1 (is a volunteer a rostered
identity that can *hold* a sign-off authority, or purely an execution target whose outcomes a
central authority attests?). Full analysis: `docs/test-validation-fleet-design.md` §2.1 T-6, §5.
Gated on the access-MCP design landing; no code depends on it yet.

> **DECIDED 2026-07-03 (owner, Nathan):** topology **(b) — plan signing goes ed25519 with a
> persistent, custodied judge key**, like sign-off attestation. The distributed access-MCP will
> have an off-box central judge that signs plans with a custodied ed25519 private key; the executor
> on the target verifies with the embedded public key. This **pairs with F3** (key custody +
> rotation + a key-id → key registry) — the judge key is now a second custodied ed25519 key
> alongside the sign-off authority, and both need the same registry/rotation machinery. Domain-tag
> separation keeps a judge signature from ever being a valid corpus attestation. No code yet; this
> is the target the distributed wrapper builds to (the loopback wrapper stays in-process HMAC).

## What's already built (P0 — no decisions needed, additive + cold-start-safe)

- `common::InventoryProvider` trait + `CoarseHostInventory` (today's os/arch/family
  default) + `ExternalInventory`.
- Engine CLI: `--inventory-keys <file|->` (identity-free config keys from an external
  inventory source → honest `config_class`, closing the A7/MH-6 gap) and `--json` (the
  `cec-diagnose/v1` envelope).
- De-id regression tests on the inventory path; the engine cold-starts byte-identically.

**The wire contract AllMyStuff codes against** (so it's robust, not "parse the last
line"): under `--json`, **stdout carries exactly one line — the `cec-diagnose/v1` JSON
envelope — and nothing else**; all human-readable trace is routed to **stderr**. So the
embedder reads stdout, parses one JSON object, and ignores stderr (or surfaces it for
debugging). The envelope carries only de-identified data: vocabulary symptoms (never the
raw request text), the **hashed** config class, the plan's action vocabulary, route,
consent level, and escalation. Fields:
`{schema_version, fault:{fingerprint,symptoms[]}, config_class, route, candidates[],
selected, consent_required, escalation, executed}` (+ additive `part_class` beside `route`
when it is `hardware_evidenced`), where each **candidate** carries ONLY
`{plan_id, max_risk, actions[]}` — the `actions` are tool-name vocabulary (e.g.
`cim_query`, `create_restore_point`) that **AllMyStuff maps to its own human-readable
labels**. The envelope deliberately omits a candidate's free-text `title`/`rationale` and
a step's `description`, because those can carry the raw request prose (hostname/user/IP/
serial); this is enforced by a de-id regression test + a process-level stdout-contract
test. **`source` (cold_model vs corpus_primed) was deliberately removed (2026-07-02,
leak-C10)** — a candidate exists with `source: corpus_primed` iff the corpus holds a
confirmed fix for exactly this `(fingerprint, config_class)`, so emitting the label turned
every diagnose into a yes/no corpus-membership oracle (corpus cartography — see
`docs/corpus-cartography-threat.md`). AllMyStuff never needed it: it maps `actions[]` to
its own human-readable labels and renders `max_risk`/`consent_required`/`escalation`;
it does not need to know whether the plan came from the corpus or a cold model, and it
gets no corpus provenance on the wire. (Honest residual: the retrieval-first hit/miss
latency and slate-count differentials are not yet equalized — tracked in FOLLOWUPS.md and
the threat doc's §3 control D.) Per D2, within `v1` new fields are additive-only; a
breaking change bumps the major and the consumer errors on an unknown one. **Enum wire
grammar (pinned 2026-07-02, before any consumer exists):** the enum-valued fields carry
frozen snake_case tokens — `route`: `software_state | hardware_evidenced | ambiguous`;
`max_risk`: `read_only | reversible | destructive`; `consent_required`:
`read_only_only | allow_reversible | allow_destructive`; `escalation`:
`auto | verifier_confirm | human_confirm` — mapped explicitly in code (never `Debug`
formatting, which a Rust rename could silently change) and frozen by a pinning test.
Candidate `source: cold_model | corpus_primed | human` was part of this grammar but the
**field itself was removed** from the wire (see above) — the enum values are retained here
only as the historical record of what was pinned before the removal.

Next: P1 (AllMyStuff-side de-id allowlist + the serde-only `diagnose` contract) — which is
where AllMyStuff first touches the engine, and where Q1–Q5 start to bite.
