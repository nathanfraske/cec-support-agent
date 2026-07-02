# Negative results

Drafted **before** `claims.md`, on purpose: negative results discipline claims. Every claim in `claims.md`
must be written to survive the items below. If a claim cannot survive a negative result here, it does not
belong in the paper. These are **real, observed** negatives from this engine's current code — not
hypothetical risks. Each cites the artifact it came from. The **Limitations** must appear in the abstract,
not buried.

> Commit-ordering: this file must be committed **before** `claims.md`. Check
> `git log --reverse -- docs/research/negative-results.md docs/research/claims.md`.

## NR-1 — Bootstrap verification cannot observe a real fix

The CLI bootstrap re-derives the post-execution signature from the **same request text** as the input
(`crates/support-agent/src/main.rs:558-559`, `signature_of(&collect_diagnostics(&args.describe))`), so the
post-vs-original diff is always empty and any completed software-state run trivially labels
`ResolvedConfirmed`. The "same instrument" guarantee (`crates/agent-core/src/verify.rs`) is only as strong as
a real re-collection, which is not wired in the CLI. **Any corpus-accuracy number from bootstrap runs is an
echo and is excluded** (see `instrumentation-inventory.md`, status `ORPHAN-BY-SELF-EVALUATION`).

## NR-2 — Sign-off is caller-asserted, not proven

`Contribution.sign_off` is a plain serde enum (`crates/corpus-client/src/schema.rs:100`) and the gate checks
only `is_confirmed()` (`crates/corpus-client/src/gate.rs:15-21`). A library embedder can construct
`Contribution{ sign_off: HumanConfirmed }` with no human in the loop and the gate passes. The "zero unsigned
rows" property is therefore a **discipline** today, not a cryptographic guarantee — it must be claimed as
such until EI-08/MH-1 attestation lands.

> **Fixed since (2026-06-14, Increments 2+9, commits `7919e56`/`7c5d9b3`; audit-hardened `11f0609`).**
> ed25519 attestation landed: with `.with_authority(pubkey)` a constructed `HumanConfirmed` is refused at
> submit and at `open`-time re-admission. The negative stands as a true observation of the pre-Increment-2
> engine and still applies to a store run **without** an authority configured (cold start).

## NR-3 — The verification verdict is computed but never bound into the row

`verify_outcome`'s `Verdict` (`crates/agent-core/src/verify.rs`, computed at `main.rs:560`) is used only to
pick an `OutcomeLabel`; the corpus row stores the label, **not** the recurring-symptom diff or the
`VerificationClass`. A `resolved` row cannot later be audited against the evidence that justified it.

> **Fixed since (2026-06-14, Increments 1+3, commits `c9af199`/`9efaa20`).** `common::Verification`
> (result + recurring diff) and `VerificationClass` are bound onto the row and gate-enforced: a resolved
> label without a matching passing verdict is refused. The negative stands as a true pre-Increment-1
> observation; any claim citing it must scope it historically.

## NR-4 — `FileCorpus` has no per-row tamper-evidence

Rows are plain appended JSONL (`crates/corpus-client/src/store.rs:181-197`) with no per-row signature or
hash chain — append-only is enforced only by OS file permissions. An operator can hand-edit a confirmed
precedent, which is then served **retrieval-first** as an authoritative fix on the next run.

> **Fixed since (2026-06-14, Increment 4, commit `8cc57a8`; audit-hardened `11f0609`).** `FileCorpus` now
> hash-chains every row (`chain_hash`, `verify_chain`) and `with_authority` re-runs the full gate over every
> at-rest row at `open`, failing closed. Residual (still open, FOLLOWUPS): the keyless chain head/tail
> anchor — a full-file rewrite by a party with file access is detectable only with an authority configured.

## NR-5 — Self-evaluation (the headline)

The corpus is read retrieval-first and is **also** the truth set the engine is measured against; the loop
labels its own outcomes and strengthens its own confirmations. Without a preregistered control lane and a
real post-fix re-collection (NR-1), reported corpus accuracy measures the loop's agreement with itself.

## Limitations (must appear in the abstract)

- **Single board family / domain** — Windows software-state + a narrow hardware-evidenced route.
- **N = 1 reviewer / owner** — one human signs off; no inter-rater reliability.
- **No external replication** — results are from one machine and one corpus instance.
- **Self-evaluation** — see NR-5; the engine evaluates a corpus it also produced.
- **In-process, ephemeral signing key** — `SigningKey::generate()` per run
  (`crates/support-agent/src/main.rs:485`; `crates/provenance/src/lib.rs:11-16`) proves intra-run integrity
  only, not which judge signed.
