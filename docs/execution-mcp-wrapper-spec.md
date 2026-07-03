# The execution-MCP wrapper spec (normative)

**Status:** normative contract, greenlit (Lane ② item 1). **Rationale + threat model:**
`docs/test-validation-fleet-design.md` §2 (this doc is its prescriptive distillation — the *what you must
build and must never build*, with a conformance checklist). **Companion contracts now in code:** the
`SandboxValidator` "lowers-an-escalation, never mints truth" contract (`crates/swarm/src/lib.rs`) and the
execution audit log (`crates/support-agent/src/audit.rs`).

The MCP through which a diagnosis agent reaches a target machine (a client OR a volunteer) is the
highest-risk runtime surface in the system. This spec fixes the line it must hold. Keywords **MUST**,
**MUST NOT**, **MUST ONLY** are binding on any wrapper implementation.

---

## 1. The one cardinal rule

The wrapper **MUST ONLY** expose the gated verb pair `diagnose` → `execute`, over the existing two-phase
`/v1/execute` path (`serve.rs:462-630`). It **MUST NOT** expose a raw on-machine tool (`registry_set`,
`download_file`, a shell), and it **MUST NOT** speak the internal agent-loop `{tool, args}` protocol
(`agent.rs`) on the wire. The internal agent protocol is not the external MCP protocol; a wrapper that
forwards an arbitrary `{tool, args}` re-exposes the raw tool surface and is a defect.

Everything below serves this rule.

## 2. Binding invariants

1. **One gated verb pair only.** `diagnose` then `execute`. No third verb; no tool-by-name.
2. **Wrap the gates, never bypass them.** Every execution **MUST** flow through `execute_signed_plan`
   (signature re-verified at the executor, `execute.rs:14-22`) → the consent-gated `Dispatcher`
   (`dispatch.rs:104-119`) → two-phase, one-shot, TTL-bound consent with the escalation recompute bound to
   the *selected* candidate (`serve.rs:472-552`). No side door, no direct dispatch.
3. **Destructive ⇒ human sign-off, unforgeable at the boundary.** The destructive floor
   (`panel/src/lib.rs:301`, `gate.rs:156-160`) is only real if the sign-off is unforgeable on the wire. The
   wrapper **MUST NOT** self-grant `AllowDestructive`, and **MUST** treat the sign-off level as an
   authenticated field, not a bare string. (In-process today `serve.rs:466-470` trusts a `"human"` string;
   off-box that becomes a forgeable claim — see the Q7 fork, §6.)
4. **Risk is reconciled on the executing box.** The wrapper **MUST** run `reconcile_risk`
   (`dispatch.rs:63-79`) against the *target box's own* registered tool set — never trust a risk label
   computed elsewhere. Reconciliation raises an understated risk; it never lowers.
5. **Out-of-vocabulary is advisory-only.** Any action outside the registered tool vocabulary **MUST** be
   rejected at the wire boundary and recorded as `EscalatedHumanUnresolved`, never executed
   (`is_executable`, `main.rs`).
6. **The target's data is de-identified by construction before anything leaves the box.** The response
   **MUST** inherit the per-endpoint egress-sink checklist verbatim (`AGENTS.md` §2.5): vocabulary-only
   bodies, errors are fixed tokens never `Display`, no prose in logs, and a ported poison-token contract
   test. A response field not needed to *use* the answer **MUST NOT** ship.
7. **Every execution is attributable.** Each execute **MUST** emit one de-identified audit record
   (`audit::ExecutionRecord`: minted plan id + opaque run id + timestamp + outcome token + hashed caller
   key). The record **MUST NOT** carry `describe`, prose, tool output, or any raw identifier. (The skeleton
   is in code with a `NullSink` default; a remote surface wires a persistent, access-controlled sink and,
   with a caller-identity layer, fills `caller_key`.)
8. **Off-box is a deliberate, audited, encrypted act.** Reaching a machine the operator does not own is a
   *remote* surface: it **MUST** require `--allow-remote` (which arms AGPL §13, `serve.rs:175-207`) **plus** a
   mesh identity **plus** encrypted transport — the "trusted box + trusted calls + encrypted" posture the
   corpus trajectory sets, applied to execution.
