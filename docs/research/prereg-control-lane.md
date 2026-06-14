# Preregistration — retrieval control lane

**Status: SCAFFOLD — NOT yet preregistered.** No corpus row carries a `lane` field yet, so nothing is
governed by this document. It becomes a binding preregistration only when filled and committed **before** the
first lane-tagged row exists.

> **VOID rule.** This document must be commit-timestamped in git **before any corpus row carries a `lane`
> field**. Check `git log -- docs/research/prereg-control-lane.md` against the first appearance of a `lane`
> field. **If lane-tagged data predates this commit, this preregistration is VOID** and the experiment must be
> re-designed and re-registered.

## §0 — Preconditions (must all hold before any lane-tagged round)

- [ ] **Real post-fix re-collection is wired** — the bootstrap echo (NR-1, `main.rs:558-559`) is replaced so
  the software-state arm can actually observe a fix. *Without this the control arm is degenerate.*
- [ ] Every corpus row carries a **lane** field and a **provenance/lane pin** (EI-01) identifying the
  knowledge state it saw.
- [ ] Inputs (held-out signature set, config classes, model/endpoint) are **frozen** for the run.

## §1–§9 — To be locked before the first lane-tagged round

1. **§1 Toggle.** The control lane is **retrieval-OFF**: it toggles `corpus.query` at
   `crates/support-agent/src/main.rs:289` (a structural switch), **not** a steering weight.
2. **§2 Assignment.** Deterministic, **signature-indexed**, agent-ungameable (the agent cannot choose which
   rounds are controls). _Lock the exact rule (e.g. every Nth signature by stable hash)._
3. **§3 Primary metric.** Resolution rate on **held-out** signatures, retrieval-first vs cold. _Lock it._
4. **§4 Secondary / derived metrics.** _Lock them._
5. **§5 Exclusions (locked before analysis).** Bootstrap-only runs (NR-1), `OffMachine`/hardware outcomes,
   `Withdrawn` tickets.
6. **§6 Run count + minimum analyzable thresholds.** _Lock N and the minimum per arm._
7. **§7 Success criterion.** **One-sided.** _Lock it._
8. **§8 Exploratory.** Anything not preregistered here is exploratory, labeled as such.
9. **§9 Self-evaluation limitation.** Disclosed per `negative-results.md` NR-5.

_Fill and commit this (with §0 satisfied) before tagging any row with a lane. Tracked in `FOLLOWUPS.md`._
