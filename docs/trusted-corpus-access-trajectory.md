# Trusted corpus access: the trajectory

**Status:** decision-ready, no code. **For:** Nathan (owner) + Chris. **Companion to:**
`docs/api-extension-design.md` (§1.1 corpus-over-API, §2.2 auth ladder, §2.5 egress rules, §5 decision log),
`docs/consolidated-work-plan.md` §3 (B3/B4), §5 (E3/E4), §6 (F1–F3), §9 (order),
`docs/integration-myown-family.md` (P3 `MeshCorpus`/`serve_corpus`, license firewall),
`docs/integration-rfc-for-chris.md` (Q1–Q5), `docs/corpus-leak-prevention.md` §3.1.

This is the trajectory **after** the API-enforcement wave. The API doc named corpus-over-API the "real W8
successor" (`api-extension-design.md` §1.1) and gated it on the read-side attestation gap (B4) and identity
(Q1). Nathan's decision (2026-07-02) sets the bar: *"anything that can even remotely access the corpus must
be attested and encrypted; keep it ONLY on a trusted box, and only allow trusted calls to it."* That decision
is not a new feature — it is a **constraint that collapses several open questions** and fixes a build order.
This doc states what is now decided, the dependency-ordered engine work to earn a servable corpus, and what
still waits on Chris.

---

## 0. The conceptual spine: three orthogonal properties, all now required

The single most important thing to hold in mind is that "safe to serve the corpus" is **three independent
guarantees**, and today's code has exactly one-and-a-half of them. Nathan's decision requires all three:

| Property | Question it answers | Mechanism | Today |
|---|---|---|---|
| **Admissibility** (de-id) | *Is the content free of identity — safe to bring in-process, print, re-serialize?* | closed-grammar leaf types + de-id image check | **DONE** (leak Phases 0–2) — read + write side |
| **Authenticity** (attestation) | *Did a trusted authority actually sign this off, or did a server fabricate it?* | ed25519 attestation over the canonical tuple | **write-side only**; read wire is a trust gap (B4) |
| **Confidentiality / access** (encryption + trusted-caller) | *Can only trusted, rostered callers reach it, over a channel no one can read?* | mesh transport / TLS + roster/loopback | **NOT built** — `HttpCorpus` is plain `reqwest`, no TLS |

The load-bearing insight for §2: **these do not substitute for each other.** A compromised corpus server can
trivially emit a `FixMapping` whose every action is frozen vocabulary, whose id is a clean slug, whose symptoms
are grammar tokens — it sails through the entire Phase-2 read-side validation
(`crates/corpus-client/src/store.rs:452-482`) — and yet was **never signed off by anyone**. De-id proves the
row *can't leak identity*; it says **nothing** about whether the row is *true*. Retrieval-first
(`corpus-primed`) means such a row becomes a candidate plan the human is asked to run. So attestation on the
read wire (B4) is necessary *even though Phase 2 is done*, and encryption/roster is necessary *even though
attestation is done* (a valid attestation is copyable; a truncated or replayed stream of individually-valid
rows still misleads — see F1). Decision #2 is the instruction to stack all three.

---

## 1. How decision #2 reshapes the plan (what is DECIDED vs still-OPEN)

Decision #2 is effectively a **partial answer to Q1 and Q2**, and it ratifies the auth-ladder decision already
in the API decision log (`api-extension-design.md` §5, 2026-07-02):

**DECIDED (Nathan can act on these now):**

- **Corpus access = rostered mesh identity or loopback — never public token HTTP.** This confirms the auth
  ladder's resolution: rung 1 (bearer token) **will not be built**; the surface climbs from rung 0 (loopback)
  straight to rung 2 (mesh rostered identity) (`api-extension-design.md` §2.2, §5). "Only allow trusted calls"
  = `serve_corpus` read gated on roster membership, write gated on `Role::Owner` (integration-myown-family P3).
