;; Rust items query for tree-sitter.
;;
;; Captures top-level items per the items[] schema (see
;; `languages/rust/items.schema.yaml`). Each top-level construct that
;; corresponds to a `kind` in the schema gets its own pattern. The
;; analyzer's `extract-items` subcommand walks these matches and produces
;; `ExtractedItem { kind, name, signature?, attributes[], span }` per
;; capture.
;;
;; Conventions:
;; - Each item kind has at least an `<kind>.def` capture (the whole node)
;;   and a `<kind>.name` capture (the identifier). Where attributes
;;   precede the item, capture them as `<kind>.attrs`.
;; - `mod_item` and `extern_crate_declaration` are intentionally excluded
;;   per items[] design: modules are first-class spec entities; extern
;;   crate is an architecture-level concern.
;;
;; Top-level only: every pattern is anchored under `(source_file ...)` so
;; only direct children of the file are extracted. Items nested inside an
;; `impl`/`trait` body (associated fns/consts/types) or a nested `mod`
;; block are deliberately NOT items — the impl/trait is the item, and its
;; methods are codegen-derived from the impl's prose. An unanchored
;; pattern matches at any depth, which would surface every method as a
;; phantom top-level `fn` and make any module with an impl report drift
;; against a spec that (correctly) does not list methods.

;; ──────────────────────────────────────────────────────────────────
;; fn
(source_file
  (function_item
    name: (identifier) @fn.name
    parameters: (parameters) @fn.params
    return_type: (_)? @fn.return_type) @fn.def)

;; struct
(source_file
  (struct_item
    name: (type_identifier) @struct.name) @struct.def)

;; enum
(source_file
  (enum_item
    name: (type_identifier) @enum.name) @enum.def)

;; union
(source_file
  (union_item
    name: (type_identifier) @union.name) @union.def)

;; trait
(source_file
  (trait_item
    name: (type_identifier) @trait.name) @trait.def)

;; impl
(source_file
  (impl_item
    trait: (_)? @impl.trait
    type: (_) @impl.type) @impl.def)

;; type alias
(source_file
  (type_item
    name: (type_identifier) @type.name) @type.def)

;; const
(source_file
  (const_item
    name: (identifier) @const.name) @const.def)

;; static
(source_file
  (static_item
    name: (identifier) @static.name) @static.def)

;; re_export — `pub use ...;` at top level
;; The grammar models `pub use foo::bar as baz;` as
;; `(use_declaration (visibility_modifier) (_ ...))`.
(source_file
  (use_declaration
    (visibility_modifier) @re_export.vis
    argument: (_) @re_export.target) @re_export.def)

;; macro_rules!
(source_file
  (macro_definition
    name: (identifier) @macro.name) @macro.def)

;; extern block — `extern "C" { ... }`
(source_file
  (foreign_mod_item) @extern_block.def)
