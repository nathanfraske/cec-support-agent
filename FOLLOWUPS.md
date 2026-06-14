# FOLLOWUPS

Standing, **append-only** backlog of everything **deferred to the future / not being implemented now** —
any non-blocking item the agent chose not to do this turn but that must be revisited. Maintained by the
agent per the SessionStart followups policy. Times are **UTC**.

**Append-only with tombstones — items are NEVER deleted.** When a follow-up is done, promoted, or dropped,
it is tombstoned in place so the deferral history stays fully auditable.

Format:
- `- [ ] [added YYYY-MM-DD HH:MM UTC] <item> — <why deferred / context / where to resume>` — open
- `- [x] [added YYYY-MM-DD HH:MM UTC · closed YYYY-MM-DD HH:MM UTC → <where it went>] <item>` — done/promoted
  (tombstone points where it went: a `PR #N`, a `TODOS.md` line, another doc, or "dropped: <reason>")

Conventions:
- NON-BLOCKING items only. Blocking work is finished in-turn, not parked here.
- Every entry carries the exact date **and time** it was added.
- Never delete a line. Flip `- [ ]` to `- [x]` and append the `· closed …` tombstone instead.
- Distinct from `TODOS.md` (the live checklist of work being done now) and `HANDOFFS.md` (resume state).

## Open

