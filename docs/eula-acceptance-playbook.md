# EULA on-screen acceptance — target-side playbook

**Audience:** an agent (or human) wiring the target-side executor. The
engine-side gate is built, merged, and tested; this is the one piece that lives
on the machine with a screen.

## The liability boundary

Some remediations install third-party software with an end-user license
agreement (SignalRGB, Thermalright Control Center, vendor utilities, …).
**Accepting a EULA is the user's act, not the shop's.** If the engine — or a
technician acting through it — clicked through a license on the user's behalf,
the shop would be accepting those terms and the liability that comes with them.
So the engine **never** accepts a EULA. It refuses to run a EULA-bearing install
unless the user accepted that specific license on screen.

## What the engine already enforces (do not rebuild)

- `Tool::requires_eula(&self) -> Option<&str>` — a tool that installs
  license-bearing software returns the EULA id (default `None`).
- `Dispatcher::eula_of(name)` — the lookup.
- `EulaAcceptances` — the set of EULA ids the user accepted on screen.
- `execute_plan` / `execute_signed_plan` take `&EulaAcceptances` and, **before
  dispatching a step**, refuse any step whose tool `requires_eula(id)` when `id`
  is not in the set — the plan stops, the installer never runs, and the step is
  recorded as `installation refused: '<id>' requires the user to accept its
  license agreement on screen`. Proven by
  `a_eula_install_is_refused_and_never_runs_without_on_screen_acceptance` (the
  installer's `invoke` is asserted never called) and its accepted-path twin.
- Fail-closed: the CLI and served paths pass `EulaAcceptances::none()` today, so
  a EULA-bearing install refuses until a target collects real acceptances.

This is orthogonal to the risk consent gate: a EULA install is often only
`Reversible` risk, so the risk gate would allow it — only the EULA gate stands
in the way. Both must pass.

## The whole target-side task

**1. Mark the EULA-bearing install tools.** For each tool in `tools-windows`
(or wherever the installers live) that runs a licensed installer, implement
`requires_eula`:

```rust
impl Tool for InstallSignalRgb {
    fn name(&self) -> &str { "install_signalrgb" }
    fn risk(&self) -> Risk { Risk::Reversible }
    fn requires_eula(&self) -> Option<&str> { Some("signalrgb") }  // <- the id
    async fn invoke(&self, _args) -> Result<ToolOutcome, ToolError> { /* run installer */ }
}
```

Use a **stable** id per license (the product/EULA name). The same id is what the
UI presents and what the acceptance records.

**2. Present each required EULA on screen and capture acceptance.** Before
executing a plan, walk its steps, and for each `dispatcher.eula_of(action) ==
Some(id)` that is not yet accepted:

- show the actual license text to the user (fetch/display the vendor EULA),
- require an explicit, logged on-screen "I accept" (checkbox + button; never
  pre-checked, never defaulted),
- on acceptance, add it: `accepted = accepted.accept(id)`.

Then execute with the accumulated set:

```rust
let accepted = EulaAcceptances::none()
    .accept("signalrgb");          // only ids the user actually accepted
execute_signed_plan(&dispatcher, &signed, &signer, granted, &accepted).await
```

If the user declines a license, simply do not add its id — the engine refuses
that step and stops the plan, which is the correct outcome (offer the rest of
the plan, or route to a human).

**3. (Optional, recommended) record the acceptance in the ledger.** The
`sign_off` authority already proves *the run was authorized*; if you want an
auditable record that *this specific license was accepted by the user at this
time* (a stronger receipt for a dispute), capture the accepted id + timestamp
into the per-machine ledger alongside the outcome. This pairs with the
consent-receipt FOLLOWUPS item.

## Surfacing it in consent (nice-to-have)

The rendered consent screen should tell the user, before they consent to the
plan, that "step N installs software that will ask you to accept its license
agreement." The engine exposes `dispatcher.eula_of(action)` so the renderer can
add that line per step. Not required for the gate to be safe — the refusal is
the hard guarantee — but it sets expectations.
