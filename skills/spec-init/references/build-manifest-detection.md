# Build manifest detection

`/spec-init` Step 2 scans the project root for build manifests and extracts fields to pre-fill the Step 4 / Step 4b interviews. For each present file, extract the fields below. Use the extracted values as defaults the user confirms or overrides — never silently overwrite.

| File | Extract |
|---|---|
| `Cargo.toml` | `[package] name`, `[package] edition`, `[package] rust-version`, `[dependencies]` names, `[workspace] members` |
| `.cargo/config.toml` | `[build] target` triple, `[env]` keys (MCU, RTOS hints), `build-std` presence |
| `rust-toolchain.toml` | `channel` |
| `CMakeLists.txt` | `project(...)` name, `set(CMAKE_SYSTEM_PROCESSOR ...)`, linked libraries |
| `sdkconfig.defaults` | all `CONFIG_*` lines (ESP-IDF) |
| `platformio.ini` | `board`, `framework`, `lib_deps` |

## Usage rules

- Show every defaulted value to the user and let them confirm or change it. Never write a value extracted from a manifest without confirmation.
- Extractions that no domain consumes are harmless — they simply aren't surfaced by any `init.md`'s interview questions.
- If multiple manifests disagree (e.g. a `Cargo.toml` workspace with nested crates), prefer the outermost manifest and note the discrepancy for the user.

## Adding a new manifest type

1. Add a row to the table above (file name + extracted fields).
2. If a field maps to a domain-specific slot, ensure the destination domain's `init.md` has an interview question that consumes it.
3. Do not add parsers that require running the manifest (`cargo metadata`, `cmake --trace`). Read-only text extraction only — `/spec-init` must work without any toolchain installed.
