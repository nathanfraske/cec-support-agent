# Security Policy

## Reporting a vulnerability

Please report security vulnerabilities privately. Use GitHub's **"Report a
vulnerability"** feature (Security → Advisories) on this repository, or email the
maintainers, rather than opening a public issue.

Include enough detail to reproduce: affected crate, version or commit, platform,
and a minimal proof of concept if you have one. We aim to acknowledge reports
within a few business days and will coordinate a fix and disclosure timeline with
you.

## Scope

This repository is the **open engine** only. The private corpus, the corpus
service, the firmware, and the PCB/FEA tooling live in separate repositories and
are out of scope here.

Security-relevant invariants this repo upholds, and which a report may concern:

- **No corpus data or model weights** are present in the tree or history. A
  finding of leaked corpus data or weights is a security issue.
- **Evidence-integrity gate.** `corpus-client` must refuse to admit any row that
  fails the truth-admission gate (`ensure_evidence_integrity`, the gate formerly
  named `ensure_signed_off`). It must be sign-off confirmed (verifier or human),
  and a **resolved** outcome must carry a **matching passing verification verdict**
  and — when its plan is destructive — **human** sign-off. A way to admit an
  unconfirmed row, a resolved row with no/contradicting verdict, or a destructive
  "fix" without human sign-off is a security issue. The sign-off itself is
  cryptographically attested (ed25519): a store configured with
  `.with_authority(pubkey)` refuses any confirmed row whose attestation is
  missing or invalid — a constructed `HumanConfirmed` enum cannot pass, at
  submit or at `open`-time re-admission. A bypass of `ensure_attested`, the
  hash-chain tamper-evidence, or owner-only revocation is likewise a security
  issue; see `docs/evidence-integrity-and-research-checklist.md` §9.
- **Consent gate.** `agent-core` must not run a state-changing tool without
  consent appropriate to its risk. A bypass is a security issue.
- **No mandatory outbound connection.** The engine must cold-start with no
  CEC-hosted service. Unexpected exfiltration or hidden network calls are a
  security issue.
- **Never-routable capabilities.** The `serve` API (`crates/support-agent/src/serve.rs`)
  exposes exactly three routes — `GET /v1/health`, `POST /v1/diagnose`, `POST /v1/execute`
  — frozen by the `router_surface_is_frozen` pinning test. Three capabilities MUST
  NEVER be reachable over the socket: sign-off **attestation** and **key generation**
  (`gen-signoff-key`), and corpus **write** (submit). The evidence-integrity keystone
  is the asymmetric split — the authority holds the ed25519 seed and the engine embeds
  only the public key — so a network-reachable attest/keygen oracle, or a corpus-write
  endpoint reachable without a rostered owner identity, would let anyone who reaches the
  socket mint or admit forged `HumanConfirmed` rows. A route (or any other change) that
  makes attestation, key generation, or corpus write network-reachable is a security issue.
- **Inference-egress opt-in.** A non-loopback `--endpoint`/`--fast-endpoint` (the
  model-inference egress carries raw request prose) is refused at startup on both the
  `diagnose` and `serve` paths unless `--allow-remote-inference` is explicitly passed.
  A silent path that sends request prose to a non-loopback host without that opt-in is a
  security issue.

## Network exposure and AGPL §13

The `serve` API is **hard-loopback by default**. Remote exposure is **mesh-only**:
there is no bearer-token auth tier, and none will be built — a non-loopback bind is
authenticated by MyOwnMesh rostered identity, not a shared secret. `--allow-remote`
is the deliberate, audited act that permits a non-loopback bind; on startup it prints
a one-line notice, because **binding beyond loopback makes this a network service, and
AGPL-3.0 §13 then requires offering that service's users the Corresponding Source** of
the engine they interact with. The auth posture and the §13 source-offer duty move
together: the same flag that opens the surface arms the obligation. Loopback,
single-operator use is not "remote network interaction" and triggers nothing.

## Supported versions

The project is pre-1.0; security fixes target the `main` branch. Pin a commit if
you need stability.

## Secret handling

Never commit secrets. CI runs `gitleaks`, and a pre-commit hook runs
`gitleaks protect`. If you believe a secret was committed, rotate it immediately
and report it through the channel above.
