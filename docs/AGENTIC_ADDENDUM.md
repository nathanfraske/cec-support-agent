# Agentic Addendum: Hooks, Memory, Panels, and the Blind Audit

This addendum specifies the agentic infrastructure for continuing `cec-support-agent`
inside a coding-agent harness. The reference harness is Claude Code (the CLI, the web/
remote-execution runner, and the Agent SDK), whose hook lifecycle, MCP configuration, and
`AGENTS.md` auto-loading are its native mechanisms; the requirements are stated so they map
to any equivalent harness. Nothing here touches the engine's runtime invariants or the
private corpus: hooks and servers operate on the repository and the tracking documents,
never on corpus data, model weights, or a sign-off seed.

The goal is narrow and practical. The engine's security properties — the de-identification
type barrier, the attestation gate, the frozen route surface, non-mappability — are held by
the agent's judgment and the compiler; this addendum adds a mechanical safety net so a
customs violation (a corpus row that leaks, an unattested write, a re-added membership
oracle, an unverified commit, a stale handoff) cannot slip through, and so the review panels
and the blind audit have real, structured data to work from. The division of labour is the
reference project's: the agent's judgment does the work and runs the suite itself; the hooks
are the backstop that makes a violation impossible to ship rather than merely discouraged.

---

## 1. Memory persistence: four files plus a durability mirror

Memory is persisted in plain Markdown at the repository root, plus a git-tracked memory
mirror under `.claude/memory/`. Each has a distinct role and lifecycle, and the SessionStart
hooks inject the operative policy for each into every session.

**`AGENTS.md` (static; the operating manual).** The `CLAUDE.md`-analogue the harness loads
automatically as project memory, so the working rules are always in context. It carries the
six runtime-invariant pointers, the tracking discipline, and — load-bearing — the two
**binding checklists** every corpus-facing change must satisfy in the same PR: the
per-endpoint **egress-sink checklist** (vocabulary-only bodies, the poison contract test,
tokens-not-Display errors, no prose in logs, attest rows crossing the wire, never let the
wire lower a gate without a signature) and the **non-mappability** rule set (leak-C10: one
answer per call, no membership differential, no behavioural oracle, minimal attested unit,
attributable calls, non-enumerable keys, budgeted). It changes rarely and only on the
owner's call. Read-mostly.

**`HANDOFFS.md` (rolling; the cross-agent baton).** The first thing the next session reads.
Three sections kept current in the same turn as the work — **Current state**, **Pick up
here** (the exact next step, concrete enough to start immediately), **Lessons learned**
(durable, append-only, never delete a lesson) — above a reverse-chronological dated handoff
log. It recovers state without re-derivation. Never rewrite history; append.

**`TODOS.md` (live; the work-now checklist).** Everything being done now, in checkbox form:
`- [ ] [added YYYY-MM-DD HH:MM UTC] <task>` active, `- [x] [… · done …]` completed (left in
place), `- [~] [… · obsolete … → <tombstone>]` obsolete. Append-only; a completed item is
flipped, never deleted.

**`FOLLOWUPS.md` (deferred; the not-now backlog).** Every non-blocking item deferred to the
future, appended in the same turn it is raised: `- [ ] [added … UTC] <item> — <why deferred
/ where to resume>`, closed by flipping to `- [x] [… · closed … → <where it went>]` with the
tombstone pointing at a PR number, a TODOS line, another doc, or `dropped: <reason>`.

**`.claude/memory/*.md` (the durability mirror).** The git-tracked copy of the agent's
persistent file-memory — `MEMORY.md` (the index), `project-repo-identity.md`,
`tracking-discipline-tombstones.md`, `wsl-ephemeral-durability.md`. Its live ephemeral
counterpart lives outside the repo and is re-seeded from this mirror after a wipe. Edit
memory through the live dir and let the Stop hook mirror it; do not hand-edit these snapshots
directly.

