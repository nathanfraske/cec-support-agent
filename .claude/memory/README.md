# Durable in-tree memory mirror (WSL-ephemeral state policy)

This directory is the **git-tracked, durable mirror** of the agent's persistent file-memory, which lives
(ephemerally) at `~/.claude/projects/-home-nathan-CEC-AutoDiagnoser/memory/`. The live dir is wiped when WSL
is reset; this mirror is not.

- The **Stop** hook (`.claude/hooks/session-end.sh`) refreshes this mirror from the live memory dir and also
  pushes a wipe-proof copy to the `ops/agent-handoff` branch on the remote.
- The **SessionStart** hook (`.claude/hooks/session-start.sh`) re-seeds the empty live dir from this mirror
  after a WSL wipe + fresh clone, so the memory + handoff are always surfaced at session start.

Do not hand-edit the `*.md` snapshots here; edit memory through the live dir and let the Stop hook mirror it.
