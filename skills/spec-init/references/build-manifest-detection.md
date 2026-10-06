# Build manifest detection

`/spec-init` Step 2 scans the project root for build manifests and extracts fields to pre-fill the Step 4 interview. For each present file, extract the fields below. Use the extracted values as defaults the user confirms or overrides — never silently overwrite. Versions, editions, licenses, descriptions, engines and dependency lists stay in the manifest: the spec never copies them.

| File | Extract | Maps to |
|---|---|---|
| `Cargo.toml` | `[package] name` | `[project].name` default |
| `Cargo.toml` | `[dependencies]` names | profile inference (cortex-m / embassy / stm32* → `firmware-layered`; otherwise → `library-api`) |
| `Cargo.toml` | `[workspace] members` | the root is a workspace: offer `--profile rust-workspace` for the root (members pre-fill `[[workspace.members]]`; `default-members` and `exclude` stay in `Cargo.toml`), then run `/spec-init` inside each member with that crate's own profile |
| `.cargo/config.toml` | `[build] target` triple | `architecture.toml [platform].target_triple` default (firmware-layered) |
| `rust-toolchain.toml` | `channel` | informational only — not written to spec |
| `package.json` | `name` | `[project].name` default (typescript) |
| `package.json` | `dependencies` names | profile inference (`next` → `nextjs-app`) |
| `pubspec.yaml` | `name` | `[project].name` default (dart) |
| `pubspec.yaml` | `dependencies.flutter.sdk` | profile inference (present → `flutter-app`) |
| `CMakeLists.txt` | `project(...)` name | `[project].name` default (C projects; on the roadmap) |
| `platformio.ini` | `board`, `framework` | informational only |

## Usage rules

- Show every defaulted value to the user and let them confirm or change it. Never write a value extracted from a manifest without confirmation.
- Extractions that no profile consumes are harmless — they simply aren't surfaced by any interview question.
- A `Cargo.toml` with a `[workspace]` table is a workspace root. Bootstrap it with `--profile rust-workspace` (its `members` list pre-fills `[[workspace.members]]`), then run `/spec-init` inside each member directory with that crate's own profile — member names must match each member's `[package] name`. Do not fold member manifests into the root: a workspace spec carries no per-member surface.
- Do not run any manifest tool (`cargo metadata`, `cmake --trace`, `pnpm ls`, `flutter pub deps`). Read-only text extraction only — `/spec-init` must work without any toolchain installed.
- **The lockfile is never a source.** `pnpm-lock.yaml`, `pubspec.lock` and `Cargo.lock` are the package manager's resolved output, not the project's declaration. Read the manifest.
- A `package.json` with a `workspaces` key, or a sibling `pnpm-workspace.yaml`, marks a JavaScript workspace root. **No shipped profile drives a JavaScript workspace root.** Bootstrap each member package with `nextjs-app` (or its own profile) and tell the user that the workspace root is theirs.

## Adding a new manifest type

1. Add a row to the table above (file name + extracted fields + destination).
2. Verify the destination field exists in the embedded `spec.schema.yaml` (`[project]`) or the relevant profile's `architecture.schema.yaml`. If not, the extraction has no consumer and should be left informational.
