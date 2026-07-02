export const meta = {
  name: 'myown-integration-design',
  description: 'Map the real APIs of AllMyStuff + MyOwnMesh and design a clean integration plan for the cec-support-agent engine (inventory seam, brain embedding over an AGPL-safe process boundary, corpus-over-mesh, identity/sign-off)',
  phases: [{ title: 'Map' }, { title: 'Design' }, { title: 'Synthesize' }],
}

const ENGINE = '/home/nathan/CEC_AutoDiagnoser'
const AMS = '/tmp/myown/AllMyStuff'
const MESH = '/tmp/myown/MyOwnMesh'

const CTX = `You are designing how the **cec-support-agent** diagnostic engine integrates cleanly into the "MyOwn family". Read the ACTUAL code in these local checkouts:
- ENGINE (cec-support-agent, AGPL-3.0): ${ENGINE} — a Rust workspace (lib crates + the support-agent binary). Pipeline: intake → diagnostics → candidate plans → judge panel → consent-gated execution → verify → sign-off-gated, de-identified corpus write-back. Retrieval-first from an inverted corpus of attested (FaultSignature, Plan, OutcomeLabel) rows. Trait/extension seams already exist: \`host_inventory()\`/\`host_config_class()\` (support-agent/src/main.rs), the \`CorpusStore\` trait with \`FileCorpus\`/\`HttpCorpus\` impls + the \`/v1/mappings\` + \`/v1/contributions\` HTTP contract (corpus-client/src/store.rs), ed25519 sign-off attestation (corpus-client + provenance), a \`SandboxValidator\` trait (swarm), and an OpenAI-compatible inference client (the \`inference\` crate). The private corpus lives OUTSIDE this repo (a separate off-tree repo) — only the client+schema are here; de-identification is by structured extraction (free text never reaches a row).
- AllMyStuff (MIT): ${AMS} — a Tauri+Svelte+Rust desktop "brain" app that inventories every device on your machines and wires them across a mesh. Crates: allmystuff-inventory (cross-platform device/hardware inventory — linux/macos/windows + types), allmystuff-bridge, allmystuff-graph, allmystuff-cli, allmystuff-protocol, allmystuff-session, allmystuff-updater. Built on MyOwnMesh. Does NOT yet reference cec-support-agent — this integration is greenfield. Its README calls cec-support-agent "the engine behind the AllMyStuff brain."
- MyOwnMesh (MIT): ${MESH} — a pure-Rust private mesh you embed. Crates: myownmesh-core (src/identity.rs = cryptographic device identity; src/protocol/{rpc,governance,handshake,topology,keepalive,features}.rs; src/services/), myownmesh-services (STUN/TURN), myownmesh-signaling (Nostr), myownmesh. Model: cryptographic device identity + "authorization not authentication" (is this mine, or am I sharing).

THE GOAL: a clean integration plan. CLEAN means, non-negotiably:
1. **The engine stays STANDALONE** — cold-start, no mesh/app required (an existing invariant). Integration is via trait seams + adapters, never by the engine hard-depending on AllMyStuff/MyOwnMesh.
2. **Dependency direction app → engine → mesh**, NEVER a cycle.
3. **LICENSE: the engine is AGPL-3.0; AllMyStuff/MyOwnMesh are MIT.** Statically linking an AGPL lib into an MIT app makes the combined work AGPL — so judge whether the clean pattern is the app driving the engine over a PROCESS/RPC boundary (the engine as a daemon/CLI) rather than linking. Be concrete about what keeps AllMyStuff MIT.
4. **No corpus leak** — de-identification + the private-corpus boundary hold even when the corpus is served/shared over the mesh.`

const MAP_SCHEMA = {
  type: 'object', additionalProperties: true,
  properties: {
    area: { type: 'string' },
    key_apis: { type: 'array', items: { type: 'object', additionalProperties: true, properties: { name: { type: 'string' }, file: { type: 'string' }, what: { type: 'string' } }, required: ['name', 'what'] } },
    data_models: { type: 'array', items: { type: 'string' } },
    integration_surface: { type: 'array', items: { type: 'string' }, description: 'Concrete seams this area exposes for the integration (with file:line).' },
    constraints: { type: 'array', items: { type: 'string' } },
  },
  required: ['area', 'key_apis', 'integration_surface'],
}

