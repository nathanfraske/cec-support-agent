---
name: wsl-ephemeral-durability
description: "WSL volume is disposable; durability hooks + the three gotchas (memory-dir _->- sanitization, no-identity clone, chmod no-op on /mnt/e DrvFs)"
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
history). The gh credential helper handles auth — no PAT needed. (3) **`chmod` is a no-op on the `/mnt/e` DrvFs mount
(verified):** `chmod 700/600` silently "succeeds" but perms stay `0o777`, so a secret on `/mnt/e` is
world-readable. `/mnt/e/secrets` already holds a real GitHub PAT + sudo password world-readable. For a secret
that must be durable (off-tree on `/mnt/e`) AND protected, use encryption-at-rest (`age`/`gpg`) or Windows
ACLs (`icacls`), never `chmod`. This is why the private-corpus ed25519 seed custody is encrypt-at-rest. The
private corpus itself is the separate off-tree repo `/mnt/e/cec-corpus-private`. See [[project-repo-identity]].
