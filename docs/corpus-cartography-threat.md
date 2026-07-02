# Corpus cartography: query-oracle resistance (the fourth property)

**Status:** decision-ready, no code. **For:** Nathan (owner) + Chris. **Companion to:**
`docs/corpus-leak-prevention.md` (proposes **leak-C10** for its §1.2 taxonomy),
`docs/api-extension-design.md` (§2.5 egress-sink rules, §1.6 never-routable, §5 decision log),
`docs/trusted-corpus-access-trajectory.md` (§0 the three guarantees; B4 served-row; F3 registry),
`docs/integration-myown-family.md` (roster / `Role::Owner`).

**The owner-raised threat (2026-07-02):** *"Can a surface expose the internal corpus by mapping
it out through trusted calls?"* This is **corpus cartography** — membership-inference / oracle-
enumeration. It is a property the leak-prevention wave does **not** cover, and does not claim to:

- **De-identification** (leak Phases 0–2) stops identity leaking **out inside a served row**.
- **Attestation** (B4) stops a **forged row coming in**.
- **Encryption / roster** (E3) stops an **outsider reading the wire** or reaching the socket.

None of the three stop a *trusted, rostered* caller from **reconstructing the corpus's
membership, coverage, structure, and fix-content** by aggregating many individually-legitimate,
de-identified, attested, encrypted queries. That is a distinct, fourth property:
**non-mappability / query-oracle resistance.**

---

## 0. The honest limit, stated first

A caller you *permit to query* is, **by design**, obtaining corpus knowledge. Serving the fix for
a real diagnosis **is** disclosing one row's content — that is the product. Perfect
confidentiality against a trusted querier is impossible; "the corpus" is exactly the set of
answers the query surface exists to give. So the goal is **not** "a rostered caller learns
nothing." The goal is three achievable properties:

1. **Do not disclose structure gratuitously** — one query answers *one* diagnosis; it must not
   also hand over membership counts, the priming graph, confirmation structure, or a
   compute-then-probe handle to *other* keys.
2. **Make bulk cartography slow, costly, attributable, and detectable** — a caller enumerating
   the corpus should be rate-limited, logged to an identity, and visible after the fact.
3. **Make "rostered" mean, explicitly, "trusted with corpus knowledge"** — so that the residual
   (a determined rostered insider maps their share) is a **stated roster-policy decision**, not a
   silent bug. This is the analogue of "`--allow-remote-inference` makes remote egress an audited
   act, not a default."

The whole of §2 rests on separating **inherent product function** (serving the fix(es) for one
real diagnosis) from **exploitable cartography** (bulk enumeration, differential probing,
structure disclosure beyond the single answer). Every control below narrows the second without
breaking the first.

---

## 1. leak-C10 — the threat class (proposed taxonomy addition)

> **Proposed for `corpus-leak-prevention.md` §1.2, ranked below C9 as a distinct axis** (it is
> *not* a serialize/print/egress leak — it needs no identity to survive de-id; it is an
> aggregation property of a legitimate surface):

| Rank | Class | Why it is distinct | Representative vectors |
|------|-------|--------------------|------------------------|
| **C10** | **Corpus cartography / query-oracle enumeration** — a caller *with legitimate query access* aggregates individually-clean responses to map the private corpus's membership, coverage, structure, and fix-content | Every prior class asks *"did identity leak out of one row?"* C10 asks *"can many de-id'd, attested rows be assembled into the map of what the corpus **knows**?"* De-id/attestation/encryption are all satisfied and it still succeeds. Orthogonal to C1–C9. | `diagnose-source-membership-oracle`, `retrieval-first-latency-differential`, `candidate-slate-structure-disclosure`, `enumerable-fnv-probe-space`, `served-provenance-priming-graph`, `no-query-budget-or-audit` |

**The adversary (two concrete shapes, both *inside* the trust boundary):**

