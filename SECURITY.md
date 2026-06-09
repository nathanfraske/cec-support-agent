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
- **Sign-off gate.** `corpus-client` must refuse to submit any outcome that is
  not verifier- or human-confirmed. A way to bypass `ensure_signed_off` is a
  security issue.
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
