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

The analyzer ships these graph-check primitives (any profile or rule pack may use them; another `kind` is
`DOMAIN-003`, and `plugin-lint` refuses it in a pack):

- **`unique`** — `field` must be unique across the scoped entities; with `fields = [..]`, the values at those paths together form the key.
- **`related_attrs_allowed`** — for each relation of `relation_scope`, the pair (`left`, `right`) read from `relation.`, `from.` or `to.` attributes must be in `allowed` (`"*"` matches any value).
- **`reference_exists`** — every value at `source_field` must name a value at `target_field`.
- **`reachability`** — every node at `nodes_field` must be reachable from `entry_field` over the `edges_from` → `edges_to` edges.
- **`acyclic`** — the `edges_from` → `edges_to` edges form no cycle (optionally scoped by `entity_scope` + `relation_scope`).
- **`layer_dependency`** — for each `relation_scope` relation between `entity_scope` entities, the `to` entity's `layer_field` must not be above the `from` entity's, per `layer_ordering`.
- **`requirements_have_satisfiers`** — every requirement that needs a module is the target of a `relation_scope` relation.
- **`entity_has_inbound_relation`** — every `entity_scope` entity (optionally only those whose `entity_attr` equals `entity_attr_value`) is the target of a `relation_scope` relation.
- **`entity_attr_inbound_count`** — each entity whose `entity_attr` equals `entity_attr_value` has exactly `expected_count` inbound `relation_scope` relations.
- **`entity_count`** — the number of `entity_scope` entities whose `entity_attr` equals `entity_attr_value` must equal `expected_count`. A project with no entity of the scope yet passes.
- **`entity_attr_target_attr_equals`** — for each `entity_scope` entity whose `entity_attr` equals `entity_attr_value`, the entity its `target_attr_field` names is a `target_entity_kind` whose `target_attr` equals `target_attr_value`.
- **`ids_in_document`** — every `entity_scope` entity's id appears in `document`.
- **`verified_by_resolves`**, **`module_reachable_through_open_question`** — declared here, run by `check` (they read the code).

Profile-specific rule names map to these primitives via the `kind` field in the `[[project.graph_checks]]` block. A block's or bridge's checks live in its inlined rule pack, `specs/specdev/checks/{blocks,bridges}/<name>.toml`, with rules named `<unit>.<rule>` plus the code-check rules `pack_entity_in_code`, `pack_attribute_matches` and `pack_key_ambiguous`; see `rule-packs.md`.

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
| `BLOCK-LINT-001..007`, `BRIDGE-LEX-001`, `GUIDANCE-001..003`, `PROSE-PROFILE-001` | Plugin-author-time lint codes (run via `specdev-analyzer plugin-lint --plugin <root>`). `GUIDANCE-001..003` = each unit's `guidance.md` is present, holds no fenced code, and is under the size cap; `PROSE-PROFILE-001` = a block's or bridge's guidance must not name a profile. |
| `BOOTSTRAP-001..003`, `INLINE-001..003`, `MERGE-001..003`, `LINEAGE-001`, `DRIFT-INVARIANT-001..003` | Build-time CLI refusals. **Exit 1**, not 2 — the tool ran and refused on your data. |
| `PACK-000..002`, `PACK-LINT-001..005` | Rule packs: a pack loaded (info), an inlined pack or query that does not load (error), and the plugin-author lint. See `rule-packs.md`. |
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
