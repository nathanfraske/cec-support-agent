# The test-validation fleet: two execution-zone MCP surfaces

**Status:** decision-ready, no code. **For:** Nathan (owner) + Chris.
**Companion to:** `docs/trusted-corpus-access-trajectory.md` (the corpus-access analogue — this doc
is its execution-side twin), `docs/corpus-cartography-threat.md` (leak-C10 — the non-mappability
discipline every corpus-touching surface inherits), `docs/integration-rfc-for-chris.md` (RFC Q1–Q6;
esp. Q4 mesh sandbox), `docs/evidence-integrity-and-research-checklist.md` (A10, EI/MH), `docs/research/prereg-control-lane.md`
(experimental-data discipline), `docs/consolidated-work-plan.md` §9 (sequence).

**The model (owner-confirmed, 2026-07-03).** The engine earns its inverted-ground-truth corpus by
having an agent PRODUCE candidate fix-flows ("test methods"), validate each in a disposable SANDBOX,
then validate against consented VOLUNTEER environments (real machines), and admit only
sign-off-gated, de-identified, verified outcomes. Two runtime MCP surfaces stand up the on-machine
EXECUTION ZONE: **(a)** the MCP through which diagnosis agents ACCESS a target machine (a client OR a
volunteer) to execute + observe, and **(b)** the MCP TEST HARNESS for the sandboxed environments.
These are the engine's highest-risk runtime surfaces — distinct from the agentic dev-tooling already
built, and distinct from the corpus-access surface the trajectory doc covers.

**The one cardinal rule, stated first.** Both MCP surfaces **WRAP the existing execution gates —
they never expose a raw on-machine tool.** The gates are already in code: two-phase consent
(`serve.rs:462-630`), ed25519 sign-off attestation (`provenance/src/lib.rs:159-231`, `gate.rs:179-194`),
HMAC plan-provenance (`provenance/src/lib.rs:41-120`, `execute.rs:14-22`), risk reconciliation
(`dispatch.rs:63-79`), escalation recompute bound to the selected candidate (`serve.rs:530-552`), and
the operation-vocabulary / advisory-only rail (`main.rs:911-917`, `dispatch.rs:46-48`). The MCP verb is
a gated `diagnose`/`execute` over the `/v1/execute` path — **never `registry_set`, `download_file`, or
a shell**. Everything below is an argument about how to hold that line across a process/box boundary
the current code has never crossed.

---

## 1. The loop, precisely — mapped onto existing code

| Stage | What happens | Existing code (reuse) | New work |
|---|---|---|---|
| **1. Agent produces a test method** | The swarm fans distinct causal hypotheses to generators; each returns a `Candidate` (an in-flight `Plan`). The bounded tool-using agent loop is the primitive. | `swarm::Generator::generate → Vec<Candidate>` (`swarm/src/lib.rs:37-48`); `Swarm::gather` (`:101-118`); `agent-core::Agent::run` JSON-in-content loop (`agent.rs:91-145`). Risk reconciled before anything runs: `Dispatcher::reconcile_risk` RAISES an understated risk to the tool's true risk (`dispatch.rs:63-79`). | The generator that emits a *reusable* fix-flow ("test method") as a first-class artifact vs a one-shot candidate. |
| **2. Sandbox validation** | The candidate is applied to a throwaway environment; a clean apply is *positive evidence* that can lower the escalation bar. | `swarm::SandboxValidator` trait + `ValidationReport{candidate_id, applied_cleanly, notes}` (`swarm/src/lib.rs:52-68`); `sandbox_validated_for(...) -> bool` (`main.rs:1077-1107`) feeds `required_escalation` (`panel/src/lib.rs:317-331`). | **The seam is wired as `None` today** (`main.rs:588`, `serve.rs:409,540`). A production `SandboxValidator` (Surface b) is the new work. |
| **3. Staged volunteer validation** | A sandbox-passed flow is applied to a real, consented volunteer machine — the same execution path as a client, on a machine the owner does not own. | **None.** The execution path exists (`execute_signed_plan`), but there is **no volunteer concept anywhere in the tree** (verified: `grep` finds only "volunteered facts" in intake, unrelated). | The entire volunteer fleet — enrollment, scoped/revocable consent, staging, the access MCP reaching off-box (Surface a). |
| **4. Verify (real post-state re-collection)** | Re-run the same diagnostic instrument on the machine; diff the post-state signature against the original. | `agent-core::verify_outcome(original, post, class)` (`verify.rs:71-91`). A `None` post → `Verdict::Unverified` → escalate, never resolve (`verify.rs:79-81`). | **`recollect_post_signature()` returns `None` unconditionally** (`main.rs:1118-1120`) — the NR-1 bootstrap echo. **No real verdict is possible until this is wired (F4).** This is the hard precondition for the whole model. |
| **5. Sign-off-gated de-identified write-back** | The de-identified triple is minted, attested, and admitted through the single truth-admission gate. | `record_outcome → Contribution::new` de-id mint (`main.rs:1006-1056`, `schema.rs:267-307`, `de_identify_plan` `:508-523`); `ensure_evidence_integrity` + `ensure_attested` (`gate.rs:91-194`); ed25519 attestation the engine cannot forge (`provenance/src/lib.rs:159-231`). | Nothing — this is the mature core. The fleet feeds it; it does not change. |
| **6. The flywheel** | The next run facing a known signature starts retrieval-first from this run's confirmed outcome. | `FileCorpus` persistence + `query` (`store.rs:401-440`); retrieval-first slate (`main.rs:489-505`, `serve.rs:343-359`); independent-confirmation counting (`store.rs:55-164`). | Nothing — reuse. |

