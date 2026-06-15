# Evidence-Integrity & Research Checklist — cec-support-agent

*Adapted from CEC-Platform's EVIDENCE-INTEGRITY policy (EI-01..EI-08) and RESEARCH CHECKLIST (PP-01..PP-13), re-aimed at the inverted-ground-truth corpus.*

> **Scope.** This repository (`cec-support-agent`) is the **open engine**. It ships **no corpus and no weights** — those live in a separate private repo. The engine ships only the corpus client + schema. Its truth is the **inverted corpus**: signed-off `(FaultSignature, Plan, OutcomeLabel)` triples earned at the sign-off gate and read back **retrieval-first**. Because CEC-Platform's server-side custody (CODEOWNERS, branch protection, two git zones) does not exist for an engine whose corpus lives elsewhere and grows **in-process one row at a time**, every adaptation below re-expresses a CEC-Platform mechanism as an **in-code runtime gate inside `corpus-client`** plus a **paired adversarial CI test** — never as a CI lint on a repo the corpus does not live in.

---

## 1. Purpose & the inverted-ground-truth integrity model

CEC-Platform earns truth top-down through a layered, mostly out-of-process gate (CI lint + CODEOWNERS + branch protection + ledger pins) that promotes human-reviewed artifacts into `corpus/promoted/**`. `cec-support-agent` earns truth **bottom-up**: there is no promoted zone to review; truth is **accreted** from signed-off, verified outcomes, and the **sign-off gate is the truth-admission boundary**. A single poisoned or mislabeled row becomes the next run's *preferred* fix, because retrieval-first scores a corpus-primed precedent at likelihood **0.8 vs 0.6** for a cold guess (`crates/panel/src/lib.rs:250-253`) and skips de-novo generation when precedent exists. **Per-row integrity is therefore the whole game.**

The boundary exists structurally and is real:

- `ensure_signed_off` (`crates/corpus-client/src/gate.rs:15`) is the single choke point. Every `CorpusStore::submit` calls it **before any state change** — in memory (`store.rs:116`), before disk (`store.rs:184`), before the network (`store.rs:251`) — and tests prove an unconfirmed row never reaches disk (`store.rs:393-409`).
- Hard negatives are **first-class truth**: every ticket emits a label (an unlabeled ticket is corpus poison, `schema.rs:24-27`); failures are stored but `fix_mappings` never offers them as fixes (`store.rs:33-37`), verified at `store.rs:334-348`.
- De-identification is **by structured extraction, not scrubbing**: `FaultSignature` is built from a fixed vocabulary and `de_identify_plan` strips every free-text field down to `{id, action, risk}` (`schema.rs:122-141`), proven by the adversarial leakage suite (`crates/corpus-client/src/lib.rs:34-127`).

But the boundary is **hollow**: `ensure_signed_off` checks exactly one bit — `contribution.sign_off.is_confirmed()` over a plain serde enum (`schema.rs:7-22, 100`). The four other integrity mechanisms (de-identification, the verification verdict, HMAC plan provenance, consent/escalation) each fire **somewhere else** in the pipeline and are **never jointly bound to the row that becomes truth**. The core adaptation is to widen the one funnel into a unified **`ensure_evidence_integrity(&EvidenceBundle)`** that admits a row only when a **real (non-bootstrap-trivial) verdict, a provenance/judge attestation, a de-identification proof, an honestly-derived config-class, and a sign-off level matching the authorizing consent** are all present and cross-checked — enforced in `corpus-client` so no library embedder (MyOwnLLM, AllMyStuff) can skip it.

---

## 2. How this is adapted from CEC-Platform

