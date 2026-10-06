# spec-init — illustrative architecture refinement per shipped profile

Step 6 in `/spec-init` reads `specs/architecture.toml` after bootstrap and offers per-profile refinements. The bootstrap-emitted templates are valid as-is; this is **interview-driven refinement, not error recovery** — only edit fields the user actually wants to change.

`architecture.toml` holds only facts a check reads: connections between modules, project-wide decisions, and the few platform facts a gate step uses. Dependencies, features, versions, editions, licenses and build profiles live in the build manifest (`Cargo.toml`, `pubspec.yaml`, `package.json`) — never copy them into the spec.

Only `firmware-layered` has a section here, because it is the one profile with an architecture field to confirm. For **every other profile** — and any user-authored one — read `${CLAUDE_PLUGIN_ROOT}/profiles/<profile>/README.md` and the emitted `specs/architecture.toml` together; the emitted template carries commented hints.

## firmware-layered

Pre-filled, confirm or change:

- `[platform].target_triple` — `thumbv8m.main-none-eabihf` (Cortex-M33 / STM32H5). Common alternatives: `thumbv7em-none-eabihf` (M4F/M7F; STM32F4/H7), `thumbv6m-none-eabi` (M0/M0+). The firmware gate step builds for this target, so it must match `.cargo/config.toml`.

The core, runtime, panic-handler, logger and HAL crates are dependencies: the user adds them to `Cargo.toml`, and the inlined chip and runtime units' guidance says which features they need.

## Connections and project decisions (any profile that has them)

- `[[connections]]` — port-to-port wiring between modules; `connection_endpoints_resolve` checks each endpoint.
- `project_decisions` — decisions that shape the whole project. A declaration only: to make a decision reach a module, link it through the module's `guided_by` or a satisfied requirement's `decided_by`.