phase('Map')

const maps = await parallel([
  () => agent(`${CTX}\n\nMAP AllMyStuff. Read ${AMS}: its top Cargo.toml, ARCHITECTURE.md, and especially crates/allmystuff-inventory/src/{types,lib,report,sys,linux,windows,macos}.rs (the device/hardware INVENTORY data model — this is the real source the engine's host_inventory()/config_class wants), crates/allmystuff-bridge (what it bridges) and allmystuff-cli. Report: the inventory data model (what fields/devices it captures), how the app is structured (GUI ↔ core ↔ bridge), and the concrete surface where AllMyStuff would (a) FEED inventory to the engine for config_class + diagnostics, and (b) DRIVE the engine's diagnose pipeline as "the brain". Cite file paths.`, { label: 'map:allmystuff', phase: 'Map', schema: MAP_SCHEMA }),
  () => agent(`${CTX}\n\nMAP MyOwnMesh. Read ${MESH}: ARCHITECTURE.md, CONNECTION-ENGINE.md, crates/myownmesh-core/src/lib.rs + identity.rs + protocol/{rpc,governance,handshake,topology}.rs + services/, and crates/myownmesh-core/README.md. Report: the EMBEDDING API (how a Rust program embeds the mesh and EXPOSES or CONSUMES an RPC service over it — the corpus would be served this way), the cryptographic DEVICE-IDENTITY model (identity.rs — could be the engine's "producing machine" attestation + the sign-off authority), and the AUTHORIZATION/GOVERNANCE model (governance.rs — "is this mine, or sharing" — how it maps to the corpus sign-off gate's HumanConfirmed/owner-authorization). Cite file paths.`, { label: 'map:myownmesh', phase: 'Map', schema: MAP_SCHEMA }),
  () => agent(`${CTX}\n\nMAP the ENGINE's embedding surface + license. Read ${ENGINE}: the trait seams — host_inventory()/host_config_class() (crates/support-agent/src/main.rs), the CorpusStore trait + HttpCorpus + the /v1 contract (crates/corpus-client/src/store.rs), the SandboxValidator trait (crates/swarm), the inference client (crates/inference), and which crates are the "embeddable surface" vs the binary (README). Read the LICENSE file + any SPDX headers. Report: exactly which seams an external app/mesh plugs into (as traits/HTTP), what's library vs binary, the cold-start/standalone invariant, and the AGPL constraint (what linking vs process-boundary means for an MIT consumer). Cite file:line.`, { label: 'map:engine', phase: 'Map', schema: MAP_SCHEMA }),
])

const mapText = JSON.stringify(maps.filter(Boolean), null, 1)
log(`Mapped ${maps.filter(Boolean).length}/3 areas`)

const DESIGN_SCHEMA = {
  type: 'object', additionalProperties: true,
  properties: {
    lens: { type: 'string' },
    approach: { type: 'string' },
    seams: { type: 'array', items: { type: 'object', additionalProperties: true, properties: { from: { type: 'string' }, to: { type: 'string' }, mechanism: { type: 'string' }, detail: { type: 'string' } }, required: ['mechanism', 'detail'] } },
    risks: { type: 'array', items: { type: 'string' } },
    steps: { type: 'array', items: { type: 'string' } },
  },
  required: ['lens', 'approach', 'seams'],
}

phase('Design')

