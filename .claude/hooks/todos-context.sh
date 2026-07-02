#!/usr/bin/env bash
# SPDX-License-Identifier: AGPL-3.0-only
# Copyright 2026 Nathan M. Fraske
#
# SessionStart hook (owner directive 2026-06-14): maintain TODOS.md as the LIVE checklist of everything
# the agent is GETTING DONE — checkbox form, checked off with the same thoroughness as FOLLOWUPS.md.
#   (1) ensures TODOS.md exists, (2) injects the standing policy.
# APPEND-ONLY: items are never deleted; completed/obsolete ones stay as timestamped history with a tombstone.
set -uo pipefail
ROOT="${CLAUDE_PROJECT_DIR:-$(git rev-parse --show-toplevel 2>/dev/null || pwd)}"
F="$ROOT/TODOS.md"

if [ ! -f "$F" ]; then
  cat > "$F" <<'EOF'
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

EOF
fi

INSTR='TODOS POLICY (project SessionStart hook, owner 2026-06-14): maintain TODOS.md at the repo root as the LIVE checklist of everything you are GETTING DONE, in checkbox form, checked off with the same thoroughness as FOLLOWUPS.md. When you START a task, append `- [ ] [added YYYY-MM-DD HH:MM UTC] <task>` using the ACTUAL current UTC date AND time. APPEND-ONLY: NEVER delete an item — when DONE flip `- [ ]` to `- [x]` and append `· done YYYY-MM-DD HH:MM UTC`; when OBSOLETE flip to `- [~]`, append `· obsolete YYYY-MM-DD HH:MM UTC → <tombstone>` and point the tombstone where it went (a FOLLOWUPS.md entry, a PR #, or another line). Completed/obsolete items STAY as timestamped history. Update TODOS.md in the SAME turn as the work. Distinct from FOLLOWUPS.md (deferred backlog) and HANDOFFS.md (resume state). TODOS.md already exists (this hook ensures it).'

python3 - "$INSTR" <<'PY' 2>/dev/null || true
import json, sys
print(json.dumps({"hookSpecificOutput": {"hookEventName": "SessionStart", "additionalContext": sys.argv[1]}}))
PY
