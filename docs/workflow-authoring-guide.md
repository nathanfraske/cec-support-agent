<!-- SPDX-License-Identifier: AGPL-3.0-only -->

# Shop workflow authoring guide — capture what you *know works*

**Who this is for:** everyone at the shop who diagnoses and fixes machines. This
is how your hard-won, proven workflows become the system's ground truth. The
diagnostic engine is only as good as what you teach it here — and it will only
ever treat as "proven" what a person like you actually stood behind.

**The one rule:** author only workflows you have **done and seen work**. A guess
belongs somewhere else (the engine can reason out guesses on its own and flags
them as unproven). What you write here is the gold standard — the thing it
reaches for first and trusts most.

There are **two kinds** of workflow, and you can write either or both:

- **Fix workflow** — "when I see *this shape* of problem, I do *these steps*, and
  I know it worked when *this*." (Your DDU example is one of these.)
- **Diagnostic workflow** — "when a machine does *this*, here's how I figure out
  *what's actually wrong* before touching anything."

Fill in one block per workflow. Plain language is fine everywhere — a field
marked `action:` is the only one that maps to a specific tool, and we help you
map it (or flag that we need to build it).

---

## Part 1 — Fix workflow

Copy this block, fill it in, one per fix you trust. Everything in `< >` is a
prompt — replace it.

```yaml
fix_workflow:
  title: <short name, e.g. "GPU driver crash — clean reinstall">
  author: <your name — so we can ask you if something's unclear>

  # HOW SURE ARE YOU? (this is what makes it "proven")
  proven:
    times_done: <roughly how many times you've run this and it worked>
    confidence: <high | medium> # only put "high" if it reliably works
    notes: <anything about when it works best / worst>

  # WHEN does this apply? — the "shape" of the problem
  shape:
    what_you_see: >
      <plain language: what the customer reports and what you observe.
       e.g. "display artifacts and game crashes; event log shows TDR /
       display-driver-stopped-responding">
    symptoms: [ <the short signal words — we help you pick these from the
                 engine's list; e.g. tdr, display, driver, crash> ]
  applies_to: >
    <which machines this is for — be specific enough that it won't be applied
     to the wrong hardware. e.g. "NVIDIA GPUs on Windows 11" — NOT "any PC">

  # HOW will we know it worked?
  verification:
    type: <deterministic | intermittent | hardware>
    # deterministic = you can re-test right away and see pass/fail
    # intermittent  = it only happens sometimes; needs watching over time
    # hardware      = the real test is a bench/RMA, not a software recheck
    test: >
      <exactly how you confirm the fix held. e.g. "re-run the game / re-check
       the event log — the TDR is gone and it runs clean">
    if_still_broken: >
      <what it means and what you do next. e.g. "escalate — the GPU itself may
       be failing; send to bench">

  # WHAT do you do, in order?
  steps:
    - do: <plain language: what you do this step>
      action: <the tool name from the list below — or write NEEDS-TOOL>
      risk: <read-only | reversible | destructive>
      installs_licensed_software: <no | yes: name>   # yes -> the user must
                                                      # accept the EULA on screen
      why: <one line: why this step, in your words>
```

### Your DDU example, fully worked

This is exactly what you gave me — "Driver issue → DDU → Restart → Reinstall
fresh → Test" — written in the format, and honestly mapped to today's tools.

```yaml
fix_workflow:
  title: "GPU display-driver crash — DDU clean reinstall"
  author: <shop tech>
  proven:
    times_done: "dozens"
    confidence: high
    notes: "the go-to for a corrupted display driver; a plain reinstall over
            the top usually does NOT fix it — the clean wipe is the point"
  shape:
    what_you_see: >
      Display artifacts, black screens with the fans still running, or crashes
      during games; the event log shows a display-driver-stopped-responding
      (TDR) entry.
    symptoms: [ tdr, display, driver, crash ]
  applies_to: "NVIDIA or AMD GPUs on Windows 10/11 (a discrete graphics card)"
  verification:
    type: deterministic
    test: "re-run the workload that was crashing / re-check the event log — the
           TDR and crashes are gone"
    if_still_broken: "escalate to a hardware/thermal check — the card may be
                      failing or overheating"
  steps:
    - do: "Make a restore point first, so we can undo if needed"
      action: create_restore_point      # EXISTS (reversible)
      risk: reversible
      installs_licensed_software: no
      why: "safety net before removing drivers"
    - do: "Boot to safe mode and run DDU to fully remove the display driver"
      action: NEEDS-TOOL: ddu            # <- not a tool yet; propose it below
      risk: reversible
      installs_licensed_software: no
      why: "a clean slate removes the corrupted driver a normal reinstall leaves"
    - do: "Restart the PC"
      action: NEEDS-TOOL: restart        # <- not a tool yet
      risk: reversible
      installs_licensed_software: no
      why: "let Windows come up with no display driver before the fresh install"
    - do: "Download the latest driver package from the vendor"
      action: download_file              # EXISTS (reversible)
      risk: reversible
      installs_licensed_software: no
      why: "get the current official package, not whatever Windows Update pushes"
    - do: "Install the driver package fresh"
      action: NEEDS-TOOL: driver_install # <- cec-autosetep already installs
                                         #    drivers via pnputil; wire it here
      risk: reversible
      installs_licensed_software: no
      why: "clean install on top of the wiped state"
  # "Test to see if issue is still present" -> that's the `verification` block
  # above, not a step. The engine re-collects and diffs automatically.
```