const LENSES = [
  { key: 'inventory-diagnostics', prompt: `LENS: the INVENTORY + diagnostics seam. Design how AllMyStuff's allmystuff-inventory feeds the engine: a trait the ENGINE defines (e.g. an InventoryProvider) that an allmystuff adapter implements, replacing host_inventory()'s os/arch/family with real device inventory → honest config_class (closes the engine's A7/MH-6 gap) and richer diagnostic signals. Specify the adapter crate, the data mapping (AllMyStuff inventory → the engine's normalized inventory entries / config_class), and how it stays optional (cold-start unchanged).` },
  { key: 'brain-embedding-license', prompt: `LENS: brain embedding + the AGPL↔MIT boundary. Design how AllMyStuff (MIT) drives the engine (AGPL) WITHOUT becoming AGPL. Decide: engine as a separate DAEMON/CLI the app talks to over IPC/local-RPC (process boundary = no AGPL propagation into the MIT app), vs a linked lib (would force AGPL). Specify the API contract (the diagnose→candidates→judge→sign-off pipeline exposed as commands/RPC), how the app surfaces consent + sign-off to the user, and how the engine's binary is distributed alongside the app. Be concrete and correct about the license boundary.` },
  { key: 'corpus-over-mesh', prompt: `LENS: the corpus + mesh. Design serving the PRIVATE corpus over MyOwnMesh as an RPC service (the W8 HttpCorpus /v1 contract realized over the mesh — no public endpoint; "shared only on the mesh you own"). Map: MyOwnMesh device-identity → the engine's config_class/provenance "producing machine" attestation + the ed25519 sign-off authority; MyOwnMesh governance/"authorization not authentication" → the sign-off gate's HumanConfirmed / propose-then-authorize. CRITICAL: de-identification + the private-corpus boundary must still hold when rows cross the mesh. Specify the mesh service crate + the CorpusStore-over-mesh adapter.` },
  { key: 'redteam-governance', prompt: `LENS: red-team the whole integration. Enumerate the traps + the fix for each: (1) LICENSE — any path where AGPL propagates into AllMyStuff/MyOwnMesh (static link, vendoring, shared crate), and the exact boundary that prevents it; (2) DEP CYCLE — any app→engine→mesh→app cycle; keep it a DAG; (3) the STANDALONE-ENGINE invariant — anything that would make the engine require the mesh/app; (4) CORPUS LEAK over the mesh — identity in inventory-derived config_class, a mesh peer reading raw rows, the de-id boundary; (5) IDENTITY/KEY mapping — conflating the mesh device identity with the sign-off authority key incorrectly; (6) MyOwnLLM inference seam pitfalls. Return concrete defenses as risks + seams.` },
]

const designs = await parallel(LENSES.map((l) => () =>
  agent(`${CTX}\n\nThe three area maps:\n${mapText}\n\nYOUR ${l.prompt}\n\nGround every seam in the real code (cite file paths). Return the structured design.`,
    { label: `design:${l.key}`, phase: 'Design', schema: DESIGN_SCHEMA })
))

log(`Designed ${designs.filter(Boolean).length}/${LENSES.length} lenses`)

phase('Synthesize')

const PLAN_SCHEMA = {
  type: 'object', additionalProperties: true,
  properties: {
    summary: { type: 'string' },
    architecture: { type: 'string', description: 'The clean dependency graph + boundaries (app → engine[process] → mesh; trait-seam adapters), as prose + an ASCII sketch.' },
    license_resolution: { type: 'string' },
    seams: { type: 'array', items: { type: 'object', additionalProperties: true, properties: { seam: { type: 'string' }, mechanism: { type: 'string' }, where: { type: 'string' }, closes: { type: 'string' } }, required: ['seam', 'mechanism'] } },
    phases: { type: 'array', items: { type: 'object', additionalProperties: true, properties: { phase: { type: 'string' }, deliverable: { type: 'string' }, depends_on: { type: 'string' }, acceptance: { type: 'string' } }, required: ['phase', 'deliverable'] } },
    open_questions: { type: 'array', items: { type: 'string' } },
    invariants_preserved: { type: 'array', items: { type: 'string' } },
  },
  required: ['summary', 'architecture', 'license_resolution', 'seams', 'phases'],
}

const plan = await agent(
  `${CTX}\n\nThe three maps:\n${mapText}\n\nThe four lens designs:\n${JSON.stringify(designs.filter(Boolean), null, 1)}\n\nSYNTHESIZE ONE clean, phased integration plan. Resolve conflicts explicitly. It MUST: keep the engine standalone; keep deps a DAG (app→engine→mesh); resolve the AGPL↔MIT boundary correctly (be precise); preserve de-identification + the corpus boundary over the mesh. Give the dependency/boundary architecture (with an ASCII sketch), the concrete seams (each: mechanism + where + what engine gap it closes), an ORDERED phase plan (each: deliverable + depends-on + acceptance check), the invariants preserved, and the open questions that need the owner's decision.`,
  { label: 'synthesize', phase: 'Synthesize', schema: PLAN_SCHEMA }
)

return { plan, maps: maps.filter(Boolean), designs: designs.filter(Boolean) }
