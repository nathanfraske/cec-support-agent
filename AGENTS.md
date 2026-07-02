# Working rules for this repo
- Preserve the invariants in section 0 of the bootstrap doc.
- Never add corpus data, fixtures derived from it, or model weights.
- Keep the engine cold-startable: no CEC service required to build, test, or run.
- All model access goes through crates/inference over HTTP. Do not hardwire a provider.
- corpus-client must reject any contribution that is not sign-off confirmed.
- Run cargo fmt and clippy -D warnings and the test suite before every commit.

## Agent operations, durability & evidence integrity

This repo runs under WSL2 with the agent-ops layer in `.claude/` and `docs/`. Before doing significant work:

- **Read `HANDOFFS.md` first** — the cross-agent baton (current state, exact next step, append-only lessons).
  It is injected at every session start. `TODOS.md` is the live work-now checklist; `FOLLOWUPS.md` is the
  deferred backlog. All three are **append-only with tombstones** (UTC date+time; never delete a line — flip
  `[ ]`→`[x]`/`[~]` and append a tombstone). Update them in the **same turn** as the work.
- **WSL is disposable.** The durability contract (`docs/wsl-ephemeral-state-policy.md`) keeps load-bearing
  state on the git remote: an in-tree `.claude/memory/` mirror on `main` + an off-tree `ops/agent-handoff`
  branch pushed every session Stop. The hooks are wired in `.claude/settings.json`.
- **Evidence integrity** for the inverted-ground-truth corpus is specified in
  `docs/evidence-integrity-and-research-checklist.md` (the runnable checklist an agent ticks before any
  corpus write-back / before claiming a result is true). The research-discipline tree is `docs/research/`.
- **Local inference** runs through the `cec-llm-broker` on `:8080` — see `docs/local-agent-infrastructure.md`.

## Per-endpoint egress-sink checklist (binding)

Every new `serve` response type is a new egress sink. Satisfy ALL of this in the **same PR** that adds it
(binding policy, from `docs/api-extension-design.md` §2.5):

1. **Vocabulary-only bodies.** Emit only pinned enum tokens, validated slugs (`run_id`/`plan_id`), a hashed
   class, a stored/minted type, or an integer. Never `Prose` (title/description/rationale/message/summary),
   never model output, never a tool-output `Value`, never a transcript, never a path/URL.
2. **Port the poison contract test.** Plant `leakguard::POISON` into every input and assert no token survives
   the new body. No such test, no merge.
   - **2b. Assert structurally, not by substring.** Where the field is a symptom, assert closed-grammar
     membership (de-id is a transformation, not a deletion); reuse `leakguard::assert_no_poison`.
3. **Errors are tokens, never `Display`.** Return a fixed reason token, never `{e:#}` — a `Display` of a
   `GateError`/`anyhow::Error` can carry a served-plan fragment or a path.
4. **No prose in logs either.** Handler `eprintln!` diagnostics format fixed strings + error *categories*
   only — never request bodies or served rows.
5. **Attest any corpus row crossing the wire.** Ship attested rows and re-verify on receipt; never serve
   unattested aggregates.
6. **Never let the wire lower a gate without a signature.** Consent, sign-off, and sandbox evidence that
   *reduce* escalation must be cryptographically backed, else the conservative default holds.

## Non-mappability (corpus cartography, leak-C10) (binding)

A trusted, rostered caller can still map the corpus's membership/coverage/structure by aggregating many
individually-legitimate queries. Satisfy ALL of this in the **same PR** that touches a corpus-facing surface
(binding policy, from `docs/corpus-cartography-threat.md` §3b; taxonomy: `docs/corpus-leak-prevention.md`
§1.2 leak-C10):

1. **One answer per call.** No endpoint returns more than a single diagnosis's worth of corpus rows. There
   is no list / enumerate / dump / range / bulk corpus read — ever. Adding a corpus read route is a
   reportable security issue.
2. **No gratuitous membership differential.** A new response field must not add a hit-vs-miss signal
   (presence, count, a `source` label, ordering, or a score that betrays the retrieval prior) beyond what
   delivering the single answer requires. Not needed to *use* the answer ⇒ it does not ship.
3. **No behavioural oracle.** A corpus hit must not be inferable from latency, error shape, or slate size.
   Equalize timing or gate the fast path behind an identity budget.
4. **Minimal attested unit.** A served corpus row carries only what the consumer needs to verify and use it
   — never the priming graph (`primed_from`), raw confirmation counts, or derivation topology, unless a
   decision log entry explicitly authorizes it.
5. **Every corpus-touching call is attributable.** Log it to an identity (hashed key + caller id +
   timestamp; never `describe`). No identity ⇒ no off-loopback corpus read.
6. **Non-enumerable keys.** Retrieval keys (`fingerprint`, `config_class`) crossing a boundary are
   keyed/salted, never plain FNV, and never in a logged URL — so a caller cannot compute-then-probe keys.
7. **Budgeted.** Any surface a non-owner can reach carries a per-identity query budget; bulk enumeration
   must be rate-limited and visible.
