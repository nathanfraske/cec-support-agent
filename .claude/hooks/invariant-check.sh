#!/usr/bin/env bash
# SPDX-License-Identifier: AGPL-3.0-only
# Copyright 2026 Nathan M. Fraske
#
# PostToolUse hook (Write|Edit): the REACTION half of the invariant guard
# (docs/AGENTIC_ADDENDUM.md section 2c). It cannot undo the edit, but it
# surfaces feedback (exit 2 with a message) when the just-written file shows an
# unambiguous problem, so the agent fixes it immediately rather than at CI time.
# It stays deliberately narrow — only checks that do not false-positive on this
# repo's own prose/code — and is silent (exit 0) otherwise. Fuzzier structural
# checks (a re-added Serialize on a raw type, a re-added `source` envelope field)
# belong in the projectops `invariants` tool, not a grep-heuristic hook.
set -uo pipefail
ROOT="${CLAUDE_PROJECT_DIR:-$(git rev-parse --show-toplevel 2>/dev/null || pwd)}"

# Read the hook JSON from stdin into an env var (the heredoc below is python's
# stdin), so the payload is not consumed by the program text.
HOOK_INPUT="$(cat 2>/dev/null || true)" python3 - "$ROOT" <<'PY'
import json, os, re, sys
root = os.path.realpath(sys.argv[1])
try:
    data = json.loads(os.environ.get("HOOK_INPUT", "") or "{}")
except Exception:
    sys.exit(0)
ti = data.get("tool_input", {}) or {}
path = ti.get("file_path") or ti.get("path") or ""
if not path:
    sys.exit(0)
ap = os.path.realpath(path if os.path.isabs(path) else os.path.join(root, path))
if not (ap == root or ap.startswith(root + os.sep)) or not os.path.isfile(ap):
    sys.exit(0)
try:
    with open(ap, "r", errors="replace") as fh:
        body = fh.read()
except OSError:
    sys.exit(0)

msgs = []
# (1) Merge-conflict markers — always a bug, never legitimate content.
if re.search(r'^(<{7}|={7}|>{7})', body, re.MULTILINE):
    msgs.append("a merge-conflict marker (<<<<<<< / ======= / >>>>>>>) — resolve it before finishing")
# (2) A serialized corpus-row shape written into a file — the renamed-corpus-dump
#     backstop for a file the path-based PreToolUse guard did not catch. It requires
#     the 16-hex fingerprint VALUE (format!("{hash:016x}")), so a real row matches but
#     a prose description of the shape (this addendum's own section 2c) does not.
if re.search(r'"outcome"\s*:\s*\{\s*"signature"\s*:\s*\{\s*"fingerprint"\s*:\s*"[0-9a-f]{16}"', body):
    msgs.append("content shaped like a serialized corpus row (an \"outcome\" wrapping a \"signature\" with a "
                "16-hex \"fingerprint\" value) — corpus rows must never enter this repo (SECURITY.md)")
# (3) An age-encrypted seed / private key block committed as content. The markers
#     are assembled from fragments so this hook's own source never contains the
#     contiguous literal (which would self-flag on every edit to this file).
dash = "-----BEGIN "
age_marker = dash + "AGE ENCRYPTED " + "FILE-----"
key_marker = dash + "OPENSSH PRIVATE " + "KEY-----"
if age_marker in body or key_marker in body:
    msgs.append("an encrypted-seed / private-key block — sign-off seeds and keys never enter this repo")

if msgs:
    rel = os.path.relpath(ap, root)
    sys.stderr.write("invariant-check (PostToolUse) flagged %s:\n- %s\n" % (rel, "\n- ".join(msgs)))
    sys.exit(2)
sys.exit(0)
PY