**Read this table as the scope boundary.** Stages 1, 2, 5, 6 are built or seam-ready. The new work is
concentrated in **stage 2's missing backend (Surface b), stage 3's missing fleet (Surface a +
volunteers), and stage 4's missing re-collection (F4)** — and F4 gates the value of everything else,
because without a real post-state a volunteer run produces `Unverified`, which can never back a
resolved corpus row (`gate.rs:140-153`).

---

## 2. Surface (a): the target-environment access MCP — the execution-boundary threat model (the core)

This is the highest-risk surface in the system. Model it as its own boundary, analogous to leak-C10
for the corpus: a de-identified, gated surface whose *aggregate* misuse is the threat, plus a set of
single-call attacks each existing gate must already stop. **The adversary is a compromised/mis-steered
agent, a poisoned test method, a malicious plan trying to run a destructive action, a consent bypass,
an out-of-vocabulary escape, a plan-provenance forgery, or exfiltration of the target's data.**

The surface exposes exactly one gated verb pair — `diagnose` then `execute` — over the two-phase
`/v1/execute` path (`serve.rs:287-630`). It never exposes `registry_set` et al. directly. The
never-routable invariant already frozen for the socket (`serve.rs:28-40`: no attest, no keygen, no
corpus write, pinned by `router_surface_is_frozen` `:913-938`) is the model; this surface adds
**"no raw on-machine tool, ever"** to that list.

### 2.1 Per-adversary: which existing gate stops it, and the NEW guard the wrapper needs

**T-1 — Compromised/mis-steered agent runs a destructive action.**
*Existing:* the consent gate refuses any tool whose risk exceeds the granted consent, enforced at
dispatch, not the call site (`Consent::permits` `consent.rs:23-29`; `Dispatcher::dispatch` `:104-119`).
Granted consent is bound to the sign-off level (`serve.rs:579-583`): a `verifier` sign-off grants at
most `AllowReversible`; only `human` grants `AllowDestructive`. `required_escalation` forces
`HumanConfirm` for any destructive plan, any unvalidated state-changing plan, and any
hardware/ambiguous route (`panel/src/lib.rs:299-331`).
*NEW guard:* the MCP must carry the sign-off level as an authenticated field and must **never
self-grant `AllowDestructive`**. In the current in-process serve the agent, judge, and executor share
one process; when the agent is off-box the sign-off assertion becomes a *wire* claim — it must be an
ed25519-attested token, not a bare `"human"` string (`serve.rs:466-470` today trusts the string). The
destructive floor is only real if the sign-off is unforgeable at the boundary.

**T-2 — Poisoned test method mislabels its risk to slip past consent.**
*Existing:* `reconcile_risk` re-derives each step's risk from the *registered tool*, RAISES an
understated step, never lowers it, and reports the correction (`dispatch.rs:63-79`); serve runs it
before judging/consent (`serve.rs:401-404`). A model labeling a `registry_set` step `ReadOnly` cannot
thereby understate the rendered consent or slip the gate.
*NEW guard:* the wrapper must run `reconcile_risk` **on the target box, against that box's registered
tool set**, not trust a risk label computed elsewhere. Reconciliation is only sound against the actual
tool vocabulary that will execute.

