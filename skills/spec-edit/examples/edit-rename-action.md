# spec-edit — worked example: renaming an action

**Profile in this walkthrough:** `firmware-layered`. The *process* shown (grep callers, present impact, wait for confirmation) is profile-agnostic; only the specific field shapes are firmware-flavored.

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

The project is single-package (v1 specdev is single-package across the board).

**Module callers** — grep `specs/modules/*.toml` for `depends_on` containing `StatusLed`. Found:
- `specs/modules/AudioPipeline.toml` has `depends_on = ["BtA2dpSink", "I2sOutput", "StatusLed"]`. Generated source under `src/audio_pipeline/` would call `self.led.off()`. That call site updates when `/spec-gen AudioPipeline` next runs.

**Architecture wiring** — grep `specs/architecture.toml` `[[connections]]` for `StatusLed.<anything>`. Ports bind to ports, not to actions, so renaming an action does not affect wiring. No impact.

**Hardware ownership** — `StatusLed.hardware.pins` is unchanged by an action rename. No impact.

**Bindings** (Phase 7+) — if any `bindings."embassy+stm32h5-rs"` table on this module references the action by name, flag it. In this scenario, no bindings exist. No impact.

Present to the user:

> **Impact analysis for rename `StatusLed.off` → `StatusLed.turn_off`:**
> - Module callers: 1 — `AudioPipeline` (source call site `self.led.off()` will be regenerated on next `/spec-gen AudioPipeline`).
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
returns = "Result<(), LedError>"
blocking = false
isr_safe = false
description = "Turn the LED off."
```

Do **not** edit source code from `/spec-edit`. Source regeneration is `/spec-gen`'s job — and per Phase 8, it's the user driving Claude through `/spec-gen`, not the analyzer rewriting `.rs` files.

Re-run `specdev-analyzer validate --no-color`. Expect a clean report.

## Reporting back

> Renamed action `StatusLed.off` → `StatusLed.turn_off` in `specs/modules/StatusLed.toml`.
> 1 caller flagged (`AudioPipeline`); source update deferred to `/spec-gen AudioPipeline`.
> Validate: clean.