| CEC-Platform mechanism | AutoDiagnoser analog | What changed / dropped — and why |
|---|---|---|
| **Two-zone custody** (`corpus/staging/**` agent-writable/advisory; `corpus/promoted/**` human-only behind CODEOWNERS + branch protection) | The **SignOff ladder**: STAGING = `{Unconfirmed, VerifierConfirmed}` (agent/auto-reachable, reversible-only, advisory); PROMOTED = `HumanConfirmed` + route-forced human boundary (`panel/src/lib.rs:317-331`) | **Dropped** the git-zone model: the corpus is a separate private repo, so the truth boundary is an **in-process call** (`gate.rs:15`), not a server-side PR review. |
| **EI-08** server-side signature gate; machine account `nathanfraske-bot` **cannot self-approve** | Re-expressed as a **cryptographic attestation the in-code gate verifies** over `(signature, plan, label, sign_off, config_class)`, signed by a key the submitting process does **not** hold | **Changed** from GitHub branch protection to a cross-party attestation; the "asserting party ≠ approving party" property must be cryptographic, not a repo rule. |
| **EI-07** monotone-tightening law (unsigned influence on evaluation is **raise-only**) | `required_escalation` is raise-only via `.max()` (`panel/src/lib.rs:324-329`); the corpus prior touches only `likelihood`, never safety/reversibility (`lib.rs:240-254`) | **Changed framing** from "unsigned weight edits to a penalty table" to "the corpus prior + escalation are raise-only on the safety bar." Same **law**; the one EI already honored in code. |
| **Mandatory provenance; "model output is NOT a source"** (lint rejects model-sourced rows) | The row's provenance **is its verified execution outcome**: a `Pass/ProvisionalPass` verdict over a re-collected signature | **Changed** the unit of provenance from a cited external document to a real (non-bootstrap) verification verdict; the lint becomes a gate that rejects rows lacking one. |
| **Adversarially-tested de-identification** ("a scrubbing pass never adversarially tested leaks") | De-id by extraction + the seeded leakage suite (`corpus-client/src/lib.rs:34-127`) | **Kept and elevated** — this is the one discipline the repo already fully embodies. Only the *standing obligation to extend the suite on schema change* is new. |
| **CI-lint-as-enforcement** (`cec_corpus_lint.py` in `checklist.sh`) | A **library function in `corpus-client`** that runs on every `submit` | **Changed**: CI is the regression backstop, not the gate. A CI lint on a repo the corpus does not live in cannot gate a runtime write by an embedder. |
| KiCad/DRC, fab/datasheet source whitelist, two-zone git custody specifics | — | **Dropped** as platform-specific; no engine analog. |
| **PP-01..PP-13 paper-track** (claims/prereg/negative-results/instrumentation/commit-timestamp) | A `docs/research/` five-doc tree re-aimed at the inverted corpus (§4) | **Kept** as the **outer governance loop**, separate from the per-row runtime checkpoint; it is research-output governance, not row admission. |

---

## 3. The EVIDENCE-INTEGRITY checklist (EI-01..EI-08 analogs)

Each item is tagged **ENFORCED-NOW**, **PARTIAL**, or **GAP**, with its code hook point. GAP/PARTIAL items are the work the unified `ensure_evidence_integrity()` (§7) must do.

- [ ] **EI-01 — Provenance pin (which knowledge state produced this truth).** `GAP`. Add a provenance pin to `Contribution` — `{retrieval_first, primed_from_plan_ids, verification_class, generator_kind, run_id}` — derived from **observable facts (did `query()` return a hit), not an agent-supplied flag**, so a confirmation can be traced to independent vs corpus-primed origin. *Hook:* `crates/corpus-client/src/schema.rs:91-101`; lane derived at `crates/support-agent/src/main.rs:289` (`corpus.query`) and `:318` (`retrieval_first`).

- [ ] **EI-02 — Control vs augmented lane.** `GAP`. Stamp each row with a **deterministic, signature-indexed lane tag** (control = `query()` returned no hit and generation was de novo; augmented = a `CorpusPrimed` candidate was scored), computed in `corpus-client` so the agent cannot choose which rows are controls. *Hook:* `crates/support-agent/src/main.rs:289-332`; `crates/panel/src/lib.rs:248-254`.

