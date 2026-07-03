#!/usr/bin/env bash
# SPDX-License-Identifier: AGPL-3.0-only
# Copyright 2026 Nathan M. Fraske
#
# PreToolUse hook (Write|Edit): the PREVENTION half of the invariant guard
# (docs/AGENTIC_ADDENDUM.md section 2b). It HARD-BLOCKS (exit 2) a write that
# would put corpus data, model weights, or a sign-off seed into the repository
# tree — the exfil shapes scripts/githooks/pre-commit and .gitignore already
# deny, promoted here to a pre-WRITE deny that fires before the file exists and
# holds even under permission-bypass mode. It guards only writes INTO the repo;
# a corpus/tmp write elsewhere is not our concern. The private corpus lives in
# the separate cec-corpus-private repo, never here.
set -uo pipefail
ROOT="${CLAUDE_PROJECT_DIR:-$(git rev-parse --show-toplevel 2>/dev/null || pwd)}"

# Read the hook JSON from stdin into an env var: the `<<'PY'` heredoc below is
# python's stdin (the program), so the payload cannot also come from stdin.
HOOK_INPUT="$(cat 2>/dev/null || true)" python3 - "$ROOT" <<'PY'
import json, os, re, sys
root = os.path.realpath(sys.argv[1])
try:
    data = json.loads(os.environ.get("HOOK_INPUT", "") or "{}")
except Exception:
    sys.exit(0)  # unparseable input -> do not block
ti = data.get("tool_input", {}) or {}
path = ti.get("file_path") or ti.get("path") or ""
if not path:
    sys.exit(0)
ap = os.path.realpath(path if os.path.isabs(path) else os.path.join(root, path))
# Only guard writes that land inside the repository tree.
if not (ap == root or ap.startswith(root + os.sep)):
    sys.exit(0)
rel = os.path.relpath(ap, root)
# Mirrors scripts/githooks/pre-commit + .gitignore: corpus/weights dirs; model,
# db, corpus-row, and seed extensions; fix-flow YAML; any cec-corpus path.
EXFIL = re.compile(
    r'(^|/)(corpus|weights)/'
    r'|\.(gguf|safetensors|bin|sqlite|duckdb|jsonl|ndjson|seed|seedhex|env)$'
    r'|\.flow\.ya?ml$'
    r'|cec-corpus',
    re.IGNORECASE,
)
if EXFIL.search(rel):
    sys.stderr.write(
        "BLOCKED by invariant-guard (PreToolUse): '%s' matches a corpus / weights / "
        "sign-off-seed exfil shape. This engine repository holds NO corpus data, model "
        "weights, or sign-off seed (AGENTS.md, SECURITY.md, scripts/githooks/pre-commit, "
        ".gitignore). The private corpus lives in the separate cec-corpus-private repo. "
        "If this is a legitimate non-corpus file, rename it off the blocked shape.\n" % rel
    )
    sys.exit(2)
sys.exit(0)
PY
