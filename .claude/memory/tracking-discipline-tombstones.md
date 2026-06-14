---
name: tracking-discipline-tombstones
description: "Owner wants FOLLOWUPS/TODOS/HANDOFFS append-only with tombstones, UTC date+time, never deleted"
metadata: 
  node_type: memory
  type: feedback
  originSessionId: 109a36d0-7dee-4d46-bdc4-c738f00cd762
---

The owner requires strict, auditable worklist discipline in this repo: `FOLLOWUPS.md` (deferred/not-now),
`TODOS.md` (live checklist of work being done), `HANDOFFS.md` (cross-agent baton: current state + exact next
step + append-only lessons). All are **append-only with tombstones** — never delete a line; flip `- [ ]` to
`- [x]`/`- [~]` and append a tombstone pointing where it went. Every entry carries the **UTC date AND time**.

**Why:** the deferral/work history must be fully auditable and can never be silently rewritten. This is a
deliberate tightening over CEC-Platform, whose FOLLOWUPS.md used date-only and *deleted* resolved items.

**How to apply:** update the relevant file in the SAME turn as the work; record anything deferred (don't drop
it); put lessons learned in HANDOFFS.md. SessionStart hooks inject each file's policy + HANDOFFS contents.
See [[project-repo-identity]] and [[wsl-ephemeral-durability]].
