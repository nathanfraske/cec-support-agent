# cec-support-agent

The open Rust engine behind the CEC support agent and the AllMyStuff brain.

It turns a support request into a diagnosed fault, generates candidate
remediation plans, scores them with a judge panel, and stops at a sign-off gate
before anything touches a real machine. The engine is a Cargo workspace of small
library crates plus one headless CLI; the host app (MyOwnLLM) and the AllMyStuff
brain embed the libraries, and `support-agent` is the CLI face.

```
support request → intake interview (map input to a case, ask follow-ups)
   → collect diagnostics → generate candidate plans (swarm)
   → judge panel (route, score, escalate, pick best) → execute winning plan (consented)
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
| `common` | Shared types: `Plan`, `Candidate`, `FaultSignature`, `ConfigClass`, `DiagnosticEvent`; structured symptom extraction. |
| `inference` | OpenAI-compatible HTTP client with a pluggable endpoint. |
| `intake` | Intake judge: maps a support request to a structured case, asking the standard helpdesk follow-ups only for what the description leaves open. |
| `provenance` | Plan signing: the judge signs winning plans, the executor verifies before any step runs. |
| `agent-core` | The `Tool` trait, a consent-gated dispatcher, the execution loop, and outcome verification. |
| `tools-windows` | Windows tools (CIM, event log/WER, registry-with-backup, verified restore points); a stub off-Windows. |
| `panel` | Routing taxonomy (software / hardware-evidenced / ambiguous), judge, plan-level best-of-N, scoring axes, escalation ladder. |
| `swarm` | Trusted-node dispatch with hypothesis fan-out and sandbox-VM validation coordination. |
| `corpus-client` | Corpus API client + schema (outcome labels, config classes) + cold start; enforces the sign-off gate and de-identification. |
| `support-agent` | Binary: assembles the pipeline; the CLI and the embeddable entry point. |

The nine libraries are the embeddable surface; `support-agent` is the headless
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

The agent runs the full diagnostic → route → candidate → judge pipeline with no
corpus and no endpoint, using a deterministic, model-free heuristic candidate:

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

With an endpoint configured, the swarm fans out one generator per causal
hypothesis (driver regression, state corruption, configuration interaction) so
the slate is hypothesis search, not variants of one guess. If the endpoint is
unreachable, the failed generators are noted and the run continues with the
heuristic candidate.

### Model tiers: simple requests sample a lighter model

`--fast-model` (and optionally `--fast-endpoint`) names a lighter model for
simple requests, so routine work does not queue behind a heavyweight reasoning
model. Two things ride the fast tier: intake question phrasing (always — it is
the lightest, most latency-sensitive call in the pipeline), and plan
generation when the ticket is *routine* — the route is software-state and the
intake interview established every case field. Anything vague (unestablished
fields), novel (ambiguous route), or physical (hardware-evidenced) stays on
`--model`: novelty escalates, in model choice as in sign-off. With no
`--fast-model`, everything uses `--model` as before.

### Closing the loop: execute, verify, label, record (sign-off gated)

By default the run stops once the judge picks a winner. Passing `--sign-off`
executes the winning plan through the consent-gated dispatcher, verifies the
outcome by diffing a re-collected signature against the original failure
signature, emits an outcome label (`resolved.confirmed`,
`resolved.provisional`, `escalated.hardware` with a part class,
`escalated.human-unresolved`, …), and records the de-identified triple through
the corpus sign-off gate:

```bash
cargo run -p support-agent -- diagnose \
  --describe "explorer.exe crashes on login" --offline --sign-off human
