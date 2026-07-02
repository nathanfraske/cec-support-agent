# WSL-ephemeral state policy

> **Status:** implemented and verified in this repo. The Stop hook has pushed `ops/agent-handoff` to the remote with `main` untouched.
> **Owner directive:** 2026-06-14. **Scope:** `cec-support-agent` (CEC AutoDiagnoser), a Rust workspace developed under WSL2.

## TL;DR

The WSL2 Linux volume is **disposable**. Treat anything stored only on it as already lost. Every load-bearing artifact — source, agent memory, the cross-session handoff, the live worklist — must live in one of:

1. the **git remote** (committed on `main`, or pushed to the off-tree `ops/agent-handoff` branch), or
2. the **Windows filesystem** (`C:` / `E:`, e.g. the secrets file), or
3. be **rebuildable from the repo** (a checked-in provisioner / setup script).

If a thing is not in one of those three places, a `wsl --unregister` (or a botched distro move) erases it with no recovery. This document describes the policy as it is actually built here, the durability machinery that enforces it, the AutoDiagnoser-specific adaptations, and the recovery and verification procedures.

## Why this policy exists

On **2026-06-12**, in the sibling repo **CEC-Platform**, an attempt to relocate the WSL distro to the `E:` drive destroyed the entire Linux home. Almost everything came back — from the git remote, and from the idempotent `ops/provision.sh` provisioner. **One thing did not:** the previous session's agent handoff. It existed *only* on the WSL volume and was lost permanently.

The lesson: a `git clone` of `main` is necessary but not sufficient. State that is generated *between* commits to `main` — the agent's working memory, the in-flight handoff, the live TODO list — is exactly the state that lives only on the ephemeral volume at the moment of a wipe. The policy closes that gap with a durability contract that runs on **every** session, not just on commit.

## The durability contract (two halves, as implemented here)

Both halves are refreshed at every session **Stop** and re-seeded at every **SessionStart**, so the cross-session state can never again exist only on the WSL volume.

### Half 1 — the in-tree memory mirror (committed on `main`)

- The agent's live persistent memory lives under `~/.claude/projects/<sanitized-project-path>/memory/*.md`. **This live copy is disposable.**
- The **durable** copy is the git-tracked mirror at **`.claude/memory/*.md`**, committed normally on `main`.
- **On Stop**, `.claude/hooks/session-end.sh` plain-file-copies every live `…/memory/*.md` over the in-tree `.claude/memory/<basename>`. This is an ordinary working-tree write to already-tracked files — it never touches `HEAD` or the index; the *next* normal commit on `main` carries the refreshed snapshots. This keeps the versioned copy from drifting stale.
- **On SessionStart**, `.claude/hooks/session-start.sh` does the inverse **self-heal**: after a WSL wipe + `git clone`, the live memory dir is empty, so it copies each committed `.claude/memory/*.md` *into* the live dir — but **only files the live dir lacks**, never clobbering a newer live edit. This is why the mandatory handoff is surfaced at session start even on a brand-new clone.

A plain `git clone` of `main` therefore restores the durable memory, and the first session re-seeds the live dir from it. (See `.claude/memory/README.md`.)

### Half 2 — the off-tree `ops/agent-handoff` side branch (wipe-proof remote copy)

At every Stop, `session-end.sh` assembles a commit **entirely with git plumbing** and pushes it straight to `refs/heads/ops/agent-handoff` on `origin`. It is **non-disruptive by construction**: it never checks out, never touches `HEAD`, and never touches the normal index. The mechanics:

1. **Generate the handoff into a `mktemp` file** (never written to the working tree): repo state (branch, `HEAD` oneline, uncommitted-path count), the last 12 commits, the short working-tree status, a fail-soft probe of any local OpenAI-compatible inference endpoint, the full `HANDOFFS.md`, and full dumps of `current-work-handoff.md` + `MEMORY.md`.
2. `git fetch -q origin ops/agent-handoff`; point `GIT_INDEX_FILE` at a **separate temp index**; `git read-tree` the remote branch tip (or `--empty` on first run).
3. For each blob, `git hash-object -w` then `git update-index --add --cacheinfo 100644,<blob>,<path>`, writing the handoff to `docs/agent/handoff.md`, the three root tracking files to their own paths, and each memory file to `.claude/memory/<basename>`.
4. `git write-tree`; **skip the commit entirely if the tree equals the parent's tree** (no-op guard); else `git commit-tree <tree> -p <parent> -m "agent handoff <UTC> [session-end hook]"`.
5. `unset GIT_INDEX_FILE` and `git push <commit>:refs/heads/ops/agent-handoff` (PAT URL if present, else `gh` credentials).

