#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-only
# Copyright 2026 Nathan M. Fraske
#
# projectops: structured checks over the cec-support-agent repository
# (docs/AGENTIC_ADDENDUM.md section 3). It turns the verification suite, the
# security invariants, and the tracking backlog into machine-readable JSON so
# hooks, panels, CI, and the (thin) MCP server consume structured results
# instead of re-deriving them from ad-hoc greps.
#
# CARDINAL CONSTRAINT: this operates on the repository and the tracking docs
# ONLY. It never reads, serves, or enumerates the private corpus (which does not
# live in this repo at all). Pure stdlib — no third-party dependency.
#
# Subcommands (each prints one JSON object to stdout; exit 0 = all pass):
#   verify [--checks fmt,clippy,build,test,deny,gitleaks]  run the suite
#   invariants                                             fast git/grep guards
#   backlog                                                parse TODOS/FOLLOWUPS
#   leak_scan                                              the de-id / poison suite
import argparse
import json
import os
import re
import subprocess
import sys

_git_root = subprocess.run(
    ["git", "rev-parse", "--show-toplevel"], capture_output=True, text=True
).stdout.strip()
ROOT = os.environ.get("REPO_ROOT") or _git_root or os.getcwd()

# Mirrors scripts/githooks/pre-commit + .gitignore + the invariant-guard hook.
EXFIL_RE = re.compile(
    r"(^|/)(corpus|weights)/"
    r"|\.(gguf|safetensors|bin|sqlite|duckdb|jsonl|ndjson|seed|seedhex|env)$"
    r"|\.flow\.ya?ml$"
    r"|cec-corpus",
    re.IGNORECASE,
)


def run(cmd, timeout=900):
    try:
        p = subprocess.run(cmd, cwd=ROOT, capture_output=True, text=True, timeout=timeout)
        return p.returncode, p.stdout, p.stderr
    except FileNotFoundError:
        return 127, "", "%s: command not found" % cmd[0]
    except subprocess.TimeoutExpired:
        return 124, "", "timed out after %ss" % timeout


def tail(text, n=20):
    lines = [ln for ln in (text or "").splitlines() if ln.strip()]
    return "\n".join(lines[-n:])


def read(rel):
    try:
        with open(os.path.join(ROOT, rel), "r", errors="replace") as fh:
            return fh.read()
    except OSError:
        return None


def emit(obj):
    print(json.dumps(obj, indent=2))
    return 0 if obj.get("pass", True) else 1


# --------------------------------------------------------------------------- #
# verify: the cargo/gitleaks suite, each check structured.
# --------------------------------------------------------------------------- #
_SUITE = [
    ("fmt", ["cargo", "fmt", "--all", "--", "--check"]),
    ("clippy", ["cargo", "clippy", "--workspace", "--all-targets", "--", "-D", "warnings"]),
    ("build", ["cargo", "build", "--workspace"]),
    ("test", ["cargo", "test", "--workspace"]),
    ("deny", ["cargo", "deny", "check"]),
    ("gitleaks", ["gitleaks", "dir", "--config", ".gitleaks.toml", "--redact", "--exit-code", "1", "."]),
]


def cmd_verify(args):
    only = set(c.strip() for c in args.checks.split(",")) if args.checks else None
    checks = []
    for name, cmd in _SUITE:
        if only and name not in only:
            continue
        rc, out, err = run(cmd)
        if rc == 127:  # tool absent (cargo-deny / gitleaks not installed)
            checks.append({"check": name, "status": "skipped", "pass": True,
                           "detail": "%s not installed" % cmd[0]})
            continue
        checks.append({"check": name, "status": "pass" if rc == 0 else "fail",
                       "pass": rc == 0, "detail": "" if rc == 0 else tail(err or out)})
    ok = all(c["pass"] for c in checks)
    return emit({"tool": "verify", "pass": ok, "checks": checks})


# --------------------------------------------------------------------------- #
# invariants: fast git/grep security guards (no cargo).
# --------------------------------------------------------------------------- #
def _inv_no_exfil():
    rc, out, _ = run(["git", "ls-files"])
    hits = [f for f in out.splitlines() if EXFIL_RE.search(f)]
    return {"check": "no_exfil_tracked", "pass": not hits,
            "detail": "" if not hits else "tracked exfil-shaped files: " + ", ".join(hits)}


def _slice_fn(src, sig):
    # Return the brace-balanced body of the first `fn` matching `sig`.
    i = src.find(sig)
    if i < 0:
        return None
    j = src.find("{", i)
    if j < 0:
        return None
    depth, k = 0, j
    while k < len(src):
        if src[k] == "{":
            depth += 1
        elif src[k] == "}":
            depth -= 1
            if depth == 0:
                return src[j:k + 1]
        k += 1
    return src[j:]


def _inv_source_absent():
    src = read("crates/support-agent/src/main.rs")
    if src is None:
        return {"check": "source_label_absent", "pass": False, "detail": "main.rs unreadable"}
    body = _slice_fn(src, "fn diagnose_envelope")
    if body is None:
        return {"check": "source_label_absent", "pass": False, "detail": "diagnose_envelope not found"}
    # The leak-C10 negative pin: the envelope must not carry a candidate `source`.
    leaked = re.search(r'"source"\s*:', body)
    return {"check": "source_label_absent", "pass": not leaked,
            "detail": "" if not leaked else 'diagnose_envelope emits a "source" field (leak-C10 membership oracle)'}


