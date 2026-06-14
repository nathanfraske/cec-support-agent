#!/usr/bin/env bash
# SPDX-License-Identifier: AGPL-3.0-only
# Copyright 2026 Nathan M. Fraske
#
# SessionStart hook — WSL-EPHEMERAL STATE POLICY, start half (owner directive 2026-06-14).
#
# The live persistent-memory dir under ~/.claude is DISPOSABLE: a WSL wipe + `git clone` leaves it empty.
# The DURABLE source of truth is the git-tracked in-tree mirror at .claude/memory/ (refreshed by the Stop
# hook) plus the off-tree `ops/agent-handoff` branch on the remote. This hook is the start half of that
# durability contract: after a wipe it RE-SEEDS the live memory dir from the committed mirror, then injects
# the memory index + the latest agent handoff as additionalContext so every new session picks up in-flight
# work and never repeats finished tasks.
#
# Emitted as the hook's ONLY stdout (JSON); everything else goes to stderr. Fail-soft by construction.
set -uo pipefail

root="${CLAUDE_PROJECT_DIR:-$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)}"

# Persistent memory lives under ~/.claude/projects/<sanitized-project-path>/memory. The harness sanitizes
# '/', '.', AND '_' to '-' (so CEC_AutoDiagnoser -> CEC-AutoDiagnoser); CEC-Platform's hook only mapped
# '/.' which is WRONG for an underscore'd path. Derive it, then fall back to the canonical main-checkout dir
# (git worktrees sanitize to a different name but share the same project memory).
derived="$HOME/.claude/projects/$(echo "$root" | tr '/._' '---')/memory"
canonical="$HOME/.claude/projects/-home-nathan-CEC-AutoDiagnoser/memory"
memdir="$derived"; [ -d "$memdir" ] || memdir="$canonical"

# --- self-heal the live memory dir from the DURABLE in-tree mirror (WSL-ephemeral policy) -------------
# .claude/memory/ is the committed, git-tracked copy (refreshed by the Stop hook). After a WSL wipe the live
# dir is empty, so seed it from the mirror — but only files the live dir LACKS (never clobber a newer edit).
committed="$root/.claude/memory"
mkdir -p "$memdir" 2>/dev/null || true
if [ -d "$committed" ]; then
  for f in "$committed"/*.md; do
    [ -e "$f" ] || continue
    base="$(basename "$f")"
    [ -e "$memdir/$base" ] || cp "$f" "$memdir/$base" 2>/dev/null || true
  done
fi

# --- inject the memory index + latest handoff as additionalContext (stdout = the hook JSON only) ------
python3 - "$memdir" "$root" <<'PY' 2>/dev/null || true
import json, os, sys
md, root = sys.argv[1], sys.argv[2]

def rd(path, cap):
    try:
        with open(path) as f:
            t = f.read()
    except OSError:
        return None
    return t[:cap] + ("\n[...truncated by session-start hook...]" if len(t) > cap else "")

index   = rd(os.path.join(md, "MEMORY.md"), 4000)
handoff = rd(os.path.join(md, "current-work-handoff.md"), 8000)
parts = ["MANDATORY STARTUP CONTEXT (project SessionStart hook, owner 2026-06-14, WSL-ephemeral state "
         "policy): the live ~/.claude memory is disposable and was re-seeded from the in-tree mirror if a "
         "WSL wipe had emptied it. Before doing ANY work, take the persistent memory into account so you "
         "pick up in-flight work and never repeat completed tasks. Memory dir: " + md + ". The cross-agent "
         "baton is HANDOFFS.md at the repo root (injected separately) — read it first. Keep memory and the "
         "tracking files current in the same turn as significant work; the Stop hook snapshots them to the "
         "durable ops/agent-handoff branch."]
if handoff:
    parts.append("=== persistent-memory current-work-handoff.md ===\n" + handoff)
if index:
    parts.append("=== MEMORY.md (persistent-memory index — open any entry relevant to your task) ===\n" + index)
print(json.dumps({"hookSpecificOutput": {"hookEventName": "SessionStart", "additionalContext": "\n\n".join(parts)}}))
PY
exit 0