- **Rung-0 loopback:** the single operator's **compromised embedding app** (AllMyStuff). The
  socket has no auth — the trust boundary *is* the OS user (`api-extension-design.md` §2.2:
  "There is no token check anywhere in `serve.rs`"). A malicious or breached app on the box can
  issue unlimited `/v1/diagnose` calls. *(Note: the single operator who **owns** the corpus is
  not in scope — mapping your own corpus is not a threat; a compromised app acting **as** them,
  exfiltrating the map off-box, is.)*
- **Rung-2 mesh peer:** a **rostered** peer permitted to read the corpus over the mesh adapter
  (E3/`serve_corpus`, `integration-myown-family.md:109,153`). It is trusted to *read precedents*;
  it is not trusted to *reconstruct and re-host the whole corpus*. Today no distinction is drawn.

---

## 2. Concrete vectors (each verified in code)

Separating **INHERENT** (product function — the single-diagnosis answer) from **EXPLOITABLE**
(bulk/differential/structure beyond it).

### V1 — the `source: corpus_primed` membership oracle. **CURRENT. Severity: high.**
`diagnose_envelope` emits, per candidate, `"source": wire_source(&c.source)`
(`crates/support-agent/src/main.rs:1187`), where `wire_source(CorpusPrimed) => "corpus_primed"`
(`main.rs:1132-1138`). A `corpus_primed` candidate exists **iff** the corpus holds a confirmed
fix for exactly this `(fingerprint, config_class)` (`serve.rs:344-360`). So the label is a direct
**membership bit**: present ⇒ "the corpus knows this fault at this class."
- **INHERENT:** the caller needs *a* fix for their real diagnosis.
- **EXPLOITABLE:** the *label* is not needed to deliver the fix — the `actions[]` are. Emitting
  `source` gratuitously turns every diagnose into a yes/no membership probe.

### V2 — `actions[]` fix-content disclosure. **CURRENT. Severity: high (but largely inherent).**
Each candidate ships `"actions": c.plan.steps.iter().map(|s| s.action.clone())`
(`main.rs:1189`) — the full de-identified plan (the fix). On a hit this is the *entire* stored
fix content for that key.
- **INHERENT:** returning the fix for the caller's own diagnosis is the point of the product.
- **EXPLOITABLE:** combined with V4 (enumerable probe space) it lets a caller **harvest the fix
  for every key they can name**, not just the one they legitimately face. This is the core of
  cartography: `actions[]` × enumeration = the corpus's fix-content map.

### V3 — the retrieval-first hit/miss differential (latency + slate). **CURRENT. Severity: high.**
Even if V1's label were removed, the hit is still observable:
- **Latency oracle:** on a hit, `retrieval_first = true` (`serve.rs:360`) and de-novo model
  generation is **skipped** (`serve.rs:365`: `if !state.args.offline && !retrieval_first {…}`
  gates the whole `ModelGenerator` fan-out). A hit returns fast (no model round-trips); a miss
  runs the swarm and is slow. With any inference endpoint configured this is a **timing side
  channel** for membership that needs no response body at all.
- **Slate-structure oracle:** the envelope contains **one `corpus_primed` candidate per
  distinct confirmed plan** for the key (`serve.rs:344-360`), plus the echoed
  `fault.fingerprint`, `symptoms[]`, and `config_class` (`main.rs:1196-1199`). So the *count* of
  `corpus_primed` candidates discloses **how many distinct fixes** exist for that fault
  (coverage/structure), and the echoed key confirms exactly which key hit.
- **`selected` + the panel prior:** the retrieval-first likelihood prior is **0.8 for
  `CorpusPrimed` vs 0.6 otherwise** (`crates/panel/src/lib.rs:250-253`). A corpus-primed
  candidate therefore reliably becomes `selected` (`main.rs:1202`). So `selected` is a membership
  signal **even if the `source` label were stripped** — the winner's presence/behaviour betrays
  the prior.

