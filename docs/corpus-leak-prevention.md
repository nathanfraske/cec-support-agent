# Corpus Leak Prevention Methodology

**Status:** decision-ready, partially implemented. This document is the authoritative
specification for how operator/host identity is kept out of the public corpus, the
`--json` envelope, stdout/stderr, model-prompt egress, and the git tree.

**Scope of the guarantee we are buying:** an agent (or a tired human) making a *normal,
well-motivated edit* should hit a **compile error or a red CI check**, not a silent
green build that ships a leak. We are explicit below about where that is a hard
guarantee (type system) versus defense-in-depth (lints, content scans, policy).

**The one-sentence root cause:** *De-identification is enforced today as a chokepoint
**discipline** — one function (`Contribution::new` → `de_identify_plan`) that every
write path is **supposed** to flow through — rather than a **type invariant** the
compiler checks.* Every critical vector below is an instance of "a value reached a
sink without passing the chokepoint, and nothing screamed." The fix is to make
"de-identified" a property of the **type**, re-checked on **read** and at the **write
gate**, so the set *{values that can reach a sink}* is, by construction, equal to the
set *{values proven de-identified}*.

---

## 1. Threat model

### 1.1 The structural root cause

```
        WRITE PATH (today)                     THE HOLE
  Outcome{raw Plan} ──► Contribution::new ──► de_identify_plan ──► row
                              ▲                      │
                              │                      ├─ keeps step.action VERBATIM
                   the ONLY chokepoint              ├─ keeps plan.id VERBATIM
                   (a discipline, not a type)        └─ sets description = action
```

Three facts, all verified in-tree, make this a discipline and not a guarantee:

1. **`de_identify_plan` is a pass-through for the two fields it claims to scrub.**
   `crates/corpus-client/src/schema.rs` builds each row step as
   `PlanStep { description: step.action.clone(), action: step.action.clone(), risk }`
   and `Plan::new(plan.id.clone(), ...)`. So `action` and `id` are the de-id *output*,
   copied byte-for-byte. The "de-id model" only reduces `title`/`description` to the
   action vocabulary; it never validates `action` or `id` against any charset/allowlist.

2. **The same raw types are the corpus row.** `Outcome.plan: Plan`, `FaultSignature.symptoms:
   Vec<Symptom>`, `Symptom(pub String)` — the in-flight object and the stored row are the
   *same type*. There is no `RawPlan` vs `StoredPlan` split, so the type system cannot
   tell "proven clean" from "arbitrary prose." `Symptom`'s inner `String` is `pub` and
   `From<&str>` accepts anything.

3. **The gate checks structure, not content.** `gate::ensure_evidence_integrity` checks
   sign-off, label↔verdict agreement, and destructive-fix human sign-off. It never
   re-runs `extract_symptoms` or asserts `de_identify_plan` is idempotent. A row built by
   struct literal (all `Contribution` fields are `pub`) bypasses `Contribution::new`
   entirely and the gate signs it anyway.

Everything else is a consequence of this root cause being unaddressed on one of four
**boundaries**: serialize-to-row, print-to-stdout/stderr, **network egress to a model
endpoint**, and **commit-to-git**.

### 1.2 Leak vector classes, ranked