```

The sign-off level authorizes execution at a matching consent level — `verifier`
permits reversible steps, `human` permits destructive ones — and the consent
gate still refuses any step that exceeds it. The sign-off must also meet the
judge's required escalation: a hardware-evidenced or ambiguous route, and any
state-changing plan with no sandbox validation report, require `human`, so a
`verifier` sign-off is refused for those runs. The corpus, in turn, refuses any
outcome that is not verifier- or human-confirmed (`ensure_signed_off`). On a
non-Windows host the Windows tools report "unsupported", so execution halts at
the first step and the run is labeled unresolved rather than claimed fixed; on
Windows it proceeds.

### Intake: people don't like to explain things

The first stage is an intake judge (`crates/intake`) implementing step 1 of
the standard troubleshooting methodology (CompTIA A+: question the user,
identify symptoms, determine what changed). It infers everything it can from
the initial description — onset, recent change, reproducibility, scope — and,
when a terminal is attached, asks the classic helpdesk follow-ups *only* for
what is still open (at most 5, never repeated; `--no-questions` opts out;
headless runs never prompt). Every answer is run through structured
extraction, so an error code typed at the prompt lands in the fault signature
and can re-route the case. The case then drives the pipeline: reproducibility
picks the verification class (an intermittent fault is paroled, never
confirmed, by one clean re-collection), and the interview findings prime the
hypothesis generators.

The interview *structure* is deterministic and model-free, so it works at cold
start: which field is asked about, in what order, and when the funnel stops is
never up to a model. With an endpoint configured, a model-backed interviewer
(`ModelInterviewer`) only sharpens the *wording* of each question using the
case so far and the transcript; any error, timeout (the intake client uses a
short one), or non-question reply falls back to the scripted prompt, so a slow
or dead endpoint degrades to cold-start behavior instead of stalling the
interview.

### Judge-signed plans, rendered consent, bounded retry

Three more gates close the loop in code. **Provenance** (`crates/provenance`):
the judge signs the winning plan (HMAC-SHA256 over its canonical JSON) and
`execute_signed_plan` re-verifies the signature at the executor — a plan that
was tampered with, or never passed the judge, is refused before any tool runs.
Plans whose steps fall outside the agent's operation vocabulary (the
registered tools) are advisory-only and never executed. **Consent** is to a
rendered plan, never an opaque script: plain-language steps, risk classes, and
the restore-point coverage boundary, confirmed interactively (declining
withdraws the ticket). **Retry** is bounded: a failed attempt is recorded as a
hard negative and the next-best executable plan gets one chance, then the
ticket escalates instead of thrashing.

### Board identity and firmware advisories

When the evidence implicates the platform or a driver, the agent reads the
board identity over CIM (`board_info` — configuration fields only, never
serial numbers or service tags) and emits a firmware advisory: the exact
board, the installed BIOS version and date, the vendor's stable download
page (ASUS/MSI/Gigabyte/ASRock by board; Dell/HP/Lenovo by system model),
the exact model string to search, and numbered plain-language steps. A
consent-gated `download_file` tool (HTTPS-only, confined to
`Downloads\cec-support`, reports SHA-256) lets judge-signed plans fetch
files; *installing* what was fetched is a separate concern, and flashing
firmware is advisory-only by design — the agent never executes it.

### The flywheel: persistent corpus and retrieval-first

`--corpus <PATH>` swaps the in-memory store for a file-backed one (one JSON
row per line, gate-checked before anything touches disk). Outcomes persist
across runs, so the next run facing a known signature at the same config
class is served *retrieval-first*: confirmed precedents join the slate as
`CorpusPrimed` candidates (which the judge prefers under corpus priors), de
novo model generation is skipped, and hard negatives stay in the rows without
ever being offered as fixes.

### De-identification by structured extraction

A corpus row never carries free text. Fault signatures are built by structured
extraction (`common::extract_symptoms`): only a fixed fault vocabulary,
`0x`-hex codes, id-bearing terms (`event_41`, `xid_79`), and bare module names
survive — hostnames, usernames, paths, and serials are dropped because they
never match a rule. `Contribution::new` likewise strips plans to their action
vocabulary before anything can be submitted, and every row is scoped to a
`ConfigClass` (BOM revision when present, else a derived inventory hash). An
adversarial leakage test suite seeds known identifiers through the whole path
and asserts zero leakage into serialized corpus rows.

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
