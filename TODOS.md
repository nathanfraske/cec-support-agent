# TODOS

The LIVE checklist of everything the agent is **getting done**, in checkbox form, checked off as work
completes. **APPEND-ONLY** — items are never deleted; completed / obsolete ones stay as timestamped
history with a tombstone. Times are **UTC**.

Format:
- `- [ ] [added YYYY-MM-DD HH:MM UTC] <task>` — active / in progress
- `- [x] [added YYYY-MM-DD HH:MM UTC · done YYYY-MM-DD HH:MM UTC] <task>` — completed (left in place)
- `- [~] [added YYYY-MM-DD HH:MM UTC · obsolete YYYY-MM-DD HH:MM UTC → <tombstone>] <task>` — obsolete,
  with a tombstone pointing where it went (e.g. `FOLLOWUPS.md "<entry>"`, `PR #N`, another line)

Conventions:
- Every entry carries the exact date **and time** it was added, and the time it was checked off.
- Never delete a line. Flip `- [ ]` to `- [x]` (done) or `- [~]` (obsolete) and append the tombstone.
- Distinct from `FOLLOWUPS.md` (deferred / not-now backlog) and `HANDOFFS.md` (resume state).

## Active

### Session 2026-06-14 — evidence-integrity checklist + agent infra hardening

- [x] [added 2026-06-14 19:45 UTC · done 2026-06-14 19:46 UTC] Locate the real project — clone `cec-support-agent` into the `CEC_AutoDiagnoser` working dir (the GitHub `CEC_AutoDiagnoser` repo is empty; work lives in `cec-support-agent`)
- [ ] [added 2026-06-14 19:48 UTC] Recon fan-out (workflow `autodiagnoser-recon`): map current pipeline, CEC-Platform evidence-integrity policy + research checklist, the inverted-ground-truth-corpus approach, the WSL-ephemeral state policy, and the current local-agent infrastructure
- [x] [added 2026-06-14 19:48 UTC · done 2026-06-14 19:51 UTC] FOLLOWUPS.md SessionStart hook (date+time, append-only with tombstones — stricter than CEC-Platform) + seed file
- [x] [added 2026-06-14 19:48 UTC · done 2026-06-14 19:51 UTC] TODOS.md SessionStart hook (checklist, append-only with tombstones) + seed file
- [x] [added 2026-06-14 19:48 UTC · done 2026-06-14 19:51 UTC] HANDOFFS.md SessionStart hook (baton: current state + pick-up-here + lessons; injects contents at session start) + seed file
- [ ] [added 2026-06-14 19:48 UTC] WSL-ephemeral state parity: session-start + session-end durability hooks (durable in-tree memory mirror + off-tree handoff branch push), adapted to this git repo
- [ ] [added 2026-06-14 19:48 UTC] `.claude/settings.json` wiring all SessionStart + Stop hooks
- [ ] [added 2026-06-14 19:48 UTC] Design panel + write the **evidence-integrity & research checklist** adapted to the inverted-ground-truth-corpus approach (the engine's truth is accreted from signed-off outcomes)
- [ ] [added 2026-06-14 19:48 UTC] Document the **current local-agent infrastructure** (it changed — fresh doc)
- [ ] [added 2026-06-14 19:48 UTC] Final verification: hooks executable + syntax-clean, settings.json valid, all required files present; update HANDOFFS.md; commit on a branch

## Done / obsolete (history)

_(completed items stay above, in place, with their `· done` tombstone)_