- **Encrypted transport is mandatory off-box.** Corpus bytes cross the wire only over the mesh's
  authenticated/encrypted transport (myownmesh-core `Identity`/RPC) or TLS — never cleartext HTTP. This is a
  **new requirement not yet in code**: `HttpCorpus` today builds a bare `reqwest::Client::new()` against a
  `base_url` that may be `http://` (`store.rs:411-417`), and `serve` binds plaintext loopback
  (`serve.rs:176-190`). Loopback plaintext stays acceptable (the OS-user boundary); the constraint bites the
  moment a *peer* is the caller.
- **Attested rows only.** A corpus row crossing the wire must carry its ed25519 attestation so the *consumer*
  runs `ensure_attested` itself (this is B4; §2 below). The current `FixMapping` aggregate carries none
  (`schema.rs:80-88`) — decision #2 makes closing that gap a **precondition**, not a nicety.
- **Trusted box.** The private corpus stays on the owner's box (the `FileCorpus` at rest, encrypted-at-rest
  seed per the private-repo custody model); serving is opt-in and never the default (cold start unchanged).

**Q1/Q2 mapping — what decision #2 answers and what it leaves open:**

- **Q2 (inference-over-mesh)** is orthogonal to corpus and was already leaning loopback-only; decision #2
  doesn't change it but *sharpens the parallel*: corpus egress and inference egress are **separate encrypted
  knobs** on the same box. Corpus rows are de-id-safe to serve (encryption is access-control, not
  leak-prevention); raw inference prose is **not** de-id'd (`corpus-leak-prevention.md` §3.1(1)), so mesh
  inference stays a distinct, still-open decision. Do not let "corpus over mesh is fine" bleed into "inference
  over mesh is fine" — different leak class.
- **Q1 (identity unification)** is **partially answered**: corpus access *authority* is a mesh-rostered
  identity — so for a **single operator** the mesh `DeviceId` seed and the sign-off authority seed are the
  same ed25519 key (`CEC_SIGNOFF_SEED`, domain-tag-disjoint). **Still open:** the **multi-owner** case. A
  single configured public key (`admit(..., &Option<SignOffPublicKey>)`, `store.rs:17-26`; `ensure_attested`
  takes one `&SignOffPublicKey`, `gate.rs:161-176`) **cannot represent a mesh of several owners**. That is
  exactly F3 (key registry) — see §2.3. So decision #2 answers "which kind of identity" (rostered mesh) but
  **not** "how many, and how does a consumer pick the right verifying key per row" — that is engine work Nathan
  can greenlight (F3), and a policy question (who is an owner) that is Chris-adjacent via Q1.

**Net:** decision #2 turns the corpus-over-API candidate from "gated on Q1 + B4" into a concrete build:
**B4 + F1/F2/F3 are the hardening prerequisites, all pure-engine and greenlightable now; the mesh wiring (E3)
stays gated on Q1's multi-owner policy + Q3/Q5 (Chris).**

---

## 2. The dependency-ordered engine work

Each item: the de-id/attestation/encryption obligation it satisfies, size (S<½day · M=1–3d · L=week+), and gate.

### 2.1 B4 — the attested read path (authenticity on the wire). **Size M. Gate: none (greenlight now).**

**Current behavior (verified).** `HttpCorpus::query` (`store.rs:429-484`) now does read-side **admissibility**
re-validation in two layers: leaf `#[serde(try_from)]` on `StoredAction`/`StoredSymptom`/`StoredPlanId`
(`stored.rs:39-135`) makes an out-of-vocab action / non-grammar symptom / bad id **fail to deserialize**
(→ `GateError::ServedPlanInadmissible`), and a `de_identify_plan` image check catches a hand-edited derived
`title` (`store.rs:477-482`). This is real and adversary-tested (`http_query_refuses_an_adversary_seeded_served_symptom`).
But **cryptographic re-verification is absent**, and the code says so: *"Cryptographic re-verification of row
attestations on this path needs attested rows on the wire — the mappings aggregate carries none"*
(`store.rs:471-476`). `FixMapping` is `{signature, plan, confirmations}` (`schema.rs:80-88`) — a derived
aggregate that deliberately drops the per-row `label`, `sign_off`, `attestation`, and `provenance` the
attestation is computed over (`attestation_message`, `schema.rs:289-373`).

