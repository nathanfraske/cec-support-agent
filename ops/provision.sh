#!/usr/bin/env bash
# SPDX-License-Identifier: AGPL-3.0-only
# Copyright 2026 Nathan M. Fraske
#
# One idempotent script for setup / disaster recovery of a cec-support-agent
# clone (docs/AGENTIC_ADDENDUM.md "Tier 0"). It activates the dormant exfil
# pre-commit guard, makes the agent hooks executable, ensures the pinned
# toolchain components, checks for gitleaks (a hard dependency of the pre-commit
# hook), and runs the full verification suite. Safe to re-run.
#
# It does NOT provision secrets, a bot PAT, or branch protection — those are
# owner/GitHub-side actions (see BOUNDARY.md / the private WIRING.md).
set -euo pipefail
ROOT="$(git rev-parse --show-toplevel)"
cd "$ROOT"

echo "==> Activating the corpus/weights/seed exfil pre-commit guard"
git config core.hooksPath scripts/githooks
chmod +x scripts/githooks/pre-commit 2>/dev/null || true
chmod +x .claude/hooks/*.sh 2>/dev/null || true

echo "==> Toolchain (rust-toolchain.toml pins the exact version)"
if command -v rustup >/dev/null 2>&1; then
  rustup component add rustfmt clippy >/dev/null 2>&1 || true
else
  echo "    note: rustup not found — install the toolchain pinned in rust-toolchain.toml"
fi

echo "==> gitleaks presence (hard dependency of scripts/githooks/pre-commit)"
if ! command -v gitleaks >/dev/null 2>&1; then
  echo "    WARNING: gitleaks is not on PATH. The pre-commit exfil guard will REFUSE"
  echo "    every commit until it is installed (see BOUNDARY.md / private WIRING.md)."
fi

echo "==> Verification suite"
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo build --workspace
cargo test --workspace

echo "==> provision: OK — exfil guard active, hooks executable, suite green."
