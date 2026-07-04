# The corpus boundary

This repository is the **open engine**. The **private corpus** — the ground-truth
fault→fix mappings and the keys that attest them — lives in a **separate, private
repository and infrastructure** and is never present here, in any form, including
fixtures, dumps, or git history (README "Open engine, private corpus"; SECURITY
invariant 1). This file is the contract that keeps the two apart; it is mirrored in
the private repo.

## Two repos, never joined

| | Public (this repo) | Private |
| --- | --- | --- |
| Holds | the engine + the corpus **client and schema** only (`crates/corpus-client`) | the ground-truth **data**: authored flows, compiled rows, the sign-off keys |
| Data | **none** — no corpus row or model weight, ever | the corpus itself |

The private repo is off-tree and is **never** a submodule, subtree, symlink, or
relative-path sibling of this one.

## The data flow is ONE-WAY

- **May cross private → public:** *only* the **public ed25519 pubkey hex** (it
  verifies attestations, cannot mint them) and a **schema commit SHA** the private
  side pins to. Both are non-identifying.
- **May cross public → private:** *only* this repo's schema/gate/vocabulary, by the
  private side pinning a commit — code, not data.
- **MUST NEVER enter this repo:** any authored fix-flow YAML, any compiled corpus
  row / `FixMapping`, any BOM-revision string, and the secret sign-off seed.

## How a leak is caught here, not just discouraged

1. **Structured de-identification** (`corpus-client`): free text never reaches a row;
   only fixed-vocabulary symptoms and allowlisted actions survive (`de_identify_plan`,
   `extract_symptoms`). The adversarial leakage suite in `crates/corpus-client/src/lib.rs`
   seeds known identifiers through the whole path and asserts zero leakage.
2. **`.gitignore`** refuses `*.flow.yaml`, `*.jsonl`, `*.seed`, `/flows/`, `/build/`,
   and any `cec-corpus*` path.
3. **`scripts/githooks/pre-commit`** refuses the same shapes (plus corpus/weights),
   runs the CONTENT-keyed boundary gate (`cargo xtask scan-content --staged` — corpus-row
   shapes, canonical poison tokens, base64/hex-encoded smuggling; renames and `git add -f`
   do not bypass it), and runs `gitleaks` on staged content when installed (warn-and-skip
   otherwise — the required CI `secrets` + `boundary` jobs are the server-side backstop).
   Activate with `cargo xtask install-hooks`. Sanctioned synthetic literals live in
   `.boundary-allow.txt`, which is FROZEN in CI (net-new entries fail the `boundary` job).
4. **The truth-admission gate** (`ensure_evidence_integrity` + `ensure_attested`) means
   even a row that did reach a store is refused unless it is sign-off-confirmed and
   attested by a key this engine does not hold.