def _inv_frozen_route():
    src = read("crates/support-agent/src/serve.rs")
    if src is None:
        return {"check": "frozen_route_surface", "pass": False, "detail": "serve.rs unreadable"}
    body = _slice_fn(src, "fn route_surface")
    if body is None:
        return {"check": "frozen_route_surface", "pass": False, "detail": "route_surface not found"}
    routes = set(re.findall(r'"(/v1/[^"]*)"', body))
    expected = {"/v1/health", "/v1/diagnose", "/v1/execute"}
    return {"check": "frozen_route_surface", "pass": routes == expected,
            "detail": "" if routes == expected else "route surface is %s (expected %s)" % (sorted(routes), sorted(expected))}


def _inv_action_vocab():
    src = read("crates/deid/src/lib.rs")
    if src is None:
        return {"check": "action_vocabulary_sorted", "pass": False, "detail": "deid/lib.rs unreadable"}
    m = re.search(r"ACTION_VOCABULARY:\s*&\[&str\]\s*=\s*&\[(.*?)\]", src, re.DOTALL)
    if not m:
        return {"check": "action_vocabulary_sorted", "pass": False, "detail": "ACTION_VOCABULARY not found"}
    toks = re.findall(r'"([^"]+)"', m.group(1))
    ok = toks == sorted(toks) and len(toks) > 0
    return {"check": "action_vocabulary_sorted", "pass": ok, "count": len(toks),
            "detail": "" if ok else "ACTION_VOCABULARY is not sorted (binary_search relies on it): %s" % toks}


def _inv_wire_pins():
    src = read("crates/support-agent/src/main.rs") or ""
    src2 = read("crates/support-agent/src/serve.rs") or ""
    have_d = "envelope_enum_wire_values_are_pinned" in src
    have_e = "execute_wire_values_are_pinned" in src2
    ok = have_d and have_e
    missing = [n for n, h in [("cec-diagnose/v1", have_d), ("cec-execute/v1", have_e)] if not h]
    return {"check": "wire_grammar_pins_present", "pass": ok,
            "detail": "" if ok else "missing pinning test(s) for: " + ", ".join(missing)}


def _inv_hooks():
    hooks = ["invariant-guard.sh", "invariant-check.sh", "tracking-freshness.sh", "addendum-context.sh"]
    missing = []
    for h in hooks:
        p = os.path.join(ROOT, ".claude", "hooks", h)
        if not (os.path.isfile(p) and os.access(p, os.X_OK)):
            missing.append(h)
    return {"check": "agent_hooks_executable", "pass": not missing,
            "detail": "" if not missing else "missing/non-executable: " + ", ".join(missing)}


def cmd_invariants(args):
    checks = [_inv_no_exfil(), _inv_source_absent(), _inv_frozen_route(),
              _inv_action_vocab(), _inv_wire_pins(), _inv_hooks()]
    ok = all(c["pass"] for c in checks)
    return emit({"tool": "invariants", "pass": ok, "checks": checks})


# --------------------------------------------------------------------------- #
# backlog: parse the tracking files into open/closed items.
# --------------------------------------------------------------------------- #
_ITEM_RE = re.compile(r"^\s*- \[( |x|~)\] \[added (\d{4}-\d{2}-\d{2} \d{2}:\d{2} UTC)[^\]]*\]\s*(.*)")


def _parse_items(rel):
    text = read(rel) or ""
    items = []
    for ln in text.splitlines():
        m = _ITEM_RE.match(ln)
        if m:
            status = {" ": "open", "x": "done", "~": "obsolete"}[m.group(1)]
            summary = re.sub(r"\*\*|`", "", m.group(3)).strip()[:160]
            items.append({"status": status, "added": m.group(2), "summary": summary})
    return items


def cmd_backlog(args):
    todos = _parse_items("TODOS.md")
    followups = _parse_items("FOLLOWUPS.md")

    def counts(items):
        c = {"open": 0, "done": 0, "obsolete": 0}
        for it in items:
            c[it["status"]] += 1
        return c

    return emit({
        "tool": "backlog",
        "pass": True,
        "todos": {"counts": counts(todos), "open": [i for i in todos if i["status"] == "open"]},
        "followups": {"counts": counts(followups), "open": [i for i in followups if i["status"] == "open"]},
    })


# --------------------------------------------------------------------------- #
# leak_scan: the de-identification / poison / leakage suite.
# --------------------------------------------------------------------------- #
def cmd_leak_scan(args):
    # Test-name filter: the leakage suite + poison harness + de-id gate tests.
    rc, out, err = run(["cargo", "test", "--workspace", "--", "leak", "poison", "de_identif", "deid", "attest"])
    if rc == 127:
        return emit({"tool": "leak_scan", "pass": False, "status": "skipped", "detail": "cargo not installed"})
    # cargo test exits non-zero if any matched test fails OR (harmlessly) if a
    # crate matched zero tests is not the case here; treat rc==0 as pass.
    return emit({"tool": "leak_scan", "pass": rc == 0,
                 "detail": "" if rc == 0 else tail(err or out)})


def main(argv=None):
    ap = argparse.ArgumentParser(prog="projectops", description="structured checks over cec-support-agent")
    sub = ap.add_subparsers(dest="cmd", required=True)
    v = sub.add_parser("verify"); v.add_argument("--checks", default=None,
        help="comma list subset of fmt,clippy,build,test,deny,gitleaks"); v.set_defaults(fn=cmd_verify)
    sub.add_parser("invariants").set_defaults(fn=cmd_invariants)
    sub.add_parser("backlog").set_defaults(fn=cmd_backlog)
    sub.add_parser("leak_scan").set_defaults(fn=cmd_leak_scan)
    args = ap.parse_args(argv)
    return args.fn(args)


if __name__ == "__main__":
    sys.exit(main())
