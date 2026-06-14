#!/usr/bin/env bash
# SPDX-License-Identifier: AGPL-3.0-only
# Copyright 2026 Nathan M. Fraske
#
# SessionStart hook (owner directive 2026-06-14): make sure NO deferred / not-now item is ever dropped,
# and that the deferral record is DURABLE and AUDITABLE.
#   (1) ensures FOLLOWUPS.md exists at the repo root, and
#   (2) injects a standing instruction telling the agent to record every deferred / pending / consider-later
#       item there with the DATE AND TIME it was added, and to TOMBSTONE (never delete) items when removed.
#
# This is the STRICTER variant of the CEC-Platform followups policy: there, resolved items were deleted;
# here they are append-only with a tombstone so the deferral history can never be silently rewritten.
set -uo pipefail
ROOT="${CLAUDE_PROJECT_DIR:-$(git rev-parse --show-toplevel 2>/dev/null || pwd)}"
F="$ROOT/FOLLOWUPS.md"

if [ ! -f "$F" ]; then
  cat > "$F" <<'EOF'
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

EOF
fi

INSTR='FOLLOWUPS POLICY (project SessionStart hook, owner 2026-06-14): whenever you DEFER an action, note something PENDING future authorization, or jot a "consider later" / "do this later" side note — ANY non-blocking item deferred to the future — you MUST append it to FOLLOWUPS.md at the repo root in the SAME turn you raise it, as: `- [ ] [added YYYY-MM-DD HH:MM UTC] <item> — <why deferred / context / where to resume>` using the ACTUAL current UTC date AND time. FOLLOWUPS.md is APPEND-ONLY WITH TOMBSTONES: never delete a line. When an item is done, promoted, or dropped, flip `- [ ]` to `- [x]` and append `· closed YYYY-MM-DD HH:MM UTC → <where it went>` (a PR #, a TODOS.md line, another doc, or "dropped: <reason>"). Never drop a deferred follow-up; it always lands in FOLLOWUPS.md and stays there as auditable history. Distinct from TODOS.md (live work-now checklist) and HANDOFFS.md (resume state). FOLLOWUPS.md already exists (this hook ensures it).'

python3 - "$INSTR" <<'PY' 2>/dev/null || true
import json, sys
print(json.dumps({"hookSpecificOutput": {"hookEventName": "SessionStart", "additionalContext": sys.argv[1]}}))
PY
