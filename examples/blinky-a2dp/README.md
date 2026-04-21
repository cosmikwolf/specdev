# blinky-a2dp

A Bluetooth A2DP audio sink firmware example. Streams PCM frames from a paired Bluetooth source to an I2S DAC and surfaces connection status on a DotStar RGB LED.

Target: ESP32 with Bluetooth and I2S.

## Modules

| Module          | Layer      | Purpose                                                   |
| --------------- | ---------- | --------------------------------------------------------- |
| `StatusLed`     | driver     | APA102 DotStar driver with a 3-state pattern FSM.         |
| `BtA2dpSink`    | driver     | Bluetooth A2DP sink with a 4-state connection FSM.        |
| `I2sOutput`     | driver     | I2S DAC driver; single-threaded, owns the peripheral.     |
| `AudioPipeline` | middleware | Bridges Bluetooth → I2S, drives the LED, reports drift.   |

## Files

- `spec.toml` — project config (domain, language, source root).
- `specs/architecture.toml` — cross-module port connections.
- `specs/modules/*.toml` — one TOML per module.
- `specs/requirements/audio.toml` — requirements referenced by modules via `satisfies`.

## Run the analyzer

From the repo root:

```
specdev-analyzer validate --project specdev/examples/blinky-a2dp
specdev-analyzer extract  --project specdev/examples/blinky-a2dp --output /tmp/blinky-out
specdev-analyzer audit    --project specdev/examples/blinky-a2dp
```

Expected: clean validation, six YAML files written under `/tmp/blinky-out/` (four modules + `architecture.yaml` + `requirements.yaml`), audit reports Match for every extracted module.

## Regenerate code

Inside Claude Code with the plugin installed:

```
/spec-gen --all
```

This emits Rust / C / C++ source under `src/` following the project's configured language and the codegen guides shipped with the plugin.