**The work.** Introduce a **served-row type** that carries what `ensure_attested` needs — the attested
`StoredOutcome` + `SignOffAttestation` + `RowProvenance` (i.e. essentially the `Contribution` minus
`integrity`) — and serve *those*, not aggregates. The consumer then, per row: deserialize (Layer-1 admissibility
already fires), run `ensure_attested(row, key)` (authenticity), and **aggregate `fix_mappings` client-side**
(`store.rs:67-142`, already pure over rows). One design call: aggregation moves to the consumer (or is done
both sides). This is correct — the consumer must not trust a server-computed `confirmations` count either.

**Composition with Phase 2 — authenticity vs admissibility are DIFFERENT guarantees (state this explicitly in
the PR).** Phase 2's `try_from` guards answer *"is this content de-identified?"*; B4's `ensure_attested`
answers *"did a trusted authority sign this exact tuple?"*. They are orthogonal and **both** required: a
server can serve a perfectly de-id-clean, never-attested row (passes Phase 2, fails B4) or — once F3 lands — a
genuinely-attested row that is nonetheless one an out-of-roster authority signed (passes admissibility, fails
authority selection). Keep the two error paths distinct: `ServedPlanInadmissible` (Phase 2, deserialize-time)
vs `AttestationMissing`/`AttestationInvalid` (B4, post-deserialize). Do **not** collapse them — an operator
debugging a refusal must know whether the server sent *identity* or sent an *unsigned* row.

**De-id/attestation/encryption obligation:** attestation (primary); rides on admissibility (Phase 2, done);
encryption is B4-adjacent but lands in E3's transport (§2.4).

### 2.2 F1 + F2 — chain-head anchor + canonical chain_hash (integrity of the *sequence*). **F1: M, F2: S. Gate: none for the engine change; F1's mesh-distribution half is Q5 (Chris).**

**Why attestation alone is insufficient.** Each row's attestation is *independently* valid. An attacker (or a
buggy serving node) can therefore **truncate the tail** — serve rows 0..k of a k+n chain — and every served
row still verifies. The hash chain (`verify_chain`, `store.rs:149-171`; `chain_hash`, `schema.rs:380-400`)
detects edits/reorders/mid-stream removal but **cannot detect trailing-row removal** without an external
length anchor (`RowIntegrity` doc, `schema.rs:112-124`; FOLLOWUPS "Chain integrity — key or anchor the head").
A truncated corpus silently drops the very `Reopened` rows that demote a bad fix (`fix_mappings` net-of-reopens
logic, `store.rs:124-142`) — i.e. truncation *re-promotes retracted fixes*. That is a real safety regression a
consuming peer must be able to refuse.

**F1 (M).** Anchor the chain head: an authority-signed `(head_hash, count)` anchor (or HMAC the chain with a
store-held secret). Then a consumer that holds the anchor detects truncation (count mismatch) and re-keys the
chain to an integrity boundary rather than a keyless recompute. Today the chain is keyless — "recomputable by
anyone with file-write access" (`store.rs:279-282`, `HANDOFFS` lesson 2026-06-14 23:15). F1's *engine* half
(produce + verify the anchor locally) is greenlightable now; its **mesh-distribution** half — how a consuming
peer *obtains* the anchor so a serving node can't ship a matching truncated anchor — is **Q5**, Chris.

**F2 (S).** `chain_hash` currently hashes the `serde_json::to_vec` image of the row (`schema.rs:384`) — stable
for same-code recompute, **coupled to struct field order**, not cross-language. The moment a **peer consumes
the chain** (verifies it independently, possibly in another impl), it must switch to the serde-independent
canonical encoder already used for attestation/plan signatures (`provenance::canonical`, `provenance/src/lib.rs:99-120`;
`attestation_message`, `schema.rs:289-373`). F2 is the small prerequisite that makes F1's anchor
**cross-impl verifiable**. Do F2 before any peer consumes the chain; F1's anchor should be built on the F2
encoding from day one to avoid a second migration.