The side-branch tree carries: **`docs/agent/handoff.md` + `HANDOFFS.md` + `FOLLOWUPS.md` + `TODOS.md` + the `.claude/memory/` mirror.**

The whole Stop path is **fail-soft and time-bounded**: `exec 2>/dev/null` silences all noise, every git/network call is wrapped in `timeout 60`, and every failure path exits `0`. A Stop hook must never block or fail the session.

**Together:** Half 1 makes the state recoverable by a plain `git clone` of `main` (and re-seeds the live session); Half 2 guarantees a *current* copy is on the remote at the end of **every** session, even between commits to `main`.

## The tracking files

Three Markdown files at the repo root carry cross-session work state. Each is created (if absent) and re-injected by a dedicated SessionStart hook, and all three are pushed to the side branch by the Stop hook.

| File | Role | Discipline | Maintained by |
|------|------|-----------|---------------|
| **`HANDOFFS.md`** | The **cross-agent baton**: current state, the exact next step ("Pick up here"), and append-only lessons learned. Read first. | Append-only lessons; reverse-chronological handoff log. **Injected verbatim at SessionStart** so every agent reads the baton before doing anything. | `.claude/hooks/handoffs-context.sh` |
| **`FOLLOWUPS.md`** | The **deferred backlog** — every non-blocking "not now / consider later / pending authorization" item. | **Append-only with tombstones**; never delete a line — flip `- [ ]` to `- [x]` and append `· closed <UTC> → <where it went>`. Every entry carries the **date *and* time** it was added. | `.claude/hooks/followups-context.sh` |
| **`TODOS.md`** | The **live checklist** of work being done *now*. | **Append-only with tombstones**; `- [x]` for done, `- [~]` for obsolete (with a tombstone). Date **and** time on every entry. | `.claude/hooks/todos-context.sh` |

The three are deliberately distinct: `TODOS.md` is work-now, `FOLLOWUPS.md` is deferred-later, `HANDOFFS.md` is resume-state. The append-only-with-tombstones discipline (a deliberate tightening over CEC-Platform, where resolved follow-ups were deleted) makes the deferral and work history fully auditable — it can never be silently rewritten.

These are wired alongside `session-start.sh` in `.claude/settings.json`:

- **SessionStart** → `session-start.sh`, `followups-context.sh`, `todos-context.sh`, `handoffs-context.sh`
- **Stop** → `session-end.sh`

## AutoDiagnoser-specific adaptations (and why)

This repo diverges from the CEC-Platform reference in exactly two load-bearing ways. Both are bugs that CEC-Platform never hits but this repo *would*, so they are fixed in the ported hooks.

### 1. Memory-dir sanitization must also map `_` → `-`

Claude Code stores per-project memory under a **sanitized** project path: it replaces `/`, `.`, **and `_`** with `-`. CEC-Platform's original hook derived the dir with `tr '/.' '--'`, which leaves underscores intact — fine for `CEC-Platform` (it has a hyphen, not an underscore), but **wrong for `CEC_AutoDiagnoser`**:

- naive derivation: `/home/nathan/CEC_AutoDiagnoser` → `-home-nathan-CEC_AutoDiagnoser` ❌ (underscore preserved — this path does not exist)
- correct sanitization: `-home-nathan-CEC-AutoDiagnoser` ✅

A wrong derivation fails **silently**: the seed/refresh targets a non-existent dir, the live memory is never re-seeded after a wipe, and the mirror never refreshes. Both hooks therefore derive the dir with **`tr '/._' '---'`** and use the corrected canonical fallback **`-home-nathan-CEC-AutoDiagnoser`** when the derived path does not exist:

```sh
memdir="$HOME/.claude/projects/$(echo "$ROOT" | tr '/._' '---')/memory"
[ -d "$memdir" ] || memdir="$HOME/.claude/projects/-home-nathan-CEC-AutoDiagnoser/memory"
```

### 2. A pristine post-wipe clone has no git identity — supply a fallback

`git commit-tree` **refuses** to build a commit with no `user.name` / `user.email` configured — it dies with `*** Please tell me who you are` / `empty ident name`. A fresh post-wipe `git clone` has exactly that: no identity. Because the Stop hook runs with `exec 2>/dev/null` and exits `0` on every failure, an unconfigured identity would kill the durability push **silently** — the precise failure mode this policy exists to prevent.

`session-end.sh` therefore exports a git identity for the process, preferring the configured identity and falling back to a stable bot identity:

