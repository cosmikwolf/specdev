# Bridge artifact templates

Exact shapes for the files `/spec-create-bridge` scaffolds. Placeholders:
`<a>`/`<b>` = lex-ordered endpoint block names (`a < b`); `<lang>` = target
language (default `rust`); `<table>` = the binding-table name; `<Case>`/`<case>`
= a discrete worked-example case. Fill every `TODO:` from the interview (Step 1)
and the **resolved real API** (Step 2) — never from memory.

The two shipped canonical bridges are the live precedents; read the one whose
shape matches the pairing:
- `bridges/embassy+stm32h5-rs/` — `no_std`, cross-target (embedded).
- `bridges/egui+nih-plug/` — `std`, host-target (GUI/audio).

---

## 1. `bindings.schema.yaml`

```yaml
$schema: "https://json-schema.org/draft/2020-12/schema"
$id: "https://specdev.dev/bridges/<a>+<b>.v1"
title: "<a>+<b> bridge (v1)"
description: |
  Schema fragment contributed by the `<a>+<b>` bridge. Adds shape under
  bindings."<a>+<b>" on `Module`. TODO: one paragraph on what it wires.
  The bridge does NOT introduce new entity kinds — it contributes a
  relation materialized from each <table>[] entry. See `reduction.yaml`.
type: object
properties:
  bindings:
    type: object
    properties:
      "<a>+<b>":
        type: object
        additionalProperties: false
        properties:
          <table>:
            description: |
              TODO: what each entry declares ("this <X> uses this <Y>").
            type: array
            items: { $ref: "#/$defs/<Entry>" }

$defs:
  <Entry>:
    type: object
    additionalProperties: false
    required: [<endpoint_a>, <endpoint_b>]     # TODO: the two endpoint refs
    properties:
      <endpoint_a>:
        description: |
          References an `<a>.<...>[].name` declared on this SAME module.
          Cross-module wiring is not supported in v1 (co-residency).
        type: string
        pattern: "^[A-Za-z_][A-Za-z0-9_]*$"     # inline Identifier — do NOT $ref a profile enum
      <endpoint_b>:
        description: |
          References a `<b>.<...>[].name` declared on this SAME module.
        type: string
        pattern: "^[A-Za-z_][A-Za-z0-9_]*$"
      # TODO: attribute fields (handoff / widget_kind / range_hint / …).
      # Model closed sets as `enum` with a `default`.
```

**Invariants:** `$id` matches the directory (`<a>+<b>.v1`); the single
`bindings.<a>+<b>` namespace key; `additionalProperties: false` on the table
body; endpoint patterns **self-contained** — never `$ref` a profile's
`enums.schema.yaml` (the bridge must compose against any profile carrying both
blocks). "Self-contained" allows either style, and the two shipped bridges differ:
a local `$defs` + `$ref` (as above), **or** a literal `pattern:` on each property
(no `$defs`). Both lint clean; the only rule is *never `$ref` a profile enum*.

---

## 2. `reduction.yaml`

```yaml
metaschema_version: "v1"

# <a>+<b> bridge reduction. Each <table>[] binding materializes one
# `<relation>` relation from the <a>/<b> entity to the other. The relation
# extends `depends_on` so acyclic / layer / reachability primitives run over
# bridge-contributed wiring with zero domain knowledge. Bridges declare
# relations ONLY — the metaschema rejects bridge-qualified entity kinds.

entities: []

relations:
  - kind: <relation>                    # TODO: semantic name, e.g. peripheral_owned_by
    extends: depends_on                 # a canonical relation kind
    from_kind: <a-dir>.<Entity>         # qualifier = block DIR name (may be hyphenated: nih-plug.Parameter)
    to_kind: <b-dir>.<Entity>           # NOT the underscore table key — see the trap in Invariants below
    from_spec: "specs/modules/*.toml"
    for_each: 'bindings."<a>+<b>".<table>[]'   # quoted bridge segment (path-selector strips the quotes)
    endpoints:
      from: "${root.name}.${.<endpoint_a>}"
      to: "${root.name}.${.<endpoint_b>}"
    attrs:
      # TODO: <attr>: ".<field>"   (binding fields → relation attributes codegen reads)
```

**Invariants:** `entities: []` always; exactly the relations, each `extends` a
canonical kind; `for_each` uses the quoted `"<a>+<b>"` segment. A binding that
references a nonexistent block entity surfaces as `REDUCE_UNRESOLVED` at
validate — no new graph-check primitive needed.

