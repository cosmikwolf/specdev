# `languages/` — per-language item vocabulary

Fourth top-level plugin tier, peer to `blocks/`, `bridges/`, and `profiles/`. Each subdirectory holds the per-language artifacts that the analyzer needs to extract items from source and that profiles `$ref` into their `module.schema.yaml` `items[]` field.

## What lives here

For each language `<lang>`:

- **`items.schema.yaml`** — the JSON Schema 2020-12 fragment defining the `Item` shape for this language. Profiles' `module.schema.yaml` `items[]` field `$ref`s this schema via the `{{language}}` placeholder.
- **`syntax.md`**, **`build.md`**, **`audit.scm`** — tree-sitter-backed authoring helpers. The `tree-sitter-<lang>` adapters in `specdev-analyzer/src/lang/` `include_str!` from `audit.scm`.

## Status

- `rust/items.schema.yaml` — used by `extract-items` and by Rust profile module schemas.
- `c/`, `cpp/` — `items.schema.yaml` for C is on the roadmap; current artifacts cover the existing tree-sitter-backed adapters.
- `dart/` — full tier: `audit.scm`, `items.schema.yaml`, `items.scm`, `imports.scm`, `syntax.md`, `build.md` (pub/flutter toolchain). Used by `extract-items` and (once a Dart profile ships) by Dart profile module schemas.
- `typescript/` — full tier: `audit.scm`, `items.schema.yaml`, `items.scm`, `imports.scm`, `syntax.md`, `build.md` (Node/pnpm toolchain). One language, two extensions (`.ts`, `.tsx`) and two grammars; the adapter picks the grammar per resolved file and the one items query compiles under both. Visibility is `export` (top level) and the absence of `private`/`protected`/`#` (members). Used by `extract-items` and (once a TypeScript profile ships) by TypeScript profile module schemas.

## Sparseness rule

Every `Item` carries `satisfies` (≥1 entry) except where a language's schema marks it optional (Rust's `kind = "re_export"`, whose inherently-contractual `pub use` nature makes `satisfies` optional) or where the item is a test (`test = true` carries `verifies` instead). Helper functions that don't satisfy a requirement do **not** appear in `items[]`. This keeps the items array bounded — typical module: 5–20 items, not 100+.

## Kind vocabulary is data

`imports.scm` has one capture, `@import.source`: the module specifier of each import. The adapter maps a specifier to a package name (`LanguageAdapter::import_package`), and `report imports` compares those names with what the module may rely on (its own and the architecture-level `external_deps`, plus the deps of the blocks and bridges it uses). A language without the file is reported `checked: false`, never clean. Rust ships none: cargo already refuses a crate that a package does not declare.

The analyzer holds no compile-time item-kind list. `items.schema.yaml`'s `kind` enum is the authority: it validates spec TOML (via the profile `$ref`), and the analyzer loads the same inlined copy at runtime for the `items diff` / `report coverage` kind gate. The `items.scm` capture prefixes (`<kind>.def` / `<kind>.name`, plus the `inherent_<kind>.*` + `inherent.host` member convention) must use the same kind strings — the extractor derives each item's `kind` from the capture prefix. Adding or removing a kind is an edit to these two files only.