**T-3 — Malicious plan carrying a destructive action.**
*Existing:* destructive ⇒ human sign-off floor, enforced twice — at the panel (`Escalation::HumanConfirm`,
`panel/src/lib.rs:301`) and at the corpus gate (`GateError::DestructiveFixNeedsHuman`, `gate.rs:156-160`,
so an embedder cannot mint a destructive "fix" with only a verifier sign-off). (Note: the current
`windows_tools()` set has no `Destructive` tool — `registry_set`/`download_file` are `Reversible`,
`tools-windows/src/lib.rs:211,441` — so today the floor is latent; it bites the moment a destructive
tool is registered.)
*NEW guard:* the operation vocabulary the MCP exposes is a **frozen allowlist of gated verbs**, never
"run this tool by name." Adding a destructive tool is a deliberate edit that must ship with the
human-floor test, the same discipline as the frozen router.

**T-4 — Consent bypass (replay, stale consent, wrong-candidate).**
*Existing:* the session is one-shot and TTL-bound — consumed on execute, expired after 15 min, so a
stale consent cannot authorize execution far from the diagnosis it was given for (`serve.rs:87,472-483`).
The escalation gate binds to the candidate **actually selected** by `plan_id`, recomputed for that
plan — never the judge's winner — closing the downgrade defect where a verifier sign-off could run a
reversible sibling the panel independently routes to `HumanConfirm` (`serve.rs:530-552`; regression
test `:779-853`).
*NEW guard:* over the wire, the `session_id` must be bound to the *caller identity* (a rostered mesh
peer), not a bare loopback token — otherwise a compromised app on the box replays another caller's
session within the TTL. Consent is to a *rendered plan*; the rendering the human approved must be the
bytes the signature covers (already true in-process via `provenance::canonical` binding title +
descriptions, `:99-120`) — the MCP must transmit the rendered plan, not a plan-id the target re-expands.

**T-5 — Out-of-vocabulary escape (a free-text "review" step, a shell).**
*Existing:* `is_executable` requires every step's action to be a registered tool; anything else is
advisory-only, never executed, and recorded as `EscalatedHumanUnresolved` (`main.rs:911-917`,
`serve.rs:556-577`). The tools themselves validate arguments before touching the OS: `safe_identifier`
(bare identifiers only, `tools-windows/src/lib.rs:60-69`), `validated_url` (https-only, injection-safe
`:407-416`), `validated_file_name` (bare name, no path escape `:420-430`).
*NEW guard:* the MCP must reject any verb outside `{diagnose, execute}` and any action outside the
registered vocabulary **at the wire boundary**, before dispatch — a wrapper that forwards an arbitrary
`{"tool": ..., "args": ...}` (the shape the internal `Agent` loop speaks, `agent.rs:122-134`) would
re-expose the raw tool surface. The internal agent protocol must never become the external MCP protocol.

**T-6 — Plan-provenance forgery.**
*Existing:* `execute_signed_plan` re-verifies the plan signature at the executor before any step runs;
a plan modified after signing, signed with another key, or never signed is refused without touching a
tool (`execute.rs:14-22`; `provenance::canonical` binds id, title, and every step's action + rendered
description + risk, `:99-120`, tampering test `:271-290`).
***Gap / NEW guard (load-bearing):*** **plan signing is symmetric HMAC** because "the judge and the
executor are the same process" (`provenance/src/lib.rs:141-154`, `SignedPlan` is in-process only,
`:35-46`), and the key is a **fresh, ephemeral `SigningKey::generate()` per run** (`serve.rs:587`,
`main.rs:733`) — which "proves intra-run integrity only, not which judge signed" (`negative-results.md`
Limitations). A distributed execution MCP where the diagnosing agent/judge is off-box and the executor
is on the target **breaks the same-process assumption** — a symmetric key shared across the boundary is
a signing oracle on the wire, and an ephemeral per-run key has no persistent judge custody to attribute
to. Either the **judge must run on the target box** (HMAC stays in-process; the agent sends diagnostics,
the target judges + signs + executes locally), or **plan signing must go ed25519** like sign-off, with a
persistent, custodied judge key. This is a genuine fork — see the open question for Nathan/Chris.

**T-7 — Exfiltration of the target's data.**
*Existing:* de-identified by construction, in the type system. `ToolOutcome` has no `Serialize` — raw
tool payload (`data: Value`, raw CIM among it) cannot reach a row or the wire (`tool.rs:6-32`).
`AgentRun`/`AgentStep` have no `Serialize` (`agent.rs:147-179`). The `/v1/execute` envelope emits only
action names + ok flags, **never step summaries** ("tool output can carry machine identity",
`serve.rs:617-629`); the post-execution envelope is de-identified and a test asserts none of five
planted identifiers (hostname/user/MAC/serial/IP) survive (`serve.rs:855-911`). `board_info` selects
configuration fields only, never serial numbers/asset tags (`tools-windows/src/lib.rs:353-395`). The
re-collected signature keeps only vocabulary terms, hex codes, prefixed ids, module names
(`signature_of` `main.rs:1129-1137`).
*NEW guard:* the MCP response contract **inherits the per-endpoint egress-sink checklist verbatim**
(`AGENTS.md:29-49`): vocabulary-only bodies, errors are fixed tokens never `Display`, no prose in logs,
and a ported poison-token contract test (plant `leakguard::POISON` into every input, assert no token
survives). A new response field that is not needed to *use* the answer does not ship. The only thing
that leaves the target box is the de-identified envelope + the de-identified, attested corpus row.

