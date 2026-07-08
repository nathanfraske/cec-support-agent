<!-- SPDX-License-Identifier: AGPL-3.0-only -->

# Corpus lifecycle & retrieval — design (owner ask 2026-07-08)

> **STATUS 2026-07-08: scoping.** Owner asked to build five features in one arc:
> (1) the corpus query **service**, (2) **retrieval-as-partial**, (3) the
> **config-transition trigger**, (4) gated **workflow retirement**, and (5) a
> corpus-entry **formatting/intent page**. This doc scopes all five, marks the two
> decisions that are genuinely the owner's, and records the build order. Precision-
> critical surfaces (retirement is a NEW corpus-mutation gate; the service is a new
> egress) get the §7 blind-audit treatment.

The through-line: the inverted corpus has, so far, only ever *grown* (a gated
write admits a signed triple). These five features give it a **lifecycle** —
things get *retrieved* (service, partial), *triggered by context* (transition),
*retired* (deprecation), and *authored cleanly* (formatting page) — without ever
weakening the one-way, tamper-evident, de-identified guarantees that make it
trustworthy.

---

## 1. Retrieval-as-partial  ·  *unambiguous, building now*

**Today:** `fix_mappings` counts a row toward a plan's confirmations only when
`label.is_resolved()`. A `ResolvedPartial` row is admitted and chained but backs
no retrievable mapping — so a proven per-symptom benefit is invisible to
retrieval.

**Change:** a `ResolvedPartial` row contributes a **partial** mapping keyed on
its proven `cleared` set. Retrieval can then offer a plan as *"known to clear
{cleared} at this config class (may not fully resolve)"* — a legitimate,
confidence-weighted partial step, distinct from a full-resolution mapping.

- `FixMapping` gains a `kind: MappingKind { Full, Partial { cleared } }` (or an
  equivalent additive discriminator) — **additive**, default `Full`, so existing
  served/serialized mappings are byte-compatible.
- Partial confirmations aggregate on the SAME independence key (`confirmation_key`)
  and the SAME `Reopened` net-cancellation as full mappings — a partial's benefit
  earns confidence exactly as rigorously as a full fix, just scoped to `cleared`.
- Full and partial mappings for the same plan are **separate** mappings (a plan
  can both fully-resolve some tickets and partially-clear others); the pipeline
  ranks a full mapping above a partial one at equal confirmation weight.
- The retry loop (future) consumes a partial mapping as PROGRESS (`remaining`
  becomes the next working set) — the DDU multi-step case. Filed, not built here.

## 2. Config-transition trigger  ·  *unambiguous scope (minimal primitive), building now*

**The 5070→5080 case.** A large share of experience-heavy fixes are triggered by
a **config transition** (a part swapped, Windows updated) rather than a symptom
from nowhere: swap a 5070 for a 5080 and you must DDU even after a clean install.
`ConfigClass` today is a single opaque hash — two configs are "same" or
"different", and you cannot see *what* changed. Recognizing a transition needs
**structured** inventory.

**Minimal composable primitive (this arc):**

- A `StructuredInventory` of **categorized** entries (`gpu`, `cpu`, `os`, `ram`,
  …) → normalized value, built from the same inventory the `DerivedHash` already
  consumes (so no new collection surface).
- `ConfigTransition::between(prior, current)` → the per-category deltas
  (`gpu: nvidia_5070 → nvidia_5080`), each classed as `WithinFamily` (same vendor/
  category, e.g. NVIDIA→NVIDIA) vs `CrossFamily` vs `Added`/`Removed`.
- A transition emits a **trigger token** (a de-id-grammar member, e.g.
  `transition_gpu_within_nvidia`) that can enter a fault signature's context, so
  the corpus can carry a row keyed to *"this fault shape AFTER a within-NVIDIA GPU
  swap"* and retrieval can prime the DDU workflow for exactly that transition.

**Deferred (FOLLOWUPS):** the per-machine **config ledger** that STORES each
machine's inventory history so a transition can be detected autonomously (today
the primitive takes `prior` + `current` as inputs; who persists `prior` is the
ledger's job). The primitive is useful immediately (a tech supplies "was 5070,
now 5080") and composes with the ledger later.

## 3. The corpus query service  ·  *unambiguous architecture (D3), building this arc*

