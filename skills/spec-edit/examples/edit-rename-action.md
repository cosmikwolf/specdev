# spec-edit — worked example: renaming an action

**Domain in this walkthrough:** firmware. The *process* shown (grep callers, bucket impact, wait for confirmation) is domain-agnostic; only the specific action name is firmware-flavored.

**Target project:** `specdev/examples/blinky-a2dp/` (flat / single-package).

**Scenario:** the user runs `/spec-edit StatusLed` with the change "rename action `off` to `turn_off`".

## Before

`specs/modules/StatusLed.toml` (excerpt):

```toml
[[actions]]
name = "off"
params = []
returns = "Result<(), LedError>"
blocking = false
isr_safe = false
description = "Turn the LED off."
```

## Step 5 — impact analysis

**Same-package impact** (blinky-a2dp is single-package):
- `specs/modules/AudioPipeline.toml` has `depends_on = ["BtA2dpSink", "I2sOutput", "StatusLed"]`. Generated source under `src/audio_pipeline/` would call `self.led.off()`. That call site must update when `/spec-gen` next runs.

**Cross-package impact:**
- None. The project has no `[[packages]]`.

**Project-wide impact:**
- `specs/architecture.toml` — check `[[connections]]`. Ports bind to ports, not to actions, so renaming an action does not affect wiring. No impact.
- Hardware ownership: `StatusLed.hardware.pins` unchanged. No impact.

Present to the user:

> **Impact analysis for rename `StatusLed.off` → `StatusLed.turn_off`:**
> - Same-package: 1 caller — `AudioPipeline` (source call site `self.led.off()` will be fixed on next `/spec-gen`).
> - Cross-package: none.
> - Project-wide: none.
>
> Proceed?

Wait for confirmation before editing.

## After confirmation

Rewrite the `[[actions]]` block:

```toml
[[actions]]
name = "turn_off"
params = []
returns = "Result<(), LedError>"
blocking = false
isr_safe = false
description = "Turn the LED off."
```

Do **not** edit source code from `/spec-edit`. Source regeneration is `/spec-gen`'s job.

Re-run `specdev-analyzer validate --project <project_root> --no-color`. Expect a clean report.

## Reporting back

> Renamed action `StatusLed.off` → `StatusLed.turn_off` in `specs/modules/StatusLed.toml`.
> 1 caller flagged (`AudioPipeline`); source update deferred to `/spec-gen`.
> Validate: clean.
