#!/usr/bin/env bash
# SPDX-License-Identifier: AGPL-3.0-only
# Copyright 2026 Nathan M. Fraske
#
# Stop hook (docs/AGENTIC_ADDENDUM.md section 2d): a soft, single-pass nudge if
# this branch changed engine code (crates/) without updating the tracking files
# (HANDOFFS.md / TODOS.md) — the same-turn discipline the agent-ops layer depends
# on. It is a REMINDER, not a hard gate: it fires at most once (respects
# stop_hook_active) and never blocks the durability push (session-end.sh runs
# regardless). The full cargo suite is deliberately NOT run here (a multi-minute
# compile on every turn end is impractical); that is CI's job and, once built,
# the projectops `verify` tool's.
set -uo pipefail
ROOT="${CLAUDE_PROJECT_DIR:-$(git rev-parse --show-toplevel 2>/dev/null || pwd)}"
input="$(cat 2>/dev/null || true)"

# Recursion guard: allow the stop on the second pass.
active="$(printf '%s' "$input" | python3 -c "import json,sys
try: print(json.load(sys.stdin).get('stop_hook_active', False))
except Exception: print(False)" 2>/dev/null || echo False)"
[ "$active" = "True" ] && exit 0

cd "$ROOT" 2>/dev/null || exit 0
git rev-parse --git-dir >/dev/null 2>&1 || exit 0
git rev-parse --verify -q origin/main >/dev/null 2>&1 || exit 0

changed="$(git diff --name-only origin/main...HEAD 2>/dev/null || true)"
[ -z "$changed" ] && exit 0
printf '%s\n' "$changed" | grep -qE '^crates/' || exit 0          # no engine code changed
printf '%s\n' "$changed" | grep -qE '^(HANDOFFS|TODOS)\.md$' && exit 0  # tracking already updated

echo "tracking-freshness (Stop): this branch changed crates/ but did not update HANDOFFS.md or TODOS.md. \
The agent-ops discipline is to update the baton and the checklist in the SAME turn as the work \
(append-only, UTC-timestamped, tombstoned). Add the entries before finishing, or confirm this is intentional." >&2
exit 2
