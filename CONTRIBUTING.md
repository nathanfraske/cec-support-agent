# Contributing

Thanks for helping build the open engine behind the CEC support agent.

## Before you start

Read `AGENTS.md` and section 0 of the bootstrap doc. The invariants there are
non-negotiable; a change that breaks one will not be merged. In particular:

- **Never** add corpus data, fixtures derived from it, or model weights — not in
  the tree, not in fixtures, not in git history. The pre-commit hook and
  `.gitignore` enforce this, but treat it as your responsibility too.
- Keep the engine **cold-startable**: it must build, test, and run with no
  CEC-hosted service and no mandatory outbound connection.
- All model access goes through `crates/inference` over HTTP. Do not hardwire a
  provider, model, or GPU runtime anywhere else.
- `corpus-client` must reject any contribution that is not verifier- or
  human-confirmed. Keep the gate in code.

## Local setup

```bash
rustup toolchain install stable
rustup component add rustfmt clippy
git config core.hooksPath scripts/githooks
chmod +x scripts/githooks/pre-commit
```

## Before every commit

The CI gates are reproduced locally; run them before you push:

```bash
cargo fmt --all
cargo clippy --workspace --all-targets -- -D warnings
cargo build --workspace
cargo test --workspace
cargo deny check        # licenses + advisories (cargo install cargo-deny --locked)
gitleaks detect         # no secrets
```

## Pull requests

- One focused change per PR; keep the diff readable.
- New crates follow the existing dependency direction: `support-agent` depends on
  all libraries; `agent-core` on `common` and `inference`; `tools-windows` on
  `common` and `agent-core`; `panel`, `swarm`, and `corpus-client` on `common`.
- Add tests for new behavior. Tests must not require network access or a live
  model — mock the `Completer` / `CorpusStore` traits instead.
- `main` is protected: PRs require the CI checks to pass and one review.

## Reporting security issues

See [SECURITY.md](SECURITY.md). Do not open public issues for vulnerabilities.
