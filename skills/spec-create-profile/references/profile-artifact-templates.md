# Profile artifact templates

Profiles are authored **copy-and-adapt**: clone the nearest shipped profile's tree
and change the load-bearing files. This doc gives the full required-file inventory,
the shapes of the files you actually *change* (the rest are near-verbatim reuse),
and the invariants `bootstrap`/`validate` enforce. Placeholders: `<name>` = profile
name (`^[a-z][a-z0-9-]*$`); `<lang>` = language.

Precedents (clone the closest): `profiles/library-api/` (standalone library, no
layers), `profiles/firmware-layered/` (no_std binary, 5-tier + metadata schema),
`profiles/audio-plugin/` (plugin/cdylib, 4-tier, features), `profiles/rust-workspace/`
(workspace root).

---

## Required-file inventory (bootstrap fails if any is missing)

```
profiles/<name>/
├── README.md                              # author — overview, when-to-use, graph-check list
├── profile.yaml                           # ADAPT — the analyzer dispatch manifest
├── spec.toml.template                     # adapt — {{project_name}}/{{profile}}/{{language}}
├── domain.toml.template                   # ADAPT — [domain] + [validation.rules] + graph_checks
├── architecture.toml.template             # reuse/adapt — empty/example
├── requirements.toml.template             # reuse/adapt — empty/example
├── decisions.toml.template                # reuse — triad skeleton
├── open_questions.toml.template           # reuse — triad skeleton (open/default_committed/closed)
├── reductions/project.yaml                # ADAPT — TOML → universal entities/relations
├── schemas/module.schema.yaml             # ADAPT — the load-bearing module shape
├── schemas/architecture.schema.yaml       # adapt — additionalProperties:false, CrateReference $ref
├── schemas/requirements.schema.yaml       # reuse — Requirement+Source, `nature` field
├── schemas/enums.schema.yaml              # reuse+extend — Identifier/ModuleRef/RequirementId/…
├── schemas/decisions.schema.yaml          # reuse shape; rewrite $id + title
├── schemas/open_questions.schema.yaml     # reuse shape; rewrite $id + title
├── schemas/<aux>.schema.yaml              # optional — e.g. firmware's metadata.schema.yaml
└── codegen/<lang>/module.conventions.md   # AUTHOR — any *.conventions.md (name it for what it
                                            # describes, e.g. module/workspace — NOT the profile); ≥1 per language
```

---

## `profile.yaml` (ADAPT — the dispatch contract)

Validated against `specdev-analyzer/src/embedded/profile.schema.yaml`. Required
top-level keys: `name`, `language`. Shape:

```yaml
# Manifest — drives analyzer dispatch for <name> projects.
# Schema: specdev-analyzer/src/embedded/profile.schema.yaml.
name: <name>
language: <lang>

cargo_toml:
  shape: library                  # CLOSED enum: library | binary | plugin | workspace
  default_edition: "2021"
  lib: {}                         # or [[bin]]/cdylib config appropriate to `shape`
  features:                       # descriptive: the renderer dispatches on `shape` and
    synthesize_default: true      #   reads architecture.toml tables by fixed key; nothing
    drain_from: ["features"]      #   reads these keys — keep them truthful for the shape
  architecture_deps:              # descriptive, same rule
    typed_slots: []               # named infra-dep slots (platform/runtime/logger for firmware)
    open_arrays: ["external_deps"]

bootstrap:
  steps: []                       # workspace profiles add steps here

context:
  module_fields_for_prompt: [items, satisfies, depends_on, context_files, external_deps]
  context_subcommand_extras: []   # e.g. {field: visibility, from_attr: visibility}

surface:
  has_modules: true
  has_requirements: true
  has_architecture: true
```

Pick `cargo_toml.shape` from the closed enum. A shape not in the enum needs
analyzer work — out of the skill's scope.

---

## `schemas/module.schema.yaml` (ADAPT — load-bearing)

The single most important file: it defines what a module spec carries for this
shape. Invariants:

