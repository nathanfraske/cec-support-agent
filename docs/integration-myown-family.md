# Integrating cec-support-agent into the MyOwn family

How the **cec-support-agent** engine (this repo, **AGPL-3.0**) wires cleanly into the
MyOwn family — **AllMyStuff** (the device-inventory + mesh-wiring "brain" app, MIT),
**MyOwnMesh** (the private-mesh substrate, MIT), and **MyOwnLLM** (local inference) —
without compromising the engine's standalone nature, the corpus's de-identification
boundary, or AllMyStuff's MIT license.

> Grounded in the actual code of all three repos (file paths cited). The design
> reuses patterns AllMyStuff *already* uses for the MyOwnMesh daemon, so it is
> additive, not speculative.

## Cardinal rules (non-negotiable)

1. **The engine stays standalone.** Cold-start (empty `LocalCorpus`, no endpoint, no
   mesh) keeps working. Every engine change is *additive* — a new trait with a coarse
   default, new CLI flags — never a hard dependency on AllMyStuff or MyOwnMesh.
2. **Dependency direction is a DAG: app → engine (process) → mesh.** No cycle, ever.
3. **The license boundary IS a process boundary.** See below.
4. **De-identification holds over any transport.** A corpus row is de-identified *by
   construction* before it can exist, so serving it over HTTP or the mesh cannot leak.

## The architecture

Three rings, one direction, two link-clusters separated by a **process boundary**:

```
 +----------------------------- MIT -----------------------------+
 |  AllMyStuff app (Tauri + Svelte + Rust)                       |
 |    allmystuff-inventory ─▶ inventory_to_config_keys()         |  ← de-id ALLOWLIST (app side)
 |    allmystuff-protocol::diagnose  (serde-only HAND-MIRROR)    |  ← license firewall: no cec-* dep
 |    gui/src-tauri: bundle_cec_engine_sidecar() + externalBin   |
 |        │  spawn child + stdin/stdout JSON                     |
 +--------│-----------------------------------------------------+
          │   ═════ PROCESS / RPC BOUNDARY (spawn, no link) ═════
          ▼
 +============================= AGPL ============================+
 |  cec-support-agent daemon/CLI (STANDALONE, cold-start)        |
 |    diagnose: intake→diag→candidates→judge→consent→verify→     |
 |              sign-off-gated, de-identified corpus write-back   |
 |    seams: --inventory-keys ─▶ common::InventoryProvider (NEW) |
 |           --json           ─▶ ConfigClass::from_inventory     |
 |           CorpusStore (Local|File|Http|Mesh), SandboxValidator|
 |                                                              |
 |  ── AGPL adapter crate that ships WITH the daemon ──          |
 |  corpus-mesh-adapter  ── links corpus-client (AGPL)           |
 |        │                 + myownmesh-core (MIT)               |
 +========│=====================================================+
          │  ordinary library link, entirely inside the AGPL ring
          ▼
 +-------------------------- MIT -------------------------------+
 |  MyOwnMesh: myownmesh-core (Mesh::open / Rpc::serve|call /    |
 |  Identity ed25519 / roster authorized_devices / Role::Owner) |
 +--------------------------------------------------------------+
```

The arrow **app → engine** crosses a process boundary (sidecar spawn + stdio/JSON) —
arms-length IPC + mere aggregation under AGPL, so AllMyStuff is **not** a derivative
work. The arrow **engine → mesh** is an ordinary link, but it sits entirely inside the
AGPL ring, so it relicenses nothing MIT.

## The license resolution (AGPL ↔ MIT)

