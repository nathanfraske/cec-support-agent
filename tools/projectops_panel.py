#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-only
# Copyright 2026 Nathan M. Fraske
#
# projectops panel generator (docs/AGENTIC_ADDENDUM.md section 4). Runs the
# projectops checks and renders their JSON into a self-contained, theme-aware
# HTML dashboard — the review surface the reference addendum describes, produced
# as a static snapshot (a rendered page cannot call an MCP URL under the harness
# CSP, so the data is baked in at generation time). Regenerate to refresh.
#
# The output is CONTENT ONLY (a <style> block plus the dashboard markup) so it
# drops straight into the harness Artifact wrapper and also opens fine on its
# own. It reads the repo + tracking docs only, never the private corpus.
#
#   python3 tools/projectops_panel.py [--verify] [-o panel.html]
#     --verify   also run the (slow) cargo/gitleaks suite; else it is marked
#                "not run in this snapshot" and left to `projectops verify`.
import argparse
import datetime
import html
import json
import os
import subprocess
import sys

HERE = os.path.dirname(os.path.realpath(__file__))
CLI = os.path.join(HERE, "projectops.py")
ROOT = os.environ.get("REPO_ROOT") or subprocess.run(
    ["git", "rev-parse", "--show-toplevel"], capture_output=True, text=True).stdout.strip() or os.getcwd()


def cli(sub, *extra):
    p = subprocess.run([sys.executable, CLI, sub, *extra], capture_output=True, text=True, cwd=ROOT)
    try:
        return json.loads(p.stdout)
    except Exception:
        return {"tool": sub, "pass": False, "error": (p.stderr or p.stdout or "no output").strip()[:400]}


def head_sha():
    r = subprocess.run(["git", "log", "-1", "--format=%h %s"], capture_output=True, text=True, cwd=ROOT)
    return r.stdout.strip()[:72] or "(unknown)"


E = lambda s: html.escape(str(s), quote=True)