| Rank | Class | Why it is the rank it is | Representative vectors |
|------|-------|--------------------------|------------------------|
| **C1** | **Trusted-chokepoint pass-through** — the de-id function itself copies identity through | The one field the de-id model trusts is unvalidated; ships green through the existing test | `plan-action-verbatim-no-charset`, `plan-id-never-deidentified`, `symptom-type-public-no-construction-discipline`, `contribution-struct-literal-bypass` |
| **C2** | **Model / inference network egress** — raw `--describe` + transcript sent to an endpoint by design | Same PII the corpus de-id exists to stop, on a network boundary the de-id model does not touch *at all*; worsens under the active MyOwn integration | `inference-channel-raw-statement`, `model-generator-raw-event-message`, `model-prompt-not-a-row-but-egress`, `inference-error-body-echo` |
| **C3** | **Raw-type serialization / print** — any new sink that serializes a domain object or prints a prose field | Blanket `derive(Serialize)`+`Debug` on every type; one `to_string`/`json!`/`eprintln!` leaks; the D1 class | `domain-types-blanket-serialize-debug`, `raw-domain-type-serialize`, `new-envelope-field`, `new-corpus-row-field`, `tooloutcome-data-raw-cim`, `agentrun-public-serialize-no-deid`, `human-trace-stdout-*` |
| **C4** | **Read-side trust** — served/deserialized rows are treated as already de-identified | De-id is write-side only; `serde` bypasses every smart constructor; a peer/hand-edited row leaks on print or re-serialize | `http-query-response-not-re-deid`, `served-plan-title-printed-verbatim`, `served-symptoms-echoed-in-envelope` |
| **C5** | **Extractor shape-heuristic admission** — the allowlist admits identity by *shape* | `is_stop_code_name` keeps any `ALL_CAPS_UNDERSCORE` token; `module_name` keeps any `stem.exe`; these are denylists-of-shape, not dictionaries — they pass asset tags, AD groups, in-house binaries | `stop-code-rule-keeps-asset-tags-verbatim`, `module-name-rule-keeps-custom-binaries-verbatim`, `extractor-charset-widening`, `prefix-id-grammar-numeric-passthrough` |
| **C6** | **Verbatim opaque slots** — same-charset fields stored as-is | `BomRevision(String)`, `run_id`, `part_class`, inventory keys: a hostname/serial is indistinguishable from a legit token; some are stored *unhashed* | `envelope-config-class-key`, `bom-revision-verbatim`, `opaque-token-fields-same-charset-identity`, `escalated-hardware-part-class-string`, `inventory-keys-*` |
| **C7** | **Hash-as-safety fallacy** — unsalted FNV emitted in envelope + URL | "It's hashed so it's safe" is false for low-cardinality/identity inputs; reversible by dictionary; a stable per-host correlation handle in logs | `unsalted-fnv-fingerprint-preimage`, `httpcorpus-fingerprint-in-url` |
| **C8** | **Commit-time / git tree** — corpus rows, recon JSON, PII in source/docs/messages | Rails are name/secret-shaped, never content-shaped; hook is dormant; recon artifacts already hold live infra identity | `no-content-gate-anywhere`, `hook-dormant-and-gitleaks-uninstalled`, `recon-infra-json-unignored`, `rename-extension-bypass`, `git-add-f-defeats-gitignore-entirely`, `test-fixture-pii-is-the-house-style`, `doc-prose-leaks-infra-already-committed`, `gitleaks-cant-see-the-ed25519-seed-shape`, `commit-message-and-branch-leak-surface` |
| **C9** | **Meta — false-confidence test** — the proof-of-no-leak avoids the trusted fields | The single de-id test seeds `describe/title/description` but uses clean `action`/`id`/label, so a C1 regression stays green | `leakage-suite-coverage-gap`, `attestation-message-new-field-unbound`, `vocabulary-snapshot-drift`, `ci-gate-runs-only-check-no-lint-no-gitleaks` |
| **C10** | **Corpus cartography / query-oracle enumeration** — a caller with legitimate query access aggregates individually-clean, de-identified, attested responses to map the corpus's membership/coverage/structure/fix-content; orthogonal to C1-C9 (needs no identity to survive de-id); enforced by differential-minimization + budget + audit + roster, never by a type. See `docs/corpus-cartography-threat.md`. | Every prior class asks "did identity leak out of one row?"; C10 asks "can many de-id'd, attested rows be assembled into the map of what the corpus knows?" — de-id/attestation/encryption are all satisfied and it still succeeds | `diagnose-source-membership-oracle`, `retrieval-first-latency-differential`, `candidate-slate-structure-disclosure`, `enumerable-fnv-probe-space`, `served-provenance-priming-graph`, `no-query-budget-or-audit` |

**The honest framing:** C3 and C8 are the "easy accidental leak" — those we *can* convert
to hard stops cheaply. C1 is the keystone — fix it and the strongest vectors die. **C2,
C4, C5, C7 are not closable by a serialize-side type wrapper alone**; they need read-side
re-validation, a prompt-payload chokepoint, closed dictionaries, and keyed hashing
respectively. Any methodology that claims a single `DeIdentified<T>` newtype "solves it"
is wrong — and the red-team proved exactly that.

---

## 2. Layered defense

Four layers. Each enforces one invariant and converts a documented agent mistake into a
**compile error** (L1) or a **red CI check** (L2–L4). The red-team fixes are folded in;
where a layer is defense-in-depth rather than a guarantee, it is labeled as such.

### Layer 1 — Type-system chokepoint: validating mints, leaf-level prose typing, split raw/stored types

**Invariant:** *A value can cross a serialize/print boundary **iff** it is, by its type, a
value that was produced by a validating de-id mint — and the mint enforces **content**
(a positive allowlist), not merely **provenance**.*

This is the keystone. The red-team showed three ways a naive `DeIdentified<T>` is defeated;
the design below closes all three.

**1a. Split the types so the corpus row is not the in-flight object.**
Introduce `StoredPlan` / `StoredSymptom` (in a new `de-id` crate) as the **only**
`Serialize + Deserialize` corpus-bound payload. The in-flight `Plan` / `Candidate` /
`Outcome` / `DiagnosticEvent` / `ToolOutcome` / `AgentRun` **lose `Serialize`** (keep
`Clone`, keep `Debug` *only* if sealed — see 1d). This is what makes `to_string(&candidate)`
a hard `rustc E0277`. Crucially it also resolves the red-team blocker that "you can't remove
`Deserialize` from `Plan` because `FileCorpus::open`/`HttpCorpus::query` need it" — they now
deserialize `StoredPlan`, and a raw `Plan` genuinely has no serde path.

**1b. Type the *leaf* prose, not just the container.**
Every leak-bearing field is a `String` today (`Plan.title`, `PlanStep.description`,
`Candidate.rationale`, `DiagnosticEvent.message`, `StepResult.summary`), and
`String: Serialize + Display` forever. Removing `Serialize` from the *struct* does nothing
for `json!({"why": c.rationale})` or the pre-existing `render_consent(&plan)` that copies
`plan.title` out into a printable `String`. **Fix:** replace those fields with
`Prose(String)` — private field, **no `Serialize`, no `Display`** — exposed only via an
explicit `into_inner()` that the egress lint denylists. Then `format!("{}", plan.title)`
fails to compile.

