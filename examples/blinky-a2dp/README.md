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
```

Expected: clean validation (`0 error(s), 0 warning(s), 0 info(s)`) and six YAML files written under `/tmp/blinky-out/` (four modules + `architecture.yaml` + `requirements.yaml`).

## Generate code

Inside Claude Code with the plugin installed:

```
/spec-gen --all
```

Expected output: one Rust file per module is emitted under `src/` — `status_led.rs`, `bt_a2dp_sink.rs`, `i2s_output.rs`, `audio_pipeline.rs` — following the `languages/rust` and `codegen/firmware-rust` guides shipped with the plugin. The skill appends each module to `src/lib.rs` (or creates it).

## Audit drift

Once source exists, `audit` cross-checks it against the spec:

```
specdev-analyzer audit --project specdev/examples/blinky-a2dp --show-uncovered
```

On a freshly-generated tree this reports `4 match, 0 missing, 0 mismatch, 0 uncovered`. Before `/spec-gen --all` runs, the same command reports four `missing` entries — one per module without a source file.