### 2.2 The invariants this surface must hold (restate as binding rules)

1. **One gated verb pair only.** `diagnose` → `execute`, over `/v1/execute`. Never `registry_set`,
   `download_file`, a shell, or the internal agent-loop `{tool,args}` protocol.
2. **Destructive ⇒ human sign-off floor**, unforgeable at the boundary (T-1, T-3, T-6).
3. **Every action attributable + logged.** There is **no execution audit log today** (the twin of the
   cartography V7 gap, `corpus-cartography-threat.md` §2 V7). A remote execution surface must log
   *who executed which plan on which box, when* — hashed keys + caller id + timestamp, never `describe`.
4. **The target's raw data is de-identified by construction before anything leaves the box** (T-7).
5. **Off-box is a deliberate, audited act.** `serve` is loopback-only by default and refuses a
   non-loopback bind without `--allow-remote` (`serve.rs:175-207`), which arms AGPL §13. Reaching a
   volunteer/client machine is a *remote* surface: it requires `--allow-remote` + a mesh identity +
   encrypted transport — the exact "trusted box + trusted calls + encrypted" posture the trajectory
   doc sets for the corpus (`trusted-corpus-access-trajectory.md` §0), now applied to execution.

---

## 3. Surface (b): the sandbox test-harness MCP

The production `SandboxValidator` the seam currently wires as `None` (`main.rs:588`; `serve.rs:409,540`
both pass `None` to `sandbox_validated_for`). Its job is narrow and its power is deliberately bounded.

**Contract.** Apply a candidate fix-flow to a throwaway environment, observe the post-state, and return
`ValidationReport{candidate_id, applied_cleanly, notes}` (`swarm/src/lib.rs:52-68`). `applied_cleanly`
feeds `sandbox_validated_for → bool → required_escalation` (`main.rs:1077-1107`, `panel/src/lib.rs:317-331`).

**What "disposable + isolated + reproducible" requires.**
- **Disposable + holds no user data** — the swarm doc's own words: "disposable sandbox VMs that hold no
  user data" (`swarm/src/lib.rs:8-9`). A VM/snapshot backend that reverts to a clean snapshot per
  candidate, or a MyOwnMesh peer standing in as the disposable node (**RFC Q4**,
  `integration-rfc-for-chris.md:79-82` — a `MeshSandboxValidator`; **state the Q4 dependency: this is
  Chris/owner-gated**).
- **Isolated** — network-isolated so a poisoned test method cannot reach real infra, and so nothing
  observed in the sandbox can egress. The sandbox is a *fresh* environment, not a copy of a real one,
  so there is no user PII in it to leak.
- **Reproducible** — reset to an identical baseline between candidates, so `applied_cleanly` means the
  same thing every run and a flaky apply is not read as evidence.

**The load-bearing bound: it LOWERS an escalation, it never RAISES trust without a signature.** This
is already the semantics and must stay: a clean apply moves a *reversible, software-state* plan from
`HumanConfirm` down to `VerifierConfirm` (`panel/src/lib.rs:327-331` — the `requires_consent() &&
!sandbox_validated` clause is what forces the human floor when unvalidated). It **cannot** lower a
destructive plan (always `HumanConfirm`, `:301`), a hardware/ambiguous route (`:324-326`), or produce
a corpus verdict. A dirty apply or a validation error stays conservative — `sandbox_validated_for`
returns `false` on both (`main.rs:1099-1106`), so "unvalidated equals escalate" holds.

**What it must NOT do.**
- **Not a way to launder an unvalidated fix into the corpus.** A clean sandbox apply is **not** a
  `Verdict::Pass`. The corpus verdict comes only from `verify_outcome` over a *real* re-collection on
  the actual target (`verify.rs:71-91`), and a resolved row is gated on a matching passing verdict +
  sign-off + attestation (`gate.rs:140-160`). Sandbox evidence and the corpus verdict are **different
  quantities from different machines** — keep them separate. (The current `ValidationReport` returns
  only `applied_cleanly`, not a post-state signature; do **not** extend it to emit a verdict that could
  be mistaken for the real one.)