The three tracking files are all **append-only with tombstones, UTC date-and-time on every
entry, never a deleted line** — a deliberate tightening over the antecedent CEC-Platform
policy, which used date-only and deleted resolved items. The distinction is strict:
HANDOFFS is resume-state, TODOS is work-now, FOLLOWUPS is deferred-later.

---

## 2. The hooks

Hooks live in `.claude/settings.json` (project-committed, so they are shared) under the
`hooks` key, plus a user-level Stop gate the runner supplies. Handler type is `command`
throughout. What exists today, and what is worth adding, are marked.

### 2a. SessionStart: load the baton, re-seed memory (exists)

Four `SessionStart` command hooks fire in order, each emitting
`hookSpecificOutput.additionalContext`:

- **`session-start.sh`** re-seeds the ephemeral live-memory dir from the in-tree
  `.claude/memory/` mirror **only for files it is missing** (it never clobbers a newer live
  edit), then injects the MANDATORY STARTUP CONTEXT preface plus `MEMORY.md` (capped 4000
  chars) and the current work-handoff (capped 8000). It establishes the "read the persistent
  memory and the baton before doing any work" baseline. It does **not** run the verification
  suite.
- **`followups-context.sh`**, **`todos-context.sh`**, **`handoffs-context.sh`** each create
  their file if absent and inject that file's standing POLICY (the append-only-with-tombstone
  rules above) plus, for HANDOFFS, the current baton content (capped 12000 chars).

`SessionStart` re-runs on resume, so the baseline is never stale.

### 2b. PreToolUse: the invariant guard (built — prevention)

`PreToolUse` fires before a tool runs and blocks it by exiting 2, even under bypass mode.
Matched to `Write|Edit`, **`invariant-guard.sh`** is the hard guard that a corpus/weights/
seed file cannot be written into the tree in the first place. It parses `tool_input.file_path`
and blocks (exit 2, with a message naming the shape) any write **inside the repo** whose path
matches the exfil regex `scripts/githooks/pre-commit` and `.gitignore` already deny — a
`corpus/`/`weights/` path, a `.gguf`/`.safetensors`/`.bin`/`.sqlite`/`.duckdb`/`.jsonl`/
`.ndjson`/`.seed`/`.seedhex`/`.env` extension, a `*.flow.yaml`, anything matching
`cec-corpus`. This promotes the dormant pre-commit block (inert until `core.hooksPath` is set,
and only firing at commit) to a pre-**write** deny, and writes outside the repo pass through.

The block is deliberately **path-only**, because that is the check with effectively zero
false positives — no one legitimately writes a `corpus/` file or a `.seed` into the engine
repo. The fuzzier content-level customs (a re-added `"source"` envelope field, a `Serialize`
derive back on a raw domain type, a new `route_surface()` route) are **not** hard-blocked
here — a grep heuristic would false-positive on this repo's own prose and on the legitimate
serde on stored types. Those are surfaced non-blocking in §2c and belong properly in the
`projectops` `invariants` tool (§3), on top of the compiler and the negative pins that already
catch them.

### 2c. PostToolUse: per-edit reaction (built)

`PostToolUse` fires after a successful edit and cannot undo it, but it can surface feedback.
Matched to `Write|Edit`, **`invariant-check.sh`** reads the just-written file and surfaces
(exit 2, feedback) only on **unambiguous** problems — a merge-conflict marker, content shaped
like a serialized corpus row (`"outcome":{"signature":{"fingerprint"…}`, the renamed-dump
backstop the path guard cannot catch), or an encrypted-seed / private-key block — and is
silent otherwise. It is intentionally narrow so it never false-positives on the repo's own
documentation, which discusses `attestation`/`sign_off`/`fingerprint` as prose.

### 2d. Stop: the completion gate

Two things happen at Stop.

