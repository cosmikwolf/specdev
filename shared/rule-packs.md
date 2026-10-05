# Rule packs — a unit's `checks.toml`

A block or bridge may ship its checks as data: `checks.toml` at the unit root, plus query files under
`languages/<lang>/`. `inline` copies them into the project:

| Unit file | Inlined to |
|---|---|
| `checks.toml` | `specs/specdev/checks/{blocks,bridges}/<name>.toml` |
| `languages/<lang>/<file>.scm` | `specs/specdev/queries/{blocks,bridges}/<name>/<lang>/<file>.scm` |

Every language's query files are copied, whatever the project's language. The inlined copies are the ones that
run: a project changes a check by editing its copy, and `drift` reports the edit. `validate` runs the pack's graph
checks; `check` also runs its code checks. A profile has no pack: its checks live in the project's `domain.toml`.

## Shape

```toml
[rules]                        # default severities; optional
"<unit>.pins_unique" = "warning"

[[graph_checks]]               # the domain.toml graph-check shape
kind = "unique"
rule = "<unit>.pins_unique"
scope = "<unit>.Pin"
fields = ["attrs.port", "attrs.pin"]

[[code_checks]]                # spec ↔ code
language = "rust"
query = "languages/rust/tasks.scm"
entity_kind = "<unit>.Task"
key_capture = "task.name"
arg_key_capture = "arg.key"
arg_value_capture = "arg.value"

[code_checks.attributes]       # source argument → spec attribute
priority = "priority"
```

Quote a dotted key in `[rules]`: an unquoted `<unit>.rule = "error"` is a TOML table, not a key.

## Rule names and severity

- Every graph-check rule and every `[rules]` key is named `<unit>.<rule>` (`plugin-lint` checks it). Packs share
  one severity namespace with the project's `domain.toml`; a bare name could collide with a profile's rule.
- Code checks report three generic rules, with the entity named in each finding: `pack_entity_in_code` (a spec
  entity the code does not have), `pack_attribute_matches` (a mapped attribute that differs, including one side
  giving it and the other not), and `pack_key_ambiguous` (the code has the key at more than one place; nothing is
  compared until one remains).
- Severity: the project's `domain.toml` `[validation.rules]`, else the pack's `[rules]`, else `error`.

## Graph checks

Any primitive in `rule-ids.md` works. Two exist for packs:

- **`unique` with `fields`** — each entity of `scope` is keyed on the values at `fields` together. An entity missing
  a key part is not keyed; the schema owns presence.
- **`related_attrs_allowed`** — for each relation of `relation_scope`, read `left` and `right` (each
  `relation.<attr>`, `from.<attr>` or `to.<attr>`). The pair must be in `allowed`; `"*"` matches any value. A
  relation with an unresolved endpoint or a missing attribute is skipped.

The schema requires each pack primitive's arguments (`field` or `fields` for `unique`; `relation_scope`, `left`,
`right` and `allowed` for `related_attrs_allowed`), so a check cannot ship that checks nothing.

## Code checks

The query runs over every source file under the project's source root. Each match's `key_capture` is a code key;
`arg_key_capture` and `arg_value_capture` give that key's arguments. A spec entity of `entity_kind` matches the code
key equal to its name without the module prefix (`Ticker.on_tick` → `on_tick`).

- A missing entity is reported unless it is planned (the entry, its unit table or its module).
- A found entity always has its attributes compared; a found planned entry also warns `stale_planned`.
- An entity whose module has no source is left to `source_resolves`.

A pattern can let other attributes and comments stand between an attribute and its item with
`[(attribute_item) (line_comment) (block_comment)]*` between two `.` anchors (see the rtic pack). Write a second
query pattern that captures the key alone when the code form can carry no arguments (`#[idle]`), or
a bare entity is reported missing.

## Lint and findings

`plugin-lint` codes: `PACK-LINT-001` (does not parse or fails the schema), `PACK-LINT-002` (query missing, no
adapter for its language, or does not compile), `PACK-LINT-003` (query lacks a capture the check names),
`PACK-LINT-004` (unknown primitive), `PACK-LINT-005` (rule name without the unit prefix).

In a project: `PACK-000` (info, a pack loaded), `PACK-001` (error, an inlined pack does not load; none of its checks
ran), `PACK-002` (error, an inlined query does not compile or lacks a capture).
