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

## Supported versions

The project is pre-1.0; security fixes target the `main` branch. Pin a commit if
you need stability.

## Secret handling

Never commit secrets. CI runs `gitleaks`, and a pre-commit hook runs
`gitleaks protect`. If you believe a secret was committed, rotate it immediately
and report it through the channel above.