- **Not a rigged-result trust bypass.** Because a sandbox can at most downgrade a reversible
  software-state plan to `VerifierConfirm` — and even that still requires a signature to write the row
  — a rigged sandbox result cannot mint truth. It can waste a verifier's time; it cannot admit a fix.
  The signature (ed25519 attestation, `provenance/src/lib.rs:159-231`) remains the truth-admission
  boundary the sandbox never touches.

### 3.1 Reproducing a target's environment *quickly* — the image mechanism

The sandbox is only useful if it can stand up an environment matching the target's `config_class`
fast enough to validate a fix while the ticket is live. The mechanism, and the boundary it cannot
cross:

**`config_class` is the image key — not "a Windows build."** `ConfigClass`
(`common/src/config_class.rs:3-9`) is defined as *"the comparability key for a machine: which corpus
rows (and golden baselines) it may be matched against"* — so the same key that already matches a
ticket to corpus rows is the key that selects a golden baseline. It is either a `BomRevision` (a CEC
build, `:13-14`) or a `DerivedHash` — an order-independent, normalized hash over inventory entries
(CIM hardware + driver inventory, `:15-18,31-42`). **The granularity is a modeling choice of what
goes into that inventory vector**, and the shipped test keys on `"os:windows 11 23h2"` — a
*release-branch* string, not a monthly build number (`common/src/lib.rs:85-89`). Under the current
model a machine's class is therefore `{release branch} × {hardware/driver inventory}`: two machines
on 23H2 with the same hardware are the *same* class regardless of which Tuesday's cumulative update
each carries.

**Golden image + differencing disk + warm pool = fast.** Keep a small library of base golden images
keyed by config_class. To reproduce one, boot a **differencing (copy-on-write) disk** off the base,
apply the candidate, snapshot, discard — the base is never mutated and a clone is seconds, not a
multi-GB capture. A **pre-warmed pool** of already-booted clones removes even the boot latency.
Reset-to-snapshot between candidates is what makes `applied_cleanly` mean the same thing every run
(the "reproducible" requirement above).

**Do you capture a golden image per Windows update? No.** Two reasons:
1. **Most updates don't change the class.** A monthly cumulative update does not move
   `windows 11 23h2` — same release branch — so it mints no new config_class and needs no new image.
   A new class is minted only when something you *key on* changes: a **feature update**
   (23H2 → 24H2), or a driver/firmware component (which may *arrive via* Windows Update but registers
   as a change in the hardware/driver inventory, not the OS-patch string).
2. **When you do need a patched image, no machine "downloads" it.** Windows servicing packages
   (`.msu`/`.cab`) are pulled from the Microsoft Update Catalog and **injected offline into the base
   image via DISM** (`Add-Package`) — one automated pipeline produces the patched image as a
   copy-on-write delta on the base. No physical box downloads anything; no per-KB manual capture.
   Images are **demand-driven**: materialize a class's image when a real fault at that class needs
   validation, not preemptively for every KB the moment it ships.

**The hard boundary a VM cannot cross — and why the volunteer fleet exists.** A golden image
reproduces the *software-state* dimension of a config_class cheaply and offline: OS release,
cumulative patch level, installed software, registry state. It **cannot synthesize real silicon** —
the actual OEM driver stack bound to physical hardware, firmware/EC/BIOS, a specific GPU or NIC —
and that is exactly the `DerivedHash` inventory dimension (CIM hardware + driver inventory) that most
often *discriminates a fault*. So the two validation stages split along precisely this line:
- **Sandbox validation (Surface b)** covers the software-state classes a VM can hold — the cheap,
  offline, automatable dimension, where the golden-image mechanism above lives.
- **Volunteer-environment validation (Surface a on a volunteer, §4)** covers the hardware/firmware/
  driver classes a VM cannot reproduce — the real-machine dimension. A "new Windows update" is cheap
  (an offline DISM delta); a *new hardware config_class* is what genuinely needs a real box, and that
  — not the OS-update cadence — is the volunteer fleet's reason to exist.
- This split *is* the escalation semantics already in code: a clean sandbox apply lowers a reversible
  **software-state** plan to `VerifierConfirm`, but a hardware/ambiguous route is never lowered by a
  sandbox (`panel/src/lib.rs:324-326`) — precisely because the sandbox never reproduced the hardware.

