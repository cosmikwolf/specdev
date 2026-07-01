# Block artifact templates

Exact shapes for the files `/spec-create-block` scaffolds. Placeholders: `<name>`
= block directory name (lowercase-hyphenated, concrete); `<key>` = the schema
**table key** — the same string with hyphens turned to underscores (`nih-plug` →
`nih_plug`); `<lang>` = target language (default `rust`); `<Kind>`/`<plural>` =
an entity kind and its table name; `<case>` = a discrete worked-example case.
Fill every `TODO:` from the interview (Step 1) and the **resolved real API**
(Step 2) — never from memory.

Live precedents (read the one whose shape matches):
- `blocks/embassy/` — two entity kinds, no relations, `no_std` cross-target reference.
- `blocks/egui/` — declares an intra-block relation (`region_in_panel`), `std` host reference.
- `blocks/nih-plug/` — hyphenated name → `nih_plug` table key; `std` host reference.

⚠️ **Hyphen/underscore:** `<name>` (dir, `$id`, reduction entity qualifier) keeps
the hyphen; `<key>` (schema table property + `for_each` path) uses the underscore.
`nih-plug` → dir `nih-plug/`, `$id .../nih-plug.v1`, entity `nih-plug.Parameter`,
but table `[nih_plug]` and `for_each: "nih_plug.parameters[]"`.

---

## 1. `schema.yaml`

```yaml
$schema: "https://json-schema.org/draft/2020-12/schema"
$id: "https://specdev.dev/blocks/<name>.v1"          # BLOCK-LINT-006 checks this exactly
title: "<name> block (v1)"
description: >
  Schema fragment contributed by the `<name>` block. Adds a namespaced
  `[<key>]` table to `Module`. TODO: one paragraph — what concrete thing it
  models, which entity kinds, what is deferred to v1.x.
  Self-contained: identifier patterns are inlined, not $ref'd against a
  profile's enums.schema.yaml, so the block composes against any profile.
type: object
properties:
  <key>:                                # underscore form of <name>
    type: object
    additionalProperties: false         # LEAF — additionalProperties, NOT unevaluatedProperties
    properties:
      <plural>:
        description: >
          TODO: what each entry declares.
        type: array
        items: { $ref: "#/$defs/<Kind>" }
      # ... more entity-kind tables

$defs:
  Identifier:
    description: snake_case or PascalCase identifier usable as a Rust ident.
    type: string
    pattern: "^[A-Za-z_][A-Za-z0-9_]*$"
  # RustTypeExpression (free-form Rust type; validated at codegen, not schema):
  #   type: string, minLength: 1

  <Kind>:
    type: object
    additionalProperties: false
    required: [name]                    # plus any other mandatory fields
    properties:
      name: { $ref: "#/$defs/Identifier" }
      # TODO: kind-specific fields — enums with `default`, integers with
      # minimum/maximum, RustTypeExpression strings. One per schema field you
      # want surfaced as a reduction attr.
```