```sh
_id_name="$(git config user.name  2>/dev/null || true)"; [ -n "$_id_name"  ] || _id_name="cec-agent-handoff[bot]"
_id_email="$(git config user.email 2>/dev/null || true)"; [ -n "$_id_email" ] || _id_email="agent-handoff@cec.direct"
export GIT_AUTHOR_NAME="$_id_name"    GIT_AUTHOR_EMAIL="$_id_email" \
       GIT_COMMITTER_NAME="$_id_name" GIT_COMMITTER_EMAIL="$_id_email"
```

**Authentication:** the push uses the `gh` credential helper, so **no PAT is required** today — `gh` auth is sufficient for the side-branch push. (A dedicated bot PAT scoped to this repo is an *optional* future hardening, not a requirement; see below. If `CEC_BOT_PAT` is ever provided via `ops/secrets/load-secrets.sh`, the hook rewrites the `origin` URL to `https://x-access-token:<PAT>@github.com/…` and prefers it.)

## Recovery procedure (after a WSL wipe)

1. **Reinstall WSL** and the Windows-side GPU/driver prerequisites for your workstation.
2. **`git clone`** the repo to its canonical checkout path (`/home/nathan/CEC_AutoDiagnoser`).
3. **Start a Claude session.** The SessionStart hooks self-heal automatically:
   - `session-start.sh` re-seeds the empty live memory dir from the in-tree `.claude/memory/` mirror and injects the memory index + `current-work-handoff.md`.
   - `handoffs-context.sh`, `followups-context.sh`, `todos-context.sh` recreate (if missing) and inject `HANDOFFS.md` / `FOLLOWUPS.md` / `TODOS.md`.
4. **For the very latest state** (anything since the last commit to `main`), read the **off-tree snapshot** on the `ops/agent-handoff` branch — `docs/agent/handoff.md` plus the mirrored tracking files and memory. It holds the most recent Stop-hook push, which may be newer than `main`.

No manual copying is needed: a clone of `main` plus a first session start restores the durable state; the side branch supplies anything committed after the last `main` commit.

## Verification

Confirm the contract holds without disturbing `main`:

```sh
# Half 2 exists on the remote:
git ls-remote origin ops/agent-handoff          # -> a commit hash on refs/heads/ops/agent-handoff

# Record main's tip, then run the Stop hook directly:
git rev-parse main
echo '{}' | .claude/hooks/session-end.sh ; echo "exit=$?"   # always exits 0

# main is untouched (HEAD, index, and ref unchanged):
git rev-parse main                               # same hash as before
git status --porcelain                           # no new staged/committed changes from the hook

# The side branch advanced (and carries the expected paths):
git ls-remote origin ops/agent-handoff           # new hash
git fetch -q origin ops/agent-handoff
git ls-tree -r --name-only FETCH_HEAD            # docs/agent/handoff.md, HANDOFFS.md,
                                                 # FOLLOWUPS.md, TODOS.md, .claude/memory/*.md
```

A healthy run: the Stop hook exits `0`, `main`'s tip and the working tree are unchanged, and `ls-remote` shows a fresh commit on `ops/agent-handoff` whose tree contains the handoff, the three tracking files, and the memory mirror. (If nothing changed since the last push, the no-op guard skips the commit and the side-branch hash stays the same — also correct.)

## Optional future hardening

The policy is complete as-is. The following are *optional* tightenings — phrased so they can be pasted straight into `FOLLOWUPS.md`:

```
- [ ] [added 2026-06-14 00:00 UTC] Provision a bot PAT scoped to cec-support-agent contents:write, placed in /mnt/e/secrets/cec-bot.env (survives-WSL) and consumed via ops/secrets/load-secrets.sh — so the ops/agent-handoff push authenticates as the bot instead of the owner gh login — why deferred: gh credential fallback works today; PAT is a least-privilege/auditability upgrade, not a blocker
- [ ] [added 2026-06-14 00:00 UTC] Add a cargo-shaped ops/provision.sh (cargo build/test/clippy + githook install) so disaster recovery is one idempotent script — why deferred: do NOT copy CEC-Platform's KiCad/CUDA/broker provisioner; this repo is a Rust workspace and needs its own
- [ ] [added 2026-06-14 00:00 UTC] Add claude-rc survivability units (tmux + systemd --user claude-rc@.service with Restart=always, rc-recover.sh) repointed to the AutoDiagnoser ops path, so a dropped WSL console never orphans the agent — why deferred: nice-to-have resilience layer, independent of the durability contract
```

---

*Implemented in: `.claude/settings.json`, `.claude/hooks/session-start.sh`, `.claude/hooks/session-end.sh`, `.claude/hooks/handoffs-context.sh`, `.claude/hooks/followups-context.sh`, `.claude/hooks/todos-context.sh`, `.claude/memory/` (+ `README.md`). Durable copies live on `main` (in-tree mirror) and on the `ops/agent-handoff` remote branch (off-tree snapshot).*