**Endpoints: two shapes, and `${root.name}` on both sides is only the first.**
The template above is the **intra-module** shape, which is what the five bridges
before `next-auth+prisma` all use. A **named cross-module** endpoint is equally
valid: one side stays `${root.name}.<...>` and the other is a name the binding
carries, resolved against another module's entity. `bridges/next-auth+prisma`
is the worked precedent, verified by dumping `specdev-analyzer reduce` — the
edge materializes with `from: Auth.auth` and `to: Db.datasource`, two different
modules.

⚠️ **Select the field the endpoint reads.** In the cross-module shape, point
`for_each` at the naming field itself and read `${.}`:

```yaml
    for_each: 'bindings."<a>+<b>".<field>'   # the STRING, not the table
    endpoints:
      from: "${root.name}.<suffix>"
      to: "${.}.<suffix>"
```

If `for_each` instead selects the binding **object** and an endpoint template
reads a **child** field, omitting that field aborts the reducer —
`template path ... resolved to nothing`, exit 2 — **even when the field is
`required`**, because reduction runs before schema enforcement. Selecting the
field makes that unreachable: absent field, no pick, no iteration, and the
author gets the ordinary "required property" error instead of a crash.

⚠️ **Hyphen/underscore trap on `from_kind`/`to_kind`.** These are qualified by the
block's **directory name** — which may be hyphenated (`nih-plug.Parameter`,
`stm32h5-rs.Peripheral`). That is a *different string* from the schema **table
key**, which is often the underscore form (`nih_plug`) and is what the `for_each`
path and the binding fields use (`bindings."<a>+<b>"` +
`<a-dir-underscored>.<table>[]`). The block's `reduction.yaml` gives you the
`<Entity>` kind (e.g. `Parameter`), **not** the qualifier — take the qualifier
from the block directory name. Getting this wrong yields a silent
`REDUCE_UNRESOLVED` (the kind never resolves), not a schema error.

---

## 3. `contract.yaml`

```yaml
# Manifest — <a>+<b> bridge dispatch contract.
# Schema: specdev-analyzer/src/embedded/bridge_contract.schema.yaml.

binding_locality: endpoint_module        # closed enum — pick per the two values below

diagnostic_template: |
  the {bridge_name} bridge requires intra-module wiring:
  binding endpoints must reference declarations on the same module that
  carries the bindings table. Move the binding or move the declaration.
```

The endpoint pair is encoded in the directory name — do not restate it in the
body. `binding_locality` is a **closed 2-value enum** — pick by *where the bound
entities live*, not by copying whichever precedent you happened to open:

- `endpoint_module` — both endpoints on the **same** module that carries the
  bindings (intra-module; the template above fits this case). Precedent:
  `embassy+stm32h5-rs`.
- `plugin_shell_unique` — the binding targets the project's **single
  `plugin_shell`-layer module** (auto-resolves when exactly one exists).
  Precedent: `egui+nih-plug` — whose `diagnostic_template` reads very differently
  and substitutes `{parameter}` / `{decl_module}` / `{binding_module}`.

`diagnostic_template` may substitute `{bridge_name}`, `{parameter}`,
`{decl_module}`, `{binding_module}`. Copy the template from **the precedent whose
`binding_locality` you chose**, not the other one.

---

## 4. `codegen/<lang>/README.md`

Prose that maps a `bindings."<a>+<b>".<table>` entry to source. Structure it by
concern (crate deps, entry-point integration, the per-case handoff, error
handling). Requirements:

- **Profile-agnostic (`PROSE-PROFILE-001`).** May name the two constituent
  *blocks* — that is the composition seam this bridge owns. Must **not** name a
  *profile*: write "the project's profile's `module.conventions.md`", never a
  specific profile name. (If genuinely profile-bound, mark the line
  `PROFILE-REF-OK` and name it deliberately — rare.)
- **Worked example per case, from the resolved API (Step 2).** Each documented
  type carries its **full real generic arity** (this is what `REF-ARITY-001`
  checks against the compiled reference). Show the constructor with real arg
  order/count, ancillary config (`Config`, pull-ups), and any extra
  interrupt/DMA wiring.
- **Bridge prose wins over block prose** for the cross-block wiring — say so
  where the join-product type replaces a block's standalone type.

Every generic type named here must be used at the same arity as in
`reference/<lang>/src/<case>.rs`, or `REF-ARITY-001` fires.

---