Every engine crate is `AGPL-3.0-only`; AllMyStuff and MyOwnMesh are `MIT`. Cargo-linking
**any** `cec-*` crate into AllMyStuff would make the combined binary a derivative work
under AGPL (and attach §13's network-source obligation). **The clean pattern is a
process boundary, not a link — and AllMyStuff already proves it in code for the
identical case:**

- **Sidecar bundling** — `gui/src-tauri/build.rs` already stages the `myownmesh` daemon
  (`.myownmesh-rev` → `binaries/myownmesh-<triple>`) and ships it via
  `tauri.conf.json` `externalBin`. **Reuse identically:** `bundle_cec_engine_sidecar()`
  + `binaries/cec-support-agent`. The engine is a separately-distributed *process*.
- **Serde-only wire mirror** — `allmystuff-protocol` deps are `serde`/`serde_json` only;
  `control.rs` is "a hand-kept mirror of the MyOwnMesh daemon's control protocol …
  rather than depending on the engine workspace" (and MyOwnLLM does the same). **Reuse
  identically:** an `allmystuff-protocol::diagnose` module that hand-mirrors the engine's
  request/result JSON. A drift surfaces as a parse error, never a silent link.

**What keeps AllMyStuff MIT:** (a) zero `cec-*` cargo edge — enforce with a CI guard
(`cargo metadata` must contain no `AGPL-3.0-only` package); (b) the engine reached only
over the spawned-process boundary; (c) the only crate that links the AGPL engine to MIT
`myownmesh-core` (`corpus-mesh-adapter`) is itself AGPL and ships inside the engine
daemon's workspace, never in the app. The AGPL obligation lands on the **engine binary**
(correct), not on AllMyStuff. **Inventory de-id splits cleanly by side:** the
selection/allowlist (`inventory_to_config_keys`) lives app-side (MIT, depends only on
`allmystuff-inventory` + serde, emits identity-free strings); the engine keeps an
independent in-tree de-id *regression* test on its `--inventory-keys` input (it never
trusts an app-side filter). Defense on both sides of one boundary, no cross-link.

## The seams

| Seam | Mechanism | Closes / enables |
| --- | --- | --- |
| **Inventory → config_class** | App `inventory_to_config_keys()` → engine `--inventory-keys` → new `common::InventoryProvider` trait → `ConfigClass::from_inventory` | The engine's **A7/MH-6** honest-config-class gap (today just os/arch/family) — richer, real-hardware retrieval scoping |
| **Brain embedding** | AllMyStuff spawns the engine as a Tauri **sidecar**; drives `diagnose --json` over stdio; renders plan + per-step risk + consent | AllMyStuff becomes the engine's UI **without** linking AGPL — two-phase consent (no execution before human sign-off) |
| **Corpus over the mesh** | New AGPL `corpus-mesh-adapter`: `MeshCorpus` (4th `CorpusStore` impl) + `serve_corpus` over `myownmesh-core` RPC, gated on roster (read) + `Role::Owner` (write) | **W8** realized *privately* — no public endpoint; the corpus is shared only on the mesh you own |
| **Identity unification** | The device's mesh `Identity` seed → `CEC_SIGNOFF_SEED` (engine already reads it via `from_seed_hex`); matching `CEC_SIGNOFF_PUBKEY` | One ed25519 key is both mesh `DeviceId` and sign-off authority (single-operator); domain-tag-disjoint so a mesh-auth sig is never a valid attestation |
| **Inference → MyOwnLLM** | The engine's OpenAI-compatible client default loopback; any non-loopback/mesh completer behind an explicit privacy opt-in | Local inference; raw symptom prose never egresses without consent |

## ⚠️ One real engine finding (independent of the integration)

`HttpCorpus::query` (`crates/corpus-client/src/store.rs:425-453`) returns the server's
`FixMapping`s **without re-verifying anything** — no `admit()`, no attestation check.
The submit path is gated, but the *read* path trusts the corpus server entirely. Over
your own rostered mesh this is bounded by the roster, but it's a genuine trust gap: the
new `MeshCorpus` **must verify the ed25519 attestation on every row it receives on the
query path** (P3 acceptance (d)), and the same hardening should be considered for
`HttpCorpus`. Filed in `FOLLOWUPS.md`.

## Phased rollout

Each phase is independently shippable with an acceptance check; the engine stays
green and standalone throughout.

- **P0 — Engine machine-output + inventory seams** *(in-tree, AGPL, additive)*. Add
  `--json` (versioned result envelope; human output stays default) and `--inventory-keys
  <file|->`; add `common::inventory.rs` with the `InventoryProvider` trait +
  `CoarseHostInventory` (today's os/arch/family default) + `ExternalInventory`; an
  engine-side de-id regression test on the `--inventory-keys` path. **Accept:** a bare
  `diagnose` yields a byte-identical `DerivedHash` to today; cold-start unchanged; a
  planted hostname/mac in `--inventory-keys` survives only as a one-way hash. *(depends: none)*
- **P1 — App-side de-id allowlist + serde-only diagnose contract** *(MIT, in AllMyStuff)*.
  `inventory_to_config_keys()` (KEEP allowlist; explicit DROP of hostname/mac/ip/serial;
  memory bucketed); `allmystuff-protocol::diagnose` serde-only mirror. **Accept:**
  `cargo metadata` shows no AGPL package / no `cec-*` dep; a fixture seeding
  hostname/mac/ip/serial appears in **zero** emitted config keys. *(depends: P0)*
- **P2 — Sidecar bundle + driver command** *(MIT, in AllMyStuff)*.
  `bundle_cec_engine_sidecar()` (clone of the myownmesh one; pinned `.cec-engine-rev`;
  zero-byte stub + `CEC_ENGINE_BIN` override); a `diagnose_run` Tauri command (two-phase:
  plan, then `--sign-off` on human consent); Svelte UI. **Accept:** the app bundles and
  spawns the engine over the process boundary; no execution before consent; a build with
  no engine binary still compiles and simply never advertises `diagnose` (graceful
  degrade); the CI guard fails if any AGPL package enters the graph. *(depends: P0, P1)*
- **P3 — `corpus-mesh-adapter`** *(AGPL, ships with the engine daemon)*. `MeshCorpus`
  (query re-verifies `ensure_attested` on every received row; submit calls `admit()`
  before the wire); `serve_corpus` gating read on roster + write on `Role::Owner`, backed
  by `FileCorpus.with_authority(pubkey)`. **Accept:** cold-start stays green (serving is
  opt-in); a stranger is refused read; a rostered non-owner is refused write; a forged
  `HumanConfirmed` is refused by `admit()` even from an owner; a query row with an invalid
  attestation is rejected (read-path gap closed); the leakage suite shows zero seeded
  identifier in the on-wire JSON; the adapter is absent from AllMyStuff's graph. *(depends: P0)*
- **P4 — Identity unification + egress policy** *(optional, link-free)*. Wire the mesh
  `Identity` seed → `CEC_SIGNOFF_SEED`; keep inference loopback by default, mesh/non-loopback
  behind a privacy opt-in; add the three CI guards (no AGPL in the app; engine builds with
  no corpus/endpoint/mesh; no `engine → allmystuff/myownmesh` edge / no cycle). **Accept:**
  `hex(mesh seed) → from_seed_hex → public_key` matches the mesh `DeviceId` expectation;
  domain-tag disjointness (a mesh-auth sig is never a valid attestation); inference defaults
  loopback. *(depends: P3)*

## Invariants preserved

Standalone/cold-start engine · DAG `app → engine(process) → mesh` · AllMyStuff stays MIT
(zero `cec-*` edge, process boundary) · de-identification structural + pre-transport ·
corpus read=rostered / write=`Role::Owner` on the wire · sign-off asymmetry (engine holds
only the public key) · human-in-the-loop consent floor for destructive steps · graceful
degrade when no engine binary is bundled.

## Open questions (need the owner's call)

1. **Single-shot CLI vs persistent daemon** — P2 drives a per-diagnose child
   (`diagnose --json`, nothing to orphan, simplest); a long-lived daemon is faster but
   adds lifecycle. Recommend single-shot first.
2. **Result-envelope versioning** — `--json` becomes a contract the moment AllMyStuff
   parses it; confirm it carries a `schema_version` and a compatibility policy.
3. **Identity-unification scope** — sharing one ed25519 seed across the mesh `DeviceId`
   and the sign-off authority is clean for a *single operator* who is both; a split
   deployment keeps them separate. Which model?
4. **Inference egress over the mesh** — raw symptom free-text is **not** de-identified;
   is fanning inference to a peer's MyOwnLLM over the mesh ever desired, or loopback-only?
5. **`myownmesh-core` pin coordination** — the adapter must git-tag-pin the **same**
   `myownmesh-core` tag AllMyStuff uses; confirm a single source of that pin.
6. **SandboxValidator over the mesh** — a `MeshSandboxValidator` (a disposable mesh node
   supplying positive validation evidence to *lower* an escalation) — in scope or later?
7. **Tail-truncation residual** — serving over the mesh inherits the keyless-chain tail
   gap (FOLLOWUPS); the committed anchor covers it for a file, but a mesh peer needs the
   anchor too.