**De-id/attestation/encryption obligation:** integrity/authenticity of the sequence (a fourth facet of
authenticity). No de-id impact (hashes over already-de-id'd rows). No encryption impact.

### 2.3 F3 — key-id → key registry. **Size M. Gate: none (greenlight now). THE LINCHPIN.**

**The argument that F3 is the keystone.** Everything else in this trajectory quietly assumes the consumer knows
*which public key* to verify a row against, and the current code assumes there is exactly **one**:

- `admit()` threads a single `&Option<SignOffPublicKey>` (`store.rs:17-26`); `ensure_attested` verifies against
  one `&SignOffPublicKey` (`gate.rs:161-176`).
- The row **already carries** `authority_id` — the first 16 hex chars of the signing key
  (`SignOffAttestation.authority_id`, `schema.rs:135`; `SignOffPublicKey::id()`, `provenance/src/lib.rs:223`)
  — but **nothing reads it to select a key.** The plumbing for a registry is half-present and unused.
- `FileCorpus::with_authority` re-admits *every at-rest row* under the one configured key
  (`store.rs:327-341`), so a corpus accreted under an old/other key becomes **un-openable** after rotation
  (`store.rs:324-326`; HANDOFFS lesson: "rotation now needs a key-id → key registry ... a prerequisite for
  rotation, not just a nicety").

F3 makes `authority_id → verifying-key` a first-class registry and threads it through `ensure_attested`/`admit`/
`with_authority`. That single change **unblocks three otherwise-separate things at once**:

1. **(a) Rotation.** A rotated key still verifies historical rows because the registry keeps the old key,
   selected per row by `authority_id`. Without F3, rotation bricks the at-rest corpus (above).
2. **(b) Multi-owner mesh (Q1).** A mesh has several owners; each row is verified against *the roster member
   who signed it*. A single-pubkey gate **cannot represent a set of owners** — this is the exact reason
   `api-extension-design.md` §1.1/§5 and consolidated §5 gate the corpus **submit/serve** side (E3) on Q1.
   F3 *is* the engine mechanism Q1's answer needs.
3. **(c) Verifier-vs-human distinct authorities.** Today one key signs both `VerifierConfirmed` and
   `HumanConfirmed` (the level is *bound* in the tuple, `schema.rs:344`, but one key attests both tiers,
   FOLLOWUPS "verifier vs human authorities"). A registry keyed by `authority_id` with an associated **role**
   makes the two trust tiers *cryptographically separable* — a verifier key physically cannot mint a
   `HumanConfirmed` row.

So F3 is the node the dependency graph funnels through: **B4** (consumer picks a key per served row) needs it
the moment there is >1 owner; **E3/E4** (multi-owner mesh serve) are Q1-blocked, and Q1's implementation *is*
F3; **rotation** (H3/W9) needs it. It is pure-engine, no Chris input required to *build the mechanism* (only to
decide *policy* — who is an owner, which is Q1). **Build F3 before any Q1-dependent work.**

**De-id/attestation/encryption obligation:** authenticity (key selection + role separation). No de-id/encryption impact.

### 2.4 The corpus-serving endpoint / MeshCorpus (P3 / E3). **Size L. Gate: Q1 (owner policy) + Q3/Q5 (Chris) + B4/F1/F2/F3 (engine).**

This is the W8 successor realized privately. Concretely, "**trusted box + trusted calls + encrypted**" means:

**What it IS:**

- **Transport:** the mesh's authenticated + encrypted channel (myownmesh-core `Identity`/RPC), or TLS — never
  cleartext. Loopback plaintext is the only unencrypted case, and only for the same-box operator.
- **Caller authority:** `serve_corpus` gates **read on roster membership**, **write on `Role::Owner`**
  (integration-myown-family P3; the mesh `roster`/`authorized_devices`/`Role::Owner` primitives). No token tier.
- **Rows:** **attested rows only** on the query path (B4) — `MeshCorpus::query` runs `ensure_attested` on every
  received row (P3 acceptance (d)), selecting the key via the F3 registry; plus the sequence anchor (F1) so a
  truncating node is caught, cross-impl-verifiable (F2). `MeshCorpus::submit` runs `admit()` before the wire
  (the write path is already gated: `store.rs:486-505`).
- **At rest:** `FileCorpus.with_authority(registry)` re-admits history (F3-aware), backed by the
  encrypted-at-rest seed custody the private repo already uses.

**What it MUST NOT be** (consistent with the API decision log, `api-extension-design.md` §1.6, §3, §5, and the
frozen router):

- **No public HTTP, no bearer-token tier.** The `serve` router surface is **frozen** to
  `GET /v1/health`, `POST /v1/diagnose`, `POST /v1/execute` by `router_surface_is_frozen`
  (`serve.rs:262-268, 814-838`). Corpus read/write is **not** a route on that surface and must not become one —
  the corpus is served over the **mesh adapter** (`corpus-mesh-adapter`, an AGPL crate that ships with the
  engine), never the public API face. Adding a corpus route to `serve` is a reportable security issue
  (`serve.rs:28-40` never-routable invariant).
- **No attestation/keygen over any network** — the seed never becomes network-reachable; the engine holds only
  the public key(s) (`serve.rs:28-40`; `provenance` asymmetry). This is invariant, not a trade-off.
- **No server-trusted aggregates** — the consumer re-verifies and re-aggregates; a server-supplied
  `confirmations` is advisory at best.

**De-id/attestation/encryption obligation:** all three — admissibility (Phase 2, done), authenticity
(B4+F3+F1/F2), confidentiality/access (mesh transport + roster). This is the item where decision #2 is fully
discharged.

---

## 3. Gated on Chris vs greenlightable now

**Greenlight now (pure-engine, no cross-repo dependency):**

- **B4** attested read path (served-row type + consumer-side `ensure_attested` + client aggregation). *M.*
- **F2** canonical `chain_hash`. *S.*
- **F1** chain-head anchor — the *engine* half (produce/verify locally). *M.*
- **F3** key-id → key registry (rotation + role-separated verifier/human + multi-key select). *M.*

None of these need Chris. They harden the existing `FileCorpus`/`HttpCorpus` paths and are valuable even before
any mesh exists (rotation, tamper-truncation defense, read-wire authenticity all stand alone).

**Gated on Chris / owner policy:**

- **Q1 (multi-owner policy)** — *who* is an owner, and whether sign-off authority is portable across devices.
  F3 builds the *mechanism*; Q1 decides the *roster policy* that populates it. Gates E3/E4.
- **Q3 (`myownmesh-core` pin)** — the adapter and AllMyStuff must pin the *same* mesh-core tag; single source
  of that pin. Gates E3 (the adapter cannot be written against a moving target).
- **Q5 (anchor distribution)** — how a consuming peer obtains F1's head+count anchor out-of-band so a serving
  node can't ship a self-consistent truncated anchor. This is F1's *mesh* half — Chris decides whether the
  rostered-owner trust model suffices or the anchor rides a separate handshake.

**Q2 (mesh inference)** and **Q4 (mesh sandbox)** are not on this corpus critical path but share the same
"encrypted, trusted-caller" posture when they land.

---

## 4. Recommended sequence (extends `consolidated-work-plan.md` §9)

Continuing the §9 numbering (Phase F is the hardening backlog; this refines its order and inserts the corpus
serve item as the E3 dependency):

```
[after B3/B4 serve wave, leak Phases 1–2 DONE]
→ F2  (canonical chain_hash)                 S   ── do first: cheap, and F1 builds on it
→ F3  (key-id → key registry)                M   ── THE LINCHPIN; unblocks rotation + Q1 + role split
→ B4  (attested read path: served-row type)  M   ── needs F3 for multi-key select; single-key works interim
→ F1  (chain-head anchor, engine half)       M   ── on the F2 encoding; Q5 gates the mesh-distribution half
   ── (F2→F3→B4→F1 is the critical path; all four are greenlightable now) ──
→ E0  Q1 (owner policy) + Q3/Q5 (Chris)       ── blocking, zero-code
→ E3 = P3  corpus-mesh-adapter / MeshCorpus / serve_corpus   L   ── needs B4+F1+F2+F3 + Q1/Q3/Q5
→ E4 = P4  identity unification + egress policy               M   ── Q1
```

**Critical path called out:** **F3 must precede any Q1-dependent E3/E4 work.** The consolidated plan already
says E3/E4 are Q1-gated and that Q1's multi-authority question is "the same registry work as F3" (§5). Making
that explicit: *do not wait on Chris to build F3.* F3 (and F2 before it, and B4/F1 around it) can all land while
Q1/Q3/Q5 are still in Chris's court, so that the day Q1 is answered, E3 is a wiring exercise against finished
primitives rather than a fresh crypto design. The single highest-leverage move is **F3**, because it is
simultaneously the rotation fix (H3/W9), the multi-owner enabler (Q1/E3), and the verifier/human separation —
three backlog items collapsed into one M-sized change.

One ordering subtlety: **B4 can ship single-key first** (interim, `authority_id` ignored, verifies against the
one configured key) and gain multi-key selection when F3 lands — but building F3 first avoids a throwaway
single-key B4 and a second migration. Recommended: F2 → F3 → B4 → F1.

---

## 5. Anti-scope — what this trajectory explicitly does NOT do

- **Does not add a corpus route to the `serve` HTTP surface.** The frozen router
  (`serve.rs:262-268, 814-838`) stays health/diagnose/execute. Corpus travels the mesh adapter, never the
  public API face. No `POST /v1/corpus/*` on `serve`.
- **Does not introduce a bearer-token / public-HTTP corpus tier.** Rung 1 stays unbuilt (decision log §5);
  loopback → mesh-rostered, nothing between.
- **Does not put attestation, keygen, or the sign-off seed on any network.** The asymmetric split is invariant;
  the engine holds only public keys. No `attest`/`sign` oracle, ever.
- **Does not de-identify inference/model egress.** Corpus over mesh is de-id-safe by construction; raw prompt
  prose is a *separate* accepted-risk boundary (`corpus-leak-prevention.md` §3.1(1)) handled by `PromptPayload`
  + `--allow-remote-inference`, not by this trajectory. Q2 is not answered here.
- **Does not build the mesh transport itself** (myownmesh-core is Chris's, MIT) — only the AGPL adapter and the
  engine-side verify/anchor primitives.
- **Does not decide roster policy (Q1) or anchor-distribution policy (Q5).** It builds the mechanisms those
  answers will configure; it does not pre-empt the owner/Chris decision.
- **Does not fix the C7 unsalted-FNV correlation handle** *(FIXED 2026-07-04: keyed HMAC `cec-fingerprint-v2` + POST-body retrieval keys — see leak doc §3.1(2) BUILT note)* (`config_class.key()`/fingerprint reversibility,
  `corpus-leak-prevention.md` §3.1(2)). Encrypting the transport hides the handle in flight but the keyed-HMAC
  change is a distinct F-track item; noted, not in scope here.

---

## 6. Decision log entry (proposed — for Nathan to ratify)

### 2026-07-02 — Trusted corpus access posture (owner: Nathan)

- **Corpus access requires all three of admissibility + authenticity + confidentiality** (§0). De-id (done)
  does not substitute for attestation (B4) which does not substitute for encryption/roster (E3). Serving is
  opt-in; cold start unchanged.
- **Read wire carries attested rows, not aggregates** (B4) — the consumer runs `ensure_attested` per row and
  aggregates client-side. Precondition for any peer read.
- **Transport is encrypted end-to-end** (mesh `Identity`/RPC or TLS); loopback plaintext only for the same-box
  operator. `HttpCorpus`/`serve` plaintext defaults are not a peer transport.
- **The key registry (F3) is the linchpin** — rotation, multi-owner (Q1), and verifier/human separation all
  route through it; build it before Q1-dependent E3/E4.
- **The frozen `serve` router and the never-routable invariant hold** — corpus travels the mesh adapter, never
  a public route; attest/keygen/seed never network-reachable.
