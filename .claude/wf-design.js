export const meta = {
  name: 'autodiagnoser-evidence-checklist',
  description: 'Design panel (4 diverse lenses + adversary) → synthesize the evidence-integrity & research checklist adapted to the inverted-ground-truth corpus; author the local-agent-infra + WSL-policy docs in parallel',
  phases: [{ title: 'Design' }, { title: 'Author' }],
}

// Shared context: the recon findings are on disk as JSON the agents should read.
const RECON = '/home/nathan/CEC_AutoDiagnoser/.claude/recon'
const REPO = '/home/nathan/CEC_AutoDiagnoser'
const CTX = `CONTEXT YOU MUST READ FIRST (structured recon already done — do not re-derive):
- ${RECON}/evidence.json — CEC-Platform's EVIDENCE-INTEGRITY policy (EI-01..EI-08) + RESEARCH CHECKLIST (paper-track PP-01..PP-13): two-zone corpus custody, taint pins, monotone-tightening law EI-07, signature gate EI-08, source revocation EI-06, claims/prereg/negative-results/instrumentation discipline. Verbatim excerpts included.
- ${RECON}/inverted.json — the INVERTED GROUND-TRUTH CORPUS model: truth is accreted bottom-up from signed-off, verified outcomes (the sign-off gate is the truth-admission boundary); hard negatives are first-class truth; de-identification by structured extraction; retrieval-first flywheel; per-row integrity is load-bearing. Includes 9 implications_for_evidence_integrity.
- ${RECON}/pipeline.json — the cec-support-agent pipeline (8 stages), the 11 existing evidence_integrity_points (with file:line), the 8 where_checklist_would_hook points, and the 11 gaps.
You MAY also read the actual Rust source under ${REPO}/crates/*/src/*.rs and ${REPO}/README.md to ground claims in real code (cite file:line).
This repo (cec-support-agent) is the OPEN ENGINE; the corpus + weights are PRIVATE and elsewhere. It ships only the corpus client + schema. Its truth is the inverted corpus: (FaultSignature, Plan, OutcomeLabel) triples earned at the sign-off gate.`

const PROPOSAL_SCHEMA = {
  type: 'object',
  additionalProperties: true,
  properties: {
    lens: { type: 'string' },
    thesis: { type: 'string' },
    adaptations: {
      type: 'array',
      items: {
        type: 'object',
        additionalProperties: true,
        properties: {
          cec_platform_mechanism: { type: 'string' },
          autodiagnoser_analog: { type: 'string' },
          code_hook_point: { type: 'string' },
          enforced_now_or_gap: { type: 'string' },
          checklist_item: { type: 'string' },
        },
        required: ['cec_platform_mechanism', 'autodiagnoser_analog', 'checklist_item'],
      },
    },
    must_have_checklist_items: { type: 'array', items: { type: 'string' } },
    what_to_drop_or_change_vs_cec_platform: { type: 'array', items: { type: 'string' } },
    deferred_followups: { type: 'array', items: { type: 'string' } },
    open_risks: { type: 'array', items: { type: 'string' } },
  },
  required: ['lens', 'thesis', 'adaptations', 'must_have_checklist_items'],
}

phase('Design')

