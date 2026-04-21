# firmware — spec-init extensions

This file is loaded by `/spec-init` after the user selects the `firmware` domain. It declares the domain-specific portion of project scaffolding.

## Extra interview questions

None. MCU, RTOS, and environment have no schema-valid destination: `firmware/v1/architecture.schema.yaml` defines only `connections` with `additionalProperties: false`, and `spec.toml` has no slot for them. Per-module `hardware` (pins, peripherals, clocks) lives on each module's own TOML, not at the architecture level. Capture MCU / RTOS / environment in the project's README or build files (`Cargo.toml`, `sdkconfig.defaults`, `platformio.ini`) — `/spec-init` does not persist them.

## architecture.toml skeleton

Emit an empty file. `connections` is optional; entries are added by `/spec-design` as modules land and need wiring.

## Extra scaffold directories

None.

## Extra scaffold files

None.

## Post-write notes

Cross-module port connections are authored by `/spec-design` once two modules exist and need wiring. Per-module hardware (pins, peripherals) is declared in each module TOML, not in `architecture.toml`.
