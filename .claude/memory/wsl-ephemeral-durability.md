---
name: wsl-ephemeral-durability
description: "WSL volume is disposable; durability hooks + the two gotchas (memory-dir _->- sanitization, no-identity clone)"
metadata: 
  node_type: memory
  type: project
  originSessionId: 109a36d0-7dee-4d46-bdc4-c738f00cd762
---

This box runs under WSL2 and the Linux volume is treated as **disposable** — load-bearing state must live on
the git remote, the Windows filesystem, or be rebuildable from the repo. The durability contract (implemented
in `.claude/hooks/session-start.sh` + `session-end.sh`, documented in `docs/wsl-ephemeral-state-policy.md`):
HALF 1 = in-tree `.claude/memory/` mirror committed on `main` (Stop hook refreshes it from live
`~/.claude/.../memory`; SessionStart re-seeds the empty live dir after a wipe); HALF 2 = off-tree
`ops/agent-handoff` branch pushed every Stop via git plumbing (never touches HEAD/index; carries
HANDOFFS/FOLLOWUPS/TODOS + memory + a generated handoff).

**Two verified gotchas for this repo (do not relearn):** (1) the project-memory dir sanitizes `_`→`-`, so the
derivation must be `tr '/._' '---'` with canonical fallback `-home-nathan-CEC-AutoDiagnoser` — CEC-Platform's
`tr '/.' '--'` is WRONG here. (2) A pristine post-wipe `git clone` has NO git identity, so `git commit-tree`
(and plain `git commit`) fails "empty ident name"; `session-end.sh` exports a `GIT_*` bot fallback, and for
manual commits set repo-local `user.name=nathanfraske` / `user.email=nathanfraske@cec.direct` (matches repo
history). The gh credential helper handles auth — no PAT needed. See [[project-repo-identity]].