const proposals = await parallel([
  () => agent(`${CTX}

YOUR LENS: RUNTIME, CODE-ENFORCED INTEGRITY. Map CEC-Platform's EI-01..EI-08 to concrete cec-support-agent mechanisms IN CODE. The recon found the load-bearing gap: there is NO unified evidence-integrity checkpoint — the sign-off gate (corpus-client/src/gate.rs ensure_signed_off) checks ONLY sign_off.is_confirmed(), and SignOff is a caller-asserted enum with no proof behind it; the verification verdict, provenance attestation, de-identification proof, and config-class are never jointly bound to the row being written. Design the adaptation so that the inverted corpus admits a row ONLY through one checkpoint that binds: (a) a real verification Verdict (not bootstrap-trivial), (b) a provenance/judge-signature attestation, (c) a de-identification proof (leakage suite green + structural), (d) a present, honestly-derived ConfigClass, (e) sign-off level matching the consent/escalation that authorized execution. For EACH EI-01..08 give the AutoDiagnoser analog, the exact code hook point (file:line from pipeline.json), enforced-now-vs-gap, and a tickable checklist item. Be concrete and Rust-accurate. Return the structured proposal.`, { label: 'design:runtime-code', phase: 'Design', schema: PROPOSAL_SCHEMA }),

  () => agent(`${CTX}

YOUR LENS: RESEARCH / PAPER-TRACK DISCIPLINE. Adapt CEC-Platform's research checklist (PP-01..PP-13: at-most-two falsifiable claims each with a named kill experiment and [CITE NEEDED] gaps you must NOT invent; preregistration commit-timestamped BEFORE the data exists; negative-results-first to discipline claims; the no-orphan instrumentation inventory where 'zero rows may read hope to compute later'; commit-timestamp ordering as the honesty guarantee; the dark-seat/QUORUM-not-FULL honesty rule for a seat reading an unsigned corpus) to cec-support-agent AS A RESEARCH ARTIFACT. The engine's real research claims are things like: retrieval-first from a signed-off corpus beats cold generation; the sign-off gate prevents corpus poison; de-identification-by-extraction yields zero leakage; hard-negative quarantine prevents re-offering known-bad fixes. Define: the exact research-discipline files this repo should carry (e.g. docs/research/{claims,prereg,negative-results,instrumentation-inventory,README}.md) and what each enforces, adapted to the inverted corpus (where 'gold is computed, not authored' and self-evaluation is the central threat). Give tickable checklist items. Return the structured proposal.`, { label: 'design:research-track', phase: 'Design', schema: PROPOSAL_SCHEMA }),

  () => agent(`${CTX}

YOUR LENS: GOVERNANCE, CUSTODY & THE ACTUAL RUNNABLE CHECKLIST. CEC-Platform uses two-zone custody (staging = agent-writable, ADVISORY-only, can never block; promoted = human-only behind CODEOWNERS + branch protection, the only blocking zone) and an owner-only signature gate the bot account cannot self-approve. cec-support-agent's analog of the two zones is the SignOff ladder (Unconfirmed / VerifierConfirmed / HumanConfirmed) and the route-driven escalation (hardware/ambiguous ⇒ human). Design: the custody model (which writes are advisory vs truth-admitting; verifier vs human as the boundary; what an agent on a machine account may and may not do to the corpus), the truth-admission boundary, and — most important — THE ACTUAL ORDERED, TICKABLE CHECKLIST an agent or contributor runs (1) before any corpus write-back, and (2) before claiming a result/finding is true. Make it concrete enough to literally tick. Specify how it is ENFORCED (runtime gate + CI/test invariant + the 'a checklist item with no adversarial test silently regresses' rule), and how the FOLLOWUPS/TODOS/HANDOFFS files plug in. Return the structured proposal.`, { label: 'design:governance-checklist', phase: 'Design', schema: PROPOSAL_SCHEMA }),

  () => agent(`${CTX}

YOUR LENS: ADVERSARIAL RED-TEAM. Enumerate the concrete ways the INVERTED corpus (truth accreted from signed-off outcomes, read retrieval-first) can be POISONED, LEAKED, or CONTAMINATED, and for each, the single checklist item that catches it. Draw on inverted.json's 9 implications and pipeline.json's 11 gaps. Cover at minimum: a caller constructing Contribution{ sign_off: HumanConfirmed } to bypass the gate (gap: sign-off is caller-asserted, not proven); the bootstrap re-deriving the post-signature from the same request text so any run trivially labels ResolvedConfirmed (gap: verification can't observe a real fix); a verdict never bound into the row so 'resolved' can't be audited; cross-ConfigClass retrieval laundering a fact into an unverified context; a hand-edited FileCorpus JSONL row served later as authoritative (no per-row tamper-evidence); model output entering a plan/row un-validated and un-de-identified; an eval/holdout case admitted as corpus truth (train/serve contamination); a retracted claim becoming truth (the T-104 case). Produce an attack→which-checklist-item-catches-it mapping, rank the 11 gaps as CRITICAL-NOW vs DEFERRABLE (with why), and list the must-have checklist items. Return the structured proposal.`, { label: 'design:red-team', phase: 'Design', schema: PROPOSAL_SCHEMA }),
])

const good = proposals.filter(Boolean)
log(`Design panel returned ${good.length}/4 proposals`)

phase('Author')

const proposalsJson = JSON.stringify(good)

