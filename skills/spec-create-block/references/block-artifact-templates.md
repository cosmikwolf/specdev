# Block artifact templates

Exact shapes for the files `/spec-create-block` scaffolds. Placeholders: `<name>`
= block directory name (lowercase-hyphenated, concrete); `<key>` = the schema
**table key** — the same string with hyphens turned to underscores (`nih-plug` →
`nih_plug`); `<lang>` = target language (default `rust`); `<Kind>`/`<plural>` =
an entity kind and its table name. Fill every `TODO:` from the interview
(Step 1) and the rules verified against the real API (Step 2) — never from
memory.

Live precedents (read the one whose shape matches):
- `blocks/embassy/` — two entity kinds, no relations.
- `blocks/egui/` — declares an intra-block relation (`region_in_panel`).
- `blocks/nih-plug/` — hyphenated name → `nih_plug` table key.
- `blocks/rtic/` (code check: tasks against `#[task(...)]`) and `blocks/mk20dx/` (graph check: `unique` over two
  fields) — a rule pack. Its shape is in `${CLAUDE_PLUGIN_ROOT}/shared/rule-packs.md`.

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
  # A free-form type expression is a code fact: prefer intent (what the
  # value means) over a type the code already holds.

  <Kind>:
    type: object
    additionalProperties: false
    required: [name]                    # plus any other mandatory fields
    properties:
      name: { $ref: "#/$defs/Identifier" }
      # TODO: kind-specific fields — enums with `default`, integers with
      # minimum/maximum. Intent only — no type or signature strings. One per
      # schema field you want surfaced as a reduction attr.
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

## 3. `guidance.md`

Rules for whoever writes or edits code in a module that uses the block — the
facts no check can verify. Inlined to `specs/specdev/guidance/blocks/<name>.md`
in every project that inlines the block, and listed by `context --module`.

```markdown
# <name> — implementation guidance

Rules an agent follows when it writes or edits code for a module with a
`[<key>]` table. No check verifies them; review does.

## <Concern>

- <One rule per bullet: what to do or never do, and why in one clause.>
```

Hard rules (`plugin-lint` checks the first three):
- **Present** (`GUIDANCE-001`), **no fenced code** (`GUIDANCE-002`), **at most
  4,096 bytes** (`GUIDANCE-003`). Rules, not code: a code sample is a template
  that rots against the crate it names.
- **No version numbers.** Name the behaviour; tell the reader to check the
  version in the project's manifest where an API differs between releases.
- **Leg 3 — no sibling-block names.** If the reader "needs" another block here,
  that coupling belongs in the `<name>+<other>` bridge's guidance.
- **PROSE-PROFILE-001 — no profile name.** Write "the project's profile".
- **A choice is not guidance.** A project-specific choice belongs in that
  project's decisions; guidance holds the rules every project using the block
  shares.

---

## 4. `README.md` (author-facing catalog — never inlined)

Sections: `# `<name>` block`, `## When to use`, `## What this block contributes`,
`## What this block does NOT contribute`, `## Sharp-test rationale` (all three
legs, explicitly), plus companion-block/bridge affinity and deferred items.
Naming a profile or a companion block **here** is fine — this file is never
inlined, so it *should* say "pairs with the X block via the `<name>+X` bridge".
