#!/usr/bin/env bash
# SPDX-License-Identifier: AGPL-3.0-only
# Copyright 2026 Nathan M. Fraske
#
# SessionStart hook: surface the AGENTIC ADDENDUM (docs/AGENTIC_ADDENDUM.md) so every session
# knows the agentic infrastructure spec + the fully-blind audit method (section 7) exist, and
# when to reach for them. It injects a pointer only, not the whole (static, long) document.
set -uo pipefail
ROOT="${CLAUDE_PROJECT_DIR:-$(git rev-parse --show-toplevel 2>/dev/null || pwd)}"
[ -f "$ROOT/docs/AGENTIC_ADDENDUM.md" ] || exit 0

python3 - <<'PY' 2>/dev/null || true
import json
instr = (
    "AGENTIC ADDENDUM POINTER (project SessionStart hook): the agentic infrastructure — the hook "
    "lifecycle, the four tracking files (HANDOFFS/TODOS/FOLLOWUPS) + the .claude/memory mirror, the "
    "projectops/panels surface, and THE FULLY-BLIND AUDIT method — is specified in "
    "docs/AGENTIC_ADDENDUM.md. Consult its section 7 BEFORE verifying any crypto/de-id kernel "
    "(attestation_message, chain_hash, content_hash, de_identify_plan, confirmation_key, "
    "ensure_attested, the wire envelope, endpoint_is_loopback) or any frozen constant "
    "(ACTION_VOCABULARY, POISON, the stop-code/module dictionaries, the domain-tag prefixes, the "
    "cec-diagnose/v1 + cec-execute/v1 wire grammar): a shared blind spot between the code and its own "
    "tests hides defects a normal review inherits. This project's adversarial audit found an "
    "escalation-gate bypass and a confirmation-replay hole exactly that way."
)
print(json.dumps({"hookSpecificOutput": {"hookEventName": "SessionStart", "additionalContext": instr}}))
PY
