# Profile artifact templates

Profiles are authored **copy-and-adapt**: clone the nearest shipped profile's tree
and change the load-bearing files. This doc gives the file inventory, the shapes of
the files you actually *change* (the rest are near-verbatim reuse), and the
invariants `bootstrap`/`check` enforce. Placeholders: `<name>` = profile name
(`^[a-z][a-z0-9-]*$`); `<lang>` = language.

Precedents (clone the closest): `profiles/library-api/` (standalone library, no
layers), `profiles/firmware-layered/` (no_std binary, 5-tier + metadata schema),
`profiles/audio-plugin/` (cdylib plugin, 4-tier, parameter bindings),
`profiles/rust-workspace/` (workspace root), `profiles/nextjs-app/` (roles, several
paired files, two gates), `profiles/python-library/` (gates per tool under
`toolchains:`), `profiles/ts-library/` and `profiles/dart-package/` (libraries in
those languages).

---

## File inventory

```
profiles/<name>/
├── README.md                              # author — overview, when-to-use, graph checks, CI toolchain list
├── profile.yaml                           # ADAPT — the analyzer dispatch manifest
├── guidance.md                            # AUTHOR — implementation guidance (rules only, ≤ 4,096 bytes)
├── spec.toml.template                     # adapt — {{project_name}}/{{profile}}/{{language}}
├── domain.toml.template                   # ADAPT — [domain] + [validation.rules] + graph_checks
├── architecture.toml.template             # optional — omit when the architecture schema has nothing to fill
├── reductions/project.yaml                # ADAPT — TOML → universal entities/relations
├── schemas/module.schema.yaml             # ADAPT — the load-bearing module shape
├── schemas/architecture.schema.yaml       # adapt — additionalProperties:false; only fields a check reads
├── schemas/requirements.schema.yaml       # reuse — Requirement+Source, `nature` field
├── schemas/enums.schema.yaml              # reuse+extend — Identifier/ModuleRef/RequirementId/…
├── schemas/decisions.schema.yaml          # reuse shape; rewrite $id + title
├── schemas/open_questions.schema.yaml     # reuse shape; rewrite $id + title
├── schemas/<aux>.schema.yaml              # optional — e.g. firmware's metadata.schema.yaml
└── templates/specdev-ci.yml.template      # adapt — the CI workflow (a paired file)
```

---

## `profile.yaml` (ADAPT — the dispatch contract)

Validated against `specdev-analyzer/src/embedded/profile.schema.yaml`. Required
top-level keys: `name`, `language`. Shape:

```yaml
# Manifest — drives analyzer dispatch for <name> projects.
# Schema: specdev-analyzer/src/embedded/profile.schema.yaml.
contract_version: 3

name: <name>
language: <lang>

# Config files written once when absent by `specdev-analyzer bootstrap`,
# then owned by the project.
paired_files:
  - { template_file: templates/specdev-ci.yml.template, target: .github/workflows/specdev.yml }

# Compile gates, as data: rendered into scripts/specdev-gates.sh after
# `specdev-analyzer check`; the first non-zero exit fails the run.
gates:
  - label: <label>
    steps:
      - [<program>, <arg>]

bootstrap:
  steps: []                       # workspace profiles add steps here

context:
  context_subcommand_extras: []   # e.g. {field: visibility, from_attr: visibility}

surface:
  has_modules: true
  has_requirements: true
  has_architecture: true
```

---

## `schemas/module.schema.yaml` (ADAPT — load-bearing)

The single most important file: it defines what a module spec carries for this
shape. Invariants:

- **Root uses `unevaluatedProperties: false`, NEVER `additionalProperties: false`.**
  Block contributions arrive through the aggregator's `allOf` slot, and
  `additionalProperties` is not `allOf`-aware — it would reject every inlined
  block's table. (This is the inverse of a *block* table, which is a leaf and DOES
  use `additionalProperties: false`.)
- The optional `items[]` references `inlined/languages/{{language}}/items.schema.yaml`
  — the one place bootstrap substitutes `{{language}}`.
- `$id`: `https://specdev.dev/profiles/<name>/module.schema.yaml`.
- `$ref` `enums.schema.yaml` for `Identifier`/`ModuleRef`/`RequirementId`/etc.
- Include the universal fields: `name`, `description`, `source_file`, `planned`,
  `satisfies`, `depends_on`, `guided_by`, `context_files`.
- **No field that restates the code or the manifest** — no types, signatures,
  dependency entries or feature lists.

Author the fields *this* shape needs (firmware: `layer`/`actions`/`ports`/
`state_machines`/`hardware`/`metadata`; library: `visibility`/`items`).

---

## `domain.toml.template` (ADAPT — the graph-check set)

```toml
[domain]
name = "{{project_name}}"

[validation]
# ...
[validation.rules]
requirements_have_satisfiers = "error"     # every profile ships this
implementation_unblocked     = "error"     # the open-question gate, run by `check`
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
  # each with kind, extends, from  (ledger kinds read the per-entry
  # globs, e.g. from: "specs/requirements/*.toml")
relations:
  # each with kind, extends, from_kind, to_kind, from_spec, for_each, endpoints
```

Maps the profile's TOML files to the universal graph. Copy the precedent's and
adjust for the fields your module schema added.

---

## Reuse points (copy the shape — do NOT re-author; rewrite each file's `$id` to `<name>`)

- **Enum vocabularies** — `Identifier`, `ModuleRef`, `RequirementId`,
  `QualifiedPort` in `enums.schema.yaml#/$defs/`. Stable across profiles; copy from
  any precedent, then add profile-specific enums (e.g. `LayerKind`).
- **`requirements_have_satisfiers`** — every profile ships this graph check; the
  `nature = "module"` default exempts non-module requirements, and a `planned`
  requirement is exempt.
- **Ledger schemas** — `requirements`, `decisions`, `open_questions` are stable;
  copy the shape (rewrite the `$id`). Open-question statuses: `open` /
  `default_committed` / `closed`, plus `hard_gate`.
- **Placeholder vocabulary (closed)** — `{{project_name}}`, `{{project_identifier}}`
  (the name with each `-` as `_`), `{{profile}}`, `{{language}}`. Any other `{{...}}` → `BOOTSTRAP-002`.

---

## `guidance.md` (AUTHOR)

Rules for code in this shape that no check can verify. Rules only: no fenced code
(`GUIDANCE-002`), no version numbers, at most 4,096 bytes (`GUIDANCE-003`).
**Name the role a block fills, not a specific block** — a profile is pluralistic;
list frameworks only as `e.g.` examples, unless the profile is genuinely
single-framework and says so. Inlined to `specs/specdev/guidance/profile.md`;
`context --module` lists it.

---

## `templates/specdev-ci.yml.template` (ADAPT)

Copy the closest shipped profile's workflow. It installs the toolchain the gates
need (and Rust, to build the private analyzer from `vars.SPECDEV_ANALYZER_REPO` with
`secrets.SPECDEV_ANALYZER_TOKEN`), then runs `sh scripts/specdev-gates.sh`. The
README's toolchain list names the same tools.

---

## Gate (bootstrap round-trip)

1. `specdev-analyzer bootstrap --profile <name> --language <lang> --name smoke --plugin-root <root> <scratch-dir>`
   — the trailing `<scratch-dir>` **TARGET is required** (omitting it exits 2).
   Catches missing files / malformed manifest (`BOOTSTRAP-00x`).
2. Author a minimal module and its code → `specdev-analyzer check --no-color` →
   zero errors.
3. `sh scripts/specdev-gates.sh` → exit 0.
4. `specdev-analyzer plugin-lint --plugin <root>`.