- **`session-end.sh` (exists; the durability keystone).** Refreshes the in-tree memory
  mirror from the live dir, generates `docs/agent/handoff.md` (repo state, recent commits,
  working-tree changes, local-broker liveness, the full baton, a memory snapshot), and
  commits it plus `HANDOFFS.md`/`TODOS.md`/`FOLLOWUPS.md`/the memory files to the orphan
  **`ops/agent-handoff`** branch via git plumbing — a temp index and `commit-tree`, so it
  never touches HEAD or the working index. It pushes with a bot PAT if `CEC_BOT_PAT` is set,
  else the ambient credential helper, wrapped in a 60s timeout and `|| true` so it never
  fails the session. This is the wipe-proof, off-tree half of the WSL-ephemeral durability
  contract; the in-tree `.claude/memory/` mirror on `main` is the other half.
- **The user-level `stop-hook-git-check.sh` gate (exists).** Refuses to end the turn while
  the tree is dirty (uncommitted or untracked files), while commits are unverified (no
  signature, or a committer email other than `noreply@anthropic.com`, when `commit.gpgsign`
  is set), or while commits are unpushed. It guards recursion with `stop_hook_active` and
  exits 2 with the specific remediation. It is a backstop, not a substitute for the agent
  committing and pushing its own work.
- **`tracking-freshness.sh` (built; a soft nudge).** If the branch changed `crates/` (vs
  `origin/main`) without also updating `HANDOFFS.md` or `TODOS.md`, it surfaces a one-time
  reminder to keep the baton and checklist current in the same turn. It respects
  `stop_hook_active` (single pass, never a loop) and never blocks the durability push.

**Still proposed: fold the verification suite into the Stop gate.** The full `cargo` suite is
deliberately **not** run at Stop — a multi-minute compile on every turn end is impractical;
that is CI's job, and, once built, the `projectops` `verify` tool's (§3), which the gate can
call selectively. The freshness nudge above is the fast, always-on half.

### 2f. Activation: `ops/provision.sh`

The guards above ship inert until the hooks are executable and the dormant pre-commit exfil
guard is switched on. **`ops/provision.sh`** is the one idempotent activator: it runs `git
config core.hooksPath scripts/githooks`, `chmod +x`'s the hooks, ensures the pinned toolchain
components, warns if `gitleaks` is absent (a hard dependency of the pre-commit hook), and runs
the full suite. Safe to re-run; it is also the disaster-recovery script for a fresh clone. It
does not provision secrets, a bot PAT, or branch protection — those stay owner/GitHub-side.

### 2e. Note on the two settings layers

The repo's `.claude/settings.json` (committed, shared) is layered under a runner-level
settings file that supplies the Stop git-check and a git-identity SessionStart hook. Repo
hooks are the project's; the runner hooks are the harness's. Keep security-relevant guards
(the exfil block, the invariant guard) in the committed repo layer so the team shares them.

---

## 3. MCP and tooling standup (built)

The **`projectops`** surface is built as two pieces: **`tools/projectops.py`** (a pure-stdlib
CLI emitting structured JSON — the keystone everything else consumes) and
**`tools/projectops_server.py`** (a minimal MCP stdio server, raw JSON-RPC 2.0 with no
third-party SDK, wired in **`.mcp.json`** and addressed `mcp__projectops__<tool>`). A cardinal
constraint, enforced by construction: it operates on the repository and the tracking docs
**only** — it never reads, serves, or enumerates the private corpus (which does not live in
this repo at all), consistent with the never-routable invariant. The surface:

- **`verify`** — runs the suite (`cargo fmt --all -- --check`, `cargo clippy --workspace
  --all-targets -- -D warnings`, `cargo build --workspace`, `cargo test --workspace`, `cargo
  deny check`, the gitleaks scan) and returns structured JSON: each check with name, pass or
  fail, and the failing lines; an absent tool (cargo-deny/gitleaks) is reported `skipped`, not
  failed. A `--checks` subset keeps a fast call fast. This is what a Stop verify-gate and the
  verification panel call.
