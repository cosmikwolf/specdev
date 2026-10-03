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
- **`entity_count`** — the number of `entity_scope` entities whose `entity_attr` equals `entity_attr_value` must equal `expected_count`. A project with no entity of the scope yet passes.

Profile-specific rule names map to these primitives via the `kind` field in the `[[project.graph_checks]]` block.

## Schema and resolver error codes

Distinct from rule names, the analyzer emits framework-level codes that are NOT in `domain.toml`:

| Code family | Meaning |
|---|---|
| `LAYOUT-001..006` | Project tree shape (missing `specs/specdev/`, missing schema, etc.). |
| `SCHEMA`, `SCHEMA-ARCH`, `SCHEMA-REQ` | A TOML file violates its JSON Schema. |
| `METASCHEMA-001..005` | Inlined block/bridge schema is misshapen (kind grammar, `extends`, etc.). |
| `METASCHEMA-006` | A reduction relation reads its edges from neither or both of `from_spec` and `match`, or its `match` is misshapen. A relation like that would build no edges and say nothing. |
| `AGGREGATOR-005` | The aggregator failed to compile after inline. |
| `DOMAIN-003`, `REDUCE_UNRESOLVED` | A reduction's `extends` chain or relation endpoint references a kind/name that doesn't exist. |
| `BLOCK-LINT-001..007`, `BRIDGE-LEX-001`, `GUIDANCE-001..003`, `PROSE-PROFILE-001`, `REF-COVERAGE-001`, `REF-ARITY-001` | Plugin-author-time lint codes (run via `specdev-analyzer plugin-lint --plugin <root>`). `GUIDANCE-001..003` = each unit's `guidance.md` is present, holds no fenced code, and is under the size cap; `PROSE-PROFILE-001` = a block's or bridge's guidance must not name a profile; `REF-COVERAGE-001`/`REF-ARITY-001` = reference-crate checks, until R-G removes the crates. |
| `BOOTSTRAP-001..003`, `INLINE-001..003`, `MERGE-001..003`, `LINEAGE-001`, `DRIFT-INVARIANT-001..003` | Build-time CLI refusals. **Exit 1**, not 2 — the tool ran and refused on your data. |
| `PLUGIN-001` | Plugin root could not be resolved. Exit 2 — nothing ran. |
| `PLUGIN-AHEAD-001` | Info from `validate`: the plugin root named by `--plugin-root`, `SPECDEV_PLUGIN_ROOT` or `CLAUDE_PLUGIN_ROOT` holds newer copies of files this project inlined or bootstrapped (decided by content hash). Run `/spec-drift`. Silent when no plugin root is named. |

One of these protects files the tool did not write, added after a brownfield project lost
hand-authored content:

- **`BOOTSTRAP-003`** — `bootstrap` found a file already sitting where it planned to write. It names
  every collision at once and writes nothing. `--force` overwrites.

`INLINE-001..003` are easy to cite backwards; the binary's meanings are:

| Code | Means |
|---|---|
| `INLINE-001` | The block or bridge is not in the plugin tree, or is missing `schema.yaml` / `reduction.yaml`. |
| `INLINE-002` | Already inlined (use `--merge`), **or** a file collision at a destination path. |
| `INLINE-003` | A bridge endpoint block is not inlined yet. Also what a misspelled bridge name produces — the endpoint check runs first, so the typo is reported as a missing endpoint. |

These codes are stable across profiles and live in the analyzer source (`specdev-analyzer/src/lib.rs` docstring).

## Skill usage

- `/spec-validate`, `/spec-design`, `/spec-edit` translate findings against the project's own `domain.toml`. Read it first.
- Do not invent rule names. If a name appears in analyzer output that isn't in the project's `domain.toml`, the project is mid-bootstrap or `domain.toml` was hand-edited and got out of sync — flag it for the user, don't paper over it.