**RFC Q4 tie-in.** The disposable node need not be a VM the engine hosts: a MyOwnMesh peer can *be*
the disposable sandbox for the software-state dimension (**Q4**, `integration-rfc-for-chris.md:79-82`)
— Chris/owner-gated, defer-able because "unvalidated ⇒ escalate" holds without it. A mesh peer
volunteering as a *real-hardware* node is the same idea extended to the dimension a VM can't reach,
which folds Q4 into the volunteer-fleet identity question (Q1).

---

## 4. The volunteer-environment fleet (the new, socially-heavy part)

**State the absence plainly: there is no volunteer concept in the code at all.** `Consent`
(`consent.rs:9-19`) is a *per-action-risk* grant (ReadOnly / Reversible / Destructive), not an
*enrollment*. There is no volunteer roster, no scoped/revocable enrollment consent, no volunteer
identity, and no lane field. This is the largest greenfield in the model, and it is mostly *policy and
infrastructure*, not engine mechanics.

**Enrollment + informed, scoped, revocable consent (NEW).** The engine's action-consent is necessary
but not sufficient. A volunteer must give, *before any run*: (i) informed enrollment consent naming
what runs on their machine and what data leaves it; (ii) a *scope* — a maximum risk tier they permit
(mapping to `Consent` + the sign-off floor); (iii) revocation that removes them from the fleet and
stops future dispatch. None of this exists; it is new engine state + a legal/consent framework.

**Staging discipline (map to existing gates).** A fix reaches a volunteer **only after** Surface (b)
sandbox validation (`sandbox_validated == true`). A *destructive* class reaches a volunteer **only**
with an explicit consent tier: `Consent::AllowDestructive` **and** a `HumanConfirmed` sign-off
(`serve.rs:579-583`, `gate.rs:156-160`). The staging order is: generate (stage 1) → sandbox-pass
(stage 2, Surface b) → volunteer-dispatch (stage 3, Surface a on a volunteer) → verify (stage 4) →
write-back (stage 5). No stage may be skipped; the sandbox pass is a *precondition* of volunteer
dispatch, not a substitute for the volunteer verdict.

**Blast-radius / isolation.** One volunteer machine per run; a restore point captured before any
change (`CreateRestorePoint` positively verifies the checkpoint exists, not merely requested,
`tools-windows/src/lib.rs:298-348`; `RegistrySet` exports a unique backup per write, refusing to
clobber, `:201-279`); halt-on-first-failure so a remediation never keeps applying after an error
(`execute.rs:51-53`); bounded retry then escalate (`MAX_ATTEMPTS = 2`, `main.rs:905`). State the
restore-point coverage boundary in consent: it covers system files/registry/drivers, **not** firmware
or user files (`render_consent` already says this, `main.rs:944-952`).

**What data flows OFF a volunteer machine — de-identified by construction.** Exactly what leaves a
client leaves a volunteer: the `cec-execute/v1` envelope (action vocabulary + ok flags,
`serve.rs:617-629`) and the de-identified, attested corpus row (vocabulary `Symptom`s, hashed
`ConfigClass::DerivedHash`, minted `StoredAction`, `schema.rs:508-523`). **Never** raw prose/PII —
`Plan.title`/`PlanStep.description` are `Prose` with no `Serialize` (`common/src/plan.rs:26-55`), and
the de-id mint drops them and reconstructs the title from the action vocabulary (`schema.rs:508-523`).
The volunteer machine is just a *target* running Surface (a); the egress-sink checklist is the contract.

**The non-mappability angle (leak-C10, and a NEW axis).** "Which volunteer validated which fault→fix"
is **both** corpus structure **and** a volunteer-identity graph — strictly more sensitive than the
corpus-cartography threat, which assumes de-identified rows. Protect it:
- The corpus row must carry **no volunteer identifier**. `RowProvenance.run_id` is OS entropy, an
  opaque token (`schema.rs:222-231`, `main.rs:1310-1314`); `primed_from` is plan-ids, not identities.
  Do **not** add a `volunteer_id` field — provenance-minimization (cartography control C) applies.
- The volunteer↔run linkage lives only in a **rostered, budgeted, audited** enrollment ledger on the
  trusted box — **never a queryable membership oracle**. The non-mappability rule set already binds
  every corpus-touching surface (`AGENTS.md:50-73`): one answer per call, no gratuitous membership
  differential, attributable, budgeted. Extend rule 4 ("minimal attested unit") to say the served/
  written unit carries no fleet-participation signal.

**Corpus-poisoning defense (a rigged or coerced volunteer must not mint truth).**
- **Independent-confirmation counting.** One volunteer run = one `run_id` = one confirmation;
  re-submitting the same run does not inflate (`confirmation_key` dedups by `run_id`,
  `store.rs:55-61`); a self-primed (circular) row counts for nothing (`:57`). N *independent*
  volunteers are required for real confidence (`store.rs:770-815`). A coerced single volunteer
  produces at most one confirmation.
