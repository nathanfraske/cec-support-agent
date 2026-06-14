export const meta = {
  name: 'autodiagnoser-recon',
  description: 'Recon: map cec-support-agent pipeline + CEC-Platform evidence integrity, research checklist, inverted ground truth, WSL-ephemeral policy, and local-agent infra',
  phases: [{ title: 'Recon' }],
}

const RO = 'You are a READ-ONLY recon agent. Do NOT write, edit, or create any files. Only read and report. Your final output is structured data, not prose for a human.'

const pipelineSchema = {
  type: 'object',
  additionalProperties: true,
  properties: {
    summary: { type: 'string' },
    crates: {
      type: 'array',
      items: {
        type: 'object',
        additionalProperties: true,
        properties: {
          name: { type: 'string' },
          role: { type: 'string' },
          key_files: { type: 'array', items: { type: 'string' } },
          evidence_provenance_relevance: { type: 'string' },
        },
        required: ['name', 'role'],
      },
    },
    pipeline_stages: { type: 'array', items: { type: 'string' } },
    evidence_integrity_points: { type: 'array', items: { type: 'string' } },
    corpus_and_ground_truth: { type: 'string' },
    where_checklist_would_hook: { type: 'array', items: { type: 'string' } },
    gaps: { type: 'array', items: { type: 'string' } },
  },
  required: ['summary', 'crates', 'pipeline_stages', 'evidence_integrity_points', 'where_checklist_would_hook'],
}

const evidenceSchema = {
  type: 'object',
  additionalProperties: true,
  properties: {
    found: { type: 'boolean' },
    source_files: { type: 'array', items: { type: 'string' } },
    what_it_is: { type: 'string' },
    checklist_items: { type: 'array', items: { type: 'string' } },
    evidence_integrity_rules: { type: 'array', items: { type: 'string' } },
    how_enforced: { type: 'string' },
    verbatim_excerpts: { type: 'array', items: { type: 'string' } },
  },
  required: ['found', 'source_files', 'what_it_is', 'checklist_items', 'evidence_integrity_rules'],
}

const invertedSchema = {
  type: 'object',
  additionalProperties: true,
  properties: {
    definition: { type: 'string' },
    how_it_works: { type: 'string' },
    contrast_with_normal_ground_truth: { type: 'string' },
    implications_for_evidence_integrity: { type: 'array', items: { type: 'string' } },
    relevant_files: { type: 'array', items: { type: 'string' } },
    verbatim_excerpts: { type: 'array', items: { type: 'string' } },
  },
  required: ['definition', 'how_it_works', 'implications_for_evidence_integrity', 'relevant_files'],
}

const wslSchema = {
  type: 'object',
  additionalProperties: true,
  properties: {
    policy_summary: { type: 'string' },
    components: {
      type: 'array',
      items: {
        type: 'object',
        additionalProperties: true,
        properties: { name: { type: 'string' }, path: { type: 'string' }, role: { type: 'string' }, mechanism: { type: 'string' } },
        required: ['name', 'role'],
      },
    },
    durability_contract: { type: 'string' },
    parity_steps_for_target_repo: { type: 'array', items: { type: 'string' } },
    adaptations_needed: { type: 'array', items: { type: 'string' } },
    secrets_handling: { type: 'string' },
  },
  required: ['policy_summary', 'components', 'parity_steps_for_target_repo', 'adaptations_needed'],
}

const infraSchema = {
  type: 'object',
  additionalProperties: true,
  properties: {
    current_state_summary: { type: 'string' },
    components: {
      type: 'array',
      items: {
        type: 'object',
        additionalProperties: true,
        properties: { name: { type: 'string' }, path: { type: 'string' }, role: { type: 'string' }, how_to_run: { type: 'string' }, ports_or_services: { type: 'string' } },
        required: ['name', 'role'],
      },
    },
    how_local_agents_run_now: { type: 'string' },
    what_changed: { type: 'array', items: { type: 'string' } },
    files_and_scripts: { type: 'array', items: { type: 'string' } },
    open_questions: { type: 'array', items: { type: 'string' } },
  },
  required: ['current_state_summary', 'components', 'how_local_agents_run_now', 'files_and_scripts'],
}

phase('Recon')