**Not** a route on the loopback engine — its route surface is frozen and
deliberately excludes corpus/attest/keygen (`router_surface_is_frozen`). The
service is a **separate authenticated read API** (D3: "serve over an
authenticated API, comms over MyOwnMesh"), its own binary/module:

- `POST /v1/mappings/query` — body carries `{fault signature, config class}`
  (the POST-body retrieval key from the migration), returns the `FixMapping`s
  (full + partial), each as a **Q6-minimal attested row** (attestation +
  provenance *commitment*, never the raw run id / priming graph).
- **B4 attested reads:** the response is verifiable by the consumer against the
  configured `SignOffPublicKey` — the service holds only the public key, so a
  compromised server cannot forge a passing attestation; the client re-runs the
  read-side de-id re-validation (`ServedPlanInadmissible`) and the attestation
  check before anything reaches the retrieval-first slate.
- **Auth:** a bearer credential gate (the "authenticated API"); bind is
  non-loopback-allowed ONLY when auth is configured (fail-closed, mirroring
  `validate_bind`). Retirement (feature 4) is honored server-side: a retired plan
  is never served.
- The service is READ-ONLY: no write/attest/keygen route exists on it either
  (writes stay on the gated local submit path). Its route surface is frozen by
  its own pinned-surface test.

## 4. Workflow retirement  ·  *DECISION NEEDED — gating posture*

**Owner ask:** *"a way to retire workflows from the corpus after we can prove
they are no longer useful or have been deprecated completely… very heavily
gated, because it is very hard to prove that a workflow is no longer useful at
all ever in any scenario, but having the workflows adapt over time as new methods
are found is still a nice to have."*

**Non-negotiable properties (from the ask + the corpus's invariants):**

- **Never deletes.** The corpus is append-only and tamper-evident (the v2 hash
  chain). Retirement is a NEW appended, signed record that *supersedes* a mapping
  for retrieval — the history stays auditable forever. (Deleting a row would
  break the chain and erase truth; retirement hides a mapping from *retrieval*,
  it does not unmake the fact that it once resolved something.)
- **Config-class-scoped by default.** "No longer useful" is almost always *"no
  longer useful at THIS config class / after THIS transition"*, not "never again
  in any scenario". A retirement names the config class it deprecates the plan
  for; a global retirement is a distinct, even-more-gated act.
- **Heavily gated = human sign-off.** Enacting a retirement requires
  `HumanConfirmed` sign-off + a signed reason (`Deprecated` / `SupersededBy(id)`
  / `ProvenHarmfulNow`), attested exactly like a resolved row — so an embedder or
  a compromised server cannot mint a retirement.
- **Adaptation = supersession.** A new workflow proven better at a config class
  can be *linked* as the successor (`SupersededBy`), so retrieval prefers the
  successor while the retired one stays on record.

**THE DECISION (Q-retire):** may the system *autonomously propose* a retirement
candidate (evidence — sustained independent failures / `Reopened` events at a
config class — flags "this looks deprecated", a human confirms), or is retirement
**human-initiated only** (no autonomous proposal at all)? Recommendation:
**propose-but-never-enact** — evidence surfaces a candidate, a human always
enacts. This gives the "adapt over time" nice-to-have without ever letting the
machine remove a fix on its own.

Mechanism either way: `RetirementRecord { plan_id, config_class, reason,
superseded_by? }`, gated (`ensure_evidence_integrity` extended: a retirement needs
HumanConfirmed + attestation), chained like any row, and folded into
`fix_mappings` (a plan retired for a class is filtered exactly where `revoked` is
today — so retirement is the attested, auditable evolution of the `revoked`
primitive).

## 5. Corpus-entry formatting / intent page  ·  *DECISION NEEDED — surface*

**Owner ask:** *"a page to format out corpus entries and be able to have them
simplified down to the proper format, of course with a double check that all
intent was captured correctly with no ambiguity."*

The hard, valuable part is the **intent double-check**: a staff-authored workflow
(free prose, shop vocabulary) must become the canonical **de-identified** corpus
format (frozen `ACTION_VOCABULARY` actions, symptom-grammar tokens, no identity),
AND the author must be shown — in plain language — *exactly what will be stored*,
with any dropped detail or ambiguous step flagged, before they confirm. This is
the write-side complement to the authoring guide.

**Pipeline (surface-independent core, built in the engine):**

1. **Normalize** — map the authored steps to `ACTION_VOCABULARY` + de-id grammar
   via the existing `de_identify_plan` mint; reject/flag anything out-of-vocabulary
   (an unknown action, an identity-shaped token) rather than silently dropping it.
2. **Read-back** — re-expand the canonical form to plain language ("this will be
   stored as: DDU → restart → reinstall driver package → verify") and show it
   beside the original, so the author sees precisely what was captured.
3. **Ambiguity check** — flag steps that mapped to nothing, mapped ambiguously
   (one authored step → multiple candidate actions), or lost qualifiers; require
   explicit author resolution. (Optional enhancement: an LLM-assisted semantic
   diff using the existing judge/inference layer to catch intent that survived
   the grammar but changed meaning. Noted; gated behind the owner's call.)
4. **Confirm** — only an explicitly confirmed, unambiguous entry is eligible to
   enter the gated submit path.

**THE DECISION (Q-page):** what is the "page"? (a) a **rendered web page**
(Artifact/HTML) staff use directly, backed by an engine normalization capability;
(b) a **CLI tool** in the engine (normalize + read-back + confirm in the
terminal); (c) an **API endpoint only** (normalization + verification over the
authenticated API, the front-end built separately). Recommendation: **(a)** — the
audience is shop staff, "page" was the ask, and the read-back/confirm loop is
inherently visual; the engine still owns the normalization so the page is a thin
client over a tested core.

---

## Build order & gating

1. **Retrieval-as-partial** (extends `fix_mappings`; additive `MappingKind`). ✅ clear.
2. **Config-transition primitive** (new `common` module; self-contained). ✅ clear.
3. **Corpus query service** (new authenticated read module; serves 1's mappings,
   honors 4's retirement). Architecture clear (D3); build after 1 + 4's record type.
4. **Workflow retirement** (new attested lifecycle record + gate extension +
   `fix_mappings` fold). ⛔ needs Q-retire. §7 blind-audit (new corpus-mutation gate).
5. **Formatting/intent page** (engine normalizer core + the chosen surface).
   ⛔ needs Q-page.

Each lands as a green sub-step (tests + clippy) with red-on-revert on every gate/
crypto change, per the established discipline. Retirement and the service get a
packet-only §7 blind audit before merge (new gate + new egress).