CSS = """
:root{
  --bg:#0e1316; --surface:#151b1f; --surface-2:#1b2329; --border:#26313a;
  --text:#c9d2d8; --muted:#7c8891; --faint:#4f5a62;
  --accent:#3fb0a3; --accent-dim:#2b6f68;
  --good:#4bb37e; --warn:#d69b3f; --crit:#e0655d;
  --good-bg:rgba(75,179,126,.13); --warn-bg:rgba(214,155,63,.14); --crit-bg:rgba(224,101,93,.14);
  --muted-bg:rgba(124,137,145,.13);
  --sans:ui-sans-serif,system-ui,-apple-system,"Segoe UI",Roboto,Helvetica,Arial,sans-serif;
  --mono:ui-monospace,"SF Mono",SFMono-Regular,Menlo,Consolas,"Liberation Mono",monospace;
}
@media (prefers-color-scheme: light){:root{
  --bg:#f2f4f5; --surface:#ffffff; --surface-2:#f6f8f9; --border:#dde4e8;
  --text:#18242b; --muted:#5c6a72; --faint:#94a2aa;
  --accent:#237f74; --accent-dim:#7cc0b8;
  --good:#2f9e63; --warn:#b0762a; --crit:#c8524b;
  --good-bg:rgba(47,158,99,.10); --warn-bg:rgba(176,118,42,.11); --crit-bg:rgba(200,82,75,.10);
  --muted-bg:rgba(92,106,114,.10);
}}
:root[data-theme="light"]{
  --bg:#f2f4f5; --surface:#ffffff; --surface-2:#f6f8f9; --border:#dde4e8;
  --text:#18242b; --muted:#5c6a72; --faint:#94a2aa;
  --accent:#237f74; --accent-dim:#7cc0b8;
  --good:#2f9e63; --warn:#b0762a; --crit:#c8524b;
  --good-bg:rgba(47,158,99,.10); --warn-bg:rgba(176,118,42,.11); --crit-bg:rgba(200,82,75,.10);
  --muted-bg:rgba(92,106,114,.10);
}
:root[data-theme="dark"]{
  --bg:#0e1316; --surface:#151b1f; --surface-2:#1b2329; --border:#26313a;
  --text:#c9d2d8; --muted:#7c8891; --faint:#4f5a62;
  --accent:#3fb0a3; --accent-dim:#2b6f68;
  --good:#4bb37e; --warn:#d69b3f; --crit:#e0655d;
  --good-bg:rgba(75,179,126,.13); --warn-bg:rgba(214,155,63,.14); --crit-bg:rgba(224,101,93,.14);
  --muted-bg:rgba(124,137,145,.13);
}
*{box-sizing:border-box;}
.dash{max-width:1000px;margin:0 auto;padding:34px 22px 60px;color:var(--text);
  background:var(--bg);font-family:var(--sans);line-height:1.5;
  font-size:15px;-webkit-font-smoothing:antialiased;}
.dash :is(h1,h2,h3){text-wrap:balance;margin:0;}
.mono{font-family:var(--mono);font-variant-numeric:tabular-nums;}
.eyebrow{font-family:var(--mono);text-transform:uppercase;letter-spacing:.14em;
  font-size:11px;color:var(--muted);}

/* header */
.masthead{display:flex;justify-content:space-between;align-items:flex-end;gap:20px;
  border-bottom:1px solid var(--border);padding-bottom:18px;margin-bottom:24px;}
.brand h1{font-family:var(--mono);font-size:23px;font-weight:600;letter-spacing:-.01em;}
.brand h1 .dot{color:var(--accent);}
.brand .sub{color:var(--muted);font-size:13px;margin-top:5px;}
.brand .sub .mono{color:var(--faint);}
.health{display:flex;align-items:center;gap:10px;padding:8px 15px;border-radius:9px;
  border:1px solid var(--border);background:var(--surface);white-space:nowrap;}
.health .lamp{width:11px;height:11px;border-radius:50%;box-shadow:0 0 0 3px var(--muted-bg);}
.health .txt{font-family:var(--mono);font-size:13px;font-weight:600;letter-spacing:.02em;}

/* tiles */
.tiles{display:grid;grid-template-columns:repeat(4,1fr);gap:12px;margin-bottom:26px;}
@media (max-width:720px){.tiles{grid-template-columns:repeat(2,1fr);}}
.tile{background:var(--surface);border:1px solid var(--border);border-radius:11px;
  padding:15px 16px;position:relative;overflow:hidden;}
.tile::before{content:"";position:absolute;left:0;top:0;bottom:0;width:3px;background:var(--stripe,var(--accent-dim));}
.tile .k{font-family:var(--mono);text-transform:uppercase;letter-spacing:.12em;font-size:10.5px;color:var(--muted);}
.tile .v{font-family:var(--mono);font-variant-numeric:tabular-nums;font-size:30px;font-weight:600;
  line-height:1.05;margin-top:9px;color:var(--text);}
.tile .d{font-size:12px;color:var(--muted);margin-top:6px;}

/* sections */
.section{margin-top:30px;}
.section > .head{display:flex;align-items:baseline;justify-content:space-between;gap:12px;margin-bottom:12px;}
.section > .head h2{font-size:14px;font-weight:600;letter-spacing:.02em;
  text-transform:uppercase;font-family:var(--mono);color:var(--text);}
.section > .head .meta{font-size:12px;color:var(--muted);}
.card{background:var(--surface);border:1px solid var(--border);border-radius:11px;overflow:hidden;}

/* check rows */
.row{display:flex;align-items:flex-start;gap:13px;padding:12px 16px;border-top:1px solid var(--border);
  position:relative;}
.row:first-child{border-top:none;}
.row.fail{background:var(--crit-bg);}
.row.fail::before{content:"";position:absolute;left:0;top:0;bottom:0;width:3px;background:var(--crit);}
.row .name{font-family:var(--mono);font-size:13.5px;color:var(--text);min-width:0;flex:0 0 auto;}
.row .detail{color:var(--muted);font-size:12.5px;margin-left:auto;text-align:right;
  max-width:58%;overflow-wrap:anywhere;font-family:var(--mono);}
.row.ok .detail{color:var(--faint);}

/* pills */
.pill{font-family:var(--mono);font-size:11px;font-weight:600;letter-spacing:.06em;text-transform:uppercase;
  padding:3px 9px;border-radius:999px;white-space:nowrap;flex:0 0 auto;}
.pill.pass{color:var(--good);background:var(--good-bg);}
.pill.fail{color:var(--crit);background:var(--crit-bg);}
.pill.skip{color:var(--muted);background:var(--muted-bg);}
.pill.warn{color:var(--warn);background:var(--warn-bg);}

/* backlog */
.cols{display:grid;grid-template-columns:1fr 1fr;gap:12px;}
@media (max-width:720px){.cols{grid-template-columns:1fr;}}
.blk .cap{display:flex;justify-content:space-between;align-items:baseline;padding:12px 16px;border-bottom:1px solid var(--border);}
.blk .cap .t{font-family:var(--mono);font-size:12.5px;font-weight:600;letter-spacing:.04em;color:var(--text);}
.blk .cap .c{font-family:var(--mono);font-size:12px;color:var(--muted);}
.item{padding:10px 16px;border-top:1px solid var(--border);font-size:12.5px;color:var(--text);}
.item:first-of-type{border-top:none;}
.item .when{font-family:var(--mono);font-size:11px;color:var(--faint);display:block;margin-bottom:3px;}
.empty{padding:14px 16px;color:var(--muted);font-size:12.5px;}

/* blind audit */
.audit{padding:16px;color:var(--muted);font-size:13px;line-height:1.6;}
.audit strong{color:var(--text);font-weight:600;}
.audit .idle{display:inline-block;margin-top:10px;font-family:var(--mono);font-size:11.5px;
  color:var(--muted);background:var(--muted-bg);padding:4px 10px;border-radius:7px;}

.foot{margin-top:34px;padding-top:16px;border-top:1px solid var(--border);
  color:var(--faint);font-size:11.5px;font-family:var(--mono);display:flex;
  justify-content:space-between;gap:12px;flex-wrap:wrap;}
"""


