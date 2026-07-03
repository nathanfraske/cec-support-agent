#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-only
# Copyright 2026 Nathan M. Fraske
#
# Minimal MCP stdio server exposing tools/projectops.py as callable tools
# (docs/AGENTIC_ADDENDUM.md section 3). It speaks JSON-RPC 2.0 over
# line-delimited stdio directly — no third-party MCP SDK — matching the repo's
# no-mandatory-dependency posture and keeping it testable without a client.
# It shells out to projectops.py and returns its JSON; it never touches the
# private corpus. Addressed as mcp__projectops__<tool> once wired in .mcp.json.
import json
import os
import subprocess
import sys

HERE = os.path.dirname(os.path.realpath(__file__))
CLI = os.path.join(HERE, "projectops.py")
PROTOCOL_VERSION = "2024-11-05"

TOOLS = [
    {
        "name": "verify",
        "description": "Run the cec-support-agent verification suite (fmt, clippy, build, test, "
                       "cargo-deny, gitleaks) and return each check's pass/fail + failing lines as "
                       "JSON. Slow — it compiles the workspace. Optional `checks` is a comma list "
                       "subset of fmt,clippy,build,test,deny,gitleaks.",
        "inputSchema": {"type": "object", "properties": {"checks": {"type": "string"}}, "required": []},
    },
    {
        "name": "invariants",
        "description": "Fast git/grep security-invariant checks (no cargo): no corpus/weights file is "
                       "tracked; the diagnose envelope carries no `source` membership label (leak-C10); "
                       "the /v1 route surface is frozen to health/diagnose/execute; ACTION_VOCABULARY is "
                       "sorted; the wire-grammar pinning tests are present; the agent hooks are executable.",
        "inputSchema": {"type": "object", "properties": {}, "required": []},
    },
    {
        "name": "backlog",
        "description": "Parse TODOS.md and FOLLOWUPS.md into open/done/obsolete counts and the open items.",
        "inputSchema": {"type": "object", "properties": {}, "required": []},
    },
    {
        "name": "leak_scan",
        "description": "Run the de-identification / poison / leakage / attestation test suite and return pass/fail.",
        "inputSchema": {"type": "object", "properties": {}, "required": []},
    },
]
TOOL_NAMES = {t["name"] for t in TOOLS}


def _send(msg):
    sys.stdout.write(json.dumps(msg) + "\n")
    sys.stdout.flush()


def _result(mid, res):
    _send({"jsonrpc": "2.0", "id": mid, "result": res})


def _error(mid, code, message):
    _send({"jsonrpc": "2.0", "id": mid, "error": {"code": code, "message": message}})


def _call_tool(name, args):
    cmd = [sys.executable, CLI, name]
    if name == "verify" and args.get("checks"):
        cmd += ["--checks", str(args["checks"])]
    p = subprocess.run(cmd, capture_output=True, text=True,
                       cwd=os.environ.get("REPO_ROOT", os.path.dirname(HERE)))
    text = (p.stdout or "").strip() or (p.stderr or "").strip() or "(no output)"
    return {"content": [{"type": "text", "text": text}], "isError": p.returncode != 0}


def main():
    for line in sys.stdin:
        line = line.strip()
        if not line:
            continue
        try:
            msg = json.loads(line)
        except Exception:
            continue
        mid = msg.get("id")
        method = msg.get("method")
        if method == "initialize":
            _result(mid, {"protocolVersion": PROTOCOL_VERSION,
                          "capabilities": {"tools": {}},
                          "serverInfo": {"name": "projectops", "version": "0.1.0"}})
        elif method == "notifications/initialized":
            pass  # notification: no response
        elif method == "ping":
            _result(mid, {})
        elif method == "tools/list":
            _result(mid, {"tools": TOOLS})
        elif method == "tools/call":
            params = msg.get("params", {}) or {}
            name = params.get("name")
            if name not in TOOL_NAMES:
                _error(mid, -32602, "unknown tool: %s" % name)
                continue
            try:
                _result(mid, _call_tool(name, params.get("arguments", {}) or {}))
            except Exception as exc:
                _result(mid, {"content": [{"type": "text", "text": "tool error: %s" % exc}], "isError": True})
        elif mid is not None:
            _error(mid, -32601, "method not found: %s" % method)
        # unknown notifications (no id) are ignored
    return 0


if __name__ == "__main__":
    sys.exit(main())