- **`invariants`** — the fast git/grep security checks (no cargo): no corpus/weights file is
  tracked; the diagnose envelope carries no `source` membership label (leak-C10); the `/v1`
  `route_surface()` is frozen to health/diagnose/execute; `ACTION_VOCABULARY` is sorted; the
  wire-grammar pinning tests are present; the agent hooks are executable. Each returns a named
  pass/fail so a regression is attributable — this is the fast structured guard the PreToolUse
  hook deliberately leaves to a typed tool rather than a grep-block. (A deeper `no raw type
  derives Serialize` check and the full vocabulary-vs-registry drift belong here next; the type
  system and the drift test cover them today.)
- **`backlog`** — parses `TODOS.md` and `FOLLOWUPS.md` into open/done/obsolete counts and the
  open items with their UTC timestamps.
- **`leak_scan`** — runs the de-identification / poison / leakage / attestation test slice as
  one callable check.

The GitHub MCP server (PR/CI/issue operations) and the connected Drive server round out the
surface. A panel backed by MCP must be a real server: a rendered HTML artifact cannot call an
MCP URL, so panels read `projectops` through the harness, not via `fetch()`.

---

## 4. Panels (built)

The review surface is built as **`tools/projectops_panel.py`** — it runs the §3 checks and
renders their JSON into one self-contained, theme-aware HTML dashboard. Because the harness
CSP forbids a rendered page from calling an MCP URL, the panel is a **static snapshot**: the
data is baked in at generation time, and you regenerate to refresh (`python3
tools/projectops_panel.py --verify -o panel.html`). It is content-only HTML, so it drops
straight into the harness Artifact wrapper and also opens on its own. Its sections:

- **Verification.** Each suite check (fmt, clippy `-D warnings`, build, test, `cargo deny`,
  gitleaks) as a pass/fail/skipped pill with its failing lines; an absent optional tool
  (cargo-deny, gitleaks) reads `skipped`, not failed. `--verify` runs the full suite; without
  it the panel marks verification "not run in this snapshot."
- **Security invariants.** The six fast `projectops invariants` guards (no exfil tracked; no
  `source` membership label; frozen `/v1` route surface; sorted `ACTION_VOCABULARY`; wire pins
  present; hooks executable), each a named pass/fail.
- **Backlog.** The open `TODOS.md`/`FOLLOWUPS.md` items with counts and UTC stamps.
- **Blind audit.** The §7 method with an idle "no active run" state (a run is on demand); the
  surface reports the packet, the verdicts, and which findings survived and were source-verified.

A summary tile row (verify · invariants N/M · open backlog · hooks) puts the health read
before the detail, state is encoded in form (status pills, a severity stripe on a failing
row) as well as number, and the semantic good/warn/crit palette is kept separate from the
teal accent. The common requirement stands: the panel is only as good as `projectops`
emitting structured output and the tracking/constant formats staying parseable (§5).

*Still open (FOLLOWUPS): the panels render a snapshot but nothing yet regenerates them on a
schedule or a Stop; and the invariants set can deepen (a real no-raw-`Serialize` check, the
full vocabulary/registry drift). Building the panel dogfood-surfaced and fixed a `verify`
bug — a missing cargo **subcommand** (`cargo deny` absent) exits non-127, so it was read as
`fail` rather than `skipped`; `projectops.py` now treats "no such command" as skipped too.*

---

## 5. Stable formats the tooling depends on

- **Tracking bullets** hold the exact templates in §1 (the `- [ ] [added … UTC]` forms and
  their tombstones), so the backlog tool parses them deterministically.
- **HANDOFFS** keeps its Current state / Pick up here / Lessons learned / dated handoff log
  shape.
