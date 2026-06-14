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
- [x] [added 2026-06-14 19:48 UTC · done 2026-06-14 20:00 UTC] Recon fan-out (workflow `autodiagnoser-recon`, 5 agents): mapped current pipeline + 11 evidence-integrity points + 8 checklist hook points + 11 gaps; CEC-Platform EI-01..08 + research paper-track PP-01..13; the inverted-ground-truth-corpus model; the WSL-ephemeral state policy (parity spec); and the current local-agent infra (cec-llm-broker :8080, hybrid WSL-docker + Windows-native seats). Findings saved to `.claude/recon/*.json`
- [x] [added 2026-06-14 19:48 UTC · done 2026-06-14 19:51 UTC] FOLLOWUPS.md SessionStart hook (date+time, append-only with tombstones — stricter than CEC-Platform) + seed file
- [x] [added 2026-06-14 19:48 UTC · done 2026-06-14 19:51 UTC] TODOS.md SessionStart hook (checklist, append-only with tombstones) + seed file
- [x] [added 2026-06-14 19:48 UTC · done 2026-06-14 19:51 UTC] HANDOFFS.md SessionStart hook (baton: current state + pick-up-here + lessons; injects contents at session start) + seed file
- [x] [added 2026-06-14 19:48 UTC · done 2026-06-14 19:57 UTC] WSL-ephemeral state parity: session-start + session-end durability hooks (durable in-tree memory mirror + off-tree handoff branch push), adapted to this git repo — VERIFIED end-to-end: Stop hook pushed `ops/agent-handoff` to the remote with `main` untouched; hardened against the no-git-identity post-wipe gotcha
- [x] [added 2026-06-14 19:48 UTC · done 2026-06-14 19:57 UTC] `.claude/settings.json` wiring all SessionStart (session-start + followups + todos + handoffs) + Stop (session-end) hooks — valid JSON, all 5 hooks present + executable
- [x] [added 2026-06-14 19:48 UTC · done 2026-06-14 20:08 UTC] Design panel (4 lenses: runtime-code / research-track / governance / red-team, workflow `autodiagnoser-evidence-checklist`) + write the **evidence-integrity & research checklist** adapted to the inverted-ground-truth-corpus approach → `docs/evidence-integrity-and-research-checklist.md` (the engine's truth is accreted from signed-off outcomes; sign-off gate = truth-admission boundary; unified `ensure_evidence_integrity()` design + runnable checklist + attack→defense table)
- [x] [added 2026-06-14 19:48 UTC · done 2026-06-14 20:08 UTC] Document the **current local-agent infrastructure** (changed) → `docs/local-agent-infrastructure.md` (cec-llm-broker :8080 front door, hybrid WSL-docker + Windows-native seats, what changed)
- [x] [added 2026-06-14 20:00 UTC · done 2026-06-14 20:08 UTC] Document the **WSL-ephemeral state policy** as implemented → `docs/wsl-ephemeral-state-policy.md`
- [x] [added 2026-06-14 20:09 UTC · done 2026-06-14 20:10 UTC] Copy the checklist's deferred GAP items + WSL optional-hardening into `FOLLOWUPS.md` (14 items, dated/timed)
- [x] [added 2026-06-14 20:09 UTC · done 2026-06-14 20:11 UTC] Scaffold the `docs/research/` tree (README + negative-results [populated] + claims/prereg [disciplined scaffolds] + instrumentation-inventory), commit-ordering discipline intact
- [x] [added 2026-06-14 20:11 UTC · done 2026-06-14 20:11 UTC] Add agent-ops/durability/evidence-integrity pointer section to `AGENTS.md`
- [x] [added 2026-06-14 19:48 UTC · done 2026-06-14 20:12 UTC] Final verification: all 5 hooks executable + syntax-clean + valid JSON, settings.json valid, all 14 required files present, infra doc well-formed
- [x] [added 2026-06-14 20:12 UTC · done 2026-06-14 20:15 UTC] Update HANDOFFS.md final state; commit everything on branch `feat/agent-ops-evidence-integrity` (commit `69d659d`; set repo-local git identity nathanfraske@cec.direct — the same no-identity gotcha the WSL doc warns about)
- [ ] [added 2026-06-14 20:15 UTC] **Push `feat/agent-ops-evidence-integrity` to origin** (and/or open a PR) — DURABILITY GAP: the commit is currently only on the ephemeral WSL volume; the Stop hook's `ops/agent-handoff` snapshot covers the tracking files + memory + handoff, but NOT `docs/` or `.claude/hooks/`. Awaiting owner go-ahead (push only when asked).

## Done / obsolete (history)

_(completed items stay above, in place, with their `· done` tombstone)_
