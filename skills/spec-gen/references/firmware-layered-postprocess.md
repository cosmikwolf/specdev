# spec-gen — firmware-layered post-processing (memory.x)

This reference applies only when `spec.toml [project].profile == "firmware-layered"`. For library-api, audio-plugin, rust-workspace, or any user-authored profile, skip — `memory.x` is a firmware linker concern and is not relevant.

## memory.x emission

`architecture.toml [platform].mcu_part` drives the linker script. Cases:

- **`mcu_part` is unset** — emit a placeholder `memory.x` with a `// FILL IN` comment in `MEMORY { FLASH ... RAM ... }` and surface a warning in the report. The user should set `mcu_part` (e.g. `"stm32h563"`, `"stm32f411"`, `"nrf52840"`) and re-run.
- **A hand-authored `memory.x` already exists** — preserve it. Do not overwrite. Surface a one-line note in the report so the user can confirm the hand-authored version still matches `mcu_part`.
- **`mcu_part` is set and no `memory.x` exists** — emit one. The MCU-specific MEMORY layout comes from the relevant inlined MCU block's codegen prose. For STM32H5 specifically (`mcu_part = "stm32h5XX"`), see `codegen/rust/blocks/stm32h5-rs/README.md`'s memory layout section. Otherwise emit a generic `MEMORY { FLASH ... RAM ... }` skeleton with a `// FILL IN` comment.

The compile gate (single-package Step 8 / workspace W4) catches an inconsistent `memory.x` because `cargo check --target <triple>` will fail to link without a valid layout. Trust that gate; do not try to validate `memory.x` content beyond emitting it.
