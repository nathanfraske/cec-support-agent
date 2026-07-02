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

**Q2. Inference over the mesh — loopback only, or fan out to a peer's MyOwnLLM?** Raw
symptom free-text (the user's description) is **NOT** de-identified before it reaches the
model — only the *corpus row* is. So sending inference to a peer's MyOwnLLM over the mesh
would expose raw prose to that peer.
- **Lean:** **loopback-only by default**; any non-loopback/mesh inference endpoint gated
  behind an explicit per-use privacy opt-in. **Decision:** is mesh inference ever in
  scope, or do we hard-require local inference?

**Q3. `myownmesh-core` pin — single source of truth?** Both AllMyStuff and the engine's
`corpus-mesh-adapter` must build against the **same** `myownmesh-core` version (git tag,
not crates.io). Where does that pin live so the two never drift? **Decision:** a shared
`.myownmesh-rev`-style file, or a documented manual bump?

**Q4. `MeshSandboxValidator` — in scope now or later?** The engine has a `SandboxValidator`
trait: a clean apply in a disposable sandbox is positive evidence that can *lower* an
escalation. Over the mesh, a peer node could be that disposable sandbox. **Decision:**
build a `MeshSandboxValidator` in this round, or defer (the conservative default —
"unvalidated ⇒ escalate" — already holds without it)?

**Q5. Tail-truncation anchor for mesh peers.** The corpus hash chain can't self-detect a
dropped tail; we close that with a committed head+count *anchor* in the corpus repo. When
the corpus is served over the mesh, a **consuming peer** needs that anchor too (else a
serving node could silently truncate). **Decision:** distribute the anchor as part of the
mesh corpus handshake, or is the rostered-owner trust model enough?

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
`{plan_id, source, max_risk, actions[]}` — the `actions` are tool-name vocabulary (e.g.
`cim_query`, `create_restore_point`) that **AllMyStuff maps to its own human-readable
labels**. The envelope deliberately omits a candidate's free-text `title`/`rationale` and
a step's `description`, because those can carry the raw request prose (hostname/user/IP/
serial); this is enforced by a de-id regression test + a process-level stdout-contract
test. Per D2, within `v1` new fields are additive-only; a breaking change bumps the major
and the consumer errors on an unknown one. **Enum wire grammar (pinned 2026-07-02, before
any consumer exists):** the enum-valued fields carry frozen snake_case tokens — `route`:
`software_state | hardware_evidenced | ambiguous`; candidate `source`:
`cold_model | corpus_primed | human`; `max_risk`: `read_only | reversible | destructive`;
`consent_required`: `read_only_only | allow_reversible | allow_destructive`; `escalation`:
`auto | verifier_confirm | human_confirm` — mapped explicitly in code (never `Debug`
formatting, which a Rust rename could silently change) and frozen by a pinning test.

Next: P1 (AllMyStuff-side de-id allowlist + the serde-only `diagnose` contract) — which is
where AllMyStuff first touches the engine, and where Q1–Q5 start to bite.
