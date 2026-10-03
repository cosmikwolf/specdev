# spec-edit — worked example: renaming an action

**Profile in this walkthrough:** `firmware-layered`. The *process* shown (grep callers, present impact, wait for confirmation) is profile-agnostic; only the specific field shapes are firmware-flavored.

**Scenario:** the user runs `/spec-edit StatusLed` with the change "rename action `off` to `turn_off`".

## Before

`specs/modules/StatusLed.toml` (excerpt):

```toml
[[actions]]
name = "off"
params = []
blocking = false
isr_safe = false
description = "Turn the LED off."
```

## Step 5 — impact analysis

The project is single-package (v1 specdev is single-package across the board).

**Module callers** — grep `specs/modules/*.toml` for `depends_on` containing `StatusLed`. Found:
- `specs/modules/AudioPipeline.toml` has `depends_on = ["BtA2dpSink", "I2sOutput", "StatusLed"]`. Its source under `src/audio_pipeline/` calls `self.led.off()`; that call site changes when the code is updated to match the spec.

**Architecture wiring** — grep `specs/architecture.toml` `[[connections]]` for `StatusLed.<anything>`. Ports bind to ports, not to actions, so renaming an action does not affect wiring. No impact.

**Hardware ownership** — `StatusLed.hardware.pins` is unchanged by an action rename. No impact.

**Bindings** — if any `bindings."embassy+stm32h5-rs"` table on this module references the action by name, flag it. In this scenario, no bindings exist. No impact.

Present to the user:

> **Impact analysis for rename `StatusLed.off` → `StatusLed.turn_off`:**
> - Module callers: 1 — `AudioPipeline` (its source calls `self.led.off()`; update it with the rename).
> - Architecture wiring: none.
> - Hardware ownership: none.
> - Bindings: none.
>
> Proceed?

Wait for confirmation before editing.

## After confirmation

Rewrite the `[[actions]]` block:

```toml
[[actions]]
name = "turn_off"
params = []
blocking = false
isr_safe = false
description = "Turn the LED off."
```

Do **not** edit source code from `/spec-edit`. The code change is a separate step: the user (or the agent at their request) renames the method in `StatusLed` and the call in `AudioPipeline`, after reading `specdev-analyzer context --module StatusLed`.

Re-run `specdev-analyzer validate --no-color`. Expect a clean report.

## Reporting back

> Renamed action `StatusLed.off` → `StatusLed.turn_off` in `specs/modules/StatusLed.toml`.
> 1 caller flagged (`AudioPipeline`); the code still says `off` until it is updated.
> Validate: clean.
