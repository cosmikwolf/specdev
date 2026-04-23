# spec-validate — annotated report walkthrough

**Domain in this walkthrough:** firmware. Rule ids and their severities come from `domains/firmware/domain.toml` `[validation.rules]`. Other domains declare their own rule ids — consult the loaded `<Domain dir>/domain.toml` for the canonical list. The *presentation pattern* shown below is domain-agnostic.

**Target project:** `specdev/examples/blinky-a2dp/` (modified to introduce defects for illustration).

## Raw analyzer output (hypothetical)

```
$ specdev-analyzer validate --project . --no-color
error [LAYER001]: module `AudioPipeline` depends on `FlightController` but `middleware` cannot depend on `application`.
  at specs/modules/AudioPipeline.toml:6
error [META010]: required metadata category `ConcurrencyModel` missing.
  at specs/modules/I2sOutput.toml
warning [FSM022]: state `Calibrating` unreachable from initial `off`.
  at specs/modules/StatusLed.toml:90 (state_machines.name = "LedFSM")
error [PORT030]: connection endpoint `AudioPipeline.led_command` not found.
  at specs/architecture.toml:12
warning [REQ001]: requirement `REQ-AUDIO-003` not satisfied by any module.
  at specs/requirements/audio.toml:14

3 errors, 2 warnings — exit 1
```

## Step 3 presentation pattern

Group by severity, then by rule category. Do not dump raw output — translate each rule id into a one-line explanation and a concrete fix hint. Pull rule definitions from `<Domain dir>/domain.toml`'s `[validation.rules]` table when a rule id is unfamiliar.

### Errors (3)

**Layer violations (LAYER001)** — 1 finding
- `AudioPipeline` → `FlightController`. `middleware` cannot depend on `application` per the firmware domain's `[layers.allowed_deps]`. Fix options: invert the dependency, or move shared types into a lower layer. Use `/spec-edit AudioPipeline` so the rename's same-package / cross-package impact is analyzed.

**Missing metadata (META010)** — 1 finding
- `I2sOutput` is missing `[metadata.ConcurrencyModel]`. Firmware modules must declare `ConcurrencyModel.protection`. `/spec-edit I2sOutput` to add.

**Architecture wiring (PORT030)** — 1 finding
- `specs/architecture.toml:12` references a port `AudioPipeline.led_command`, but no such port exists on that module (the real port is `led_cmd`). Either rename the port in `AudioPipeline.toml` or fix the connection entry directly.

### Warnings (2)

**Unreachable state (FSM022)** — 1 finding
- `StatusLed.LedFSM` has a `Calibrating` state with no incoming transitions. Either add a transition or delete the state. `/spec-edit StatusLed`.

**Unsatisfied requirement (REQ001)** — 1 finding
- `REQ-AUDIO-003` declared in `specs/requirements/audio.toml:14` is not referenced by any module's `satisfies[]`. Either satisfy it in the relevant module or remove the requirement.

## Step 4 handoff (per the skill's read-only contract)

Do not edit files from `/spec-validate`. Tell the user:

- LAYER001 on `AudioPipeline` → `/spec-edit AudioPipeline`.
- META010 on `I2sOutput` → `/spec-edit I2sOutput`.
- FSM022 on `StatusLed.LedFSM` → `/spec-edit StatusLed`.
- PORT030 in `architecture.toml` → edit `specs/architecture.toml` directly (no skill currently owns that file).
- REQ001 → `/spec-edit` on whichever module should satisfy the requirement, or edit the requirement TOML to remove it.

Re-invoke `/spec-validate` after fixes to confirm a clean report.