- **Sign-off asymmetry.** The volunteer's machine does **not** hold the sign-off seed; the human/
  verifier authority does (`provenance/src/lib.rs:141-198` — the engine embeds only the public key).
  A volunteer cannot self-attest a `HumanConfirmed` row; the gate refuses an unattested/forged one
  (`gate.rs:179-194`, `store.rs:854-930`). This is what keeps a compromised volunteer box from
  minting truth.
- **Reopened demotion.** A "fix" a volunteer confirmed that later recurs is demoted net-of-reopens
  out of retrieval (`store.rs:89-164`, run-deduped so a replayed reopen cannot over-demote).
- **The control lane.** Volunteer results are experimental data — a lane tag routes some rounds
  retrieval-OFF as controls (`prereg-control-lane.md`).

**Research / prereg discipline.** Volunteer results are experimental data governed by
`docs/research/prereg-control-lane.md`: a **lane tag** (no `lane` field exists yet — the prereg is a
SCAFFOLD, VOID if data predates its commit, `prereg-control-lane.md:3-11`), and **negative results
kept** — a failed volunteer fix enters the corpus as a hard negative (`EscalatedHumanUnresolved` /
`Verdict::Fail`), never discarded (`gate.rs:85-88`, hard negatives admitted `:378-388`). **The §0
precondition binds here:** "real post-fix re-collection is wired … *without this the control arm is
degenerate*" (`prereg-control-lane.md:14-15`). A volunteer arm on the current `None` re-collection
stub (`main.rs:1118-1120`) produces only `Unverified` — it cannot back a resolved row, so **F4 is the
gate on the entire fleet producing usable data.**

---

## 5. What is greenlightable now vs gated

**Greenlight now — pure-engine / design-first (no infra, no Chris):**
- **The gated MCP wrapper spec over `/v1/execute`** — the frozen verb contract (`{diagnose, execute}`
  only), the egress-sink checklist inheritance, and the T-1..T-7 gate map above. This is a design
  doc + a trait/route shape, and it hardens the existing serve surface regardless of whether a
  volunteer ever exists.
- **The `SandboxValidator` production contract** — nail the "lowers-only, never raises-without-a-
  signature" rule and the disposable/isolated/reproducible requirements in the trait's doc, and add a
  test that a clean report cannot produce a resolved row. No VM needed to specify it.
- **The volunteer de-id / consent *data contract*** — what leaves a volunteer box (envelope + attested
  row, nothing else), and the invariant that the corpus row carries no volunteer identifier. A design
  artifact + a poison-token contract test.
- **An execution audit-log skeleton** (hashed key + plan-id + timestamp) — the execution twin of the
  cartography V7 / MH-1 query-side log; log-only, identity fills in at rung-2.

