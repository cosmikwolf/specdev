# Build manifest detection

`/spec-init` Step 2 scans the project root for build manifests and extracts fields to pre-fill the Step 4 interview. For each present file, extract the fields below. Use the extracted values as defaults the user confirms or overrides — never silently overwrite.

| File | Extract | Maps to |
|---|---|---|
| `Cargo.toml` | `[package] name` | `[project].name` default |
| `Cargo.toml` | `[package] edition` | `[project].edition` (optional) default |
| `Cargo.toml` | `[package] rust-version` | `[msrv]` default in `architecture.toml` (library-api) |
| `Cargo.toml` | `[package] license` | `[project].license` (optional) default |
| `Cargo.toml` | `[package] description` | `[project].description` (optional) default |
| `Cargo.toml` | `[dependencies]` names | profile inference (cortex-m / embassy / stm32* → `firmware-layered`; otherwise → `library-api`) |
| `Cargo.toml` | `[workspace] members` | warn the user; v1 of specdev is single-package |
| `.cargo/config.toml` | `[build] target` triple | `architecture.toml [platform].target_triple` default (firmware-layered) |
| `rust-toolchain.toml` | `channel` | informational only — not written to spec |
| `CMakeLists.txt` | `project(...)` name | `[project].name` default (C projects; on the roadmap) |
| `platformio.ini` | `board`, `framework` | informational only |

## Usage rules

- Show every defaulted value to the user and let them confirm or change it. Never write a value extracted from a manifest without confirmation.
- Extractions that no profile consumes are harmless — they simply aren't surfaced by any interview question.
- If multiple manifests disagree (e.g. a `Cargo.toml` workspace with nested crates), prefer the outermost manifest and warn the user that v1 specdev is single-package only (multi-package support is on the roadmap).
- Do not run any manifest tool (`cargo metadata`, `cmake --trace`). Read-only text extraction only — `/spec-init` must work without any toolchain installed.

## Adding a new manifest type

1. Add a row to the table above (file name + extracted fields + destination).
2. Verify the destination field exists in the embedded `spec.schema.yaml` (`[project]`) or the relevant profile's `architecture.schema.yaml`. If not, the extraction has no consumer and should be left informational.
