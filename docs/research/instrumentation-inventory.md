# Instrumentation inventory

One table mapping **every number the paper will claim** → **the mechanism that produces it** → **where it
lives (file:line)** → **its current status** → **the precondition to flip it to DONE**. The rule:
**zero rows may read "hope to compute later."** Every row has a named, existing or in-progress mechanism.
**A number with no mechanism does not appear in the paper.**

**No-orphan audit:** every metric named in `claims.md` and `prereg-control-lane.md` must appear as a row here;
no row reads hope-to-compute-later.

| # | Number / claim | Mechanism | Where (file:line) | Partition | Status | Precondition to flip |
|---|---|---|---|---|---|---|
| 1 | Zero PII leakage into serialized rows | Adversarial seeded-identifier leakage suite | `crates/corpus-client/src/lib.rs:34-127` | n/a | **DONE** | — |
| 2 | Zero unsigned rows admitted | Sign-off gate `ensure_signed_off` | `crates/corpus-client/src/gate.rs:15` | n/a | **PARTIAL** (enum asserted, not proven — NR-2) | EI-08/MH-1 attestation |
| 3 | Hard negatives never re-offered as fixes | Quarantine filter + test | `store.rs:35-38` / `store.rs:334-348` | n/a | **DONE** | — |
| 4 | Retrieval-first resolution-rate uplift (C1) | Retrieval-first vs preregistered control lane | `main.rs:289,318` + `panel/src/lib.rs:248-254` | influenced vs uninfluenced | **BUILDING** | control lane (prereg §0) |
| 5 | "Verified-resolved" rate | Verdict bound into the row | `verify.rs` → `schema.rs:91-101` | n/a | **BUILDING** | verdict binding (NR-3 / MH-2) |
| 6 | Bootstrap `ResolvedConfirmed` rate | Bootstrap collector | `main.rs:558-559` | excluded | **ORPHAN-BY-SELF-EVALUATION** | excluded until real re-collection (NR-1) |
| 7 | Config-class retrieval scoping holds | Class-scoped `fix_mappings` + test | `store.rs:33-34` / `store.rs:425-436` | n/a | **DONE** (scoping) / **PARTIAL** (honest derivation — MH-6) | real CIM-derived `config_class` |

## Standing disciplines

- **No single-arm corpus numbers.** No corpus-accuracy/uplift number outside the preregistered retrieval-OFF
  vs retrieval-first comparison.
- **Dark-seat / QUORUM-not-FULL honesty.** Cold-start (empty-corpus) runs are partitioned out of any
  corpus-backed claim; a verifier reading an empty corpus reports lower confidence, never a flat full claim.
- **A row marked DONE ships only with a paired adversarial test** — a checklist item with no adversarial test
  silently regresses (`docs/evidence-integrity-and-research-checklist.md` §6).
