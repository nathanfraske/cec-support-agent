# cec-support-agent

The open Rust engine behind the CEC support agent and the AllMyStuff brain.

It turns a support request into a diagnosed fault, generates candidate
remediation plans, scores them with a judge panel, and stops at a sign-off gate
before anything touches a real machine. The engine is a Cargo workspace of small
library crates plus one headless CLI; the host app (MyOwnLLM) and the AllMyStuff
brain embed the libraries, and `support-agent` is the CLI face.

```
support request → collect diagnostics → generate candidate plans (swarm)
   → judge panel (score, escalate, pick best) → execute winning plan (consented)
   → verify outcome → sign-off → write to corpus (de-identified)
```

## Open engine, private corpus

This repository is the **open engine only**. The **private corpus** — the
fault-signature→fix mappings and the accumulated outcomes — and any model
weights derived from it live in a **separate, private repository and
infrastructure**. They are never present here, in any form, including fixtures,
dumps, or git history. This repo ships only the corpus *client* and the *schema*
(`crates/corpus-client`). The pre-commit hook and `.gitignore` actively block
corpus data and model weights from entering the tree.

## Runtime invariants

The engine holds to six runtime invariants. They are enforced in code and CI,
not just documented:

1. **Open engine, private corpus.** No corpus data or derived weights exist in
   this repository, ever. Only the corpus client and schema live here.
2. **Cold start.** The engine builds, tests, and runs with an empty (or
   self-hosted) corpus and a local inference endpoint. No CEC-hosted service is
   required for any of those.
3. **Inference over HTTP.** Models are reached through an OpenAI-compatible HTTP
   client (`crates/inference`). There is no build-time dependency on a specific
   model, provider, or GPU runtime.
4. **Self-host parity.** Every hosted convenience has a local equivalent. No
   outbound connection is mandatory.
5. **Sign-off gate.** `corpus-client` refuses to submit any outcome that is not
   verifier-confirmed or human-confirmed. This is enforced in code
   (`ensure_signed_off`), not in documentation.
6. **Static, cross-platform binaries.** Rust on the stable toolchain, built to
   static binaries for Windows, Linux (x86_64 and aarch64/Pi), and macOS
   (x86_64 and Apple Silicon).

## Workspace layout

| Crate | Role |
| --- | --- |
| `common` | Shared types: `Plan`, `Candidate`, `FaultSignature`, `DiagnosticEvent`. |
| `inference` | OpenAI-compatible HTTP client with a pluggable endpoint. |
| `agent-core` | The `Tool` trait, a consent-gated dispatcher, and the execution loop. |
| `tools-windows` | Windows tools (CIM, event log/WER, registry-with-backup); a stub off-Windows. |
| `panel` | Judge, plan-level best-of-N, scoring axes, escalation ladder. |
| `swarm` | Trusted-node dispatch and sandbox-VM validation coordination. |
| `corpus-client` | Corpus API client + schema + cold start; enforces the sign-off gate. |
| `support-agent` | Binary: assembles the pipeline; the CLI and the embeddable entry point. |

The seven libraries are the embeddable surface; `support-agent` is the headless
CLI. `tools-windows` compiles everywhere — its Windows implementations sit
behind `#[cfg(windows)]` so `cargo build --workspace` succeeds on every host.

## Quick start

```bash
# Build, lint, and test the whole workspace.
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo build --workspace
cargo test --workspace
```

### Cold start (no corpus, no live model)

The agent runs the full diagnostic → candidate → judge pipeline with no corpus
and no endpoint, using a deterministic, model-free heuristic candidate:

```bash
cargo run -p support-agent -- diagnose \
  --describe "explorer.exe crashes on login with WER bucket 0x1234" --offline
```

### Cold start against a local OpenAI-compatible endpoint

Point it at any local server that speaks the OpenAI Chat Completions API
(llama.cpp, vLLM, LM Studio, Ollama, …). No CEC service is involved:

```bash
cargo run -p support-agent -- diagnose \
  --describe "explorer.exe crashes on login" \
  --endpoint http://localhost:8080/v1 --model my-local-model
```

If the endpoint is unreachable, the run continues with the heuristic candidate.

## Development

A pre-commit hook runs `cargo fmt --check`, `gitleaks protect`, and a guard that
blocks corpus data and model weights:

```bash
git config core.hooksPath scripts/githooks
chmod +x scripts/githooks/pre-commit
```

CI mirrors the local gates: `fmt`, `clippy -D warnings`, `build`, and `test`
across Linux/Windows/macOS, plus `cargo deny check` (licenses + advisories) and
a gitleaks secret scan. See `AGENTS.md` for the working rules and `CONTRIBUTING.md`.

## License

AGPL-3.0-only. See [LICENSE](LICENSE) and [NOTICE](NOTICE).

This is the engine-protection lever: the GNU AGPL's network-use clause
(section 13) means anyone who runs a modified version of this engine as a
network service must offer that service's users the corresponding source. The
private corpus and any derived model weights are separate works under separate
terms and are not covered by this license.