**Invariants:** `$id` = `https://specdev.dev/blocks/<name>.v1` (BLOCK-LINT-006);
`additionalProperties: false` on the `[<key>]` table and each nested object (a
block table is a leaf — `unevaluatedProperties` is only for aggregator targets
like a profile's module schema); `Identifier` etc. inlined in `$defs`, never
`$ref`-ing a profile enum.

---

## 2. `reduction.yaml`

```yaml
metaschema_version: "v1"

# <name> block reduction. Each entity kind materializes under the `<name>`
# namespace (file location establishes it → kinds resolve to `<name>.<Kind>`).
# All extend Module; the qualified name combines parent module + local name so
# multiple modules' entities never collide.

entities:
  - kind: <Kind>
    extends: Module
    from: "specs/modules/*.toml"
    for_each: "<key>.<plural>[]"         # underscore table key
    name: "${root.name}.${.name}"
    attrs:
      # TODO: <field>: ".<field>"   (1:1 with the schema properties you exposed)

relations: []                            # [] unless the block has an INTRA-block relation
# Intra-block relation (only if one entity kind references another WITHIN the block,
# e.g. egui Region.panel → Panel):
#   relations:
#     - kind: <relation>
#       extends: depends_on
#       from_kind: <name>.<KindA>        # path-qualified with the block DIR name (hyphenated)
#       to_kind: <name>.<KindB>
#       from_spec: "specs/modules/*.toml"
#       for_each: "<key>.<pluralA>[]"
#       endpoints: { from: "${root.name}.${.name}", to: "${root.name}.${.<field>}" }
```

**Invariants:** `metaschema_version: "v1"`; every non-canonical entity `extends`
Module (or a canonical kind); `for_each` rooted at the `<key>` namespace; a
cross-block reference is **not** allowed here (that's a bridge). Getting the
`from_kind`/`to_kind` qualifier wrong (underscore instead of the hyphenated dir
name) yields a silent `REDUCE_UNRESOLVED`, not a schema error.

---

## 3. `codegen/<lang>/README.md`

The load-bearing artifact (the shipped blocks run ~1500–1700 tokens — advisory,
not lint-gated; thin prose yields thin codegen). Structure by concern; each
section gets at least one explicit **do-NOT** rule. Suggested sections:

1. **Crate dependencies** — the crate(s) from `deps.yaml`, with the feature notes.
2. **Type emission** — how the `[<key>]` entries become Rust types/decls.
3. **Per-kind emission** — a worked example per entity kind, types drawn from
   Step 2's **resolved real API** with full generic arity (this is what
   `REF-ARITY-001` checks against the reference).
4. **Error handling / lifetimes / ownership** as relevant.

Hard rules:
- **Leg 3 — no sibling-block names.** Do not mention another block's types. If the
  reader "needs" embassy here, that coupling belongs in the `<name>+embassy`
  bridge, not this prose. An explicit "do NOT add <other-block>" line is allowed
  and good; an implicit reach is a violation.
- **PROSE-PROFILE-001 — no profile name.** Write "the project's profile's
  `module.conventions.md`", never a specific profile. The block layers on whatever
  profile the project uses.

---

## 4. `codegen/<lang>/deps.yaml`

```yaml
# Manifest — Cargo deps the <name> block contributes.
# Schema: specdev-analyzer/src/embedded/block_deps.schema.yaml.
crates:
  - name: <crate>
    version: "<x.y>"                     # or pin a git rev under `source`
    features: [<...>]                    # TODO: from the resolved API
    source: crates-io
recommended_external_deps: []            # surfaced to the user, NOT auto-injected
```

For a reference **opt-out** block (unreleased API), document the reason here (a
comment) and in `README.md`, and ship no `reference/`.

---

## 5. `README.md` (author-facing catalog — never inlined)

Sections: `# `<name>` block`, `## When to use`, `## What this block contributes`,
`## What this block does NOT contribute`, `## Sharp-test rationale` (all three
legs, explicitly), plus companion-block/bridge affinity and deferred items.
Naming a profile or a companion block **here** is fine — this file is never
inlined, so it *should* say "pairs with the X block via the `<name>+X` bridge".

---

## 6. Reference crate — `reference/<lang>/` (discrete-case blocks)

A **sibling** to `codegen/`, never nested. Inline copies only
`codegen/<lang>/README.md` + `deps.yaml`, so `reference/` is CI-only by structure.
Invariant: "a lib that **type-checks without link**."

### `reference/<lang>/Cargo.toml`
```toml
# CI-only reference crate for the <name> block codegen prose. Type-checks the
# worked examples in ../../codegen/<lang>/README.md against the pinned API.
# NOT inlined — this dir is a sibling to codegen/.
[package]
name = "<name>-reference"                # hyphens ok in a package name
version = "0.0.0"
edition = "2021"
publish = false

[lib]

[dependencies]
# Mirror what the block contributes at codegen; pin git deps by rev.
<crate> = { version = "<x.y>", features = [<...>] }
```

### `reference/<lang>/.cargo/config.toml` (cross-target only)
```toml
[build]
target = "<triple>"                      # e.g. thumbv8m.main-none-eabihf; OMIT for host-target
```

### `reference/<lang>/src/lib.rs`
```rust
//! Compiled reference examples for the <name> block codegen prose. CI-only.
#![no_std]                               // OMIT for a std host-target reference
pub mod <case_1>;
pub mod <case_2>;
```

### `reference/<lang>/src/<case>.rs`
```rust
//! <Case> reference — mirrors the README worked example. Annotate types
//! explicitly so the compiler checks arity.
use <crate>::<...>;
// real constructor / signature: exact from the resolved API (Step 2)
```

### `reference/<lang>/reference_cases.yaml`
```yaml
# Reference-crate coverage manifest. CI-only — never inlined. Drives
# REF-COVERAGE-001 (each name → src/<name>.rs) and REF-ARITY-001.
reference_cases:
  - <case_1>
  - <case_2>
```

**Case-name rules:** opaque strings that are valid module file stems; avoid Rust
keywords (fold an `enum`-like case into a combined file). Declare only the cases
you actually ship a compiled example for.

**Match the README (`REF-ARITY-001`):** the check compares generic arity **by base
type name** between the README and these reference sources — write the type text
identically in both (`Vec<u8, 16>` in the README ⇒ `Vec<u8, 16>` here). **Also
commit** a `.gitignore` (ignoring `target/`) and the resolved `Cargo.lock`, as the
shipped reference crates do, for reproducible CI.
