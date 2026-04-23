# specdev example projects

Canonical reference projects used by the specdev skills. Each project is a complete, validated specdev workspace. Skills point here instead of re-listing module names, so adding or renaming an example only requires updating this file.

Identify the target project's domain from its `spec.toml` (`[project] domain`), then use the matching project below.

## Firmware — `blinky-a2dp/`

Four-module firmware architecture: three drivers and one middleware. Illustrates the full firmware module surface.

| File | Layer | Illustrates |
|---|---|---|
| `specs/modules/StatusLed.toml` | driver | Hardware block with multiple pins; named-transition state machine; multiple action signatures; error variants |
| `specs/modules/I2sOutput.toml` | driver | Peripheral driver with hardware pins and actions, no state machine |
| `specs/modules/BtA2dpSink.toml` | driver | External-dependency driver with incoming-data port |
| `specs/modules/AudioPipeline.toml` | middleware | `depends_on` three drivers; multiple ports (`audio_in`, `audio_write`, `led_cmd`); `satisfies` requirements; `shared_state` in `ConcurrencyModel` |

Domain-specific details:
- `layer` values come from `domains/firmware/domain.toml` `[layers] order`: PAC → HAL → driver → middleware → application.
- Metadata categories (`MemoryModel`, `ConcurrencyModel`, `ErrorHandling`, `Ownership`) and their legal values (e.g. `allocation = "static_alloc"`, `strategy = "result_type"`) are firmware-specific.

## Egui — `counter-egui/`

Minimal egui app with a view and a shared-state store.

| File | Kind | Illustrates |
|---|---|---|
| `specs/modules/CounterView.toml` | view | Egui view module shape |
| `specs/stores/CounterStore.toml` | store | Shared-state container (lives under `specs/stores/`, not `specs/modules/`) |

Domain-specific details:
- Egui projects use `specs/stores/` (see `domains/egui/init.md`); that directory does not exist in firmware projects.
- Metadata categories and layer semantics differ from firmware; consult `domains/egui/author.md` for conventions.

## Adding a new example

1. Create a complete, validating project under `specdev/examples/<name>/`.
2. Add a section above under the matching domain heading with a table of modules and what each illustrates.
3. No skill files need to change — they reference this README, not individual modules.
