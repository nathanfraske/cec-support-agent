# F4 Windows collector — drop-in playbook

**Audience:** an agent (or human) implementing the *one* remaining piece of the
autonomous-verification / self-learning loop on a real Windows box. Everything
else is built, merged, and proven against a mock; this is the only part that
needs live hardware.

## What is already done (do not rebuild)

The re-collection **seam** and the whole downstream loop are on `main`:

- `trait PostFixCollector` in `crates/support-agent/src/main.rs` — the seam.
- `NullCollector` (returns `None`) is the default → every outcome is
  `Unverified` → escalate. This is what a non-Windows build sees today.
- `recollect_post_signature(collector)` turns a collector's events into the
  post-fix `FaultSignature` via `signature_of` (the SAME extractor that built
  the original signature — "fixed" is judged by the same instrument that
  established "broken").
- `verify_outcome` → `label_for` → `record_outcome` already:
  - a clean re-collection (fault gone) → `Verdict::Pass` → `ResolvedConfirmed`
    (or `ProvisionalPass`/`ResolvedProvisional` for intermittent faults),
  - recorded under a **`VerifierConfirmed`** sign-off with the verifier's
    ed25519 attestation → admitted to an attestation-enforcing corpus **with no
    human in the loop**,
  - a recurring symptom → `Verdict::Fail` → `EscalatedHumanUnresolved` (a hard
    negative), never a resolved row.
- Proven end-to-end by `a_verified_clean_recollection_earns_a_resolved_row_with_no_human`
  and `a_recollection_that_still_sees_the_fault_escalates_not_resolves` (mock
  collector), and red-on-revert against the collector.

## The entire task

**1. Implement one trait** in `crates/tools-windows` (a `WindowsPostFixCollector`)
that re-runs the same diagnostic collectors the ORIGINAL diagnosis used, and
returns them as `DiagnosticEvent`s:

```rust
// crates/tools-windows/src/collector.rs (new)
pub struct WindowsPostFixCollector { /* handles/config as needed */ }

impl WindowsPostFixCollector {
    /// Re-run the live collectors: recent event log (System/Application),
    /// WER + WHEA records, and the CIM state that seeded the original
    /// signature. Return them as DiagnosticEvent — do NOT pre-filter to
    /// vocabulary; `signature_of`/`extract_symptoms` does the de-id extraction,
    /// keeping the read side identical to the write side.
    pub fn recollect_events(&self) -> Option<Vec<DiagnosticEvent>> {
        // ... query Win32 / event log via the existing tools-windows plumbing ...
    }
}
```

Wrap it behind the seam (either impl `PostFixCollector` directly if the trait is
made `pub`, or adapt in `main.rs`):

```rust
impl PostFixCollector for WindowsPostFixCollector {
    fn recollect(&self) -> Option<Vec<DiagnosticEvent>> { self.recollect_events() }
}
```

**2. Swap one function** — the single wired swap point:

```rust
// crates/support-agent/src/main.rs
fn post_fix_collector() -> Box<dyn PostFixCollector> {
    // was: Box::new(NullCollector)
    Box::new(tools_windows::WindowsPostFixCollector::new(/* ... */))
}
```

Gate it so non-Windows builds keep `NullCollector`:

```rust
#[cfg(windows)]
fn post_fix_collector() -> Box<dyn PostFixCollector> {
    Box::new(tools_windows::WindowsPostFixCollector::new())
}
#[cfg(not(windows))]
fn post_fix_collector() -> Box<dyn PostFixCollector> {
    Box::new(NullCollector)
}
```

That is the whole change. No verdict, sign-off, attestation, corpus, or gate
code moves.

## Correctness bar (match the existing discipline)

- **Same instrument.** The re-collection must run the SAME collectors as the
  original diagnosis, or the diff is meaningless. Feed raw events through
  `signature_of` — never hand-build a signature.
- **Fail closed.** If re-collection cannot run (tool error, access denied),
  return `None`, not an empty/clean signature. `None` → `Unverified` → escalate.
  This is the single most important rule: **absence of evidence is not evidence
  of a fix.** The engine now backs this up — `recollect_post_signature` maps a
  `Some(vec![])` (ran but observed nothing) to `None`/Unverified, so an empty
  return can never be scored as a pass (blind-audit finding 2026-07-08). But do
  not rely on that: return `None` on failure explicitly.
- **Prove coverage — re-exercise the fault, do not just look.** A `Pass`
  requires that the re-collection actually covered the fault's domain: re-run
  the reproducing workload (the thing that triggered the original symptoms),
  then collect — collecting a healthy-looking snapshot from a machine that never
  re-ran the failing path is a false pass the engine cannot detect. This is the
  collector's contract (M2); the engine trusts that a `Some(events)` came from a
  real, fault-covering re-observation, so the collector must make that true.
- **Autonomous verification REQUIRES a configured verifier authority.** A store
  with no sign-off authority (cold start) accepts a `VerifierConfirmed` row on
  the flag alone — fine for a human operator's manual `--sign-off verifier`, but
  for the AUTONOMOUS loop it means an unattested self-asserted resolved row.
  **Never run autonomous verification against a no-authority store.** Provision
  `CEC_SIGNOFF_SEED`/`CEC_SIGNOFF_PUBKEY` (the verifier key) so every
  autonomously-resolved row is ed25519-attested by the custodied verifier, and a
  compromised box cannot mint corpus truth. See `docs/operator-runbook.md` §1
  and the FOLLOWUPS "autonomous verification authority" item.
- **Intermittent faults.** `verification_class_for` already routes intermittent
  faults to `ProvisionalPass`/`ResolvedProvisional` (monitored parole,
  auto-reopen on recurrence). The collector does not decide this — it just
  reports what it observed.
- **The verifier key is a custodied authority.** Autonomous `VerifierConfirmed`
  sign-off uses the verifier's ed25519 seed (RFC Q1/Q7 — a central custodied
  key, not a per-machine key). Provision it like the human sign-off seed
  (`CEC_SIGNOFF_SEED` custody model); a compromised box must not be able to mint
  resolved rows. See `docs/operator-runbook.md`.

## Test it on the box

Run a known-fixable fault end to end under `--sign-off verifier`:
re-collection should observe the fault gone, the run should print
`verification (Deterministic): Pass` and `outcome label: ResolvedConfirmed`,
and the corpus should gain a retrievable precedent with **no** human sign-off.
Then run an unfixed fault: it must print `Fail` / escalate and record a hard
negative, never a resolved row.