- [ ] **EI-03 — Untainted-only corroboration.** `GAP`. `fix_mappings` increments `confirmations` for **any** matching resolved row (`store.rs:39-50`) — a corpus-primed plan can inflate its own confirmation count (laundering). Increment **only** from rows whose pin shows the plan was generated independently (not `primed_from` this mapping's `plan_id`); corpus-primed confirmations are tracked separately and never alone promote a mapping. The test `confirmations_aggregate_per_plan` (`store.rs:411-423`) submits the **identical row twice and asserts `confirmations==2`** — exactly the self-corroboration this forbids. *Hook:* `crates/corpus-client/src/store.rs:39-50`.

- [ ] **EI-05 — Corroboration budget / dormancy.** `GAP`. A mapping is offered on **any** resolved row regardless of how many `Reopened`/failed rows for the same `(signature, config_class, plan.id)` followed it (`store.rs:35`). Offer a mapping only when its **net evidence** (resolved confirmations minus `Reopened`/failed rows for the same key) stays positive; a plan crossing a configurable failure budget with no net positive goes **dormant** and is no longer retrieval-preferred. *Hook:* `crates/corpus-client/src/store.rs:26-53`; labels at `schema.rs:51-61`.

- [ ] **EI-06 — Owner-only source revocation / retraction.** `GAP`. `FileCorpus` is append-only JSONL with no revocation; a bad row, once written, is served retrieval-first forever. Provide an **owner-only revocation** (`HumanConfirmed`-level, not agent-settable): a revoked plan-id/source is consulted in `fix_mappings` and at the checkpoint, refused on read with a **named reason**, and cannot accrue further confirmations; a revoked source **poisons every dependent row**. An `OutcomeLabel::Reopened` (`schema.rs:37`) must demote the prior resolved mapping. *(This is the T-104 "a retracted claim must not become truth" case.)* *Hook:* `crates/corpus-client/src/store.rs:26-53`; `gate.rs:15`.

- [ ] **EI-07 — Monotone-tightening law (raise-only on the safety bar).** `ENFORCED-NOW` — the one EI with a real in-code analog. `required_escalation` only ever raises the bar via `.max()` (a non-software route → `HumanConfirm`, an unvalidated state-changing plan → `HumanConfirm`, independent of judge confidence, `panel/src/lib.rs:324-329`); the corpus prior raises only `likelihood`, never safety/reversibility (`lib.rs:240-254`). **One GAP to lock:** assert it as an **invariant** so a future edit cannot let a `CorpusPrimed` prior lower escalation or touch the safety axis, and so `required_escalation` is provably never below `escalation_for`'s base. *Hook:* `crates/panel/src/lib.rs:240-254, 317-331`.

- [x] **EI-08 — Signature gate: sign-off must be ATTESTED, not asserted.** `IMPLEMENTED (library — Increment 2, ed25519).` The sign-off authority (`provenance::SignOffAuthority`) holds an ed25519 private key and signs the canonical `(signature, plan, label, sign_off, config_class)` tuple; the engine embeds only `SignOffPublicKey` and re-verifies at the gate (`ensure_attested`). A store configured with `.with_authority(pubkey)` refuses any confirmed row whose attestation is missing or does not verify — so a `Contribution{ sign_off: HumanConfirmed }` constructed by the submitting process is refused (it cannot mint a valid signature without the private key). **Remaining (see §8 / FOLLOWUPS):** operator/CLI wiring to generate+supply the key and produce attestations at human sign-off time; key rotation / multi-key registry; separate verifier-vs-human authorities. *Hook:* `crates/corpus-client/src/gate.rs` (`ensure_attested`); `crates/provenance/src/lib.rs` (`SignOffAuthority`/`SignOffPublicKey`/`SignOffSignature`); `crates/corpus-client/src/schema.rs` (`SignOffAttestation`, `attested_by`, `attestation_message`).

**The unified checkpoint (the must-have that binds EI-01..EI-08):**

- [ ] **ONE funnel.** Replace the hollow `ensure_signed_off(&Contribution){ sign_off.is_confirmed() }` with **`ensure_evidence_integrity(&EvidenceBundle)`** that all three stores call before any state change (`store.rs:116`, `store.rs:184`, `store.rs:251`), reporting **which** of (a)–(e) failed via a structured per-item `GateError` (mirroring EI-06's named-reason refusals). It admits a row ONLY when it jointly binds:
  - **(a)** a real **Verdict** (`Pass`/`ProvisionalPass`, `crates/agent-core/src/verify.rs`) over a **re-collected** post-signature — rejecting the bootstrap-trivial case where `post` is re-derived from the same request text (`main.rs:558-559`), which structurally cannot observe a fix;
  - **(b)** a **judge/provenance attestation** over `(signature, plan, label, sign_off, config_class)` from a **non-ephemeral, identified** key (closing `provenance/src/lib.rs:11-16`);
  - **(c)** a **de-id proof**: leakage-suite-green for the current schema/vocabulary version **+** a structural re-check that the stored plan equals `de_identify_plan(plan)`;
  - **(d)** a present, **honestly-derived `ConfigClass`** — rejecting the OS+ARCH bootstrap stub (`main.rs:742-747`);
  - **(e)** a `sign_off` level **matching the consent/escalation** that authorized execution (human for destructive/hardware/ambiguous per `panel/src/lib.rs:317-331`; verifier only for reversible software-state) — enforced in `corpus-client`, not only at the CLI (`main.rs:428-444`).

---

## 4. The RESEARCH checklist (PP analogs)

The dominant validity threat is **self-evaluation**: the same loop that writes the corpus reads it back (retrieval-first), strengthens it (confirmation counts), and labels its own outcomes — and in the CLI bootstrap the verifier re-derives the post-signature from the **same request text** (`main.rs:558-559`), so any software-state run trivially self-confirms. A paper reporting corpus accuracy without controlling for this is reporting its own echo.

**`docs/research/` tree (port of CEC-Platform's five-doc set — none exist in this repo today):**

- [ ] `README.md` — index + **M1–M4 milestone ladder** + the **commit-ordering honesty rule** ("read `git log -- docs/research/` before trusting any 'we predicted X'"). M1 = first signed-off row clearing the gate with a **bound** verdict; M2 = preregistered retrieval-OFF vs retrieval-first run to its committed N on held-out signatures, every row carrying a lane pin; M3 = ablation (drop the `CorpusPrimed` prior / drop retrieval); M4 = preprint with cites filled and limitations stated. *Discussion/uplift is not written before M2 exists.*
- [ ] `claims.md` — **at most two** falsifiable claims, each one sentence with a single named kill experiment:
  - **C1**: "Retrieval-first from a sign-off-gated corpus resolves held-out same-config-class signatures at a higher rate than cold generation." **KILL**: the preregistered retrieval-OFF lane shows no uplift, *or* uplift vanishes once bootstrap self-confirmations are excluded. *(GAP — no control lane to kill it yet.)*
  - **C2**: "The sign-off gate + de-identification-by-extraction admit zero unsigned and zero PII-bearing rows, and hard negatives are never retrieved as fixes." **KILL**: the leakage suite finds one seeded identifier in a serialized row, *or* a `Contribution{sign_off: HumanConfirmed}` constructed without a real human action passes the gate (**it does today — must be worded as a discipline claim, not a cryptographic guarantee**). *(Leakage/quarantine half ENFORCED; "zero unsigned" half PARTIAL.)*
  - Every prior-art gap tagged **`[CITE NEEDED] (do NOT invent)`** and left empty until a real citation is found.
- [ ] `negative-results.md` — **commit-timestamped BEFORE `claims.md`**; surfaces at **abstract level**: **NR-1** bootstrap self-confirmation (`main.rs:558-559`); **NR-2** caller-asserted, unproven `SignOff` (`schema.rs:100` / `gate.rs:16`); **NR-3** verdict computed but never bound into the row; **NR-4** `FileCorpus` has no per-row tamper-evidence (`store.rs:181-197`); **NR-5** self-evaluation (the headline). **Limitations** in the abstract: single board family, single reviewer/owner (N=1), no external replication, self-evaluation, in-process ephemeral signing key (`provenance/src/lib.rs:51`).
- [ ] `prereg-control-lane.md` — **commit-timestamped BEFORE the first corpus row carries a `lane`/`corpus_state` field** (else VOID). Locks a **deterministic, agent-ungameable, signature-indexed** retrieval-OFF control lane (toggles `corpus.query` at `main.rs:289`, not a steering weight), the primary metric (resolution rate on held-out signatures, retrieval-first vs cold), exclusions (bootstrap-only runs, `OffMachine`/hardware, `Withdrawn`), N + minimum-analyzable counts, and a **one-sided** success criterion. **§0 precondition:** real post-fix re-collection must be wired or the software-state arm is degenerate.
- [ ] `instrumentation-inventory.md` — every claimed number → named mechanism → file:line → status (`DONE`/`BUILDING`/`ORPHAN-BY-SELF-EVALUATION`) → precondition-to-flip; a **partition column** (influenced/uninfluenced). **Zero rows may read "hope to compute later."** The bootstrap `ResolvedConfirmed` rate is marked **ORPHAN/excluded** until real re-collection is wired.

| Number / claim | Mechanism | file:line | Status |
|---|---|---|---|
| zero PII leakage | adversarial leakage suite | `corpus-client/src/lib.rs:34-127` | DONE |
| zero unsigned rows (discipline) | sign-off gate | `corpus-client/src/gate.rs:15` | DONE (PARTIAL: enum not proven) |
| hard negatives never re-offered | quarantine filter + test | `store.rs:35-38` / `store.rs:334-348` | DONE |
| resolution-rate uplift (C1) | retrieval-first vs control lane | `main.rs:289,318` + `panel:248-254` | BUILDING (no lane yet) |
| "verified-resolved" rate | verdict bound into row | `verify.rs` → `schema.rs:91-101` | BUILDING (verdict dropped) |
| bootstrap `ResolvedConfirmed` rate | bootstrap collector | `main.rs:558-559` | **ORPHAN-BY-SELF-EVALUATION** |

**Standing research disciplines:**

- [ ] **No-orphan rule.** Every number in `claims.md`/`prereg` is one inventory row with a file:line mechanism; a metric with no backing mechanism fails CI, not just review.
- [ ] **No single-arm corpus numbers.** No corpus-accuracy/uplift number outside the preregistered retrieval-OFF vs retrieval-first comparison.
- [ ] **Dark-seat / QUORUM-not-FULL honesty.** Cold-start (empty-corpus) runs are **partitioned out** of any corpus-backed claim; a verifier reading an empty corpus reports lower confidence, never a flat full claim.
- [ ] **Commit-timestamp honesty.** Any "we predicted/ensured X" rests on an artifact whose `git log` timestamp predates the data. *(Risk: the recon env reports this working dir is **not a git repo** — `docs/research/` must live where `git log` is meaningful or this guarantee is unenforceable.)*

---

## 5. THE RUNNABLE CHECKLIST

The ordered list an agent/contributor literally ticks. **Checklist A** is enforced inside `corpus-client` (embedders cannot skip it); **Checklist B** is the evidence-honesty discipline before claiming a finding is true.

### Custody model (state before either checklist)

> **TWO ZONES on the SignOff ladder.** **STAGING** = `{Unconfirmed, VerifierConfirmed}` — agent/auto-verifier-reachable, admissible **only for reversible/software-state plans**; advisory (may prime generation, may **never** lower an escalation bar, may never self-promote). **PROMOTED** = `HumanConfirmed` **and** the route-forced human boundary (hardware-evidenced/ambiguous, `panel/src/lib.rs:324-326`) — the **only** blocking, **agent-unforgeable** zone, minted only by an owner-bound action the machine account cannot self-issue. **Boundary** = `corpus-client/src/gate.rs:15`. **RULE:** an agent on a machine account may write STAGING rows freely (subject to de-id + gate) but may **never** mint a PROMOTED row by self-asserting `HumanConfirmed`.

### A — Before ANY corpus write-back

- [ ] **A1. DE-ID PROVEN.** The serialized row leaks zero seeded identifiers (leakage suite green, `corpus-client/src/lib.rs:34-127`) **and** if fault vocabulary / plan shape / schema changed this turn, the suite was extended. *Runtime:* `Contribution::new` de-identifies by construction (`schema.rs:107-141`). *CI:* leakage suite + a test asserting `de_identify_plan` output carries no field beyond `{id, action, risk}`.
- [ ] **A2. SIGN-OFF LEVEL LEGITIMATE FOR THE PLAN'S RISK.** `VerifierConfirmed` only for reversible/software-state; destructive or hardware/ambiguous-routed plans require `HumanConfirmed` (`panel/src/lib.rs:317-331`). *Runtime:* move the route+risk check from the CLI (`main.rs:428-434`) **into the gate**. *CI:* test that `submit()` rejects `VerifierConfirmed` on a destructive/hardware-evidenced contribution.
- [ ] **A3. HUMANCONFIRMED IS OWNER-ATTESTED, NOT SELF-ASSERTED.** A `HumanConfirmed` row carries an owner-key attestation over `(signature, plan, label, config_class, sign_off)`, verifiable with a key the running process does **not** hold. *Runtime:* extend `provenance` (`lib.rs:63-80`) to a `SignedContribution`; `ensure_evidence_integrity` verifies it. *CI:* a process holding only the bot key cannot produce a passing `HumanConfirmed` row. **Closes the #1 governance gap (EI-08).**
- [ ] **A4. RESOLVED LABELS ARE EVIDENCE-BACKED OVER THE SAME INSTRUMENT.** `ResolvedConfirmed`/`ResolvedProvisional` only if a `Pass`/`ProvisionalPass` verdict (`agent-core/src/verify.rs`) was produced over a **re-collected** signature that is **not structurally identical** to the original input; the verdict + `VerificationClass` are recorded on the row. *Runtime:* refuse a resolved label when `post == original-by-construction` (catches `main.rs:558-559`); carry the verdict into `Outcome` (`schema.rs:76-83`). *CI:* a bootstrap run cannot mint `ResolvedConfirmed`.
- [ ] **A5. MONOTONE / NO-LOOSENING + INDEPENDENT CONFIRMATIONS.** No agent-reachable (`Unconfirmed`/`VerifierConfirmed`) row lowered an escalation bar, auto-resolved, or self-promoted; the confirmation count increments **only from an independent run** (distinct `run_id`/lane pin). *Runtime:* independence check in `fix_mappings` (`store.rs:39-50`). *CI:* submitting the same row twice does **not** double the confirmation count (today it does — `confirmations_aggregate_per_plan`).
- [ ] **A6. HARD-NEGATIVE QUARANTINE HOLDS.** A non-resolved label is admitted (labeled + signed, never dropped) **and** is never retrievable as a fix (only `is_resolved()` rows back a `FixMapping`, `store.rs:35-38`). *CI:* keep `hard_negatives_are_stored_but_not_retrieved_as_fixes` (`store.rs:334-348`) green.
- [ ] **A7. CONFIG-CLASS HONESTLY DERIVED + SCOPING PRESERVED.** The row carries a `config_class` from real inventory (CIM hardware/driver or BOM revision), **not** OS+ARCH (`main.rs:742-747`), attested to the producing machine; retrieval stays class-scoped (`store.rs:33-34`). *CI:* keep `retrieval_is_scoped_to_the_config_class` (`store.rs:425-436`) green; a coarse OS+ARCH-only class is flagged. *(Until real CIM derivation lands, log a FOLLOWUPS line.)*
- [ ] **A8. APPEND-ONLY / TAMPER-EVIDENT + RE-VALIDATED ON LOAD.** The write is append-only (`store.rs:181-197`); an unparseable corpus is an error not silent loss (`store.rs:142-144`); each row carries the A3 attestation, and `FileCorpus::open` (`store.rs:136-157`) re-runs the integrity check on every loaded row and drops/errors failures — so a hand-edited precedent is never served. *CI:* a tampered JSONL line (label flipped, plan swapped) fails verification on reopen.
- [ ] **A9. NO EVAL/HOLDOUT CASE ADMITTED AS TRUTH.** Contributions carry an **origin tag** (field vs eval/holdout/synthetic); the gate refuses eval-origin rows into the served corpus, and the served-query path is provably disjoint from any holdout set. *CI:* an eval-origin contribution is rejected by `submit`; a query can never return a holdout-derived row.
- [x] **A10. CURATED GROUND TRUTH IS GATED THE SAME WAY (reproducible, no seed).** Hand-authored ground truth (the YAML fix-flow path, in the private corpus repo) passes the **same** admissibility + de-identification gate as runtime write-back, enforced **reproducibly and without the signing seed**: the curated-ingestion compiler validates every flow — vocabulary-only symptoms (the single-extractor-token de-id rule), allowlisted actions, every label↔verdict / destructive↔human coupling, and `ensure_evidence_integrity` — and refuses anything inadmissible or identity-bearing. *Runtime:* it de-identifies (`de_identify_plan`) and ed25519-attests every row; an unattested/forged/hand-edited row is refused at `FileCorpus::open` (A8). *CI/test:* a **seedless `corpus-ingest check`** runs in the private repo's merge-gate workflow on every proposed entry branch — a bot may push but **cannot merge** (propose-then-authorize) — and the crate's tests assert that an identity-bearing symptom (e.g. a spaced hostname masquerading as a module) and an inadmissible coupling are rejected. **So every entry that backs a paper claim is admissible and de-identified _by construction_, provably, before it can be merged.** *(This is the curated half; A1's leakage suite is the runtime half.)*

### B — Before claiming a result/finding is true

- [ ] **B1. PROVENANCE NAMED.** Every number/claim maps to a named existing mechanism (file:line) — the no-orphan rule. A claim with no mechanism does not ship.
- [ ] **B2. EVIDENCE, NOT MODEL OUTPUT.** The claim rests on a verified outcome (a verdict, a passing test, a measured value), never on model-generated prose. A retracted/no-conclusion trace is not reported as true (the T-104 zero-tolerance case).
- [ ] **B3. FALSIFIABLE + KILL EXPERIMENT.** One falsifiable sentence with the single experiment that would kill it; ≤2 claims total. A claim without a named kill experiment is exploratory, not a result.
- [ ] **B4. NEGATIVE RESULTS FIRST.** Limitations bounding the claim are written **before** the claim and not buried (bootstrap self-confirm `main.rs:558-559`; ephemeral key `provenance/src/lib.rs:11-16`); the claim must survive them.
- [ ] **B5. ORDERING/TIMESTAMP HONESTY.** Any "we predicted/ensured X" rests on a commit-timestamped artifact predating the data — check `git log`; the append-only `FOLLOWUPS/TODOS/HANDOFFS` ledger gives the auditable trail.
- [ ] **B6. CITE GAPS TAGGED, NOT INVENTED.** External prior-art claims are tagged `[CITE NEEDED]` and never fabricated.

---

## 6. Adversarial: attack → defense

> **The iron rule (already native to this repo's de-id design):** *a checklist item with no adversarial test silently regresses.* Every must-have ships **only** with (1) a runtime gate (in `corpus-client` for A-items) **and** (2) a CI/test invariant in `cargo test --workspace` that fails if the gate is removed.

| # | Attack on the inverted corpus | Defense (checklist item) | Code hook | Status |
|---|---|---|---|---|
| MH-1 | Construct `Contribution{sign_off: HumanConfirmed}` directly — the gate passes (one forgeable enum) | **Sign-off attested, not asserted**: gate requires a verifiable ed25519 attestation by an authority whose private key the submitting process does not hold | `gate.rs` `ensure_attested`; `provenance` `SignOffAuthority` | **DONE (library, Increment 2)** — store `.with_authority(pubkey)`; operator wiring + rotation deferred |
| MH-2 | Flip/forge a label; a "resolved" row is unauditable because no evidence is stored | **Verdict bound into the row** (`Pass`/`ProvisionalPass` + `VerificationClass` + recurring-symptom diff); gate rejects an `is_resolved()` row whose verdict isn't a pass | `schema.rs:91-101`; `verify.rs` | GAP |
| MH-3 | Every completed software-state run auto-mints `ResolvedConfirmed` (post re-derived from request text, diff always empty) | **Resolved requires a real re-collection** from a live instrument, attested by a distinct collection-run id; bootstrap-echo rows are advisory-only | `main.rs:558-559` | GAP |
| MH-4 | Hand-edit a confirmed JSONL precedent; it is reloaded and served retrieval-first, bypassing the gate | **Per-row tamper-evidence + re-validate on load**: `FileCorpus::open` re-runs the integrity check; append-only by signature/hash-chain, not OS perms | `store.rs:138-144,181-197` | GAP |
| MH-5 | Untrusted model prose enters a `PlanStep` with unreconciled risk (claims `ReadOnly` for a destructive action) | **Model output is not a source**: risk-vs-action reconciliation + de-id at generation; advisory (out-of-vocabulary) plans never back a `FixMapping` or count as executed/resolved | `main.rs:878-886`; `schema.rs:122-141` | GAP |
| MH-6 | Coarse/forged `config_class` collapses distinct hardware; a fix laundered across unverified contexts | **Config-class honestly derived + attested**; scoping preserved (`store.rs:33-34`) | `main.rs:742-747` | PARTIAL (scoping tested) |
| MH-7 | An eval/holdout case admitted as a signed row contaminates train/serve (system retrieves the case it's measured on) | **No eval/holdout case admitted**: origin tag; served set provably disjoint from holdout (CL-19 analog) | `schema.rs:91-101`; `gate.rs:15` | GAP |
| MH-8 | A retracted/proven-wrong fix stays permanent truth and keeps being preferred (T-104) | **Retraction/revocation poisons dependents**: owner-only revocation withdraws the row + its `FixMappings`/confirmations; `Reopened` demotes the prior mapping | `store.rs:26-53`; `schema.rs:37` | GAP |
| MH-9 | A schema/vocabulary change opens a leak the suite does not cover | **Standing leakage gate**: suite stays green in CI and is **extended** on every schema/vocabulary/plan-shape change; no item ships without an adversarial test | `corpus-client/src/lib.rs:34-127` | PARTIAL (suite green; obligation new) |
| EI-03/A5 | Re-submit the identical row to manufacture false confidence (`confirmations==2` from one run) | **Independent-confirmation guard** keyed on `run_id`/lane | `store.rs:39-50,411-423` | GAP |

**Sequencing:** **MH-1 is the keystone — now DONE at the library level (Increment 2, ed25519).** With the attestation in place, MH-2/3/4/7/8 can bind verdict/origin/revocation to a row that a caller cannot forge. Remaining keystone work is operator wiring (supplying/holding the authority key) and rotation — see §8 / FOLLOWUPS.

---

## 7. Enforcement plan

**Runtime gate (authoritative).** Replace `ensure_signed_off` (`gate.rs:15`) with **`ensure_evidence_integrity(&EvidenceBundle) -> Result<(), GateError>`**, where `GateError` becomes a **structured per-failed-item enum** (so a refusal names which of A1–A9 failed, mirroring EI-06's named reasons). All three stores call it before any state change (`store.rs:116`, `store.rs:184`, `store.rs:251`). Enforcement lives in **`corpus-client`** — never only in the CLI — so MyOwnLLM/AllMyStuff embedders cannot bypass it. The `EvidenceBundle` widens `Contribution` to carry: the bound verdict + `VerificationClass`, the provenance/judge attestation (`SignedContribution`), the de-id proof marker, the honestly-derived `ConfigClass`, the origin tag, and the lane/provenance pin.

**CI / test invariants (regression backstop).** Each A-item ships with a paired adversarial test in `cargo test --workspace` / `.github/workflows/ci.yml`: extend the leakage suite (`corpus-client/src/lib.rs:34-127`) so adding a serialized field without a zero-leakage assertion fails the build; add the monotone-tightening invariant (EI-07); add the independence test (a duplicate row does not inflate confirmations); add the tamper-evidence-on-reopen test. Mirror the no-orphan instrumentation audit as a doc/test lint. **Extend `SECURITY.md`'s invariant list** to name each new gate (sign-off attestation, verdict binding, real re-collection, tamper-evidence, revocation, origin isolation) so a bypass is a **reportable security issue**.

**Human-side governance ledger (`FOLLOWUPS.md` / `TODOS.md` / `HANDOFFS.md`).** These are append-only **with tombstones** (never delete a line — flip `[ ]→[x]` and append a closed/done tombstone; UTC date+time), injected every `SessionStart` by `.claude/hooks/{followups,todos,handoffs}-context.sh` (wired in `.claude/settings.json`) and mirrored off-tree to `ops/agent-handoff` by `session-end.sh`. Wiring: every checklist item **deferred** this turn → a `FOLLOWUPS.md` line (with where-to-resume); every item **in flight** → `TODOS.md`; the resume baton + lessons → `HANDOFFS.md` — all in the same turn. **Activate the dormant custody guard:** `core.hooksPath` is **not set** in this clone, so the corpus/weights pre-commit exfil guard is dormant until `git config core.hooksPath scripts/githooks` runs (HANDOFFS lesson 2026-06-14 19:50) — a one-line custody hole to close.

---

## 8. Deferred items (copy verbatim into `FOLLOWUPS.md`)

> These were the deferred items at authoring time. **`FOLLOWUPS.md` is the live tracker** (append-only with
> tombstones); **§9 below records what has since been implemented.** Items already shipped are struck through
> here for orientation.

- [x] ~~**[EI-08 / MH-1 — keystone]** Implement owner-key attestation over the contribution tuple.~~ **DONE (library, Increment 2, ed25519)** — see §9. Operator wiring + rotation remain in FOLLOWUPS.
- [ ] **[Custody]** Decide the non-ephemeral judge key custody / rotation / audit-log retention path (`crates/provenance/src/lib.rs:11-16`); `SigningKey::generate()` at `crates/support-agent/src/main.rs:485` mints a fresh per-run key with no judge identity — the EI-08 attestation cannot be real without an identified, persistent judge.
- [ ] **[Canonicalization]** Replace serde field-order canonicalization with a sorted/canonical-JSON encoder before signatures are cross-version/cross-language verified. Resume: `crates/provenance/src/lib.rs:88-91`.
- [ ] **[MH-3 / NR-1]** Wire a real post-fix re-collection that replaces the bootstrap echo `signature_of(&collect_diagnostics(&args.describe))` so the bound verdict reflects a genuine post-state diff and `ResolvedConfirmed` cannot be trivially minted. Resume: `crates/support-agent/src/main.rs:558-559`.
- [x] ~~**[MH-2 / EI-01]** ...bind the `verify.rs` Verdict + recurring-symptom diff into `Outcome`...~~ **PARTIALLY DONE (Increment 1)**: the Verdict + recurring diff are bound (`Outcome.verification: Option<common::Verification>`) and the gate rejects a resolved label without a matching pass. Remaining: carry `VerificationClass` + a provenance/lane pin — see §9 and FOLLOWUPS.
- [ ] **[EI-03 / A5]** Add a run-independence guard to confirmation aggregation keyed on `run_id`/lane, with a test that a duplicate row does not inflate the count. Resume: `crates/corpus-client/src/store.rs:39-50,411-423`.
- [ ] **[MH-4 / MH-8 / EI-06]** Add per-row tamper-evidence (signature or hash chain) + an owner-only revocation/retraction list to `FileCorpus`; re-verify on `FileCorpus::open`; have `fix_mappings` honor revocation and let `OutcomeLabel::Reopened` demote a prior resolved mapping. Resume: `crates/corpus-client/src/store.rs:26-53,136-157,181-197`.
- [ ] **[MH-6 / A7]** Derive `config_class` from real CIM hardware/driver inventory (or BOM revision) instead of OS+ARCH, attested to the producing machine. Resume: `crates/support-agent/src/main.rs:742-747`.
- [ ] **[MH-5]** Validate model-generated steps (claimed-risk-vs-actual-action reconciliation) and de-identify at generation; add inference-channel provenance (no cert pinning / endpoint / model attestation today) so a swapped endpoint is visible on the row. Resume: `crates/support-agent/src/main.rs:878-886`.
- [ ] **[Sandbox evidence]** Provide a production `SandboxValidator` impl (the `swarm` trait has none; the CLI hardcodes `sandbox_validated=false`, `main.rs:376`) and decide whether sandbox evidence is bound into the row, so "unvalidated equals escalate" is backed by positive validation evidence.
- [x] ~~**[Research tree]** Create `docs/research/` ...~~ **DONE (scaffolded)** — `docs/research/` exists (README + populated negative-results + instrumentation-inventory + claims/prereg scaffolds). Filling claims/prereg per the commit-ordering discipline remains in FOLLOWUPS.
- [ ] **[Custody activation]** Run `git config core.hooksPath scripts/githooks` to activate the corpus/weights pre-commit exfil guard (dormant in fresh clones) — *still open*. `SECURITY.md`'s invariant list **has** been extended to name the strengthened gate (Increment 1).

---

## 9. Implementation status (changelog)

The §1–§7 design is the spec; this records what has been built against it on branch
`feat/agent-ops-evidence-integrity`. `FOLLOWUPS.md` carries the remaining engine work.

### Increment 1 — structured gate + bound verdict (commit `c9af199`)
- `ensure_signed_off` → **`ensure_evidence_integrity`** with a structured `GateError`
  (`Unconfirmed` / `ResolvedWithoutPass` / `LabelVerdictMismatch` / `DestructiveFixNeedsHuman`).
- The verifier's verdict is **bound into the row**: `common::Verification { result, recurring }` on
  `Outcome` (`Verdict::to_verification()`); a **resolved label now requires a matching passing verdict**, and
  a **resolved destructive plan requires human sign-off** — enforced inside `corpus-client`, not just the CLI.
- Hard negatives are still admitted freely (a failure is truth too).

### Increment 2 — MH-1 keystone: ed25519 sign-off attestation (commit `<this>`)
- Owner chose the **asymmetric** trust model. `provenance::SignOffAuthority` holds an ed25519 **private** key
  and signs the canonical `(signature, plan, label, sign_off, config_class)` tuple
  (`attestation_message`); the engine embeds only `SignOffPublicKey` and re-verifies (`ensure_attested`).
- `Contribution` gains an optional `attestation` (`SignOffAttestation`) set by `attested_by(&authority)`.
  Stores gain **`.with_authority(pubkey)`**; when configured they refuse any confirmed row whose attestation
  is missing or invalid. A `Contribution{ sign_off: HumanConfirmed }` built by the submitting process is
  therefore **refused** — it cannot produce a valid signature without the private key.
- Cold start (no authority) is unchanged (back-compat); proven by tests.
- `ed25519-dalek` added; its dependency tree is license-clean against `deny.toml`.

**Verification:** `cargo build/test --workspace` (136 tests), `cargo fmt --check`, `cargo clippy -D warnings`
all clean; cold-start CLI smoke OK. The resolved-accept path needs a Windows host to exercise live (off-Windows
the tools report unsupported); it is covered by unit tests.

**Still open (in `FOLLOWUPS.md`):** MH-1 operator/CLI wiring + key rotation + verifier-vs-human authorities;
MH-2 remainder (`VerificationClass` + lane pin); MH-3 (real post-fix re-collection, NR-1); EI-03/A5 (independent
confirmations); MH-4/8/EI-06 (tamper-evidence + revocation); MH-5 (model-output validation); MH-6 (honest
config-class); canonical-JSON plan encoding; sandbox-validation evidence; filling the research tree.