const [pipeline, evidence, inverted, wsl, infra] = await parallel([
  () => agent(`${RO}

TARGET: /home/nathan/CEC_AutoDiagnoser — the cec-support-agent Rust workspace (open engine: Diagnose → candidate plans → judge panel → sign-off-gated execution).

Map the CURRENT pipeline in depth. Read: README.md, AGENTS.md, CONTRIBUTING.md, every file under crates/*/src/*.rs (especially common/src/extract.rs, common/src/fault.rs, provenance/src/lib.rs, corpus-client/src/{gate,schema,store,lib}.rs, agent-core/src/{verify,execute,agent}.rs, panel/src/lib.rs, swarm/src/lib.rs, intake/src/lib.rs, support-agent/src/main.rs), and the PR that built it (\`git -C /home/nathan/CEC_AutoDiagnoser show c3a4000 --stat\` and diffs of key files).

I am implementing an "evidence integrity and research checklist" in this repo, adapted to its "inverted ground truth corpus" approach. So focus your map on: (a) the eight pipeline stages and which crate owns each; (b) EVERY place evidence integrity, provenance, signing, de-identification, sign-off, or verification is already enforced (cite file:line); (c) how the corpus / ground-truth / retrieval-first flywheel works in code; (d) the concrete hook points where an evidence-integrity / research checklist would attach (e.g. before corpus write-back, at verification, at provenance signing); (e) gaps where integrity is NOT yet enforced.

Be specific with file paths and line numbers. Return the structured object.`, { label: 'recon:pipeline-map', schema: pipelineSchema }),

  () => agent(`${RO}

Find and fully extract the "EVIDENCE INTEGRITY" policy and the "RESEARCH CHECKLIST" that exist in /home/nathan/CEC-Platform. The exact phrases may not grep — they may be called provenance, fact integrity, citation discipline, claim-evidence binding, audit, in-loop audit, "claims must cite", research protocol, verification protocol, or a checklist embedded in CLAUDE.md / docs/protocols / docs/research / docs/decisions.

Search thoroughly. Strong candidates to read: /home/nathan/CEC-Platform/CLAUDE.md (search it for "evidence", "integrity", "research", "checklist", "claim", "citation", "provenance", "audit", "fact"), docs/protocols/*, docs/research/*, docs/decisions/*, scripts/cec_facts.py, scripts/cec_inloop_audit.py, scripts/cec_vision_unify_evidence.py, scripts/checklist.sh, scripts/cec_corpus_lint.py, FOLLOWUPS.md, README.md, the Ground-Truth-Spec.md. Use grep -rn for the keywords across the repo.

Extract the ACTUAL checklist items and integrity rules VERBATIM (copy exact text into verbatim_excerpts), the source files, what the mechanism is, and how it is enforced (hook? CI? script? manual discipline?). If there are multiple related artifacts, capture all of them. Return the structured object. Set found=false only if you truly cannot locate anything resembling an evidence-integrity policy or research checklist after a thorough search.`, { label: 'recon:evidence-checklist', schema: evidenceSchema }),

  () => agent(`${RO}

Explain the "INVERTED GROUND TRUTH CORPUS" approach used in the CEC project. The phrase "inverted ground truth" appears in /home/nathan/CEC-Platform/scripts/cec_reasoning_bakeoff.py, /home/nathan/CEC-Platform/tests/eval/bakeoff/tasks-workload-v1.json, and /home/nathan/CEC-Platform/ops/README-claude-rc.md. Read those (grep -n "inverted" and surrounding context). Also read how the corpus works in /home/nathan/CEC_AutoDiagnoser (crates/corpus-client/src/*.rs, README sections "Open engine, private corpus", "The flywheel: persistent corpus and retrieval-first", "De-identification by structured extraction") and CEC-Platform corpus/SCHEMA.md.

I need a crisp, correct explanation of: what "inverted ground truth corpus" MEANS in this project (is the corpus itself the ground truth that's built bottom-up from verified outcomes, rather than a top-down labeled gold set? does "inverted" mean outcomes-define-truth, or hard-negatives-as-truth, or something else?), how it works mechanically, how it contrasts with a conventional/top-down ground-truth set, and — critically — what it IMPLIES for an evidence-integrity and research checklist (e.g. since truth is accreted from signed-off outcomes, integrity of each accreted row matters more; provenance of each claim; no leakage; sign-off gating as the truth-admission boundary). Quote verbatim the lines that define the approach. Return the structured object.`, { label: 'recon:inverted-ground-truth', schema: invertedSchema }),

  () => agent(`${RO}

Document /home/nathan/CEC-Platform's "WSL-EPHEMERAL STATE POLICY" in full mechanical detail, then produce a precise PARITY SPEC to replicate it in /home/nathan/CEC_AutoDiagnoser (which is a git repo, remote https://github.com/nathanfraske/cec-support-agent.git, default branch main).

Read in full: .claude/settings.json, .claude/hooks/session-start.sh, .claude/hooks/session-end.sh, .claude/hooks/followups-context.sh, .claude/hooks/todo-context.sh, .claude/hooks/v4-idle-queue.sh, ops/secrets/* (load-secrets.sh — note structure, do NOT print secret VALUES), ops/README-claude-rc.md, ops/claude-session.sh, ops/provision.sh, ops/rc-recover.sh, ops/claude-rc@.service, ops/claude-rc-tmux.sh, docker/.wslconfig.example. Also note the .claude/memory mirror dir and how it relates to ~/.claude/projects/<sanitized>/memory, and the ops/agent-handoff branch mechanism (git plumbing: hash-object/read-tree/write-tree/commit-tree, push to a side ref, never touching HEAD/index).

The durability contract: live ~/.claude state is DISPOSABLE; the durable copies are (1) the in-tree .claude/memory mirror committed normally, and (2) the off-tree ops/agent-handoff branch pushed at Stop. Explain both halves and how session-start re-seeds after a WSL wipe.

Then give parity_steps_for_target_repo: the exact list of hooks/scripts/branches to create in cec-support-agent so it survives a WSL wipe identically. Note adaptations_needed (e.g. cec-support-agent has no ops/secrets yet, no broker on a fixed port necessarily, different memory project dir -home-nathan-CEC-AutoDiagnoser). Return the structured object.`, { label: 'recon:wsl-ephemeral', schema: wslSchema }),

  () => agent(`${RO}

Document the CURRENT state of the infrastructure for running LOCAL AGENTS / LOCAL INFERENCE on this machine. The owner says this infrastructure has CHANGED recently and needs fresh documentation. Investigate across repos and the live system.

Read: /home/nathan/CEC-Platform/ops/README-claude-rc.md, ops/cec-llm-broker/*, ops/windows-serving/*, ops/claude-rc@.service, ops/claude-session.sh, ops/provision.sh, ops/claude-rc-tmux.sh, versions.env, docker/* (compose.yaml, Dockerfile.routing, README.md, xvfb-entrypoint.sh, .wslconfig.example), docs/local-compute-exploration.md, docs/local-compute-windows-native-migration.md, docs/self-hosted-router.md, docs/ai-box-upgrade-analysis-2026-06-12.md, docs/llm-manager-bench-2026-06-09.md. Also inspect the sibling repo /home/nathan/cec-llm-broker (ls + README + key scripts) and /home/nathan/cec-runs.

Probe the live machine (read-only, best-effort): \`systemctl --user list-units 2>/dev/null | grep -iE 'claude|broker|llm|cec' \`; \`ps aux | grep -iE 'broker|llama|vllm|ollama|lm-studio|tgi' | grep -v grep\`; \`ss -tlnp 2>/dev/null | grep -iE '8080|8000|11434|1234|5000' \`; \`ls -la /home/nathan/cec-llm-broker /home/nathan/cec-runs 2>/dev/null\`; \`tail -40 /home/nathan/CEC-Platform/versions.env\`; check for any Windows-native serving notes. Determine: what serves models now (Windows-native vs WSL vs docker?), what the broker is and its port, how seats/routing work, how an agent/Claude session reaches local compute, and explicitly WHAT CHANGED vs the older setup (compare docs dated earlier vs later, and the windows-native-migration doc). Return the structured object with files_and_scripts listing every relevant path, and open_questions for anything you could not determine from disk.`, { label: 'recon:local-agent-infra', schema: infraSchema }),
])

return { pipeline, evidence, inverted, wsl, infra }
