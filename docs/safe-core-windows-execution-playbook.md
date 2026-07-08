<!-- SPDX-License-Identifier: AGPL-3.0-only -->

# Safe-core Windows execution — drop-in playbook

The safe core (`crates/tools-windows/src/catalog.rs`, `SAFE_CORE`) is registered
in the engine as data-driven `CatalogTool`s: each has a plain-language name, a
description, a risk class, and the exact Windows `command` it represents. The
engine can plan, store, gate, and dispatch them today. **This playbook is for the
Windows agent that wires the actual execution.**

## What already works
- Every `SAFE_CORE` entry is a registered dispatcher tool and a
  `deid::ACTION_VOCABULARY` member (the drift test enforces this).
- Entries flagged `runnable_as_is: true` carry a COMPLETE, argument-free,
  injection-safe command and **already run** on a Windows host via
  `run_powershell` (e.g. `disk_health`, `smart_data`, `defender_status`,
  `flush_dns`, `list_network_adapters`, `process_list`).
- On non-Windows hosts every catalog tool returns "unsupported" (build stays
  green everywhere).

## What to wire (the drop-in)
Entries with `runnable_as_is: false` need **validated argument handling** before
they execute — a drive letter, a target host, a registry key, a device id. Today
their `invoke` returns a clear "needs argument wiring" error (safe: it never runs
an unvalidated command).

For each, add arg handling that mirrors the existing hand-written tools:
1. Pull the argument from `args` (e.g. `args.get("drive")`).
2. Validate it with a strict allowlist — reuse `safe_identifier` (bare
   `[A-Za-z0-9_]`) or add a purpose-built validator (a drive letter is one
   `[A-Za-z]`; a target host is a hostname/IP; a registry key is a bounded path).
   **Never interpolate an unvalidated argument into a shell command.**
3. Substitute into the entry's `command` and run via `run_powershell`.

The cleanest structure is a small per-entry closure keyed by `name`, or a typed
`args` schema per catalog entry. Keep the fixed-command path for `runnable_as_is`
entries unchanged.

## Destructive ops (`DESTRUCTIVE_OPS`) — do NOT auto-wire
These are deliberately NOT registered and NOT in the action vocabulary. To make
one usable:
1. Add its name to `deid::ACTION_VOCABULARY` (keep sorted) and register a
   `CatalogTool` for it — but ONLY behind the human-sign-off path.
2. The corpus gate already refuses a beneficial/destructive plan without
   `HumanConfirmed` sign-off (`DestructiveFixNeedsHuman`). Ensure the execution
   path likewise requires explicit human consent before dispatch, and captures the
   `reversal_note`'s snapshot/backup step first (e.g. `backup_registry_key` before
   `delete_registry_key`; `suspend_bitlocker` before `clear_tpm`; stage the GPU
   driver before `clean_gpu_drivers`).
3. Never mark a destructive op `runnable_as_is`.

## Adding a new primitive
Add a `CatalogEntry` to `SAFE_CORE` (or `DESTRUCTIVE_OPS`), then run the vocab
merge so its name enters `deid::ACTION_VOCABULARY` (the drift test will otherwise
fail), and regenerate `docs/tool-catalog.md`. The catalog tests enforce: clean
`[a-z0-9_]` names, safe-core is never destructive, destructive ops carry a
reversal note, and the two lists are disjoint.