def lamp(color):
    return '<span class="lamp" style="background:var(--%s);"></span>' % color


def pill(status):
    cls = {"pass": "pass", "fail": "fail", "skipped": "skip"}.get(status, "warn")
    label = {"pass": "pass", "fail": "fail", "skipped": "skipped"}.get(status, status)
    return '<span class="pill %s">%s</span>' % (cls, E(label))


def check_rows(checks, status_key="pass"):
    out = []
    for c in checks:
        ok = c.get("pass", False)
        status = c.get("status") or ("pass" if ok else "fail")
        row_cls = "ok" if (ok or status == "skipped") else "fail"
        name = c.get("check", "?")
        detail = c.get("detail") or ("" if ok else "")
        if status == "skipped":
            detail = c.get("detail", "not installed")
        det_html = '<span class="detail">%s</span>' % E(detail) if detail else ""
        out.append('<div class="row %s"><span class="name">%s</span>%s%s</div>'
                   % (row_cls, E(name), pill(status), det_html))
    return "".join(out)


def backlog_block(title, data):
    counts = data.get("counts", {})
    opens = data.get("open", [])
    cap = ('<div class="cap"><span class="t">%s</span>'
           '<span class="c">%d open &middot; %d done</span></div>'
           % (E(title), counts.get("open", 0), counts.get("done", 0)))
    if not opens:
        return '<div class="blk">%s<div class="empty">no open items</div></div>' % cap
    items = []
    for it in opens[:8]:
        items.append('<div class="item"><span class="when">%s</span>%s</div>'
                    % (E(it.get("added", "")), E(it.get("summary", ""))))
    more = ""
    if len(opens) > 8:
        more = '<div class="empty">+ %d more open</div>' % (len(opens) - 8)
    return '<div class="blk">%s%s%s</div>' % (cap, "".join(items), more)