### V4 — the enumerable probe space (leak-C7 compounds C10). **CURRENT. Severity: high.**
The fingerprint and config-class key are **unsalted FNV-1a** over sorted tokens
(`crates/common/src/hash.rs:5-19`; fault fingerprint `fault.rs:53`; `ConfigClass::DerivedHash`
`config_class.rs:41`). Two consequences for cartography:
- The caller controls `describe` and `inventory_keys` (`serve.rs:122-129`), and the symptom space
  is a **closed, frozen grammar** (leak Phase 2: `0x`-hex ∪ known-prefix ∪ frozen stop-code /
  module dictionaries). The same closed grammar that gives de-id its guarantee **bounds and
  enumerates the probe space** — a caller can systematically walk the finite symptom dictionary
  and harvest V2 for each key.
- For the *planned* `POST /v1/corpus/query` endpoint (`api-extension-design.md` §1.1, which takes
  `fingerprint + config_class` directly) the unsalted hash is worse: a caller can **compute keys
  offline** and probe arbitrary `(fingerprint, class)` pairs without even constructing a
  `describe` that extracts to them. leak-C7 (`corpus-leak-prevention.md:71`, §3.1(2):357-360) is
  the exact reason the probe space is *enumerable rather than opaque*.

### V5 — the `confirmations` count = confirmation-structure. **CURRENT: not on the wire. PLANNED: exposed.**
`FixMapping.confirmations` (`crates/corpus-client/src/schema.rs:87`) is the net independent-
confirmation count. Today it is used only to build a candidate *rationale* string
(`serve.rs:355`) which is **not serialized** into the envelope (`diagnose_envelope` emits only
`plan_id/source/max_risk/actions` — verified `main.rs:1182-1192`). So **the count is not
currently disclosed** — good. But the **planned** corpus-over-API response ships
`mappings:[{… confirmations …}]` explicitly (`api-extension-design.md:37-38`). Exposing it would
disclose the corpus's confirmation structure (which fixes are battle-tested vs one-off) — a
richer map than membership alone. **Flag for the §1.1 wire contract, do not add it.**

### V6 — the B4 served-row `RowProvenance` = the priming graph. **PLANNED (B4). Severity: high at design time.**
`RowProvenance` carries `run_id`, `retrieval_first`, and **`primed_from`** — the plan-ids of the
precedents that primed a run (`schema.rs:147-161`). Today it is **stored, never served**: at
execute it is written to the row (`serve.rs:487-491`, from `session.primed_from` `serve.rs:451`)
and is **not** in the execute envelope (`serve.rs:604-616` — verified absent). But B4 proposes
serving essentially *the whole `Contribution` minus `integrity`* — including `RowProvenance` — so
the consumer can run `ensure_attested` (`trusted-corpus-access-trajectory.md:105-110`). Shipping
`primed_from` on the read wire exposes the **priming graph**: which fixes were derived from which,
i.e. the corpus's internal derivation topology. That is structure far beyond any single answer.
- **This is the concern the task calls "Q6."** *Verified: there is no "Q6" defined anywhere in
  the tree (the RFC has Q1–Q5; grep across `docs/` and the repo finds none).* The substantive
  issue is real and lives in `RowProvenance` + the B4 served-row type; I recommend it be **filed
  as a real question (call it Q6: "how much provenance does a served row expose?")** against the
  B4 wire contract, resolved by **provenance-minimization** (§3, C6 below).

### V7 — volume / rate / attribution: no budget, no log, no identity. **CURRENT. Severity: high (this is what makes bulk cartography free).**
- **No query budget / rate limit.** `MAX_SESSIONS = 256` (`serve.rs:92`) is a **pending-session
  memory bound**, not a throughput knob (its own doc-comment says so, `serve.rs:90-92`); it
  rejects only when 256 diagnoses are simultaneously *un-executed* (`serve.rs:434-438`). A caller
  that diagnoses sequentially (or lets sessions TTL-expire) issues **unlimited** queries. There
  is no per-caller query budget anywhere.
