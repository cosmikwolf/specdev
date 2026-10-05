# Bridge artifact templates

Exact shapes for the files `/spec-create-bridge` scaffolds. Placeholders:
`<a>`/`<b>` = lex-ordered endpoint block names (`a < b`); `<table>` = the
binding-table name. Fill every `TODO:` from the interview (Step 1) and the rules
verified against the real API (Step 2) — never from memory.

The shipped canonical bridges are the live precedents; read the one whose shape
matches the pairing:
- `bridges/embassy+stm32h5-rs/` — intra-module, embedded.
- `bridges/egui+nih-plug/` — plugin shell, host (GUI/audio).
- `bridges/next-auth+prisma/` — named cross-module endpoint.

`bridges/egui+nih-plug/` also ships a rule pack (`checks.toml`, `related_attrs_allowed` across the binding
relation). Its shape is in `${CLAUDE_PLUGIN_ROOT}/shared/rule-packs.md`.

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
          References an `<a>.<...>[].name` declared on this SAME module
          (the intra-module shape; see the named cross-module shape below).
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
      # TODO: <attr>: ".<field>"   (binding fields → relation attributes)
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

## 4. `guidance.md`

The rules at the seam, for whoever writes the handoff code. Inlined to
`specs/specdev/guidance/bridges/<a>+<b>.md`, listed by `context --module`.

```markdown
# <a>+<b> — implementation guidance

Rules an agent follows when it writes or edits code that <does the handoff>.
The rule pack checks <the rules it covers, by `<a>+<b>.<rule>` id>; review
checks the rest. (With no rule pack: "No check verifies them; review does.")

## The join product

- <The crate that exists only because the two blocks are paired, and the rules
  for using it. Omit the section when there is none.>

## <Concern>

- <One rule per bullet.>

## Rules

- **<PAIR>-001:** <a cross-block constraint neither block can check alone; name
  its `<a>+<b>.<rule>` id when the rule pack checks it>.
```

Hard rules (`plugin-lint` checks the first three): present (`GUIDANCE-001`), no
fenced code (`GUIDANCE-002`), at most 4,096 bytes (`GUIDANCE-003`). No version
numbers. May name the two constituent blocks; never a profile
(`PROSE-PROFILE-001`).

---

## 5. `README.md` (author-facing catalog — never inlined)

Sections: `# `<a>+<b>` bridge`, `## When to use`, `## What this bridge
contributes`, `## What this bridge does NOT contribute`, `## Inline order`, plus
the co-residency rule and any deferred items. Naming a profile or a companion
block **here** is fine — this file is never inlined, so it should say "pairs
with the X profile for …". Include the earn-its-keep rationale (endpoints resolve;
the contribution names both sides; the join product belongs to the bridge, not
either block).