- **Root uses `unevaluatedProperties: false`, NEVER `additionalProperties: false`.**
  Block contributions arrive through the aggregator's `allOf` slot, and
  `additionalProperties` is not `allOf`-aware — it would reject every inlined
  block's table. (This is the inverse of a *block* table, which is a leaf and DOES
  use `additionalProperties: false`.)
- `items[]` references `inlined/languages/{{language}}/items.schema.yaml` — the one
  place bootstrap substitutes `{{language}}`.
- `$id`: `https://specdev.dev/profiles/<name>/module.schema.yaml`.
- Use the `CrateReference` `$ref` for any Cargo-dep field; `$ref` `enums.schema.yaml`
  for `Identifier`/`ModuleRef`/`RequirementId`/etc.

Author the fields *this* shape needs (firmware: `layer`/`actions`/`ports`/
`state_machines`/`hardware`/`metadata`; library: `visibility`/`items`/`feature_gates`).

---

## `domain.toml.template` (ADAPT — the graph-check set)

```toml
[domain]
name = "{{project_name}}"

[validation]
# ...
[validation.rules]
requirements_have_satisfiers = "warning"   # every profile ships this (F-006)
# <shape-specific checks> = "error" | "warning"

[[project.graph_checks]]
# one block per check; reuse existing primitives (modules_acyclic,
# layer_dependency, entity_has_inbound_relation, the *_resolves family).
# A brand-new check KIND needs analyzer work — out of scope.
```

---

## `reductions/project.yaml` (ADAPT)

```yaml
metaschema_version: "v1"
entities:
  # each with kind, extends, from  (e.g. Module extends nothing / a canonical kind)
relations:
  # each with kind, extends, from_kind, to_kind, from_spec, for_each, endpoints
```

Maps the profile's TOML files to the universal graph. Copy the precedent's and
adjust for the fields your module schema added.

---

## Reuse points (copy the shape — do NOT re-author; rewrite each file's `$id` to `<name>`)

- **`CrateReference`** — `$ref`
  `https://specdev.dev/schema/meta/v1/crate_reference.schema.yaml#/$defs/CrateReference`
  for every Cargo-dep field. Never redefine the shape.
- **Enum vocabularies** — `Identifier`, `ModuleRef`, `RequirementId`, `FeatureName`,
  `QualifiedPort` in `enums.schema.yaml#/$defs/`. Stable across profiles; copy from
  any precedent, then add profile-specific enums (e.g. `LayerKind`).
- **`requirements_have_satisfiers`** — every profile ships this graph-check; the
  `nature = "module"` default exempts non-module requirements from the orphan check.
- **Triad schemas** — `decisions.schema.yaml` / `open_questions.schema.yaml` are
  stable; copy the shape (rewrite the `$id`). Status vocab: `open` / `default_committed` / `closed`.
- **Placeholder vocabulary (closed)** — `{{project_name}}`, `{{profile}}`,
  `{{language}}`. Any other `{{...}}` → `BOOTSTRAP-002`.

---

## Codegen conventions (`codegen/<lang>/module.conventions.md`) — AUTHOR

Uniform rules mapping a module spec to source. No reference crate (profiles are
generics-free / uniform). Discipline: **name the role a block fills, not a specific
block** — a profile is pluralistic; list frameworks only as `e.g.` examples, unless
the profile is genuinely single-framework and says so. (The symmetric side of the
block/bridge `PROSE-PROFILE-001` rule.) At least one `*.conventions.md` per
supported language, or bootstrap fails.

---

## Gate (no reference crate — bootstrap round-trip instead)

1. `specdev-analyzer bootstrap --profile <name> --language <lang> --name smoke --plugin-root <root> <scratch-dir>`
   — the trailing `<scratch-dir>` **TARGET is required** (omitting it exits 2).
   Catches missing files / malformed manifest (`BOOTSTRAP-00x`).
2. Author a minimal module → `specdev-analyzer validate --no-color` → zero errors.
3. Optional: `/spec-gen` the minimal module + `cargo build` (the smoke-fixture bar).
4. `specdev-analyzer plugin-lint --plugin <root>`.