9. **Never-routable capabilities stay never-routable.** Sign-off attestation, key generation, and corpus
   write **MUST NOT** be reachable through this surface (`serve.rs:28-40`, pinned by
   `router_surface_is_frozen`). This surface *adds* "no raw on-machine tool, ever" to that frozen list.

## 3. The verb contract

- **`diagnose`** — opens a one-shot, TTL-bound session; returns the `cec-diagnose/v1` envelope (de-identified:
  hashed fingerprint, vocabulary symptoms, hashed config class, per-candidate `{plan_id, max_risk,
  actions[]}`, route/consent/escalation). It **MUST NOT** return a candidate's free-text `title`/`rationale`
  or a step `description`, and **MUST NOT** return a corpus-membership signal (the `source` label was removed
  for leak-C10).
- **`execute`** — consumes the session exactly once (`sessions.remove`, `serve.rs:472-483`); recomputes the
  escalation for the *selected* candidate; maps sign-off → consent (`HumanConfirmed→AllowDestructive`,
  `VerifierConfirmed→AllowReversible`, `Unconfirmed→ReadOnlyOnly`); signs, executes, verifies, records, and
  emits the audit record. It returns the `cec-execute/v1` envelope: `{action, ok}` per step and the label
  token — **never** a step `summary` ("tool output can carry machine identity", `serve.rs:615-616`).

The verb set and the wire grammar are **frozen** the way the router is frozen: adding a verb, a route, or a
response field is a deliberate edit that **MUST** ship with the test that pins the surface.

## 4. What it MUST NOT become (anti-scope)

- Not a raw on-machine tool MCP. Not an un-consented telemetry pipe. Not a way to reach a machine without the
  sign-off/consent gate. Not a volunteer graph exposed as a queryable oracle. Not a sandbox that mints truth.
  Not attestation/keygen/seed on any network. (Full treatment: `docs/test-validation-fleet-design.md` §6.)

## 5. Conformance checklist (what an implementation must satisfy)

- [ ] Exposes exactly `{diagnose, execute}`; a third verb or a tool-by-name request is rejected at the wire
      boundary (pinned by a frozen-surface test, the twin of `router_surface_is_frozen`).
- [ ] Every execute path goes through `execute_signed_plan` → consent-gated `Dispatcher`; no direct dispatch.
- [ ] Sign-off is an authenticated field; the wrapper cannot self-grant `AllowDestructive`; destructive ⇒
      human floor holds (regression test).
- [ ] `reconcile_risk` runs on the executing box against its own tool set.
- [ ] Out-of-vocabulary actions are advisory-only, never executed.
- [ ] Response bodies are vocabulary-only; a ported poison-token test plants `leakguard::POISON` into every
      input and asserts no token survives the response.
- [ ] One de-identified `ExecutionRecord` per execute; a poison-token test asserts no raw identifier reaches
      the audit line.
- [ ] Off-box binds require `--allow-remote` + mesh identity + encrypted transport; the AGPL §13 notice fires.
- [ ] attestation / keygen / corpus-write are not routable.

## 6. Open forks that gate the *distributed* wrapper (owner/Chris)

- **Q7 — plan-provenance signing across the execution boundary.** The current per-run *symmetric* HMAC assumes
  judge == executor in one process. Off-box that breaks. Either the judge runs on the target box (HMAC stays
  in-process) or plan signing goes ed25519 with a persistent custodied judge key. Blocks the distributed
  topology. (`docs/integration-rfc-for-chris.md` Q7.)
- **Q1 — rostered identity for a caller/volunteer.** Attributability (§2.7) and off-box trust (§2.8) need a
  caller identity to hash and roster. A loopback, single-OS-user wrapper can ship without it; a distributed
  one cannot. (`docs/integration-rfc-for-chris.md` Q1.)

Until Q7/Q1 are decided, only the **loopback** wrapper (same-process judge/executor, OS-user trust boundary)
is buildable — and it already hardens the existing `serve` surface against every single-call attack in §2.