**What this one workflow just told us:** the `create_restore_point` and
`download_file` steps run today; `ddu`, `restart`, and `driver_install` are
**tools we need to build** (and cec-autosetep already has the driver-install
half). So authoring your proven flows also writes our tool-building to-do list —
that is the point.

---

## Part 2 — Diagnostic workflow

For "how I figure out what's actually wrong" — the decision-making before any
fix. Every check here is **read-only** (you look, you don't change).

```yaml
diagnostic_workflow:
  title: <short name, e.g. "GPU trouble — is it software or hardware?">
  author: <your name>
  starting_point: >
    <what you start from — the complaint or symptom. e.g. "customer reports the
     screen glitches and games crash">
  checks:
    - look_at: <what you inspect, plain language>
      action: <event_log_query | cim_query | board_info — or NEEDS-TOOL>
      tells_you: >
        <what each possible result means and where it sends you.
         e.g. "TDR present with normal temps -> software/driver shape;
               very high temps -> cooling/hardware shape">
  conclusions:
    - shape: <a fault shape this workflow can land on>
      then: <what happens — which fix workflow, or "escalate to a person">
```

### A worked diagnostic example

```yaml
diagnostic_workflow:
  title: "GPU trouble — software or hardware?"
  author: <shop tech>
  starting_point: "customer reports screen glitches and crashes during games"
  checks:
    - look_at: "Event log for display-driver / TDR errors"
      action: event_log_query            # EXISTS (read-only)
      tells_you: >
        TDR entries with otherwise-normal hardware -> a software/driver shape,
        go to the DDU clean-reinstall fix. No display errors at all -> keep looking.
    - look_at: "GPU temperature and health under load"
      action: cim_query                  # EXISTS (read-only)
      tells_you: >
        Very high temps or thermal events -> a cooling/hardware shape, escalate
        (a driver reinstall won't fix a hot or failing card).
  conclusions:
    - shape: "software / display driver"
      then: "run the GPU display-driver DDU clean-reinstall fix workflow"
    - shape: "cooling or failing hardware"
      then: "escalate to bench — do not attempt a software fix"
```

---

## The tools that exist today (for the `action:` field)

If a step matches one of these, put its name in `action:`. If it doesn't, write
`NEEDS-TOOL: <your name for it>` and we'll build it (that is expected and
useful — it's how we grow what the engine can do).

| action | what it does | risk |
|---|---|---|
| `board_info` | read the motherboard / hardware identity | read-only |
| `cim_query` | query Windows/hardware state (temps, devices, health) | read-only |
| `event_log_query` | read the Windows event logs | read-only |
| `create_restore_point` | make a system restore point | reversible |
| `download_file` | download a file (e.g. a driver package) | reversible |
| `driver_rollback` | roll a driver back to its previous version | reversible |
| `registry_set` | set a registry value | reversible |
| `review` | flag "a person should look at this" (advisory only) | read-only |

Known gaps your workflows will likely hit (already on our radar):
`ddu`, `restart`, `driver_install`, `sfc`/`dism` repair, `uninstall_program`,
`winget_install`. Flag them with `NEEDS-TOOL:` and describe them — the more of
you that hit the same gap, the higher it goes on the build list.

---

## What makes a workflow "proven" (and what to avoid)

- **You've run it and seen it work.** Confidence `high` means it reliably fixes
  the shape it's for. If you're not sure, mark `medium` — still valuable, just
  weighted lower.
- **Scope it honestly in `applies_to`.** A driver fix for NVIDIA cards must not
  say "any PC." Wrong scope is worse than no workflow.
- **Say how you *verify*, not just what you do.** The step list without the test
  is half a workflow — the engine's whole discipline is "we only call it fixed
  when we can show the symptom is gone."
- **Name the stop points.** `if_still_broken` and any "escalate to a person"
  are as important as the fix — they're how we keep the machine from acting on a
  guess.
- **No customer specifics.** Write shapes and steps, never a customer's name,
  machine name, serial, or IP. The workflow is about the *problem class*, not a
  person. (Any real machine detail is stripped automatically downstream, but
  keep it out from the start.)

## How to submit

Fill in one block per workflow in a plain text file (or ask for the fillable
copy) and send it back to <owner>. Diagnostic and fix workflows can go in the
same file. Don't worry about getting the `symptoms:` words or `action:` names
exactly right — write it in your own words and we'll map it with you; the
knowledge is the hard part, and that's the part only you have.