## 5. `codegen/<lang>/deps.yaml` (optional)

Only when the bridge owns a **join-product** crate (one neither block may own):

```yaml
# Manifest — Cargo deps the <a>+<b> bridge contributes.
# Schema: specdev-analyzer/src/embedded/block_deps.schema.yaml.
crates:
  - name: <join-product-crate>          # TODO: exists only because <a>+<b> are paired
    version: "<x.y>"                     # or pin a git rev under `source`
    features: [<...>]                    # TODO: from the resolved API
    source: crates-io                    # or a git source block
recommended_external_deps: []
```

Omit the file entirely if the bridge introduces no new crate. Merged into
Cargo.toml AFTER block deps (architecture → modules → blocks → bridges), so
user-authored deps win the merge.

---

## 6. `README.md` (author-facing catalog — never inlined)

Sections: `# `<a>+<b>` bridge`, `## When to use`, `## What this bridge
contributes`, `## What this bridge does NOT contribute`, `## Inline order`, plus
the co-residency rule and any deferred items. Naming a profile or a companion
block **here** is fine — this file is never inlined, so it should say "pairs
with the X profile for …". Include the sharp-test rationale (concrete pairing;
codegen-complete; the join-product belongs to the bridge, not either block).

---

## 7. Reference crate — `reference/<lang>/`

A **sibling** to `codegen/`, never nested under it. Inline copies only
`codegen/<lang>/README.md` + `deps.yaml` by name, so `reference/` is
structurally CI-only. The invariant is "a lib that **type-checks without
link**."

### `reference/<lang>/Cargo.toml`
```toml
# CI-only reference crate for the <a>+<b> bridge codegen prose.
# Type-checks the worked examples in ../../codegen/<lang>/README.md against the
# pinned API. NOT inlined — this dir is CI-only, a sibling to codegen/.
[package]
name = "<a>-<b>-reference"          # hyphens; opaque
version = "0.0.0"
edition = "2021"
publish = false

[lib]

[dependencies]
# Mirror what the bridge + both blocks contribute at codegen; pin git deps by rev.
<join-product-crate> = { version = "<x.y>", features = [<...>] }
# <block deps the generated code uses>
```

### `reference/<lang>/.cargo/config.toml` (cross-target only)
```toml
[build]
target = "<triple>"                 # e.g. thumbv8m.main-none-eabihf; OMIT for a host-target reference
```

### `reference/<lang>/src/lib.rs`
```rust
//! Compiled reference examples for the <a>+<b> bridge codegen prose.
//! Each module mirrors the SHAPE `/spec-gen` emits for one case. CI-only.
#![no_std]                          // OMIT for a std host-target reference
// Shared setup that would duplicate symbols if per-case (e.g. bind_interrupts!)
// goes here, fully-qualified so case modules keep their plain names.
pub mod <case_1>;
pub mod <case_2>;
```

### `reference/<lang>/src/<case>.rs`
```rust
//! <Case> reference — mirrors the README worked example. The EXPLICIT
//! endpoint-type annotation is what makes the compiler check arity.
use <crate>::<...>;

// Annotate the endpoint type explicitly — a bare `let x = T::new(...)` with
// inference would NOT catch a wrong table type.
fn <shape>(x: <RealType<'static, ...>>) { /* mirrors emitted code */ }

pub fn build(/* entry-point args */) {
    // real constructor: exact arg order/count from the resolved API (Step 2)
}
```

### `reference/<lang>/reference_cases.yaml`
```yaml
# Reference-crate coverage manifest. CI-only — never inlined. Declares the
# discrete cases this bridge ships a COMPILED reference for. Drives
# REF-COVERAGE-001 (each name → src/<name>.rs) and REF-ARITY-001.
reference_cases:
  - <case_1>
  - <case_2>
```

**Case-name rules:** opaque strings that must be valid module file stems; avoid
Rust keywords (fold an `enum`-like case into a combined file). Declare only the
cases you actually ship a compiled example for — documenting more kinds in the
README than you build is the *completeness* gap (surfaced by plugin CI / this
skill), which is fine and separate from the coverage/arity consistency checks.

**Match the README (`REF-ARITY-001`):** the check compares generic arity **by base
type name** between the README and these reference sources — write the type text
identically in both (`I2c<'static, Async, Master>` in the README ⇒ the same here).
**Also commit** a `.gitignore` (ignoring `target/`) and the resolved `Cargo.lock`,
as the shipped reference crates do, for reproducible CI.