**1c. Mints are *validating*, with a positive allowlist and a round-trip test.**
`DeIdentified<T>` has a private field and the only `impl Serialize`. Its mints are the only
constructors, and **each runs a positive check and returns `Result`**:
- `ActionToken::parse(&str) -> Result<ActionToken, Reject>` — membership in the **frozen
  dispatcher-tool registry** (not a charset shape). `de_identify_plan` calls this on every
  `step.action` and on `plan.id`, and **returns `Result`**; an out-of-vocab action aborts
  the row instead of being copied through. This is the C1 fix.
- `symptom(&str) -> Result<Symptom, Reject>` — must satisfy the single-token charset
  `^[a-z0-9._]+$` **and** round-trip `extract_symptoms(s) == [s]`, **and** (C5) be a member
  of a **closed grammar**: `VOCABULARY ∪ 0x-hex ∪ <known-prefix>_<digits> ∪ FROZEN
  stop-code dictionary ∪ FROZEN OS/driver-module allowlist`. The shape heuristics
  `is_stop_code_name`/`module_name` are replaced by these dictionaries.
- The envelope mint takes `Vec<ActionToken>`, `FingerprintHash`, `ConfigClassKey`, enum
  tags **only — no `String`/`serde_json::Value` field anywhere**, so there is literally no
  slot to write `rationale` into. (`diagnose_envelope` returns `DeIdentified<Envelope>`,
  not a mutable `Value`.)

> **Why "validating, not just provenance" matters (red-team C1):** a `DeIdentified<T>`
> that only proves "came from a mint" is decoration — it certifies whatever the mint
> copied through. The mint **predicate** is the security boundary; it must be a positive
> allowlist with a round-trip property test, or the newtype is a vault around a sieve.

**1d. Seal `Debug` on raw types.** `format!("{outcome:?}")` / `dbg!(candidate)` leaks and is
`Display`/`Debug`, not `Serialize`. Either remove `Debug` from the raw domain types or
implement it as a redacting `Debug`. Do not "punt to a lint."

**1e. Read-side re-de-id (C4).** `DeIdentified` has **no `Deserialize`**. A served
`FixMapping` deserializes into a `StoredPlan`/`StoredSymptom` that is then **re-validated**
through the same mints before it can become a `Candidate` or be printed:
`DeIdentified::from_served(mapping) -> Result`. Add `#[serde(try_from = "String")]` on
`StoredSymptom`/`ActionToken` so a non-vocabulary value **fails to deserialize** — making
the wire/file path identical to the construction path. `serde` no longer bypasses the gate.

**1f. Seal the write gate (C1 struct-literal + C4 re-wrap).** Make `Contribution`'s fields
**private**, `Contribution::new` the only constructor. Make
`gate::ensure_evidence_integrity` **re-run `de_identify_plan` and assert idempotence**
(`de_identify_plan(&plan) == plan`) and re-run the symptom check over `signature.symptoms`,
rejecting any row whose content is not already extraction-clean. This is the only thing
standing on the **runtime corpus-write path** to `/mnt/e/cec-corpus-private`, which **no
git/CI/CODEOWNERS layer ever sees** — so it must be a content gate, not a structure gate.

**How it makes a leak a compile error:**
- `serde_json::to_string(&candidate)` → `Candidate: Serialize is not satisfied` (E0277).
- `json!({"rationale": c.rationale})` → `c.rationale` is `Prose`, not `Serialize`/`Display` → E0277.
- `ModelGenerator` setting `action: content` → de-id mint `ActionToken::parse(content)` returns `Err`; the row is refused, not minted.
- Struct-literal `Contribution{..}` outside the module → private fields → does not compile (pinned by a `trybuild` compile-fail test).
- A served row's title printed verbatim → it's `Prose` from `StoredPlan`, no `Display` → must go through `from_served` re-validation.

**Effort:** large. **Guarantee class:** hard (type system) for C1/C3/C4; depends on the
mint dictionaries being correct for C5.

### Layer 2 — Poison-set property/fuzz harness (the test that would have caught D1)

**Invariant:** *No identity planted into **any** input or intermediate field appears in
**any** sink's bytes — and the assertion is **structural** (the symptom is a member of the
closed grammar), not substring-absence, because de-id is a **transformation**, not a
deletion.*

**2a. One canonical `POISON` set + structural assertion.** A `leakguard` crate owns the
single `POISON` superset (merging today's `SEEDED_IDENTIFIERS` + envelope-test tokens + PII
*shapes*: FQDN, NetBIOS, service-tag, GUID/SID, AD group, UNC, home-path, MAC, IPv4/IPv6).
The check is **not** `!bytes.contains(token)` — that is unsound against a transforming
pipeline (the red-team C5/transform bypass: `RIG_NATHAN_DESK` → `rig_nathan_desk`,
`acmecorp_agent.dll` → kept verbatim — byte-distinct from every planted token, invisible to
substring scan). Instead: **every symptom in every sink must be a member of the closed
grammar** (re-run the de-id validator over sink output and fail on any token the validator
would not itself have produced). Drop the per-token FNV-fingerprint scan — it is unsound
(`fingerprint_of` hashes the *sorted set*, not a token, so the scanned pre-image never
appears).