- **No audit log.** `handle_diagnose` logs nothing about the query itself; `serve.rs` `eprintln!`
  only on *errors* (`serve.rs:338,394,397`). There is no record of *which caller asked what*, so
  bulk enumeration is **neither attributable nor detectable after the fact**. (This is the read
  half of the deferred FOLLOWUPS item **MH-1: "an audit log of attestations — which authority
  signed which row, when."** — cartography needs the *query-side* twin of that.)
- **No caller identity at all on rung 0** (`api-extension-design.md` §2.2). So even a log would
  have nothing to attribute to until the mesh identity of rung 2 exists.

### V-none — confirmed *absent* (invariants to keep)
- **No list / enumerate / bulk / dump endpoint exists.** The route surface is frozen to
  `GET /v1/health`, `POST /v1/diagnose`, `POST /v1/execute` (`serve.rs:262-268`) and pinned by
  `router_surface_is_frozen` (`serve.rs:813-838`). A single diagnose returns *one diagnosis's*
  slate — though note "one diagnosis's worth" is **N `corpus_primed` candidates** (all distinct
  fixes for that key), which is inherent, not a bulk endpoint. **Keep this invariant; make
  "no bulk corpus read" an explicit non-mappability rule (§3b).**

---

## 3. Enforcement design (per control: what it stops · cost · where it lands · gate)

Cost key: **S** = cheap now (<½ day) · **M** = 1–3 d / design-level · **L** = week+ / architectural.
"Greenlight" = pure-engine, no Chris. "Design" = needs a wire/contract decision. "Chris/Q1" = roster/identity policy.

| # | Control | Stops (vectors) | Cost | Lands in | Gate |
|---|---------|-----------------|------|----------|------|
| **A** | **Per-identity query budget / rate-limit** on any corpus-touching call | V3, V4, V7 (bulk enumeration) — turns "free map" into "slow, throttled map" | M | `serve` middleware (rung-2) + `serve_corpus` | needs an identity ⇒ **rung-2 / E3**; a coarse per-process cap is greenlightable now |
| **B** | **Per-identity query AUDIT LOG** (who asked which `(fingerprint,class)`, when) | V7 — makes cartography **attributable + detectable** after the fact | S–M | `serve`/`serve_corpus`; the query-side twin of FOLLOWUPS **MH-1** | log the *hashed key + caller id + timestamp* only (never `describe`); rung-2 for a real id |
| **C** | **Provenance-graph minimization** for B4 / "Q6" — serve the **minimal attested unit** | V6, V5 | S at design time (a field choice, cheap if made *before* B4 ships) | the B4 served-row type (`trusted-corpus-access-trajectory.md` §2.1) | **Design** — decide in the B4 wire contract |
| **D** | **Minimize differential signal** — drop/gate `source`; equalize hit/miss latency & slate | V1, V3 | M | `diagnose_envelope` (`main.rs:1173`) + the retrieval-first branch | **Design** — see the `source` argument below |
| **E** | **Keyed/salted HMAC fingerprint + class** (leak-C7) — make the probe space **non-enumerable** | V4 | M | `hash.rs` (per-deployment salt); keys out of GET URLs (`store.rs:434-439`) | Greenlight (it is leak §3.1(2) / F-track anyway) |
| **F** | **No list/enumerate/bulk endpoint** — hold the frozen router; make it an *invariant* | V-none (prevents the worst case) | S (it is already true) | `router_surface_is_frozen` + a new non-mappability rule (§3b) | Greenlight |
| **G** | **Roster-is-trust clarification** — explicit policy that a rostered reader is trusted with corpus **knowledge**, and bounds it | the residual (a rostered insider maps their share) | S (doc) | `integration-myown-family.md` roster policy + decision log | **Chris / Q1** (owner policy) |

### The load-bearing argument: should `source` be on the wire at all? (control D)
**Recommendation: gate it, do not emit it by default.** Walk the consumers:
- **AllMyStuff** consumes *diagnoses*, not provenance (`api-extension-design.md:35`: "**not**
  AllMyStuff — it consumes diagnoses, not raw corpus rows"). It renders a plan and a consent UI.
  It does **not** need to know *whether the plan came from the corpus vs a cold model* — the
  `actions[]`, `max_risk`, `consent_required`, and `escalation` fully drive its UI. So for the
  primary rung-0 consumer, `source` is **gratuitous structure disclosure** (V1).
- The one place provenance *is* load-bearing is the engine's own **panel prior** (0.8 vs 0.6,
  `panel/src/lib.rs:250-253`) and the **execute-phase confirmation-independence** accounting
  (`primed_from`, `fix_mappings`) — both of which are **server-side** and do not require the
  label on the *wire*.
- **Therefore:** remove `source` from the default `cec-diagnose/v1` candidate body, or gate it
  behind an explicit rung-2 provenance scope. This kills V1 outright and (with the latency/slate
  work) collapses V3 to inference-from-behaviour only. *Caveat:* V3's latency and slate-count
  differentials survive removing the label — so D is only complete if the retrieval-first branch
  is also made **timing-equalized** (e.g. always run *some* generation, or add jitter/floor) and
  the slate size is capped. Equalizing latency has a real cost (you lose the retrieval-first
  speed win); this is a **genuine trade-off for Nathan** — see the final question.

### Greenlightable now vs design-level vs Chris-gated
- **Greenlight now (pure-engine):** **E** (keyed HMAC — already an F-track item), **F** (hold the
  invariant + add the rule), a coarse **A** per-process query cap, and the **B** log skeleton
  (hashed-key + timestamp; identity fills in at rung-2). **C** costs nothing if decided *before*
  B4 ships — so **decide C now** even though B4 is later.
- **Design-level (needs a wire decision, not new crypto):** **D** (the `source`/latency
  decision), **C** (the B4 provenance-minimization contract).
- **Chris / Q1-gated:** **A**/**B** in their *full* per-identity form (need the mesh identity),
  and **G** (roster-is-trust is a policy call about who is trusted with what).

---

## 3b. Proposed NON-MAPPABILITY rule set (for `AGENTS.md`, binding — analogue of the §2.5 egress-sink checklist)

> Add beside the existing "Per-endpoint egress-sink checklist" (`AGENTS.md:25-44`). Short,
> imperative, satisfiable in the same PR as any corpus-touching change:

1. **One answer per call.** No endpoint returns more than a **single diagnosis's** worth of corpus
   rows. There is **no** list / enumerate / dump / range / bulk corpus read — ever. (The frozen
   router `router_surface_is_frozen` is the mechanical guard; adding a corpus read route is a
   reportable security issue.)
2. **No gratuitous membership differential.** A new response field must **not** add a hit-vs-miss
   signal (presence, count, `source` label, ordering, or a score that betrays the retrieval
   prior) beyond what delivering the single answer requires. If the field is not needed to *use*
   the answer, it does not ship.
3. **No behavioural oracle.** A corpus hit must not be inferable from **latency, error shape, or
   slate size**. If retrieval-first changes timing, equalize it or gate the fast path behind an
   identity budget.
4. **Minimal attested unit.** A served corpus row carries only what the consumer needs to
   *verify and use* it (attested `StoredOutcome` + attestation) — **not** the priming graph
   (`primed_from`), **not** raw confirmation counts, **not** internal derivation topology, unless
   a decision log entry explicitly authorizes it.
5. **Every corpus-touching call is attributable.** It is logged to an identity (hashed key +
   caller id + timestamp; never `describe`). No identity ⇒ no off-loopback corpus read.
6. **Non-enumerable keys.** Retrieval keys (`fingerprint`, `config_class`) crossing a boundary are
   **keyed/salted**, never plain FNV, and never in a logged URL — so a caller cannot
   compute-then-probe arbitrary keys.
7. **Budgeted.** Any surface a non-owner can reach carries a per-identity query budget; bulk
   enumeration must be rate-limited and visible.

---

## 4. What NOT to do / accepted residual (be honest)

- **Cannot prevent: single-query product disclosure.** Returning the fix for the caller's own
  diagnosis (V2) *is* the product. We minimize *around* it (no membership label, no extra
  structure, budget+log the volume); we do not break it.
- **Cannot prevent: a determined rostered insider mapping their share.** A caller with legitimate,
  budgeted, logged read access can, over time and within budget, learn the slice of the corpus
  their queries touch. Control **G** makes this an **explicit roster-policy decision** ("rostered =
  trusted with corpus knowledge, bounded by budget + audit"), not a silent hole. This is the
  direct analogue of the accepted-risk framing the leak doc uses for inference egress
  (`corpus-leak-prevention.md` §6).
- **Do not over-engineer.** Avoid: differential-privacy noise on plans (a fix must be exact to be
  usable — noising `actions[]` breaks the product); k-anonymity thresholds on retrieval (starves a
  cold/small corpus of its few precedents — the opposite of the retrieval-first goal); a
  cryptographic PIR / oblivious-query layer (enormous cost for a threat whose honest floor is
  "the caller learns their own answers"). The right posture is **cheap friction + attribution**,
  not an unbreakable oracle — because an unbreakable query oracle that still answers queries is a
  contradiction.
- **Do not** treat leak-C10 as closable by the type system. Unlike C1/C3/C4, there is **no
  `DeIdentified<T>` analogue** — the rows are *already* clean; cartography is an aggregation
  property, enforced by budget/log/roster/differential-minimization, never by a compile error.
  State this in the taxonomy so no one claims a newtype "solves" it.

---

## 5. Sequence — where these slot into the plan

Mapping onto `trusted-corpus-access-trajectory.md` §4 (F2→F3→B4→F1→E3) and
`api-extension-design.md` §4 (Phase B):

**Harden the CURRENT diagnose surface now (rung-0, pre-consumer — cheap, no Chris):**
- **D (partial): remove/gate `source`** from `cec-diagnose/v1` candidates — **S–M, do before E1**
  (AllMyStuff codes against the wire; strip the gratuitous label before a consumer hardens against
  it, same logic as pinning the enum grammar in B2). Decide the latency-equalization trade-off
  with Nathan.
- **E: keyed/salted HMAC** — already the leak-C7 / F-track item; pull it forward, it is the
  non-mappability prerequisite that makes the probe space opaque.
- **F: add the §3b rule set to `AGENTS.md`** — S, now. Makes non-mappability binding the way §2.5
  made egress-sink binding.
- **B: audit-log skeleton** (hashed key + timestamp) — S, now; the query-side twin of MH-1.

**Prerequisites for E3 mesh serving (gate the serve-to-peers surface):**
- **C: provenance-minimization is a B4 prerequisite** — decide it *in* the B4 served-row wire
  contract (before B4 ships), so `primed_from` / raw `confirmations` never reach the read wire
  (resolve the proposed **Q6**).
- **A + B (full per-identity form)** are **E3/rung-2** — they need the mesh `Identity`
  (`serve_corpus` read=roster / write=`Role::Owner`). A rostered peer read path **must not** ship
  without a per-identity query budget and an attributable log; make both **acceptance criteria for
  E3**, alongside the existing "attested rows only / encrypted transport."
- **G: roster-is-trust policy** — **Q1/Chris**, resolved as part of "who is an owner"; it is the
  policy that says a rostered reader is trusted with (budgeted, logged) corpus knowledge.

**Net:** D + E + F + B-skeleton harden the diagnose surface **now**; C is a **B4 design
precondition**; A + B-full + G are **E3 acceptance criteria** gated on rung-2 identity (Q1/Chris).
Non-mappability becomes the **fourth guarantee** stacked onto §0's admissibility + authenticity +
confidentiality — required before the corpus is served to any peer.
