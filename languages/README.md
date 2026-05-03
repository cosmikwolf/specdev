# `languages/` — per-language item vocabulary

Fourth top-level plugin tier, peer to `blocks/`, `bridges/`, and `profiles/`. Each subdirectory holds the per-language artifacts that the analyzer needs to extract items from source and that profiles `$ref` into their `module.schema.yaml` `items[]` field.

## What lives here

For each language `<lang>`:

- **`items.schema.yaml`** — the JSON Schema 2020-12 fragment defining the `Item` shape for this language. Profiles' `module.schema.yaml` `items[]` field `$ref`s this schema via the `{{language}}` placeholder. Phase 10 ships the Rust schema; Phase 14 ships the schema-into-aggregator wiring + profile module-schema migrations; Phase 14-C ships the C schema.
- **`syntax.md`**, **`build.md`**, **`audit.scm`** — pre-arc artifacts retained in v1 because the `tree-sitter-<lang>` adapters in `specdev-analyzer/src/lang/` still `include_str!` from `audit.scm`. These will be renamed/refactored when Phase 10 migrates `audit/code_parser.rs` into `items/extract.rs` (see D40).

## Status (2026-05-02)

- `rust/items.schema.yaml` — Phase 10 commit, used by `extract-items` and (post-Phase-14) by Rust profile module schemas.
- `c/`, `cpp/` — pre-arc artifacts; new items-tier work pends Phase 14-C (items[] for C) per D38 + Phase 9 (C-language stack landing) per the revised phase order.

## Sparseness rule (per items[] commitment, doc 17 forthcoming)

Every `Item` carries `satisfies` (≥1 entry) **except** `kind = "re_export"`, where the inherently-contractual nature of a `pub use` makes `satisfies` optional. Helper functions that don't satisfy a requirement do **not** appear in `items[]`. This keeps the items array bounded — typical module: 5–20 items, not 100+.