**2b. Self-deriving field coverage (red-team flow-coverage bypass).** Do **not** use a
hand-maintained `FIELD_MANIFEST` that an agent edits in the same PR. Use a derive macro that
plants a **unique** poison token into **every** `String`/`Prose` field (input *and*
intermediate) by reflection, runs the real binary end-to-end under `--json` **and**
non-`--json`, capturing **stdout and stderr**, and scans every sink. Coverage is structural
(the macro generates a token per field) and the assertion is global (no token survives
anywhere). A new free-text field gets a token automatically; an `Outcome.notes` added without
de-id fails.

**2c. Ban `serde_json::Value` on boundary types.** `ToolOutcome.data` and `AgentStep.args`
are untyped `Value` — reflection cannot see inside them, so "every field probed" is vacuous
for the two highest-fidelity raw-CIM fields. Replace with typed, allowlisted summaries.

**2d. Adversary controls the served bytes (C4).** The read-path probe must feed a served
`FixMapping` whose fields are **seeded with identity** (not hand-built clean — that repeats
the very `leakage-suite-coverage-gap`), asserting `from_served` strips it.

**2e. Verification it is real.** Must currently **pass** on the fixed tree and be shown to
**fail** when reverting the D1 fix (re-add `"rationale"` to the envelope) **and** when
making `de_identify_plan` copy `step.action` from a planted action. Both proven red before merge.

**How it makes a leak CI-fail:** runs in the existing `cargo test --workspace` merge gate
on all three OSes. A C1/C3/C4 regression that today ships green because the test avoids the
tainted field now plants there and fails.

**Effort:** medium. **Guarantee class:** strong for the current tree; defense-in-depth
against *wholly new sinks* the harness can't reach without registration (that residual is
covered by L1's type barrier and L3's lint).

### Layer 3 — CI / lint / boundary gate: egress allowlist + content gate + live hook

**Invariant (3a, egress):** *The **only** code permitted to call a byte-emitting primitive
(`print*`/`serde_json::to_*`/`write`/`tracing`/`log`/socket/`fs::write`) is a single
`egress` module that accepts `DeIdentified<_>` exclusively.* This is an **allowlist**
(one module may do I/O), **not** a denylist of macro names — the red-team showed a
name-denylist is beaten by aliasing (`use ... as emit`), generics (`fn dump<T:Serialize>`),
trait dispatch (`.to_json()`), `tracing!`/`log!`/`fs::write`, and the `schema.rs` carve-out.
Enforce with a **dylint/rustc MIR pass** that follows the value's type through bindings,
flagging any I/O/fmt primitive **outside** the `egress` module. There is **no free-text
`// allow-sink` escape**; an exception must be wrapped in `DeIdentified<T>` so the exception
is itself type-checked.

**Invariant (3b, content gate, defense-in-depth):** *No file in a commit may contain the
project's corpus-row JSON shape or an un-allowlisted PII token, keyed on **content** not
filename* — so `.json`/`.md`/`.txt`/double-extension/`git add -f`/rename bypasses all fail.
`cargo xtask scan-content` greps staged blobs (`git diff --cached` → `git show :file`) for
(i) corpus-row key co-occurrence (`signature`+`fingerprint`+`symptoms`, `attestation`+
`authority_id`+`signature`, `integrity`+`prev`+`hash`+`run_id`) and (ii) PII shapes, and
**base64/gzip/hex-decodes string literals over N bytes and re-scans the decoded bytes**
(red-team encoding bypass) and **forbids decode-at-runtime in test files**. The
`.boundary-allow.txt` is for the *existing committed synthetic literals only*; **net-new
entries fail in CI** unless an owner-only workflow approves — closing the "edit the
allowlist in the same PR" hole. Be honest: a content scanner over an infinite identity space
is **best-effort**; the real gate for tree-leak is L1 stopping the data from being *produced*
in serializable form.

**Invariant (3c, the rail actually runs):** `cargo xtask install-hooks` sets
`core.hooksPath` and downgrades the `gitleaks` hard-dependency to warn-and-skip (so the hook
is installable without gitleaks), and a **required CI `boundary` job** runs the same scanners
server-side **and asserts the hook still invokes xtask** — so an unprovisioned clone is still
gated. `.gitleaks.toml` gains project rules for `CEC_SIGNOFF_SEED` and the corpus-row shape.

**How it makes a leak CI-fail:** re-adding `"rationale"` to `diagnose_envelope` fails the
frozen-envelope-key check; a `tracing::info!(?candidate)` outside `egress` fails the dylint;
a corpus row pasted into a `.rs`/`.md` fixture fails the content scan regardless of extension.

**Effort:** large. **Guarantee class:** 3a is hard *if* implemented as a type-aware dylint
(not grep); 3b/3c are defense-in-depth.

