---
name: project-repo-identity
description: The CEC_AutoDiagnoser working dir is actually the cec-support-agent repo; where the agent-ops layer lives
metadata: 
  node_type: memory
  type: project
  originSessionId: 109a36d0-7dee-4d46-bdc4-c738f00cd762
---

The `/home/nathan/CEC_AutoDiagnoser` working dir is a clone of the **`cec-support-agent`** GitHub repo
(`https://github.com/nathanfraske/cec-support-agent.git`), NOT the GitHub repo literally named
`CEC_AutoDiagnoser` (that one is empty). It is the open Rust engine (Cargo workspace, 10 crates +
`support-agent` CLI): diagnose → candidate plans → judge panel → sign-off-gated execution → verify →
de-identified corpus write-back. The corpus + weights are PRIVATE and live elsewhere; only the corpus client +
schema are here. Truth is the **inverted corpus**: signed-off `(FaultSignature, Plan, OutcomeLabel)` triples
earned at the sign-off gate (`crates/corpus-client/src/gate.rs`) and read back retrieval-first.

The agent-ops layer (added 2026-06-14, branch `feat/agent-ops-evidence-integrity`, commit `c508970`): the
cross-agent baton + worklists are `HANDOFFS.md` / `TODOS.md` / `FOLLOWUPS.md` at the repo root (read
HANDOFFS first — it is injected at session start). Hooks in `.claude/hooks/` + `.claude/settings.json`. The
evidence-integrity design is `docs/evidence-integrity-and-research-checklist.md`; local inference is the
`cec-llm-broker` on `:8080` (`docs/local-agent-infrastructure.md`); WSL durability in
`docs/wsl-ephemeral-state-policy.md`. See [[wsl-ephemeral-durability]] and [[tracking-discipline-tombstones]].
