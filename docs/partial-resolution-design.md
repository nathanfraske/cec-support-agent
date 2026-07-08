<!-- SPDX-License-Identifier: AGPL-3.0-only -->

# Partial resolution — design

> **STATUS 2026-07-08: partial resolution BUILT (the autonomous path), blind-audited
> clean, pending merge.**
> `verify_outcome` emits `PartialPass{cleared, remaining}`; a `ResolvedPartial` row is
> gated (needs a WELL-FORMED `PartialPass` verdict — a non-empty cleared benefit AND a
> non-empty remainder, so a full clear cannot be mislabeled as a partial and steal its
> weaker credit) and admitted as
> beneficial truth (`is_beneficial`, not `is_resolved`); the cleared/introduced deltas are
> bound additively into the attestation + chain (pre-change rows stay byte-identical, no
> migration). A §7 blind audit of all six invariants (no fabricated benefit, byte-identity,
> autonomy bound, de-id, no-partial-from-empty-post, regression coherence) found NO defects;
> its one conservative note — the gate trusted rather than verified that a partial carries a
> remainder — is now closed by the well-formedness guard above. **DEFERRED (see FOLLOWUPS):** (a) autonomous regression DETECTION — the naive
> post-symptom diff cannot tell a caused regression from benign post-fix log noise, so
> `verify_outcome` does NOT emit `Regressed`; it stays a recordable outcome (label + verdict
> + gate) for a future fault-aware collector or human. (b) RETRIEVAL offering a
> `ResolvedPartial` as a partial-fix step (today it is recorded but `fix_mappings` only
> counts `is_resolved`). (c) the retry loop carrying `remaining` forward as progress.

**Owner ask (2026-07-08):** make "the fix improved things but did not fully
resolve them" a first-class outcome, *because an improvement is an improvement —
especially when we can prove the improvement happened because of the fix that was
applied.* Today the engine is binary per symptom: any original symptom that
recurs after a fix makes the whole outcome a `Fail`, so a real improvement is
thrown away as a failure.

This design adds a beneficial-but-incomplete outcome, keeps the safety
properties intact, and — crucially — grounds the "improvement was caused by the
fix" claim in evidence the engine already has.

## 1. The measurement (it's already in the diff)

Verification re-collects the post-fix signature and compares it to the original.
Today it computes only `remaining = original.recurring_in(post)`. The full
three-way diff is:

- **`cleared`** = `original.symptoms − post.symptoms` — original symptoms that
  are gone after the fix. **This is the proven benefit.**
