# spec-gen — worked examples: `<source_dir>` resolution

Step 3.5 resolves the placeholder `<source_dir>` per target module. The rule (from the skill body): if the extracted YAML carries a top-level `package:` key, look it up in `spec.toml`'s `[[packages]]` and use `source_path + src/`; otherwise use `[source] root` from `spec.toml` (typically `src/`).

This logic is domain-agnostic. The worked cases below use firmware and egui projects for concreteness; the resolution rule is identical in any future domain.

## Case 1 — flat / single-package project

**Example project:** `specdev/examples/blinky-a2dp/` (firmware domain).

`spec.toml`:

```toml
[project]
name = "blinky-a2dp"
domain = "firmware"
language = "rust"

[source]
root = "src"
layout = "flat"
```

No `[[packages]]` section, so the fallback rule applies: `<source_dir>` = `[source] root` = `src/`.

For each target module (e.g. `StatusLed`, `AudioPipeline`), the codegen guide's path pattern `src/<module_name>/mod.rs` substitutes to `src/status_led/mod.rs`, `src/audio_pipeline/mod.rs`, etc.

`specdev/examples/counter-egui/` has the same `flat` layout in a different domain — same resolution, same substitution, different module types (`CounterView.toml`, `CounterStore.toml`).

## Case 2 — multi-package workspace

**Example project:** synthetic. No reference project currently demonstrates `[[packages]]`; the shape below is what `/spec-reverse` would detect in a Rust workspace.

`spec.toml`:

```toml
[project]
name = "quadcopter"
domain = "firmware"
language = "rust"

[source]
root = "src"
layout = "workspace"

[[packages]]
name = "drivers"
source_path = "crates/drivers"

[[packages]]
name = "flight"
source_path = "crates/flight"
```

Module TOMLs live under `specs/modules/<package>/<Module>.toml`. The extracted YAML for each carries `package: <name>`.

For `specs/modules/drivers/Imu.toml` (`package: drivers`): look up in `[[packages]]` → `source_path = crates/drivers` → `<source_dir>` = `crates/drivers/src/`. Codegen substitutes `src/imu/mod.rs` → `crates/drivers/src/imu/mod.rs`.

For `specs/modules/flight/Controller.toml` (`package: flight`): `<source_dir>` = `crates/flight/src/`. Substitute to `crates/flight/src/controller/mod.rs`.

## Edge case — top-level module in a multi-package project

If a module TOML lives at `specs/modules/Util.toml` (top-level, not under any `<package>/` subdirectory) while `[[packages]]` is declared, the analyzer emits `PKG041` (info) and extracts the module without a `package:` key.

Step 3.5 falls back to rule 2 → `<source_dir>` = `[source] root` = `src/`. Codegen writes to `src/util/mod.rs`. If that isn't the intended destination, the fix is at the spec level (move the TOML under the correct package directory and re-extract), not at the codegen level.
