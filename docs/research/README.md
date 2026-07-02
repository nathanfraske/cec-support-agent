# Research track — cec-support-agent

The research-discipline tree, ported from CEC-Platform's paper-track (PP-01..PP-13) and re-aimed at the
**inverted-ground-truth corpus**. It is deliberately separate from the engineering work in
`docs/evidence-integrity-and-research-checklist.md` (which says *what to build and what to gate*); the docs
here say **what we will claim, how each claim could be killed, and which number comes from which mechanism**.

> **Everything here is commit-timestamped on purpose.** A preregistration is only worth the bytes if it
> lands in git **before** the data it governs exists. Read `git log -- docs/research/` to check ordering
> before trusting any "we predicted X" statement. If lane-tagged data predates `prereg-control-lane.md`,
> that preregistration is **VOID** and the experiment must be re-designed and re-registered.

## The dominant validity threat: self-evaluation

The same loop that **writes** the corpus **reads** it back (retrieval-first), **strengthens** it
(confirmation counts), and **labels** its own outcomes. In the CLI bootstrap the verifier even re-derives the
post-fix signature from the **same request text** (`crates/support-agent/src/main.rs:558-559`), so any
software-state run trivially self-confirms. A paper reporting corpus accuracy without controlling for this is
reporting its own echo. Every claim here must be partitionable into knowledge-influenced vs uninfluenced
rounds, and any corpus-accuracy number must come from a **preregistered retrieval-OFF vs retrieval-first**
comparison — never a single arm.

## Milestone ladder

- **M1 — first attested promotion.** One signed-off `(signature, plan, label)` row clears the truth-admission
  gate carrying a **bound, real (non-bootstrap) verification verdict** and an owner attestation
  (EI-08 / MH-1). Not just `sign_off == HumanConfirmed` over a bare enum.
- **M2 — preregistered control dataset.** The retrieval-OFF (control) vs retrieval-first (augmented) lane run
  to its committed N on **held-out** signatures, every row carrying a deterministic, agent-ungameable lane
  pin. Requires real post-fix re-collection wired first (else the software-state arm is degenerate).
- **M3 — ablation.** Drop the `CorpusPrimed` prior / drop retrieval and re-run on held-out signatures.
- **M4 — preprint.** Claims, negative results, prereg, and the instrumentation table reconciled against the
  actual M2/M3 data; `[CITE NEEDED]` gaps filled with real citations; limitations stated in the abstract.
  *We do not write the discussion section before M3.*

## Files

| File | Role | Ordering rule |
|------|------|---------------|
| `negative-results.md` | The observed negatives that discipline the claims. | Committed **before** `claims.md`. |
| `claims.md` | At most **two** falsifiable claims, each with a named kill experiment. | After `negative-results.md`. |
| `prereg-control-lane.md` | The preregistered control/augmented lane experiment. | Committed **before** any corpus row carries a `lane` field, else VOID. |
| `instrumentation-inventory.md` | Every claimed number → mechanism → file:line → status. No orphans. | Kept in sync; a number with no row does not ship. |

## Status (2026-06-14)

`negative-results.md` and `instrumentation-inventory.md` are **populated** from real, observed artifacts.
`claims.md` and `prereg-control-lane.md` are **disciplined scaffolds** — no claim is asserted and no lane is
preregistered yet, so no ordering rule is violated by their creation. They are filled only when the
preconditions in each (real re-collection wired; a control lane designed) are met, following the ordering
rules above. See `FOLLOWUPS.md` ("Research tree — fill").