def build(verify):
    inv = cli("invariants")
    bak = cli("backlog")
    now = datetime.datetime.now(datetime.timezone.utc).strftime("%Y-%m-%d %H:%M UTC")

    inv_checks = inv.get("checks", [])
    inv_pass = sum(1 for c in inv_checks if c.get("pass"))
    inv_total = len(inv_checks)
    todos = bak.get("todos", {})
    followups = bak.get("followups", {})
    open_total = todos.get("counts", {}).get("open", 0) + followups.get("counts", {}).get("open", 0)

    if verify is None:
        verify_ok, verify_txt, verify_lamp = None, "not run in this snapshot", "faint"
        verify_status_pill = '<span class="pill skip">not run</span>'
        verify_body = ('<div class="empty">The suite was not run for this snapshot. '
                       'Run <span class="mono">projectops verify</span> (or with '
                       '<span class="mono">--checks fmt,clippy</span> for a fast subset) to populate it.</div>')
    else:
        vchecks = verify.get("checks", [])
        verify_ok = verify.get("pass", False)
        verify_txt = "green" if verify_ok else "failing"
        verify_lamp = "good" if verify_ok else "crit"
        verify_status_pill = pill("pass" if verify_ok else "fail")
        verify_body = check_rows(vchecks)

    # overall health: red if invariants fail or verify fails; amber if verify not run.
    inv_ok = inv.get("pass", False)
    if not inv_ok or verify is False:
        overall_lamp, overall_txt = "crit", "attention"
    elif verify is None:
        overall_lamp, overall_txt = "warn" if inv_ok else "crit", ("nominal" if inv_ok else "attention")
    else:
        overall_lamp, overall_txt = ("good", "all green") if verify_ok and inv_ok else ("crit", "attention")

    tiles = [
        ("verify", verify_txt, "cargo + gitleaks suite",
         "good" if verify is not None and verify_ok else ("crit" if verify is False else "faint")),
        ("invariants", "%d/%d" % (inv_pass, inv_total), "security guards passing",
         "good" if inv_ok else "crit"),
        ("backlog", str(open_total), "open items (todos + followups)", "accent-dim"),
        ("hooks", "on" if any(c.get("check") == "agent_hooks_executable" and c.get("pass") for c in inv_checks) else "off",
         "pre/post/stop guards", "good" if inv_ok else "warn"),
    ]
    tile_html = "".join(
        '<div class="tile" style="--stripe:var(--%s);"><div class="k">%s</div>'
        '<div class="v">%s</div><div class="d">%s</div></div>' % (E(stripe), E(k), E(v), E(d))
        for (k, v, d, stripe) in tiles)

    audit = (
        '<div class="audit">The <strong>fully-blind audit</strong> (addendum &sect;7) verifies a crypto/de-id '
        'kernel from a self-contained packet, so no auditor inherits the code\'s own framing. It runs on '
        'demand, not on a schedule &mdash; there is no active run to show. This surface reports the packet, '
        'the panel verdicts, and which findings survived independent blind auditors and were then verified '
        'against source. This project\'s prior blind pass caught the escalation-gate bypass and the '
        'confirmation-replay hole the passing tests missed.'
        '<span class="idle">no active run</span></div>')

    body = """
<main class="dash">
  <div class="masthead">
    <div class="brand">
      <h1>projectops<span class="dot">.</span></h1>
      <div class="sub">agentic checks &middot; <span class="mono">cec-support-agent</span> &middot; <span class="mono">%(sha)s</span></div>
    </div>
    <div class="health">%(overall_lamp)s<span class="txt">%(overall_txt)s</span></div>
  </div>

  <div class="tiles">%(tiles)s</div>

  <div class="section">
    <div class="head"><h2>Verification</h2><span class="meta">%(verify_pill)s the cargo &amp; gitleaks suite</span></div>
    <div class="card">%(verify_body)s</div>
  </div>

  <div class="section">
    <div class="head"><h2>Security invariants</h2><span class="meta">%(inv_pass)d / %(inv_total)d passing &middot; fast git/grep</span></div>
    <div class="card">%(inv_rows)s</div>
  </div>

  <div class="section">
    <div class="head"><h2>Backlog</h2><span class="meta">%(open_total)d open</span></div>
    <div class="cols">
      <div class="card">%(todos)s</div>
      <div class="card">%(followups)s</div>
    </div>
  </div>

  <div class="section">
    <div class="head"><h2>Blind audit</h2><span class="meta">on demand</span></div>
    <div class="card">%(audit)s</div>
  </div>

  <div class="foot">
    <span>generated %(now)s &middot; projectops_panel.py</span>
    <span>static snapshot &mdash; regenerate to refresh</span>
  </div>
</main>
""" % {
        "sha": E(head_sha()),
        "overall_lamp": lamp(overall_lamp), "overall_txt": E(overall_txt),
        "tiles": tile_html,
        "verify_pill": verify_status_pill, "verify_body": verify_body,
        "inv_rows": check_rows(inv_checks), "inv_pass": inv_pass, "inv_total": inv_total,
        "open_total": open_total,
        "todos": backlog_block("TODOS.md", todos),
        "followups": backlog_block("FOLLOWUPS.md", followups),
        "audit": audit, "now": E(now),
    }
    return "<title>projectops &middot; agentic checks</title>\n<style>%s</style>\n%s" % (CSS, body)


def main():
    ap = argparse.ArgumentParser(description="render the projectops dashboard panel")
    ap.add_argument("--verify", action="store_true", help="also run the slow cargo/gitleaks suite")
    ap.add_argument("-o", "--output", default=None, help="output HTML file (default: stdout)")
    args = ap.parse_args()
    verify = cli("verify") if args.verify else None
    doc = build(verify)
    if args.output:
        with open(args.output, "w") as fh:
            fh.write(doc)
    else:
        sys.stdout.write(doc)
    return 0


if __name__ == "__main__":
    sys.exit(main())
