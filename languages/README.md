# `languages/` — per-language queries and item vocabulary

Fourth top-level plugin tier, peer to `blocks/`, `bridges/`, and `profiles/`. Each subdirectory holds the per-language artifacts the analyzer reads source with — the tree-sitter queries behind `check`'s test discovery and import edges, and item extraction — and the item schema that profiles `$ref` into their `module.schema.yaml` `items[]` field.

## What lives here

For each language `<lang>`:

- **`items.schema.yaml`** — the JSON Schema 2020-12 fragment defining the `Item` shape for this language. Profiles' `module.schema.yaml` `items[]` field `$ref`s this schema via the `{{language}}` placeholder.
- **`audit.scm`**, **`tests.scm`**, **`imports.scm`**, **`items.scm`** — the tree-sitter queries. The `tree-sitter-<lang>` adapters in `specdev-analyzer/src/lang/` embed a copy of each (`include_str!`): `tests.scm` drives test discovery (`verified_by`, `report tests`), `imports.scm` the import edges (`internal_edges_declared`, `report imports`), `items.scm` item extraction (`extract-items`, `items diff`).
- **`build.md`** — the language's toolchain commands, for profile authors who write a profile's compile gates.

## Status

- `rust/items.schema.yaml` — used by `extract-items` and by Rust profile module schemas.
- `c/`, `cpp/` — `audit.scm`, `tests.scm`, `imports.scm`, `build.md`; no item vocabulary yet.
- `dart/` — full tier: `audit.scm`, `items.schema.yaml`, `items.scm`, `imports.scm`, `tests.scm`, `build.md` (pub/flutter toolchain). Used by `extract-items` and by the `flutter-app` profile's module schema.
- `typescript/` — full tier: `audit.scm`, `items.schema.yaml`, `items.scm`, `imports.scm`, `tests.scm`, `build.md` (Node/pnpm toolchain). One language, two extensions (`.ts`, `.tsx`) and two grammars; the adapter picks the grammar per resolved file and the one items query compiles under both. Visibility is `export` (top level) and the absence of `private`/`protected`/`#` (members). Used by `extract-items` and by the `nextjs-app` profile's module schema.

## Sparseness rule

Every `Item` carries `satisfies` (≥1 entry) except where a language's schema marks it optional (Rust's `kind = "re_export"`, whose inherently-contractual `pub use` nature makes `satisfies` optional). Helper functions that don't satisfy a requirement do **not** appear in `items[]`. This keeps the items array bounded — typical module: 5–20 items, not 100+.

## Kind vocabulary is data

`imports.scm` has one capture, `@import.source`: the module specifier of each import. The analyzer resolves each specifier to a project file (`specdev-analyzer report imports` shows the resolution), and `check` holds the resulting module-to-module edges to the declared `depends_on` (`internal_edges_declared`). External packages are the manifest's business: the package manager already refuses one the manifest does not declare.

The analyzer holds no compile-time item-kind list. `items.schema.yaml`'s `kind` enum is the authority: it validates spec TOML (via the profile `$ref`), and the analyzer loads the same inlined copy at runtime for the `items diff` / `report coverage` kind gate. The `items.scm` capture prefixes (`<kind>.def` / `<kind>.name`, plus the `inherent_<kind>.*` + `inherent.host` member convention) must use the same kind strings — the extractor derives each item's `kind` from the capture prefix. Adding or removing a kind is an edit to these two files only.