- **The frozen security constants stay parseable and sorted** where the code requires it:
  `deid::ACTION_VOCABULARY` (8 tokens, sorted for `binary_search`), `leakguard::POISON` (9),
  `STOP_CODE_NAMES` (42, sorted), `MODULE_NAMES` (68, sorted). A drift test already ties
  `ACTION_VOCABULARY` to the dispatcher registry; keep it.
- **The wire grammar and the `GateError` variants** are the stable vocabulary the egress-sink
  checklist and the panels reference; a new one is an additive, pinned change.

---

## 6. The lifecycle, end to end

A session runs: the four `SessionStart` hooks inject the baton, the tracking policies, and
the re-seeded memory, so the agent begins knowing the state and the customs. The agent works
under the (proposed) `PreToolUse` invariant guard, which blocks a corpus/weights/seed write
or a re-added oracle before it lands, and the `PostToolUse` check, which surfaces a slip the
moment it happens. As it finishes, the agent updates `HANDOFFS.md` and `TODOS.md` in the same
turn. The Stop gate then refuses to end while the tree is dirty, the commits are unverified or
unpushed, or (proposed) the suite is red or the memory is stale; and `session-end.sh`
snapshots the tracking files and memory to the durable `ops/agent-handoff` branch by git
plumbing that never touches HEAD. The panels read structured `projectops` results throughout,
so the maintainer sees verification status, the backlog, the invariants, and any blind-audit
run without leaving the harness.

---

## 7. The fully-blind audit (for uncontaminated correctness verdicts)

Reach for this when a verdict must not be contaminated by the repository's own framing:
verifying that a **crypto or de-identification kernel, or a frozen constant, is correct**,
where a shared blind spot between the code and its own tests could hide a defect. A test
written to match a buggy output, a comment that rationalizes an error, or a prior sighted
review that inherited the same assumption will all pass a normal review, because they were
written against the same premise. A blind auditor, given only what the code claims and what it
is built on, reaches its verdict from first principles and does not inherit that premise.

This is not hypothetical for this engine. This project's own adversarial audit found, in
merged and test-covered code, an **escalation-gate bypass** (the execute gate bound to the
judge's winner instead of the candidate the caller actually selected, so a verifier sign-off
could run a Human-required sibling) and a **confirmation-replay hole** (a no-provenance row
keyed on its list index, so a byte-identical replay inflated the confirmation count) — both of
which the passing test suite missed because the tests encoded the same assumptions. Earlier, a
de-identification "proof" test was vacuous: it seeded identity into every field **except** the
two (`action`, `plan.id`) that `de_identify_plan` copied through verbatim, so it could not
fail on the leak it claimed to guard. The blind audit is the method that catches this class.

**The packet.** The auditor sees only a self-contained packet, written to the scratchpad
outside the repository, with three sections.

- **Section A — the substrate contract.** The exact semantics of every type and primitive the
  kernel calls, so totality, overflow, and rounding are judgeable without the source:
  `ed25519-dalek` sign/verify (boolean verify, no panic; the seed is the secret half, the
  engine holds only the public key); `sha256` and `HMAC-SHA256` (constant-time verify);
  `FNV-1a` (non-cryptographic, `wrapping_mul`, keys sorted then 0xff-separated for
  order-independence and concatenation-resistance); the length-prefixed canonical encoding
  discipline (`tag[len]=value`, serde-independent, mirrored across `provenance::canonical`,
  `attestation_message`, and `chain_hash`); the substrate types (`Prose` — private field, no
  `Serialize`/`Display`, redacting `Debug`; the `Stored*` corpus types — `#[serde(try_from)]`
  validating deserialization; `Risk`/`Consent`/`Escalation`/`SignOff` ordered ladders;
  `ConfigClass` binding its variant tag). Section A also states the **audit checklist** — de-
  identification (does identity survive?), attestation-binding completeness (does the signed
  message cover every field a forger could vary?), independence (can a replay inflate a
  count?), fail-closed totality (does an unparsed/None/missing input default to the safe
  answer?), non-mappability (does the output add a hit/miss differential?), fabrication and
  steering — and the **reserved-value conventions**: the frozen constants a kernel assumes
  correct. Those constants are our "reserved values": `ACTION_VOCABULARY` (must equal the
  dispatcher registry), `POISON`, `STOP_CODE_NAMES`/`MODULE_NAMES` (completeness is the whole
  guarantee of the closed-grammar de-id), the domain-tag/version prefixes
  (`cec-signoff-attestation-v3`, `cec-corpus-chain-v1`, `cec-corpus-confirmation-content-v1`,
  `cec-plan-canonical-v1`), the panel retrieval prior (0.8 vs 0.6), `SESSION_TTL`/
  `MAX_SESSIONS`, and the pinned wire tokens.

