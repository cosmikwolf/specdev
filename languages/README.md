# `languages/` — per-language item vocabulary

Fourth top-level plugin tier, peer to `blocks/`, `bridges/`, and `profiles/`. Each subdirectory holds the per-language artifacts that the analyzer needs to extract items from source and that profiles `$ref` into their `module.schema.yaml` `items[]` field.

## What lives here

For each language `<lang>`:

- **`items.schema.yaml`** — the JSON Schema 2020-12 fragment defining the `Item` shape for this language. Profiles' `module.schema.yaml` `items[]` field `$ref`s this schema via the `{{language}}` placeholder.
- **`syntax.md`**, **`build.md`**, **`audit.scm`** — tree-sitter-backed authoring helpers. The `tree-sitter-<lang>` adapters in `specdev-analyzer/src/lang/` `include_str!` from `audit.scm`.

## Status

- `rust/items.schema.yaml` — used by `extract-items` and by Rust profile module schemas.
- `c/`, `cpp/` — `items.schema.yaml` for C is on the roadmap; current artifacts cover the existing tree-sitter-backed adapters.
- `dart/` — `audit.scm`, `syntax.md`, `build.md` (pub/flutter toolchain); `items.schema.yaml` + `items.scm` are the next step for this tier.

## Sparseness rule

Every `Item` carries `satisfies` (≥1 entry) **except** `kind = "re_export"`, where the inherently-contractual nature of a `pub use` makes `satisfies` optional. Helper functions that don't satisfy a requirement do **not** appear in `items[]`. This keeps the items array bounded — typical module: 5–20 items, not 100+.
