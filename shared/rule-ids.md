# Validation rule names — where they live

Validation rule names and severities are **profile-specific** and live in each project's bootstrap-emitted `specs/specdev/domain.toml`. There is no plugin-wide canonical rule-id table any more — profiles author their own checks.

## Where to look in a project

`specs/specdev/domain.toml` ships two relevant tables:

- **`[validation.rules]`** — maps rule name → severity (`error`, `warning`, `info`). Example for firmware-layered:
  ```toml
  [validation.rules]
  unique_qualified_names        = "error"
  modules_acyclic               = "error"
  layer_dependency              = "error"
  connection_endpoints_resolve  = "error"
  requirement_satisfies_resolves = "warning"
  ```
- **`[[project.graph_checks]]`** — declares each check's `kind` (`unique`, `acyclic`, `layer_dependency`, `endpoints_resolve`, etc.), `scope`, and source fields. Read this to learn what the rule actually checks.

## Standard `kind` values

The analyzer ships these graph-check primitives (any profile may use them):

- **`unique`** — `field` must be unique across the scoped entities.
- **`acyclic`** — relations under `scope` form a DAG.
- **`layer_dependency`** / **`layer_dependency_graph`** — for each scoped relation `(from, to)`, the `to` entity's `layer_field` must be ≤ the `from` entity's per `layer_ordering`.
- **`endpoints_resolve`** — every value at `field` must name a materialized entity.
- **`reachability`** — every entity in `scope` must be reachable from a root.

Profile-specific rule names map to these primitives via the `kind` field in the `[[project.graph_checks]]` block.

## Schema and resolver error codes

Distinct from rule names, the analyzer emits framework-level codes that are NOT in `domain.toml`:

| Code family | Meaning |
|---|---|
| `LAYOUT-001..006` | Project tree shape (missing `specs/specdev/`, missing schema, etc.). |
| `SCHEMA`, `SCHEMA-ARCH`, `SCHEMA-REQ` | A TOML file violates its JSON Schema. |
| `METASCHEMA-001..005` | Inlined block/bridge schema is misshapen (kind grammar, `extends`, etc.). |
| `AGGREGATOR-005` | The aggregator failed to compile after inline. |
| `DOMAIN-003`, `REDUCE_UNRESOLVED` | A reduction's `extends` chain or relation endpoint references a kind/name that doesn't exist. |
| `BLOCK-LINT-001..007`, `BRIDGE-LEX-001` | Plugin-author-time lint codes (run via `specdev-analyzer plugin-lint --plugin <root>`). |
| `BOOTSTRAP-001..002`, `INLINE-001..003`, `MERGE-001..003`, `PLUGIN-001`, `LINEAGE-001`, `DRIFT-INVARIANT-001..003` | Build-time CLI errors (returned as exit-2). |

These codes are stable across profiles and live in the analyzer source (`specdev-analyzer/src/lib.rs` docstring).

## Skill usage

- `/spec-validate`, `/spec-design`, `/spec-edit` translate findings against the project's own `domain.toml`. Read it first.
- Do not invent rule names. If a name appears in analyzer output that isn't in the project's `domain.toml`, the project is mid-bootstrap or `domain.toml` was hand-edited and got out of sync — flag it for the user, don't paper over it.