const [checklistDoc, infraDoc, wslDoc] = await parallel([
  // 1) The centerpiece: synthesize the four proposals into ONE finished markdown document.
  () => agent(`${CTX}

The design panel produced these four proposals (runtime-code, research-track, governance-checklist, red-team):

${proposalsJson}

SYNTHESIZE them into ONE finished, publish-quality Markdown document: the EVIDENCE-INTEGRITY & RESEARCH CHECKLIST for cec-support-agent, adapted to the inverted-ground-truth corpus. Resolve conflicts; prefer the most concrete, code-accurate option; do not lose any must-have checklist item or critical gap.

Required structure (use real headings, tables where it helps, and cite file:line for code hook points):
1. Purpose & the inverted-ground-truth integrity model (why per-row integrity is the whole game; the sign-off gate as the truth-admission boundary).
2. How this is ADAPTED from CEC-Platform (a short table: CEC-Platform EI/PP mechanism → AutoDiagnoser analog → what changed/dropped and WHY — e.g. two-zone corpus custody collapses onto the SignOff ladder + route escalation; KiCad/DRC specifics dropped; the private-corpus inversion means there is no in-repo promoted/ zone to gate, so the gate is the runtime sign-off boundary + CI invariants).
3. The EVIDENCE-INTEGRITY checklist (the EI-01..EI-08 analogs as tickable '- [ ]' items, each tagged ENFORCED-NOW or GAP with the code hook point).
4. The RESEARCH checklist (the PP analogs: claims discipline, preregistration, negative-results-first, no-orphan instrumentation, commit-timestamp honesty, dark-seat/quorum — as tickable items + the docs/research/ files to carry).
5. THE RUNNABLE CHECKLIST — the ordered list an agent/contributor literally ticks (A) before any corpus write-back, (B) before claiming a finding is true. Concrete.
6. Adversarial / attack→defense table (from the red-team lens) and the rule that a checklist item with no adversarial test silently regresses.
7. Enforcement plan: runtime gate (the unified ensure_evidence_integrity() alongside gate.rs), CI/test invariants (extend the leakage suite + SECURITY.md), and how FOLLOWUPS.md / TODOS.md / HANDOFFS.md plug in.
8. Deferred items (the gaps that are real but out of scope now) — phrased so they can be copied verbatim into FOLLOWUPS.md.

Output ONLY the Markdown document as your final message — no preamble, no fences around the whole thing, do NOT write any file. I will write it to disk.`, { label: 'author:checklist-doc', phase: 'Author' }),

  // 2) Local-agent infrastructure doc (current state).
  () => agent(`You are documenting the CURRENT local-agent / local-inference infrastructure for the cec-support-agent repo's docs/. Read ${RECON}/infra.json (a fresh live-probe recon) and, if useful, /home/nathan/cec-llm-broker/README.md and /home/nathan/cec-llm-broker/models.json.

Write a publish-quality Markdown doc titled "Local-agent infrastructure" capturing the CURRENT state (the owner says it CHANGED recently and needs fresh docs). Must cover, accurately and concretely:
- The single front door: cec-llm-broker on 0.0.0.0:8080 (SYSTEM systemd unit cec-llm-broker.service), an OpenAI-compatible on-demand orchestrator/reverse proxy; how it routes by model alias, boots managed WSL docker seats on demand, arbitrates ONE RTX 5090 (30GB VRAM budget, LRU eviction, 30-min idle reap), and proxies (never manages) external Windows-native seats.
- The hybrid backends: (a) managed WSL2 docker-compose seats (vLLM + llama.cpp server-cuda, GGUFs on E:/AI Models over slow drvfs), (b) external Windows-native llama-server seats (NTFS-speed, managed:false). The 10 registered aliases + ports (table). Note what is LIVE now (deepseek-v4-flash :8007 Windows-native deep auditor) vs cold vs unreachable (cec-worker-vision-win :8090, libomp140 blocker; WSL :8012 is the live fallback).
- How an agent/pipeline reaches it: send an OpenAI request with model=alias to http://localhost:8080/v1 (host) or http://host.docker.internal:8080/v1 (in-container); fail-safe to deterministic fallback if down. For THIS repo specifically: crates/inference is the OpenAI-compatible HTTP client; point --endpoint http://localhost:8080/v1 --model <alias> (e.g. cec-worker-vision, cec-manager-fast, deepseek-v4-flash). Cold-start works with no endpoint at all.
- WHAT CHANGED recently (broker rebuilt v2 2026-06-12 after a WSL-to-E: move wiped it; Windows-native serving migration; model lineup churn — Qwen3-235B retired, MiniMax-M2.7 retired-from-CEC, default reviewer now cec-manager-fast/gpt-oss-120b, deep auditor now DeepSeek-V4-Flash-284B, default seat unified cec-worker-vision; broker now a SYSTEM systemd unit; Claude RC/session survivability rebuilt on tmux+systemd 2026-06-14).
- Operate it: systemctl/journalctl commands, where the broker source lives (/home/nathan/cec-llm-broker + the vendored CEC-Platform/ops/cec-llm-broker), and the key files.
- Open questions / known-unverified (from infra.json open_questions): the 8090 libomp140 blocker, NAT vs mirrored networking, AI-box upgrade undecided, V4 launcher on E:, etc.

Output ONLY the Markdown as your final message; do NOT write any file.`, { label: 'author:infra-doc', phase: 'Author' }),

  // 3) WSL-ephemeral state policy doc (documents what was implemented).
  () => agent(`You are writing the WSL-EPHEMERAL STATE POLICY documentation for the cec-support-agent repo's docs/. Read ${RECON}/wsl.json (the full CEC-Platform policy + the parity spec for this repo) and the ALREADY-IMPLEMENTED hooks in this repo: ${REPO}/.claude/hooks/session-start.sh, ${REPO}/.claude/hooks/session-end.sh, ${REPO}/.claude/hooks/{followups,todos,handoffs}-context.sh, ${REPO}/.claude/settings.json, ${REPO}/.claude/memory/README.md.

Write a publish-quality Markdown doc titled "WSL-ephemeral state policy" that DOCUMENTS the policy as implemented here (it is already built and verified — the Stop hook pushed branch ops/agent-handoff to the remote with main untouched). Must cover:
- The rule: the WSL2 Linux volume is DISPOSABLE; anything load-bearing must live in (1) the git remote, (2) the Windows filesystem, or (3) be rebuildable from the repo. The origin (CEC-Platform lost a session handoff in a 2026-06-12 WSL wipe).
- The durability contract's two halves AS IMPLEMENTED HERE: (HALF 1) the in-tree memory mirror at .claude/memory/ committed on main, refreshed from live ~/.claude memory by the Stop hook and re-seeded into the empty live dir by the SessionStart hook after a wipe; (HALF 2) the off-tree ops/agent-handoff branch pushed every Stop via git plumbing (hash-object→temp index→commit-tree→push a side ref) NEVER touching HEAD/index — carrying docs/agent/handoff.md + HANDOFFS.md + FOLLOWUPS.md + TODOS.md + the memory mirror.
- The tracking files: FOLLOWUPS.md (deferred, append-only with tombstones, date+time), TODOS.md (live checklist, append-only with tombstones), HANDOFFS.md (the cross-agent baton, injected at SessionStart) — and which hook maintains each.
- The two AutoDiagnoser-specific adaptations that differ from CEC-Platform and WHY: (1) the memory-dir sanitization must map '_'→'-' (tr '/._' '---', canonical fallback -home-nathan-CEC-AutoDiagnoser) because CEC_AutoDiagnoser has an underscore; (2) a pristine post-wipe clone has NO git identity, so session-end.sh exports a GIT_*_NAME/EMAIL fallback (cec-agent-handoff[bot]) or commit-tree fails 'empty ident name' and the durable push dies silently. The gh credential helper handles auth; no PAT required (a bot PAT is an optional future hardening).
- Recovery procedure (what to do after a WSL wipe: install WSL + driver, git clone, the live memory/handoff self-heals from the in-tree mirror at first session start; the latest off-tree snapshot is on the ops/agent-handoff branch).
- Verification: how to confirm it works (git ls-remote origin ops/agent-handoff; run the Stop hook; confirm main untouched).
- A short list of optional future hardening (ops/secrets bot PAT scoped to this repo; ops/provision.sh cargo-shaped; claude-rc survivability units) — phrased so they can be copied into FOLLOWUPS.md.

Output ONLY the Markdown as your final message; do NOT write any file.`, { label: 'author:wsl-doc', phase: 'Author' }),
])

return { checklistDoc, infraDoc, wslDoc, proposalCount: good.length }
