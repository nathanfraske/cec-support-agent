#!/usr/bin/env bash
# SPDX-License-Identifier: AGPL-3.0-only
# Copyright 2026 Nathan M. Fraske
#
# SessionStart hook (owner directive 2026-06-14): maintain HANDOFFS.md so that ANY agent can pick up
# EXACTLY where the previous one left off — without being confused or having to go hunting.
#   (1) ensures HANDOFFS.md exists at the repo root,
#   (2) injects its current contents as additionalContext (READ FIRST), and
#   (3) injects a standing instruction to keep it current and to log lessons learned there.
#
# HANDOFFS.md is the cross-agent baton: current state, the exact next step, and accumulated lessons.
set -uo pipefail
ROOT="${CLAUDE_PROJECT_DIR:-$(git rev-parse --show-toplevel 2>/dev/null || pwd)}"
F="$ROOT/HANDOFFS.md"

if [ ! -f "$F" ]; then
  cat > "$F" <<'EOF'
# HANDOFFS

The cross-agent baton. An agent picking up this repo should be able to read **only this file** and know
exactly where things stand, what to do next, and what was learned — without hunting. Times are **UTC**.

Keep three things current, in the same turn as the work:
1. **Current state** — where things stand right now (branch, what's done, what's in flight).
2. **Pick up here** — the exact next step(s), concrete enough to start immediately (file, command, decision).
3. **Lessons learned** — durable, append-only. Anything you discovered the hard way (a gotcha, a non-obvious
   constraint, a tool quirk, a dead end and why) so the next agent does not relearn it. Never delete a lesson.

Below "Pick up here", keep a reverse-chronological **handoff log** of dated entries so the trail is auditable.

---

## Current state

_(empty — first agent to do substantive work fills this in)_

## Pick up here

_(empty)_

## Lessons learned (append-only)

_(empty)_

## Handoff log (reverse-chronological)

EOF
fi

# Inject the current HANDOFFS.md (capped) so every session reads the baton first, plus the standing policy.
python3 - "$F" <<'PY' 2>/dev/null || true
import json, os, sys
path = sys.argv[1]
try:
    with open(path) as fh:
        body = fh.read()
except OSError:
    body = ""
cap = 12000
if len(body) > cap:
    body = body[:cap] + "\n[...truncated by handoffs-context hook — open HANDOFFS.md for the rest...]"
instr = ("HANDOFFS POLICY (project SessionStart hook, owner 2026-06-14): HANDOFFS.md at the repo root is the "
         "cross-agent baton — READ IT FIRST so you pick up exactly where the last agent left off. Keep it "
         "current in the SAME turn as significant work: update 'Current state', rewrite 'Pick up here' to the "
         "exact next step (concrete enough to start immediately), append any 'Lessons learned' (append-only — "
         "never delete a lesson), and add a dated UTC entry to the handoff log. Distinct from TODOS.md (live "
         "checklist) and FOLLOWUPS.md (deferred backlog). HANDOFFS.md already exists (this hook ensures it).")
parts = [instr]
if body.strip():
    parts.append("=== HANDOFFS.md (cross-agent baton — READ FIRST) ===\n" + body)
print(json.dumps({"hookSpecificOutput": {"hookEventName": "SessionStart", "additionalContext": "\n\n".join(parts)}}))
PY