### Layer 4 — Agent policy & ownership (codified, branch-protected — defense-in-depth)

**Invariant:** *The de-id surface (the mints, `extract.rs`, the dictionaries, the gate, the
egress module) cannot be merged without the corpus owner's review, and the rules are stated
where an agent will read them.*

- `.github/CODEOWNERS` locks `crates/de-id/**`, `corpus-client/src/schema.rs`,
  `common/src/extract.rs`, the dictionaries, `crates/leakguard/**`, the egress module, and
  `ci.yml`; branch protection requires Code-Owner review on `main`.
- `AGENTS.md` carries the **Agent Contract** (§5).

**Honest limit (red-team):** CODEOWNERS is GitHub-*merge*-time policy. It does **nothing**
for the runtime corpus-write to `/mnt/e` (covered only by L1's write gate, 1f), nothing for a
leak already in a pushed branch's history/CI logs, and degrades to advisory if branch
protection is ever disabled. It is the weakest layer and is explicitly **not** a guarantee.

---

## 3. Coverage matrix

L1 = type chokepoint, L2 = poison harness, L3 = egress lint + content gate, L4 = policy.
**Bold** = the primary hard stop. "discipline" = still relies on a human/correct-config.

| Vector | L1 | L2 | L3 | L4 | Residual |
|--------|----|----|----|----|----------|
| plan-action-verbatim-no-charset | **✅ validating mint (1c)** | ✅ | ✅ | — | hard once mint is a positive allowlist + round-trip test |
| plan-id-never-deidentified | **✅ `ActionToken`/`PlanId` parse (1c)** | ✅ | ✅ | — | hard |
| symptom-type-public-no-construction | **✅ private field + `try_from` (1c/1e)** | ✅ | — | — | hard |
| contribution-struct-literal-bypass | **✅ private fields + write gate (1f)** | ✅ | ✅ | — | hard |
| non-executable-plan-recorded | **✅ write gate re-validates action (1f)** | ✅ | — | — | hard |
| domain-types-blanket-serialize-debug | **✅ remove Serialize + seal Debug (1a/1d)** | ✅ | ✅ | — | hard |
| raw-domain-type-serialize | **✅ (1a)** | ✅ | ✅ | — | hard |
| new-envelope-field | **✅ envelope mint has no String slot (1c)** | ✅ | ✅ frozen-key check | — | hard |
| new-corpus-row-field | **✅ split type; new field on raw type not Serialize (1a)** | ✅ self-deriving coverage | — | — | hard |
| tooloutcome-data-raw-cim | **✅ ban `Value`, type the summary (1a/2c)** | ✅ | ✅ | — | hard |
| agentrun-public-serialize-no-deid | **✅ (1a)** | ✅ | ✅ | — | hard |
| human-trace-stdout-raw-request / -prose | partial (Prose 1b) | ✅ stderr+stdout capture | **✅ egress allowlist (3a)** | — | hard via 3a dylint |
| json-mode-human-trace-to-stderr | partial | **✅ (2b captures stderr)** | ✅ | — | strong |
| raw-describe-and-event-message-logged | partial (Prose 1b) | ✅ | **✅ (3a)** | — | strong |
| registry-set-summary-paths | partial (Prose on summary) | ✅ | ✅ | — | strong |
| error-and-board-unavailable-passthrough | — | partial | partial (3a if error routed via egress) | — | **discipline** — `{error:#}` is `Display`; needs Prose on error contexts |
| **http-query-response-not-re-deid** | **✅ `from_served` re-de-id (1e)** | ✅ adversary-seeded served row (2d) | — | — | hard once 1e lands |
| served-plan-title-printed-verbatim | **✅ (1e)** | ✅ | ✅ | — | hard |
| served-symptoms-echoed-in-envelope | **✅ (1e)** | ✅ | — | — | hard |
| stop-code-rule-keeps-asset-tags | **✅ closed stop-code dictionary (1c/C5)** | ✅ structural assert (2a) | — | — | hard once dictionary replaces shape heuristic |
| module-name-rule-keeps-custom-binaries | **✅ closed module allowlist (1c/C5)** | ✅ | — | — | hard |
| extractor-charset-widening | ✅ round-trip + closed grammar | ✅ property test | — | ✅ CODEOWNERS on extract.rs | strong |
| prefix-id-grammar-numeric-passthrough | ✅ grammar tightened | ✅ | — | — | strong (low-entropy residual) |
| envelope-config-class-key (BomRevision raw) | **✅ `ConfigClassKey` mint hashes/validates (1c)** | ✅ | ✅ content scan | — | hard once key routed through mint |
| bom-revision-verbatim (private repo) | n/a (private) | — | ✅ content scan | ✅ | **discipline** — needs the W6 opacity heuristic + author confirm in corpus-ingest |
| opaque-token run_id/part_class | ✅ validated mint | ✅ | — | — | strong |
| escalated-hardware-part-class-string | **✅ enum/allowlist (1c)** | ✅ | — | — | hard |
| inventory-keys-* (unvalidated, passthrough) | ✅ charset allowlist on keys + `ConfigClassKey` | ✅ | ✅ content scan | — | strong |
| unsalted-fnv-fingerprint-preimage | partial (validate inputs) | — | — | — | **discipline** → needs **keyed/salted HMAC** (C7); not closed by any layer above |
| httpcorpus-fingerprint-in-url | — | — | partial | — | **discipline** → move keys to request body + salt (C7) |
| **inference-channel-raw-statement** | **needs PromptPayload chokepoint (see §3.1)** | partial (cannot pass while feature works) | ✅ census marks the sink | — | **architectural** — see §3.1 |
| **model-generator-raw-event-message** | **needs PromptPayload (§3.1)** | partial | ✅ marks | — | **architectural** |
| model-prompt-not-a-row-but-egress | **needs PromptPayload + `--endpoint` allowlist (§3.1)** | — | ✅ marks | ✅ | **architectural** |
| inference-error-body-echo | partial (Prose/size-cap) | — | ✅ (3a) | — | strong |
| no-content-gate-anywhere | — | — | **✅ content scan (3b)** | — | defense-in-depth |
| hook-dormant-and-gitleaks-uninstalled | — | — | **✅ install-hooks + CI backstop (3c)** | — | hard (CI backstop is server-side) |
| recon-infra-json-unignored | — | — | **✅ content scan + `.claude/**` ignore** | — | defense-in-depth |
| rename-extension-bypass / git-add-f | — | — | **✅ content keyed not name (3b)** | — | defense-in-depth |
| test-fixture-pii-is-the-house-style | — | ✅ (poison set is the only PII source) | **✅ content scan + TEST_PREFIX namespace** | ✅ | defense-in-depth |
| doc-prose-leaks-infra | — | — | **✅ content scan over staged text** | — | defense-in-depth |
| gitleaks-cant-see-ed25519-seed | — | — | **✅ custom gitleaks + xtask seed regex (3b)** | — | defense-in-depth (hex-concat residual) |
| commit-message-and-branch-leak | — | — | partial (`xtask scan-msg` follow-on) | — | **discipline** |
| attestation-message-new-field-unbound | — | ✅ "every serialized field is in attestation" test | — | — | strong |
| leakage-suite-coverage-gap | — | **✅ this IS the fix (2a/2b)** | — | — | hard |
| vocabulary-snapshot-drift (private) | — | ✅ snapshot==engine test | — | ✅ | strong |
| ci-gate-runs-only-check (private) | — | — | ✅ lint+gitleaks in CI | — | strong |

### 3.1 The vectors that are NOT closable by the above — be honest

Three classes remain **architectural decisions**, not gate gaps. Stating them is part of
the methodology, not a footnote:

1. **Model / inference egress (C2)** — `ChatMessage::user(describe)` and the intake
   transcript are sent to `--endpoint` **by design**; the model needs the raw text to be
   useful. No `DeIdentified<T>` serialize-wrapper reaches a hand-built network request, and a
   poison test over the prompt body **cannot pass while the feature works**. The methodology
   takes a position: **(a)** introduce a sealed `PromptPayload` type whose constructor builds
   user/system content **only** from de-identified fields (vocabulary symptoms, enum tags,
   `case_brief()`) — `ChatMessage::user` takes `PromptPayload`, not `String` — *and/or* **(b)**
   pin `--endpoint`/`--fast-endpoint` to a **localhost/allowlisted** host with a compile-time
   default and a **runtime refusal** for non-local endpoints unless `--allow-remote-inference`
   is explicitly passed. (b) is the pragmatic minimum: it makes remote PII egress an **audited,
   explicit act** rather than a config default — critical because pointing `--endpoint` at a
   MyOwnLLM/MyOwnMesh peer is the active integration direction. This is **declared accepted-risk
   with controls**, not a hard guarantee that raw text never leaves the box.
   **BUILT — (b) done (2026-07-02, owner decision "trusted calls only").** `--endpoint` and
   `--fast-endpoint` are refused at startup on both the `diagnose` and `serve` paths when the
   host is non-loopback (`localhost` / `127.0.0.0/8` / `[::1]`) unless `--allow-remote-inference`
   is explicitly passed (`crates/support-agent/src/main.rs::validate_inference_endpoints`; the
   refusal is a fixed message that never echoes the URL). Remote inference egress is now an
   audited, explicit act, not a config default. **(a)** the sealed `PromptPayload` chokepoint
   remains the type-level follow-on (Phase 4, item 14).

2. **Unsalted FNV correlation handles (C7)** — `config_class.key()` and the fingerprint are
   emitted in the envelope and the `HttpCorpus` GET **URL**, and over an identity-bearing
   inventory key the unsalted 64-bit FNV is dictionary-reversible and a stable per-host tag in
   proxy logs. No type/content layer fixes a *reversible hash*. **Required:** switch
   `fingerprint_of`/`from_inventory` to a **keyed/salted HMAC (per-deployment salt)** and move
   retrieval keys out of logged URLs into request bodies.
   **BUILT (2026-07-04, owner salt-custody decision 2026-07-03):** `fingerprint_of` is
   HMAC-SHA256 under a per-deployment salt (`cec-fingerprint-v2`, 64-hex): the binary loads
   `CEC_FINGERPRINT_SALT` at startup like the sign-off key, refuses a salt under 16 bytes with
   a fixed no-echo message, and falls back to a documented PUBLIC cold-start default (domain
   separation only — set a real salt for the C7 property, e.g. `openssl rand -hex 32`).
   Retrieval keys travel in the `POST /v1/mappings/query` body, never the URL. Hard cutover:
   fingerprints and chain hashes changed; the private corpus re-ingests once.

3. **Commit-message / branch-name prose (C8 residual)** — an agent quoting the leaked
   identity in the message explaining the fix. The hook sees staged blobs, not log prose. A
   follow-on `xtask scan-msg` (commit-msg hook) is the only mitigation; until then this is
   **discipline**.

4. **Corpus cartography (C10)** — not closable by a type at all: a rostered caller *is*
   permitted to learn the answer to its own query, so bulk mapping of the corpus is minimized
   by differential-minimization + budget + audit + roster-is-trust policy, never eliminated.
   See `docs/corpus-cartography-threat.md` §0 for the honest limit and §3 for the control set.

---

## 4. Phased implementation plan

Each phase is independently shippable and leaves the tree green. **Phase 0 is the
highest-leverage foundation: the poison harness that would have caught D1, plus the
validating mints — because the harness gives you a red test *before* you refactor, and the
mints kill the keystone C1 class.**

### Phase 0 — Foundation: poison harness + validating mints (catches D1 at CI time)
*Highest leverage. Ship first. Independently valuable even before the type split.*
1. **`crates/leakguard`**: canonical `POISON` set + `assert_member_of_grammar(sink_bytes)`
   structural assertion (not substring). Replace the two existing local token arrays
   (`corpus-client/src/lib.rs`, the envelope test in `main.rs`) with imports of `POISON`.
2. **Self-deriving field coverage** (derive macro) over the boundary structs; drive the real
   binary under `--json`/non-`--json`, capture stdout+stderr.
3. **Validating mints** in a new `crates/de-id`: `ActionToken::parse` (dispatcher-registry
   membership), `symptom` (charset + round-trip + closed grammar), `ConfigClassKey`, `PlanId`.
   Make `de_identify_plan` **call them and return `Result`**; make `Contribution::new`
   propagate the `Err`.
4. **Verification gates:** prove the harness **fails** on (a) reverting the D1 fix and (b)
   `de_identify_plan` copying a planted `action`. Prove it **passes** on the fixed tree.

*Exit:* the D1 regression and the C1 pass-through are red in `cargo test --workspace`. No
type refactor yet — pure additive safety net + the mint predicate.

### Phase 1 — Type split + leaf typing + sealed Debug (the C1/C3 hard stops)
5. `StoredPlan`/`StoredSymptom` are the only serde corpus types; remove `Serialize` from raw
   `Plan`/`Candidate`/`Outcome`/`DiagnosticEvent`/`ToolOutcome`/`AgentRun`. Route
   `store.rs:383` write and `chain_hash` through `DeIdentified<StoredContribution>`.
6. `Prose(String)` (private, no Serialize/Display) for `title`/`description`/`rationale`/
   `message`/`summary`; fix `render_consent` to use an explicit denylisted accessor.
7. Seal/redact `Debug` on raw types. Make `Contribution` fields private; `Contribution::new`
   the only constructor. Add `trybuild` compile-fail tests pinning both.
8. **Write gate (1f):** `ensure_evidence_integrity` re-runs `de_identify_plan` + symptom
   check, asserts idempotence — closes the runtime `/mnt/e` write path.

*Exit:* `to_string(&candidate)`, struct-literal `Contribution`, and `format!("{:?}", outcome)`
all fail to compile.

### Phase 2 — Read-side re-de-id + closed dictionaries (C4/C5)
9. `from_served` re-validates every served `FixMapping`; `#[serde(try_from)]` on
   `StoredSymptom`/`ActionToken`. Poison harness drives an **adversary-seeded** served row.
10. Replace `is_stop_code_name`/`module_name` shape heuristics with **frozen dictionaries**
    (Microsoft bugcheck names; OS/driver module allowlist). Property-test the closed grammar.
11. Ban `serde_json::Value` on boundary types (`ToolOutcome.data`, `AgentStep.args` → typed).

### Phase 3 — Egress allowlist lint + content gate + live hook (C3 backstop, C8)
12. `egress` module is the only I/O site (dylint enforces, type-aware, no free-text escape).
13. `cargo xtask scan-content` (decode-and-rescan, frozen `.boundary-allow.txt`), `install-hooks`,
    required CI `boundary` job, custom gitleaks seed/row rules, `.claude/**` ignore for
    generated recon/audit/memory artifacts.

### Phase 4 — Architectural decisions (C2/C7) + policy (L4)
14. `PromptPayload` chokepoint **and/or** `--endpoint` localhost-allowlist with
    `--allow-remote-inference` audit flag. — **`--endpoint`/`--fast-endpoint` localhost-allowlist +
    `--allow-remote-inference` DONE (2026-07-02); `validate_inference_endpoints` on both the
    `diagnose` and `serve` paths. The `PromptPayload` chokepoint half remains.**
15. Keyed/salted HMAC for `fingerprint_of`/`from_inventory`; retrieval keys out of GET URLs. —
    **DONE (2026-07-04):** `cec-fingerprint-v2` keyed HMAC + `CEC_FINGERPRINT_SALT` custody +
    `POST /v1/mappings/query` body. See §3.1(2) BUILT note.
16. `CODEOWNERS` + branch protection; `AGENTS.md` Agent Contract; `xtask scan-msg` follow-on.

---

## 5. The Agent Contract

> Copy into `AGENTS.md`. This is the short rule; the gates in §2 enforce it.

**Rule 1 — Nothing reaches a sink except a `DeIdentified<_>`.** If you are about to
`serialize`, `print`, `log`, `write to a socket/file`, or build a `--json` field, the value
**must** be a `DeIdentified<T>` minted by `crates/de-id`. Raw `Plan`/`Candidate`/`Outcome`/
`Prose` do not implement `Serialize`/`Display` — if your code compiles, you are inside the
boundary; if you get `E0277: Serialize is not satisfied`, **you found a leak — mint it,
don't bypass it.**

**Rule 2 — Identity is admitted by a closed allowlist, never by shape.** New symptom/action
tokens go through `ActionToken::parse` / `symptom()` (membership + round-trip). Never widen
`extract.rs` charset/length or add a token to a dictionary without an owner review — those
files are CODEOWNERS-locked and have round-trip property tests.

**Rule 3 — Treat served and deserialized rows as untrusted.** A row from `query`/`open`/a
file is **not** de-identified for *this* process. Run it through `from_served` before you
print, re-serialize, or select it as a candidate.

**Rule 4 — The model prompt is a network egress, not a row.** Build `ChatMessage` content
**only** from `case_brief()` / vocabulary symptoms / enum tags — never `args.describe`,
`event.message`, or board serials. Do not point `--endpoint` at a non-local host without
`--allow-remote-inference`.

**Rule 5 — Never commit a corpus row, a recon artifact, a real PII token, or the sign-off
seed** — in any file extension, encoded or not, including test fixtures and docs and commit
messages. Test PII comes **only** from the `leakguard::POISON` namespace.

**The single gate that enforces the contract:** `cargo test --workspace` (poison harness +
trybuild) is the merge gate; the type system makes Rules 1/3 compile-errors; the dylint and
`cargo run -p xtask -- ci` make Rules 4/5 CI-fails; CODEOWNERS makes Rule 2 review-gated.

---

## 6. Honest assessment — guarantee vs defense-in-depth

**Hard guarantees (compile error or unbypassable CI, after Phases 0–2):**
- C1 trusted-chokepoint pass-through (validating mints + private fields + write gate).
- C3 raw-type serialization/print (split types + `Prose` + sealed `Debug`).
- C4 read-side trust (`from_served` + `try_from` deserialize).
- C9 the false-confidence test (the poison harness *is* the fix; D1 cannot ship green).

**Strong, but dictionary-dependent (only as good as the frozen lists):**
- C5 extractor shape-heuristic admission — hard **iff** the stop-code/module dictionaries are
  complete and frozen; a missing entry is a false-negative, an over-broad entry is a leak.
- C6 opaque verbatim slots — hard once routed through validated mints; `BomRevision` in the
  private repo still needs the un-implemented W6 opacity heuristic.

**Defense-in-depth (raises the bar; not a guarantee):**
- C8 git/commit content gate — best-effort over an infinite, encodable identity space; the
  real fix is L1 stopping the data being *produced* in serializable form.
- L4 CODEOWNERS — merge-time policy that degrades to advisory if branch protection is off and
  is **blind to the runtime `/mnt/e` write path** (covered only by the L1 write gate).

**Explicitly accepted-risk, not closed (C2/C7):**
- Model/inference egress carries raw PII by design; the best we buy is a `PromptPayload`
  chokepoint **and** an explicit, audited `--allow-remote-inference` act. Raw text *can* still
  leave the box if an operator opts in — that is a stated trust-boundary decision, not a bug
  the type system can prevent.
- Unsalted-FNV correlation handles need a **keyed hash** change; "hashed ≠ safe" for
  identity-bearing inputs.

**Bottom line:** this design converts the keystone C1 class and the entire easy-accidental
C3/C4/C9 surface into **hard stops at the agent's own edit site**, which the current
single-chokepoint discipline does not. It does **not** achieve "an agent literally cannot
ship a leak" in the absolute — the inference egress and the reversible-hash handles are
architectural trade-offs, and the git/policy layers are defense-in-depth. The correct claim
is: *after Phases 0–2, an agent making a normal edit cannot leak via serialization, print,
the corpus row, or a served row without a compile error or a red poison test; the residual
leak surface is reduced to (a) explicitly opting into remote inference and (b) the
dictionary/hash-keying correctness that CODEOWNERS-gated review must maintain.* That is a
defensible, decision-ready posture — and a strict improvement over a discipline that the
existing de-id test was structurally unable to verify.