- **`remaining`** = `original.symptoms ∩ post.symptoms` — original symptoms that
  still recur (today's `recurring`).
- **`new`** = `post.symptoms − original.symptoms` — symptoms present after that
  were NOT there before. **A regression the fix may have caused.**

Everything below is derived from these three sets; no new observation is needed.

## 2. Why the improvement is *attributable to the fix* (the causal claim)

The owner's requirement is not just "some symptoms cleared" but "we can prove the
fix caused it." The engine's existing structure supplies that:

- **Single controlled intervention.** A run executes ONE judge-signed plan. The
  pre-fix signature and the post-fix re-collection bracket exactly that plan and
  nothing else — there is no other intervention between them to confound it.
- **Bound to the run.** The outcome carries `run_id` + provenance, and the
  attestation binds them, so the `(pre, plan, post)` triple is one tamper-evident
  unit. A symptom in `cleared` was present before this plan and absent after it,
  with only this plan in between → attributable to this plan.
- **Independent repetition strengthens it.** If the SAME plan clears the SAME
  symptom across independent runs (distinct `run_id`, distinct machines), the
  attribution stops being one anecdote and becomes a rate: "this fix clears
  symptom X ~90% of the time." That is the corpus confidence machinery, applied
  to a per-symptom benefit instead of a whole-ticket pass.

So the causal claim is: *symptom X was present, this single signed plan ran,
symptom X is now absent, and that holds across independent runs.* That is as
strong as an on-machine, no-control-group setting allows, and it is exactly what
the owner means by "prove the improvement happened because of the fix."

## 3. The new outcome (semantics)

Add a beneficial-but-incomplete result and label, sitting between resolved and
failed:

- **`VerificationResult::PartialPass { cleared, remaining }`** — some original
  symptoms cleared, some remain, and no regression dominates. Produced when
  `cleared` is non-empty AND `remaining` is non-empty (see §4 for `new`).
- **`OutcomeLabel::ResolvedPartial { cleared, remaining }`** (name TBD;
  candidates: `Improved`, `PartiallyResolved`). Predicates:
  - `is_resolved()` = **false** — the ticket is NOT done; do not close it, keep
    going (next step / next-best plan / human).
  - `is_beneficial()` = **true** (NEW predicate) — it earns a corpus precedent,
    because the cleared delta is proven benefit.
- The row records the `cleared` set as the fix's **proven benefit** and the
  `remaining` set as **what is left**. Retrieval can then offer the fix as
  "known to clear {cleared} at this config class" even when it never fully
  resolves — a legitimate, confidence-weighted partial step.

## 4. Regression safety (the important guard)

A "fix" that clears symptom A but introduces symptom B is NOT a clean
improvement — trading one problem for another must never be silently recorded as
beneficial. So `new` (post-only symptoms) gates the outcome:

- `cleared` non-empty, `remaining` non-empty, `new` **empty** → `PartialPass`
  (clean improvement). Beneficial precedent.
- `new` **non-empty** (the fix introduced symptoms) → **NOT** a clean partial.
  Route to a distinct outcome — `Regressed { cleared, introduced }` — that is
  NOT `is_beneficial()` and escalates to a human, because a fix that causes new
  problems needs judgment, not autonomous credit. (This also makes regressions
  visible instead of hiding inside a `Fail`.)
- `cleared` empty → today's behavior unchanged: `Fail` (nothing improved) or
  `Pass` (all clear).

## 5. How it feeds the loop (this is where the value compounds)

- **Multi-step / retry.** Today a `Fail` feeds the next-best plan as a hard
  negative from scratch. A `PartialPass` should feed the next attempt as
  **progress**: the working set becomes `remaining`, so the loop chains partials
  toward full resolution instead of restarting. (This is exactly the DDU case:
  "clean install" partially resolves → the loop applies DDU to what's left.)
- **Corpus.** A `ResolvedPartial` row is admitted as a beneficial precedent; its
  confirmation counting accrues on the `cleared` benefit, so "this reliably
  clears X" becomes trustworthy even if the fix never clears everything.
- **Customer multi-fix loop.** A partial is honest, encouraging feedback:
  "that improved these things; want to try the next step for the rest?"

## 6. Surfaces this touches (implementation map)

Precision-critical (verification + outcome semantics + the de-id/crypto row), so
build in green sub-steps with red-on-revert and a §7 blind-audit pass:

- `crates/common` — the three-way diff on `FaultSignature` (`cleared`/`new`
  alongside `recurring_in`); `VerificationResult::PartialPass`;
  `Verification` carries the cleared/remaining deltas (de-identified vocabulary
  symptoms only — no new leak surface).
- `crates/agent-core/verify.rs` — `verify_outcome` computes the three-way diff
  and returns `PartialPass`/`Regressed`.
- `crates/corpus-client` — `OutcomeLabel::ResolvedPartial` + `Regressed`;
  `is_beneficial()`; **`label_tag` gains tokens for the new variants (this is in
  `attestation_message` AND `chain_canonical` — additive, but it is a wire/crypto
  surface: pin it and treat it as a migration-aware change)**; the gate admits a
  beneficial partial; the `FixMapping` can express a partial's cleared set.
- `crates/support-agent` — `label_for` maps the new verdicts; the retry loop
  carries `remaining` forward as progress; `record_outcome` records the delta.
- `docs/workflow-authoring-guide.md` — a `partial` result becomes authorable, and
  the confidence model reflects per-symptom clear rates.

## 7. Related: config-transition triggers (surfaced by the 5070→5080 case)

A large share of convoluted fixes are triggered by a **config transition** (a
part swapped, Windows updated, a setting changed) rather than a symptom appearing
from nowhere. The engine keys on `config_class` as a static comparability class;
recognizing a *transition* (old class → new class) as its own trigger — "a GPU
changed within the NVIDIA line ⇒ expect to need a display-driver clean-uninstall
even after a fresh install" — would capture exactly the experience-heavy cases
that feel most arbitrary. Scoped separately (see FOLLOWUPS); it composes with
partial resolution (the transition triggers the workflow; partial resolution
scores the multi-step fix).

## 8. Decision points — RESOLVED by the owner (2026-07-08)

- **Label name → `ResolvedPartial`** (over `Improved` / `PartiallyResolved`).
- **Autonomy → same rules as a full resolution.** A clean partial a MACHINE
  verified earns a `VerifierConfirmed` beneficial row — a proven per-symptom
  clear is as real as a full clear — with the SAME destructive-needs-human bound
  (a destructive partial still requires human sign-off).
- **Regression policy → always escalate.** A `Regressed` outcome is never
  autonomous credit (no net-positive threshold); it is `is_beneficial() == false`
  and routes to a human.

Owner's words: *"Yes, looks good to me."*
