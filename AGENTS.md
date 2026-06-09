# Working rules for this repo
- Preserve the invariants in section 0 of the bootstrap doc.
- Never add corpus data, fixtures derived from it, or model weights.
- Keep the engine cold-startable: no CEC service required to build, test, or run.
- All model access goes through crates/inference over HTTP. Do not hardwire a provider.
- corpus-client must reject any contribution that is not sign-off confirmed.
- Run cargo fmt and clippy -D warnings and the test suite before every commit.