**Infrastructure (gated on capacity / a host, not on Chris):**
- A **VM/snapshot backend** for Surface (b) — this is `F5` in the work plan ("Production
  `SandboxValidator` (disposable VM…)", `consolidated-work-plan.md:276`).
- **Real post-fix re-collection (F4)** — replace the `None` stub with a Windows-backed re-collection
  (`consolidated-work-plan.md:274`; `main.rs:1118-1120`). **Hard precondition for any real verdict.**
- **Volunteer enrollment + the consent/authorization/legal framework** — entirely new; heaviest lift,
  and mostly non-engine.

**Chris / owner-gated:**
- **Q4 mesh sandbox** — a `MeshSandboxValidator` using a mesh peer as the disposable node
  (`integration-rfc-for-chris.md:79-82`). Defer-able: the conservative "unvalidated ⇒ escalate"
  default already holds without it.
- **Q1 identity for a rostered volunteer fleet** — a volunteer is a rostered mesh identity; this is the
  same `F3` key-registry / roster work the corpus trajectory funnels through
  (`trusted-corpus-access-trajectory.md` §2.3). A volunteer fleet cannot be attributable, budgeted, or
  revocable without it.
- **The T-6 plan-signing fork** (ed25519 vs judge-on-target) — now filed as **RFC Q7**
  (`integration-rfc-for-chris.md`), paired with Q1.

**Recommended sequence (extends `consolidated-work-plan.md` §9 and the trajectory §4):**
```
[after B3/B4 serve wave; F2→F3→B4→F1 corpus-hardening underway]
→ Spec the gated MCP wrapper over /v1/execute (verb contract + egress-sink inheritance)   S   greenlight
→ SandboxValidator production CONTRACT + "lowers-only / no verdict" test                  S   greenlight
→ Execution audit-log skeleton (hashed key + plan-id + timestamp)                         S   greenlight
→ F4  real post-fix re-collection (replaces the None stub)                                M   needs Windows host — THE data gate
→ F5  production SandboxValidator VM backend (or Q4 mesh peer)                             L   infra / Q4
→ E0  Q1 (rostered identity) + the T-6 signing fork                                       —   blocking, Chris/owner
→ Volunteer enrollment + scoped/revocable consent framework                               L   infra + legal, gated on Q1
→ Staged volunteer dispatch over the access MCP (off-box, --allow-remote + mesh + TLS)    L   gated on all above
```
F4 sits early on purpose: **without it the sandbox, the fleet, and the prereg control lane all produce
only `Unverified` — no usable data.**

---

## 6. Anti-scope — what this must NOT become

- **Not a raw on-machine tool MCP.** No verb exposes `registry_set`, `download_file`, `powershell`, or
  the internal agent-loop `{tool, args}` protocol (`agent.rs:122-134`). The vocabulary is the gated
  `{diagnose, execute}` verb pair, and it is frozen the way the router is frozen (`serve.rs:913-938`).
- **Not an un-consented telemetry pipe.** Nothing leaves a volunteer without informed, scoped consent
  *and* de-identification by construction. No raw prose, no `ToolOutcome.data`, no step summaries, no
  transcript — the egress-sink checklist is binding (`AGENTS.md:29-49`).
- **Not a way to reach a machine without the sign-off/consent gate.** Every execution goes through
  `execute_signed_plan` (signature re-verified, `execute.rs:14-22`) → the consent-gated `Dispatcher`
  (`dispatch.rs:104-119`) → two-phase, one-shot, TTL-bound consent with escalation recompute bound to
  the selected candidate (`serve.rs:472-552`). No side door, no direct dispatch, no bypass.
- **Not a volunteer graph exposed as a queryable oracle.** No endpoint maps volunteer↔fix; the corpus
  row carries no volunteer identifier; the frozen router + the non-mappability rules
  (`AGENTS.md:50-73`) hold. Enrollment linkage lives in a rostered, budgeted, audited ledger on the
  trusted box — never a membership probe.
- **Not a sandbox that mints truth.** The sandbox supplies escalation *evidence* only; it never
  produces a corpus verdict and never lowers a gate without a signature (§3).
- **Not attestation/keygen/seed on any network.** The asymmetric split is invariant: the engine holds
  only public keys; the sign-off seed never becomes network-reachable (`serve.rs:28-40`,
  `provenance/src/lib.rs:141-154`). A distributed execution MCP does not change this.

---

## 7. Gaps this design surfaces (the work, stated honestly)

1. **No volunteer concept exists** — no enrollment, no scoped/revocable consent, no volunteer identity,
   no lane field. `Consent` is per-action-risk only (`consent.rs`). This is the bulk of the new work.
2. **`recollect_post_signature()` is a `None` stub** (`main.rs:1118-1120`, NR-1) — no real verdict is
   possible; every run is `Unverified`. F4 gates the value of the entire fleet.
3. **Plan signing is symmetric HMAC, judge==executor in one process** (`provenance/src/lib.rs:141-154`).
   A distributed execution MCP breaks this — see the open question.
4. **`serve` is loopback-only** (`serve.rs:175-183`); off-box execution is a new remote surface needing
   `--allow-remote` + mesh identity + encrypted transport + AGPL §13 — not built.
5. **No execution audit log** — the twin of the cartography V7 gap; a remote execution surface must be
   attributable.
6. **`SandboxValidator` returns `applied_cleanly` only** (`swarm/src/lib.rs:52-60`), and is wired as
   `None` (`main.rs:588`); the production backend and its isolation model are unbuilt (F5 / Q4).
7. **Existing tool-surface gap:** `deid::ACTION_VOCABULARY` freezes 8 members but `windows_tools()`
   registers 6 — `review` is the sanctioned advisory token, but `driver_rollback` is de-id-clean
   vocabulary with **no backing `Tool` anywhere** and no note. It de-identifies cleanly (could ride a
   corpus row) yet `dispatcher.contains("driver_rollback")` is `false`, so `is_executable`
   (`main.rs:911-917`) always marks such a plan advisory-only — it can never be dispatched today. The
   drift test is one-directional (every tool ∈ vocabulary, not the reverse), so this is uncaught. A
   fleet that registers real remediation tools must close the vocabulary↔tool set both ways.