- **Section B — the code under audit.** The kernel functions only, with the tests, prior
  audits, design docs, and cross-references left out. The arithmetic- and logic-heavy
  surfaces worth a blind pass: `attestation_message` and `chain_hash` and `content_hash` (do
  they bind every distinguishing field, unambiguously?), `de_identify_plan` and the `deid`
  mints and the `Stored*` `try_from` validators (does anything reach a row un-minted?),
  `confirmation_key` and `fix_mappings` (is the independence key replay-proof?),
  `ensure_evidence_integrity` and `ensure_attested` (is any path fail-open?), `diagnose_
  envelope` and the `wire_*` mappings (does any field betray membership?), `endpoint_is_
  loopback` (does any non-loopback host parse as loopback?), the `serve` escalation recompute
  (does it bind to the selected candidate?).

- **Section C — the declared specification.** What each kernel is supposed to compute, its
  inputs with units and ranges, and its declared output and bound — for example,
  "`attestation_message` binds exactly `(signature, plan, label, verification, sign_off,
  config_class variant + key, provenance)`, length-prefixed, excluding the attestation
  itself; two rows differing in any bound field must produce different bytes."

The packet carries the code and what it claims, and nothing of the repository's proof that it
is correct.

**The protocol.** First pilot with one agent: is the packet sufficient to reach a conclusion
with zero repository access? The pilot audits a spanning sample and reports, precisely, any
contract the packet lacked. Fold those gaps into the packet before spending on a panel. Then
run the full panel: several independent auditors, each blind, each reading only the packet,
none aware of the others or of any prior verdict, each classifying per the checklist. To test
whether they converge — the point of the exercise — double-cover the arithmetic-heavy slices
(the attestation-binding completeness, the confirmation-independence math, the loopback host
parse) or run independent passes and compare. Then **verify every flagged finding against the
real code yourself before trusting it**: the auditor is blind and may misread an intended
convention, so a finding is a lead to prove, not a verdict to accept. Reconcile: convergence
across independent blind auditors is strong evidence, and a finding that survives your own
check against the source is a real defect to surface and fix — as the escalation bypass and
the replay hole both were.

**Enforcing the blind.** Instruct each auditor to read only the packet file and to not read,
grep, glob, or open any file under the repository. If an auditor feels it cannot judge a
kernel without the source, it must not go and get it: it records exactly what the packet was
missing, which sharpens the packet rather than breaking the blind.

**Model tiering.** Panelists run on the cheapest model that accomplishes the goal: Sonnet for
reasoning-level correctness audits (the binding-completeness and independence kernels), Haiku
when the work is sheer mass over many simple units (checking each of the 42 stop-code names,
the 68 module names, or each pinned wire token against its spec), and Opus reserved for the
hardest kernels that need full depth (the attestation canonical encoding, the confirmation
math, the de-id round-trip grammar). The pilot and the per-finding verification are the same
tiering call.

The vehicle of a run is the packet plus the panel outputs, kept in the scratchpad; a durable
finding graduates into a fix and, if it reveals a class of defect, into a Section A checklist
item and — where it is a wire or corpus surface — a line in the AGENTS.md binding checklists,
so the next packet and the next PR both name it by default.
