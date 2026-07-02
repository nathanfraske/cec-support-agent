# Extending the engine API: axes, contracts, and what each extension means

**Status:** decision-ready, no code. **For:** Nathan (owner) + Chris. **Companion to:**
`docs/consolidated-work-plan.md` §3 (serve phase, B1–B5), `docs/integration-rfc-for-chris.md` (D2, Q1–Q5),
`docs/corpus-leak-prevention.md` (egress-sink methodology), `docs/integration-myown-family.md` (P1'–P4).

The engine now has an API face — `cec-support-agent serve`, a loopback HTTP service:
`POST /v1/diagnose` (`cec-diagnose/v1`), `POST /v1/execute` (two-phase consent → `cec-execute/v1`),
`GET /v1/health`, loopback-bound by default (`serve.rs:216-220`). This document evaluates *where the
API may grow next*, what each extension costs on the two axes that actually bind us — the **de-id egress
boundary** and the **additive-only wire contract (D2)** — and what we must refuse. It extends §9's
numbering (Phase B); it does not restate the pipeline.

**The one constraint that shapes everything:** every new response type is a **new egress sink** — the
exact class of bug that produced the P0 D1 leak (raw `--describe` echoed via `candidates[].rationale`).
De-id is now a *type invariant* (leak Phases 0–2 done: `StoredPlan`/`StoredSymptom` are the only
serializable corpus payloads; served rows re-validate on deserialize, `stored.rs:39-135`) — but that
invariant only covers values that flow through the stored/minted types. A hand-built `serde_json::json!`
response (how `serve.rs` builds *every* body today, e.g. `serve.rs:552-564`) bypasses the type system
entirely. So each endpoint below carries a de-id analysis, and §2.5 states the standing rule set.

---

## 1. Extension axes and candidate endpoints

Each candidate: **purpose · consumer · request/response sketch · de-id analysis · phase/gate mapping.**
Sketches respect the pinned snake_case enum grammar (RFC lines 114-120) and D2 additive-only evolution.

### 1.1 Corpus read/query + submit over the API — the W8 successor

- **Purpose.** Serve the private corpus over the engine's own API so a mesh peer / another engine instance
  can retrieve precedents and (owner-gated) submit outcomes. This is W8 ("corpus service") realized
  *privately*, superseding the public-endpoint idea (work-plan §8 H3: "W8 is superseded by B3/E3").
- **Consumer.** MyOwnMesh (corpus serving over `myownmesh-core` RPC — but an HTTP shape is the same
  contract); a second engine instance; **not** AllMyStuff (it consumes diagnoses, not raw corpus rows).
- **Sketch.**
  - `POST /v1/corpus/query` → `{fault:{fingerprint,symptoms[]}, config_class}` →
    `{schema_version:"cec-corpus/v1", mappings:[{plan:{plan_id,actions[],max_risk}, confirmations,
    label, verification}], anchor:{head,count}}`. Note the request already carries a de-identified
    signature + hashed class — the same shape `CorpusStore::query` takes internally (`store.rs:181-188`).
  - `POST /v1/corpus/submit` → an **attested** `cec-corpus/v1` row (the `StoredOutcome` +
    `SignOffAttestation`) → `{accepted:bool, reason?}`. Owner-authority-gated.
- **De-id analysis.** *This is the one candidate where the type invariant already does the heavy lifting.*
  The response body is `StoredPlan`/`StoredSymptom`/`StoredOutcome`, which are de-identified by
  construction and now **re-validating on deserialize** (`#[serde(try_from)]`, `stored.rs:40,75,110`).
  So a corpus-over-API endpoint that serializes stored types *inherits* the read-side guard: an
  out-of-vocabulary action or identity-shaped symptom fails to serialize/deserialize on the wire, exactly
  as it does for `FileCorpus::open`/`HttpCorpus::query`. **New guard needed:** the *read* wire must carry
  per-row attestations so the consumer runs `ensure_attested` itself — today `HttpCorpus::query`
  (`store.rs:425-453`) trusts the server whole (the finding in FOLLOWUPS, "attestation on the READ
  wire"; B4). A query-response envelope that ships `FixMapping` *aggregates* (which carry no attestation)
  reintroduces C4 read-side trust. Ship attested rows, not aggregates.
- **Phase/gate.** Depends on **B4** (HttpCorpus read hardening) and **leak Phase 2** (done). The *submit*
  side is **P3/E3** (`MeshCorpus`, `serve_corpus` gated read=roster/write=`Role::Owner`) and **Q1-gated**
  (multi-owner authority needs the key registry, F3). Size **L**. Do not build the write side before Q1
  is answered — a single-pubkey gate cannot represent a mesh of owners.

### 1.2 Intake interview over the API — from single-shot to a session flow

- **Purpose.** Today `/v1/diagnose` is **headless**: `handle_diagnose` builds the case from the description
  alone (`Interview::new(&request.describe).into_case()`, `serve.rs:262`). The CLI, by contrast, runs an
  interactive funnel (`next_question`/`answer`/`transcript`, `intake/src/lib.rs:287-345`) but *only when a
  terminal is attached* (`main.rs:236`). So the API consumer cannot benefit from the follow-up questions
  that sharpen a vague symptom into an established case (`Case::is_established`, `lib.rs:160-163`).
- **Consumer.** AllMyStuff (P2' consent UI could render the questions); MyOwnLLM (could answer them).
- **Sketch (session flow, mirrors the execute session model).**
  - `POST /v1/intake` → `{describe, inventory_keys?}` → `{intake_id, next_question?:{kind, prompt},
    established:bool}` where `kind` is a pinned enum (`exact_error|onset|recent_change|reproducibility|
    scope`, from `QuestionKind`, `lib.rs:172-181`) — **not** the model-sharpened `prompt` if we want to
    keep prose off the wire (see below).
  - `POST /v1/intake/{id}/answer` → `{kind, text}` → same shape; when `established` or the funnel is
    exhausted, → `{intake_id, diagnose_ready:true}`, then `/v1/diagnose` accepts `{intake_id}` in lieu of
    `describe`. Multi-turn, so TTL-bounded but *not* one-shot (unlike execute).
- **De-id analysis.** *Two leaks to reason about.* (a) The **question `prompt`** — when an endpoint is
  configured, `ModelInterviewer` sharpens the wording (`main.rs:252-254`); that text is model output and
  is *not* a stored type. Shipping it is a C3 raw-string sink. **Mitigation:** ship only the pinned
  `kind` + the *scripted* prompt (`scripted_prompt`, `lib.rs:370`, a frozen static), never the
  model-sharpened one — or accept model prose only under the same PromptPayload posture as inference
  (leak §3.1, Phase 4). (b) The **answer `text`** the caller submits is raw request prose — it is input,
  not a response, so it does not leak *out*, but it must be treated exactly like `describe`: fed to the
  case, never echoed, never stored except through the signature extractor. The `transcript`
  (`lib.rs:338-344`) is "ticket context only" and **must never enter a response** — it is verbatim prose.
- **Phase/gate.** Additive to `cec-diagnose/v1`? No — a new `cec-intake/v1` schema (a new sink → new
  poison-contract test). Not P1'-blocking (P1' codes against single-shot diagnose). Nice-to-have, **M**,
  after B6/B7. Gated on the prompt-prose decision above.

### 1.3 Execution progress / streaming — long model calls

- **Purpose.** `/v1/execute` is synchronous: it runs the whole signed-plan pipeline and returns one
  `cec-execute/v1` body (`serve.rs:524-564`). A long generation or a multi-step plan gives the caller no
  progress until completion. Streaming would surface per-step status.
- **Consumer.** AllMyStuff (a progress bar in the consent UI).
- **Transport choice — SSE vs polling vs chunked.** **Polling first.** Add `execute` an
  `{async:true}` option → `{execute_id, status:"running"}`, and `GET /v1/execute/{id}` →
  `{status:"running"|"done", steps:[{action, ok}], ...}` reusing the exact `cec-execute/v1` terminal
  body. Polling needs no new streaming framework, no long-lived connection on a loopback service, and
  degrades trivially. **SSE** (`GET /v1/execute/{id}/events`) is the eventual answer *if* per-step
  latency is high enough to matter — but it is a new content type and a new sink discipline (each event
  frame is an egress point). **Chunked** raw streaming is rejected: it invites streaming *tool output
  prose* (see §3). Decide by profiling, not by default (mirrors the daemon-mode deferral, FOLLOWUPS).
- **De-id analysis.** A progress frame must carry **only** `{action, ok}` — the same pinned vocabulary the
  terminal body already ships (`serve.rs:558-560`), never a step "summary"/"description" (those are
  `Prose`/model prose, C3). The `execute_id` is a `run_id()` slug, safe. No new type invariant covers a
  hand-built SSE frame — this is squarely §2.5 Rule territory.
- **Phase/gate.** Depends on nothing structural; gated on demonstrated latency need and on **F4** (real
  long post-fix re-collection — today `recollect_post_signature` returns `None`, so execute is fast and
  honestly halts off-Windows). Nice-to-have, **M** (polling) / **L** (SSE).

### 1.4 Inventory push — the AllMyStuff P1' seam

- **Purpose.** Feed AllMyStuff's device inventory into the config class for honest retrieval scoping.
- **Status: already built into the diagnose body — no new endpoint needed.** `DiagnoseRequest` carries
  `inventory_keys: Vec<String>` (`serve.rs:113-115`), hashed into the config class via
  `ExternalInventory` (`serve.rs:267-272`), never stored. This *is* the P1'/E1 seam over the API. A
  standalone `/v1/inventory` push endpoint (server-side inventory state) would add a stateful surface and
  a correlation handle for no benefit — **do not build it**; keep inventory per-request and stateless.
- **De-id analysis.** The app-side allowlist (`inventory_to_config_keys`, MIT side) drops
  hostname/mac/ip/serial *before* the wire; the engine re-hashes and never trusts the filter (defense on
  both sides, `integration-myown-family.md` license-resolution §). The residual is C7: the config-class
  key is an **unsalted FNV** emitted in the envelope (`config_class` field, `main.rs:1131`) —
  dictionary-reversible over a low-cardinality inventory. **New guard:** keyed/salted HMAC
  (leak §3.1(2), Phase 4 / F-track). Not blocking, but note it before the class becomes a cross-instance
  correlation tag.
- **Phase/gate.** Done (P0/B3). The HMAC hardening is Phase 4 / leak-C7.

### 1.5 Health / readiness / metrics depth

- **Purpose.** `/v1/health` returns `{status, engine, schema_versions}` (`serve.rs:228-234`) — liveness
  only. A supervising app (P2' lifecycle, B5) wants **readiness** (is the corpus loaded, is an inference
  endpoint reachable) and light **metrics** (pending sessions, corpus row count).
- **Sketch.** Extend `/v1/health` additively: `{status, engine, schema_versions, ready:bool,
  corpus:{backed:"file"|"local", rows:N}, inference:"loopback"|"none"|"remote"}`. Optional
  `GET /v1/metrics` → `{pending_sessions, sessions_max, diagnoses_total, executes_total}`.
- **De-id analysis.** Row **count** is safe; a row **sample** is not (would be a corpus egress — route it
  through §1.1's stored types if ever wanted). Session counts are integers. The one trap: do **not**
  echo the `--corpus` path or `--endpoint` URL — a filesystem path or a peer address is infra identity
  (C6/C8). Report `inference:"remote"` as a boolean-ish enum, never the URL. `serve.rs:194-197` already
  prints the path to *stderr* (operator-local) — keep it off the wire.
- **Phase/gate.** Additive within `/v1/health` (no schema bump). **S.** Useful early for B5 lifecycle.

### 1.6 Sign-off / attestation operations — argue NO

- **Candidate.** Expose `gen-signoff-key` (keypair generation) and an `attest` operation (produce a
  `HumanConfirmed` attestation) over the API.
- **Verdict: never. This is the one axis that must stay off the network entirely.** The whole
  evidence-integrity keystone (MH-1/EI-08) is the **asymmetric** split: the human/verifier authority holds
  the ed25519 *private seed*; the engine embeds only the *public key* and verifies (`HANDOFFS` lessons
  2026-06-14 21:05; the engine "holds only the public key"). An `attest` endpoint would put the seed —
  or a network-reachable oracle that signs with it — inside the serve process, collapsing the asymmetry:
  anyone who can reach the socket could mint `HumanConfirmed` rows, which is precisely the forgery the
  gate exists to refuse (`GateError::AttestationMissing/Invalid`, `gate.rs:32-39`). Even in single-operator
  self-attest mode the serve process *can* read the seed (`parse_env_authority`, `serve.rs:182`) — that is
  a local convenience for the operator's own writes, **not** a capability to be reachable by callers.
  `gen-signoff-key` is a one-time, offline, human-in-front-of-the-terminal act; there is no consumer story
  that needs it remote. **Keep both CLI-only.** The API's job is to *present verification*, never to
  *hold the authority*.
- **Phase/gate.** Anti-scope (see §3). No work item.

### 1.7 Model-tier selection passthrough

- **Purpose.** Let a caller hint fast-vs-main tier per request. Today tier is chosen *internally* by
  `is_simple_request` (`serve.rs:315`), and the endpoints/models are **server-start flags** (`Args`,
  `main.rs:48-51`) — `DiagnoseRequest` has **no** `options` field (RFC B3 sketched `options?`; it is
  **not implemented** — verified: the struct is only `{describe, inventory_keys}`, `serve.rs:108-115`).
- **Sketch.** Additive optional body field: `{... , options?:{tier?:"fast"|"quality"}}` — a *hint* that
  can only make the engine pick among **already-configured** endpoints, never supply a new URL/model
  (that would be a C2 inference-egress channel opened by an untrusted caller — refused, see §3).
- **De-id analysis.** `tier` is a closed enum → no leak. The hard line: the caller may select a tier,
  **never** an endpoint. Endpoint choice stays an operator flag so "where raw prose egresses" is an
  audited local decision (leak §3.1, `--allow-remote-inference`), not a per-request caller choice.
- **Phase/gate.** Additive to `cec-diagnose/v1`. **S.** Low priority.

### 1.8 Sandbox-validation evidence submission (Q4)

- **Purpose.** A clean apply in a disposable sandbox is positive evidence that can *lower* an escalation.
  Today the API passes **no** validator: `sandbox_validated_for(None, best, true)` (`serve.rs:358`), so
  the escalation is always conservative. Q4 asks whether a mesh peer can *be* the sandbox.
- **Sketch.** `POST /v1/execute` gains an optional `{sandbox_evidence?:{plan_id, verdict:"clean"|"failed",
  attestation}}` — an **attested** claim from a validator identity, re-checked before it may lower
  `required_escalation`. Not a free-text field: a signed verdict tag over `(plan_id, config_class,
  verdict)`.
- **De-id analysis.** The evidence is enum + signature, no prose — safe *if* typed. The real risk is
  **trust, not leak**: an unauthenticated caller asserting "clean" to downgrade a HumanConfirm to auto is
  a privilege escalation. It must be an *attested* claim (same authority model as sign-off), and absent a
  valid attestation the conservative default holds (unvalidated ⇒ escalate). Never let the wire *lower* a
  gate without cryptographic backing.
- **Phase/gate.** **Q4-gated** and depends on **F5** (a production `SandboxValidator` exists) and the key
  registry (F3, if the validator is a distinct identity). Size **M**. Defer with Q4.

---

## 2. Cross-cutting contracts

### 2.1 Versioning evolution

- **Two version scopes, kept distinct.** The **envelope** `schema_version` (`cec-diagnose/v1`,
  `cec-execute/v1`, and any `cec-corpus/v1`/`cec-intake/v1`) is *per-schema*, D2 additive-only within a
  major (RFC D2): new **optional** fields only; consumers ignore unknowns; remove/rename/retype ⇒ major
  bump and the consumer errors on an unknown major. The **path** version (`/v1/...`) is the *transport*
  major; it bumps only when the *set* of endpoints or their verbs change incompatibly, not when a body
  gains a field. A new endpoint is *not* a v2 — it is additive to v1.
- **When v2.** Only when an existing field must change meaning/type (e.g. if `escalation` ever needed a
  fourth non-additive state that breaks the pinned grammar). The pinned enum test (`main.rs:1624`) is the
  tripwire: a Rust rename fails *there*, forcing the major decision consciously.
- **Unknown-field policy, both directions.** Response→consumer: additive, consumer ignores unknown fields
  (already the AllMyStuff serde-mirror contract). Request→engine: the engine currently uses plain
  `#[derive(Deserialize)]` (`serve.rs:108,117`), which *ignores* unknown request fields — acceptable and
  forward-compatible, but **document it**: a client sending `options` today gets silent no-op, not a 400.
  If we ever want strict rejection, add `#[serde(deny_unknown_fields)]` deliberately (it is a breaking
  change for lenient clients).

### 2.2 The auth ladder

Three rungs; the engine sits on rung 0 and must climb consciously.

- **Rung 0 — loopback, no auth (today).** `validate_bind` refuses non-loopback without `--allow-remote`
  (`serve.rs:162-170`); on loopback the trust boundary *is* the OS user. There is **no token check
  anywhere** in `serve.rs` (verified). This is correct for a single-operator local service and is the
  leak-C2 posture (raw request prose crosses this surface, so exposure is deliberate).
- **Rung 1 — bearer token.** *Required the moment `--allow-remote` is used, or the socket is shared with
  a less-trusted local process.* A static token (`CEC_API_TOKEN`) checked in an axum middleware, constant
  fixed-string refusal on mismatch. This unlocks nothing new functionally; it gates *who may reach the
  existing surface*. Minimum bar before any non-loopback bind is even contemplated.
- **Rung 2 — mesh identity (Q1).** The MyOwnMesh ed25519 `Identity` authenticates the caller, and — if Q1
  unifies keys — the same identity that is the corpus sign-off authority. This is what unlocks the
  **corpus submit** side of §1.1 and the **attested sandbox evidence** of §1.8: operations that write or
  lower a gate require a *rostered* identity (`serve_corpus` read=roster/write=`Role::Owner`, P3). Rung 2
  is **Q1-blocked** and needs the key registry (F3) for multi-owner.
- **Rule:** read-only diagnose/health may stay rung 0 on loopback; anything that **writes the corpus**,
  **lowers an escalation**, or **binds non-loopback** requires at least rung 1, and corpus-write requires
  rung 2.

### 2.3 Concurrency / session semantics

- **Execute sessions (today).** `diagnose` mints a TTL-bound, one-shot `Session`
  (`serve.rs:57-78`): 15-minute TTL (`SESSION_TTL`), consumed on execute so a consent cannot be replayed
  (`serve.rs:422-427`), capped at `MAX_SESSIONS=256` as a memory bound (not a throughput knob). This is
  the right model — a stale consent must not authorize a run far from its diagnosis.
- **Idempotency.** Execute is already idempotent-by-consumption (a replayed `session_id` → 404
  `session_unknown_or_expired`). For the **async** execute of §1.3, add a client-supplied
  `idempotency_key` so a retried POST returns the same `execute_id` rather than double-running. Corpus
  submit (§1.1) similarly needs it — a re-sent row must dedupe (the corpus already run-dedups on `run_id`,
  `store.rs`, but the wire should not rely on that alone).
- **Intake sessions (§1.2)** are TTL-bound but *multi-turn* (not one-shot) — a distinct lifecycle; keep
  it in its own map, same `MAX_SESSIONS`-style bound, so an abandoned intake cannot grow memory.

### 2.4 Error taxonomy on the wire

- **Today:** `ApiError` is `{status, reason:&'static str}` with a **fixed vocabulary** built ad hoc at
  each call site (`serve.rs:89-106`, e.g. `"sign_off_below_required_escalation"`, `"session_unknown_or_
  expired"`). Good — reasons are never request-derived text. But they are *string literals*, decoupled
  from the engine's real named refusals in `GateError` (`gate.rs:13-53`: `Unconfirmed`,
  `ResolvedWithoutPass`, `LabelVerdictMismatch`, `DestructiveFixNeedsHuman`, `AttestationMissing`,
  `AttestationInvalid`, `ServedPlanInadmissible`, `RowNotDeIdentified`, `SymptomNotDeIdentified`).
- **Proposal (B6, S).** Map `GateError` variants to stable wire reason tokens (snake_case, pinned like
  the enum grammar, exhaustive match + a pinning test) so a refusal the consumer sees is the *same named
  refusal* the gate raised, and a new `GateError` variant forces a conscious wire decision. The error
  body stays `{error:"<token>"}` — **never** `{error:"{e:#}"}` (a `Display` of a `GateError` or
  `anyhow::Error` can carry a served-plan fragment or a path; that is the C3 "error-body passthrough"
  vector, leak matrix `error-and-board-unavailable-passthrough`). Refusals are tokens, not messages.

### 2.5 The per-endpoint egress-sink rule set (the standing obligation)

**Every new endpoint's response is a new egress sink.** The leak methodology's §5 Agent Contract applies
to the wire; here is the short rule set that must be satisfied *in the same PR* as any new response type:

1. **Vocabulary-only bodies.** A response field is a pinned enum token, a validated slug (`run_id`,
   `plan_id`), a hashed class, a stored/minted type (§1.1), or an integer. **Never** `Prose`
   (`title`/`description`/`rationale`/`message`/`summary`), never a model output, never a tool-output
   `Value`, never a transcript, never a path/URL.
2. **Poison contract test, ported.** Add a test in the shape of `cli_contract.rs` / `serve.rs`'s
   `diagnose_returns_..._no_request_prose` (`serve.rs:599-629`): plant the `leakguard::POISON` set into
   every input and assert no token survives the new body. A new endpoint without this test does not merge.
2b. **Structural, not substring.** Assert membership of the closed grammar where the field is a symptom
   (de-id is a transformation, not a deletion — leak §2a); reuse `leakguard::assert_no_poison`.
3. **Errors are tokens** (§2.4), never `Display`.
4. **No prose in logs either.** `eprintln!` diagnostics in a handler must not format request bodies or
   served rows (`serve.rs` logs only fixed strings + error *categories* today — keep it that way).
5. **Attestation on any corpus row crossing the wire** (§1.1): ship attested rows, re-verify on receipt.
6. **Never let the wire lower a gate without a signature** (§1.8): consent, sign-off, and sandbox
   evidence that *reduce* escalation must be cryptographically backed, else the conservative default holds.

### 2.6 AGPL §13 implications

- **For consumers: none.** AllMyStuff/MyOwnMesh reach the engine over a **process/network boundary and
  never link** any `cec-*` crate (serde-only wire mirror; zero AGPL cargo edge, CI-guarded). The API
  *is* the boundary the firewall was designed around — the same resolution as the sidecar, restated for
  HTTP (`integration-myown-family.md` license-resolution §; work-plan §3).
- **For the operator: real.** §13 attaches to *the party operating the network service*. The moment the
  engine is served to *other users over a network* (i.e. `--allow-remote`, or a mesh-served instance
  reachable by peers), the operator must offer those users the engine's Corresponding Source. Loopback,
  single-user is not "remote network interaction," so rung-0 self-host triggers nothing. **The auth
  ladder and the §13 obligation move together:** the same flag that opens the surface (rung 1+) is the
  flag that turns on the operator's source-offer duty. Note this for Chris beside Q2 — API exposure and
  the §13 duty are the *same* knob.

---

## 3. What NOT to build (anti-scope)

- **Non-loopback exposure as anything but a deliberate, authed act.** `--allow-remote` exists to make it
  a conscious decision (`serve.rs:162-170`); never default it on, never bind `0.0.0.0` in a shipped
  config, and never expose remotely without rung-1 auth. Every non-loopback bind also arms AGPL §13.
- **Raw plan / prose on the wire.** No endpoint returns a `common::Plan` with its `Prose` title/
  description, a candidate `rationale`, a `DiagnosticEvent.message`, a `ToolOutcome.data`
  (`serde_json::Value` — raw CIM), or the intake `transcript`. These are the exact fields the type split
  removed `Serialize` from; a hand-built `json!` must not re-add them. A "debug"/"explain" endpoint that
  echoes reasoning prose is the D1 leak with a friendly name — refused.
- **Corpus write from an unauthenticated caller.** Submit is rung-2 (rostered identity + `Role::Owner`);
  a loopback rung-0 caller may *read* precedents but never *write* truth. The sign-off gate is not a
  wire-optional check.
- **Attestation/keygen over the network** (§1.6) — the seed never becomes network-reachable.
- **Per-request inference endpoint/model URLs.** A caller may hint a *tier* (§1.7) among configured
  endpoints; it may never supply an endpoint. Where raw prose egresses to a model is an operator flag
  (`--allow-remote-inference`, leak §3.1), an audited local act — not a caller capability.
- **Streaming raw tool output** (§1.3) — chunked passthrough of executor prose is a C3 sink; stream only
  `{action, ok}` frames.
- **Stateful server-side inventory** (§1.4) — keep inventory per-request and stateless; a server-held
  device profile is a correlation handle for no gain.

---

## 4. Recommended sequence (extends work-plan §9, Phase B)

New items extend Phase B (the serve phase; B1–B5 already defined). **Sizes:** S < half a day · M = 1–3
days · L = a week-plus (matching §9). **The honest headline: none of these block P1'/E1** — AllMyStuff's
first touch codes against the *existing* `cec-diagnose/v1` + `cec-execute/v1` surface, and inventory push
is already in the diagnose body (§1.4). So the API-extension backlog is sequenced *after* the app client
exists, except the two cheap correctness items that should precede consumers hardening against the wire.

| # | Item | Size | Gate / depends | Priority |
|---|------|------|----------------|----------|
| **B6** | Error taxonomy → wire (map `GateError`, pinned tokens + test) | S | none | **before E1** — consumers will hardcode reason strings; pin them first |
| **B7** | `/v1/health` readiness + row/session counts (additive) | S | none | early — B5 lifecycle wants it |
| **B8** | `options.tier` hint in diagnose body (additive) | S | none | low |
| **B9** | Intake session flow (`cec-intake/v1`) | M | prompt-prose decision (§1.2) | nice-to-have, after E1/E2 |
| **B10** | Execute progress — polling (`{async}` + `GET /v1/execute/{id}`) | M | F4 (real long runs); latency-driven | defer until profiled |
| **B10b** | Execute progress — SSE | L | B10 + demonstrated need | later |
| **B11** | Sandbox-evidence submission on execute | M | **Q4**, F5, F3 | with Q4 |
| **B12** | Corpus query/submit over the API (W8 successor) | L | **B4**; submit is **P3/E3 + Q1** | mesh phase only |

**Placement in §9's order:** insert **B6/B7 alongside B3/B4** (the last cheap moment, before E1 codes
against the wire — same logic as pinning the enum grammar in B2). **B8** rides B3. **B9–B12** land in or
after **Phase E** (integration), with **B12** explicitly **E3-gated** (mesh, Q1) and **B11** **Q4-gated**.
Everything here is additive-only under D2; nothing forces a v2.

**The three calls that need Nathan/Chris are carried in the accompanying message, not edited into the
RFC.** The load-bearing recommendation: build **B6 + B7** now (cheap, pre-consumer); treat **B12** as the
real W8 and gate it on **B4 + Q1**; and hold the line that **attestation/keygen never cross the network**
(§1.6, §3) — that is the invariant the whole evidence model rests on.

---

## 5. Decision log

### 2026-07-02 — API posture (owner: Nathan)

The owner's decisions on the questions this document raises, recorded so the constraints are auditable
against the code that now enforces them.

- **Attestation / keygen / corpus write are never network-reachable — enforced (§1.6, §3).** The anti-scope
  is now a *mechanical* guard, not just prose: the `serve` router's exact route set — `GET /v1/health`,
  `POST /v1/diagnose`, `POST /v1/execute` — is frozen by the `router_surface_is_frozen` pinning test, so
  adding ANY route is a deliberate test edit. The never-routable invariant (attest, keygen, corpus write) is
  stated in `serve.rs`'s module docs and in `SECURITY.md`; a route that makes any of the three
  network-reachable is a reportable security issue.

- **Anything that can access the corpus must be attested + encrypted, trusted-box only, trusted calls only
  (§1.1, §2.2 rung 2).** A corpus-over-API endpoint, *if ever built*, ships **only** over a MyOwnMesh
  **rostered identity** or **loopback** — **never token-auth public HTTP** (there is no bearer-token tier;
  see the auth-ladder decision below). **Served rows must carry per-row attestation** so the consumer runs
  `ensure_attested` itself; today `HttpCorpus::query`'s `FixMapping` aggregate carries none (the read-wire
  gap tracked in FOLLOWUPS / B4), so **that gap must close first** — ship attested rows, not aggregates.
  **Encryption expectation:** the transport is encrypted end-to-end (mesh transport, or TLS), never cleartext
  HTTP. **No corpus endpoint exists yet** — this is a doc-level decision; the route-pinning test above is the
  mechanical guard that one is not added without this bar being met.

- **Trusted calls only (leak C2) — built now.** `--endpoint`/`--fast-endpoint` refuse a non-loopback host
  (loopback = `localhost` / `127.0.0.0/8` / `[::1]`) unless `--allow-remote-inference` is passed, on both the
  `diagnose` and `serve` paths (`validate_inference_endpoints`). This builds the pragmatic minimum of
  `corpus-leak-prevention.md` §3.1(b). See §1.7: a caller may hint a *tier*, never supply an endpoint — where
  raw prose egresses stays an audited operator flag.

- **Auth ladder resolved (§2.2).** The API stays **hard-loopback by default**; remote exposure is
  **mesh-only**. The **bearer-token rung (§2.2 rung 1) will not be built** — rung 0 (loopback) climbs
  straight to rung 2 (mesh rostered identity). `--allow-remote` prints an AGPL §13 network-service notice at
  startup, because binding beyond loopback arms the operator's §13 Corresponding-Source duty (the auth
  posture and the §13 obligation are the same knob; §2.6, `SECURITY.md`).

- **The §2.5 egress-sink checklist is binding policy** — copied into `AGENTS.md`; it must be satisfied in the
  same PR as any new response type.
