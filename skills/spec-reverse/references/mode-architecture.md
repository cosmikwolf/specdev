# spec-reverse — architecture mode

Entered from the main SKILL.md when `$ARGUMENTS` is empty or missing. Produces `specs/architecture.toml` from source-tree evidence. Do not proceed to per-module reversal in this mode; that is `/spec-reverse <Module>` or `/spec-reverse --all`.

## Step 3 — Architecture reverse

If `specs/architecture.toml` exists, ask: **Compare (dry-run diff) / Replace / Cancel**.

Scan the project root for evidence in this order:

1. **Language** — from file extensions under `src/` and build files.
2. **Build system** — `Cargo.toml` (Rust), `CMakeLists.txt` / `Makefile` / `meson.build` (C/C++), `platformio.ini`.
3. **MCU / target** — defines, linker scripts, HAL crate names, SDK includes.
4. **RTOS** — look for FreeRTOS, Zephyr, RTIC, Embassy, bare-metal markers.
5. **External packages** — dependencies in `Cargo.toml` or `#include <...>` matches against known SDKs; present unrecognized ones for confirmation.
6. **Modules** — discover from `src/` structure. For Rust workspaces, read `[workspace] members`. For single-crate Rust, treat each `src/**/mod.rs` or top-level `src/<name>.rs` as a candidate module.
7. **Peripherals and pins** — grep for peripheral initialization (`Peripherals::take`, `gpio_init`, `twi_init`, etc.), collect pin numbers and peripheral names.
8. **Connections** — look for wiring between modules (calls from module A into module B's public surface).

Present candidate `specs/architecture.toml` with confidence (HIGH/MEDIUM/LOW) per field. Wait for confirmation. Write clean TOML per `specdev/shared/comment-policy.md` — in particular, LOW markers are permitted inline next to LOW-confidence fields and are removed after the user confirms each one. Scaffold `specs/modules/` and `specs/requirements/` if absent.

Report: language, build system, target, RTOS, packages, modules discovered. Suggested next step: `/spec-reverse <Module>` or `/spec-reverse --all`.
