# Example: `blinky-a2dp`

Reference project that pairs with the specdev planning docs. A Bluetooth A2DP
audio sink streams PCM frames from a paired source to an I2S DAC, surfacing
status (discoverable / connected / underrun) on a DotStar LED.

Modules:

| Module         | Layer      | Purpose                                               |
| -------------- | ---------- | ----------------------------------------------------- |
| `StatusLed`    | driver     | APA102 DotStar LED driver with a 3-state FSM.         |
| `BtA2dpSink`   | driver     | Bluetooth A2DP sink with a 4-state connection FSM.    |
| `I2sOutput`    | driver     | I2S DAC driver (single-threaded, owns peripheral).    |
| `AudioPipeline`| middleware | Bridges Bt → I2S, drives the status LED, reports drift. |

Additional files:

- `spec.toml` — project-level config.
- `specs/architecture.toml` — cross-module port connections (D17).
- `specs/requirements/audio.toml` — requirements referenced by `satisfies`.

These specs are ports of the existing SysML fixtures under
`sysml-v2-analyzer/tests/fixtures/`. Once Phase 2 schemas land, this project
must validate cleanly end-to-end; it is the first acceptance test for the
analyzer.

## Status

Phase 1 — specs exist; schemas and validator do not yet. Running
`specdev-analyzer validate` on this project today prints a placeholder message.
