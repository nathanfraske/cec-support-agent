# Claims

**SCAFFOLD — no claim is asserted yet.** This file is committed after `negative-results.md` (the ordering
rule) but is intentionally empty of asserted claims until its preconditions are met (real post-fix
re-collection wired — NR-1; a preregistered control lane — `prereg-control-lane.md`). Filling it is tracked
in `FOLLOWUPS.md` ("Research tree — fill").

## The rules (PP-01 analog)

- **At most two claims.** Each is **one falsifiable sentence**, with the **single experiment that would kill
  it**, the current evidence, and the cite gaps.
- **Nothing is asserted here that `instrumentation-inventory.md` does not have a named mechanism for**
  (the no-orphan rule).
- Every claim must **survive every item in `negative-results.md`**.
- A claim with no named kill experiment is **exploratory**, not a result.
- **Cite gaps are tagged `[CITE NEEDED] (do NOT invent these)`** and left empty until a real citation exists.

## Candidate claims (draft, NOT yet preregistered)

> Drafts only — to be moved above the line once their preconditions and preregistration are in place.

- **C1 (draft).** *"Retrieval-first from a sign-off-gated corpus resolves held-out, same-config-class
  signatures at a higher rate than cold generation."*
  **KILL:** the preregistered retrieval-OFF lane shows no uplift, **or** uplift vanishes once bootstrap
  self-confirmations (NR-1) are excluded.
  **Evidence today:** none admissible — no control lane exists; bootstrap runs are excluded.
  **Precondition:** real re-collection (NR-1) + `prereg-control-lane.md` committed before any lane data.

- **C2 (draft).** *"The sign-off gate + de-identification-by-extraction admit zero PII-bearing rows, and hard
  negatives are never retrieved as fixes."*
  **KILL:** the adversarial leakage suite finds one seeded identifier in a serialized row
  (`crates/corpus-client/src/lib.rs:34-127`), **or** a hard-negative row is returned by `fix_mappings`.
  **Evidence today:** leakage suite green; `hard_negatives_are_stored_but_not_retrieved_as_fixes`
  (`crates/corpus-client/src/store.rs:334-348`) green. **Note:** the "zero *unsigned* rows" half is only a
  discipline until EI-08/MH-1 attestation lands (NR-2) — claim it as such.

## Cite gaps (do NOT invent these)

- `[CITE NEEDED]` Prior art on signature/attestation-gated data or model-update pipelines.
- `[CITE NEEDED]` Prior art on provenance/lineage tracking for ML training or agentic memory.
- `[CITE NEEDED]` Prior art on retrieval-augmented / case-based diagnosis with outcome-verified corpora.
