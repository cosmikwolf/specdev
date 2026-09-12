# Module TOML authoring rules

Priority-ordered rules for any skill or subagent that writes or edits a module
TOML (`/spec-design`, `/spec-edit`, the `module-designer` subagent both
dispatch, and `/spec-reverse` after the user has confirmed inferred fields).
When two rules conflict, the earlier rule wins.

1. **Schema is the contract.** Every field name must appear somewhere in the aggregator's reachable schemas. No invented keys. The schema uses `unevaluatedProperties: false`, which is `allOf`-aware — block-contributed properties pass through.
2. **Required fields by profile.** firmware-layered requires `name` + `layer`; library-api requires `name` + `visibility`. Whatever else the profile's `module.schema.yaml` lists as required must be present.
3. **Required metadata (firmware-layered).** `metadata.MemoryModel`, `metadata.ConcurrencyModel`, `metadata.ErrorHandling`, `metadata.Ownership` must all be present. Pick explicit values.
4. **Prefer inline tables for ≤3 params.** `params = [{name = "...", type = "u32", direction = "in"}]` is easier to read than an array-of-tables for short lists. Flip to `[[actions.params]]` when an action has >3 params.
5. **Naming conventions.**
   - Module names: PascalCase (`StatusLed`, `BtA2dpSink`).
   - Action names: `snake_case`.
   - Event names: `snake_case`, suffixed `_evt` (`set_solid_evt`, not `setSolid`).
   - State names: `snake_case`.
   - Error variants: `PascalCase` (they become Rust enum variants).
6. **Layer discipline (firmware-layered).** A module at layer `application` must not directly depend on a `PAC` module — walk through `middleware` / `driver` / `HAL`. The `layer_dependency` graph check enforces this.
7. **One module per file.** Each TOML holds exactly one module.
8. **Comment policy** — follow `${CLAUDE_PLUGIN_ROOT}/shared/comment-policy.md`. `/spec-design` and `/spec-edit` work from explicit user intent, not inference, so neither emits `# LOW:` markers — those are unique to `/spec-reverse` and are stripped once the user confirms each inferred field.
9. **One space either side of `=`. Never align the `=` column.** Write `name = "Db"`, not `name        = "Db"`. Two reasons, and the second is why this rule exists at all. (a) Alignment is unstable: adding one longer key re-indents every neighbour, so a one-field edit lands as an N-line diff. (b) Parallel drafters cannot pick it consistently — measured over ten modules from ten launches, **7 came back aligned, 2 plain, and 1 aligned to a different column**. The split originates in the *first* batch, where every dependency cone is empty (a cone is computed from already-declared `depends_on` edges, and the edge being declared is the one in flight), so no agent has a sibling to copy; every later agent then copies whichever sibling it happens to read, and one arbitrary coin-flip propagates through the whole project. A real project runs several modules at once precisely in that first batch. One space removes the choice. *(Profile `domain.toml` templates align their severity tables. That is a profile-authored file, not module TOML, and it is not a precedent here.)*

## The item surface (`items[]`) — per-method model

`module.items[]` carries the module's contractual public surface. The
per-profile codegen prose is the authority for emission detail
(`${CLAUDE_PLUGIN_ROOT}/profiles/<profile>/codegen/<language>/module.conventions.md`,
its "Item emission" section); these are the authoring rules that draft against
it. **Keep the two in sync — when the conventions doc changes the item model,
this section changes with it.**

The one-line rule:

> An item is a **public, contractual code symbol, identified by its
> fully-qualified path, described in prose — never by signature.** Nesting
> determines the *path*, not whether something is an item.

i. **A method is a function.** A public function reached through `RateLimiter::`
   is its own `kind = "fn"` item, named `<Type>::<member>` — the same way a free
   `render` function is an item. Public `const` / `type` associated members are
   `<Type>::<NAME>` items.
ii. **Inherent `impl` blocks dissolve into their methods.** Never author an item
    for the `impl Type { … }` block itself. Generics normalize out of the
    prefix: `RateLimiter::new`, never `RateLimiter<C>::new`.
iii. **A trait impl is one item** — `kind = "impl"`, named `<Trait> for <Type>`
     (e.g. `Display for RateLimiter`). Its method bodies are codegen-derived
     from the trait, so they are not separate items.
iv. **The type itself is an item** — `pub struct RateLimiter` → item
    `RateLimiter` (`kind = "struct"`). Private / helper members are **not**
    items; only `pub` surface is contract.
v. **Prose-first, never signatures.** Each item carries a `description` of what
   it does, not a mirror of how it is typed. Do not name parameters, return
   types, struct fields, or generics. `signature_hint` is a rare escape hatch
   (FFI, macro contracts) — reaching for it on ordinary functions turns the spec
   back into a copy of the code, which is the one failure this model exists to
   prevent.
vi. **`satisfies` shares coarsely.** Every item except `kind = "re_export"`
    needs `satisfies` (≥1 id; the schema enforces it), **but requirements are
    shared across a type's methods** — a limiter's whole surface can
    `satisfies = ["REQ-API-001"]`. Author a new `REQ-…` only for a genuinely
    distinct capability, not one-per-method. Over-granular per-method
    requirements are the tedium to avoid. Do **not** invent requirement ids: the
    ids must already exist in `specs/requirements.toml` (author it first if
    absent).

Worked example — a type and its public inherent methods (one struct item + one
item per public method, sharing coarse requirements; the trait impl is a single
item):

```toml
[[items]]
name = "RateLimiter"
kind = "struct"
description = "Token-bucket limiter keyed by a monotonic clock."
satisfies = ["REQ-API-001"]

[[items]]
name = "RateLimiter::new"
kind = "fn"
description = "Construct a limiter with the given capacity and refill rate."
satisfies = ["REQ-API-001"]

[[items]]
name = "RateLimiter::try_consume"
kind = "fn"
description = "Attempt to take n tokens; report whether the budget allowed it."
satisfies = ["REQ-API-002"]

[[items]]
name = "Display for RateLimiter"
kind = "impl"
description = "Human-readable one-line summary of remaining budget."
satisfies = ["REQ-API-009"]
```

No item names parameters, return types, or generics. A `pub` symbol left out of
`items[]` still compiles but shows as `in_source_only` in `/spec-sync-items` —
the correct signal that you have public surface the spec doesn't account for.

**Test items** are `kind = "fn"` with `test = true`: they use `verifies = [...]`
(pointing at `nature = "test"` requirements) instead of `satisfies`, and carry a
`test_kind` (`unit` / `integration` / `doc`). `satisfies` and `verifies` are
mutually exclusive on the same item. See the conventions doc's "Test items"
section for emit locations.

## Division of labor: `module-designer` subagent vs main thread

`/spec-design` and `/spec-edit` push the drafting read-path into the
`module-designer` subagent. The split:

- **The subagent owns the draft against the contract** — rules 1–7 above and the
  entire item-surface section. It reads the schema, profile prose, inlined-block
  READMEs, and siblings in an isolated context and returns a Candidate TOML
  (prose-first per-method items, coarse `satisfies`, schema-faithful fields). It
  is read-only on files; it does not validate or write.
- **The main thread owns the interview, the comment-policy carve-outs, the file
  write, and the validate gate.** Rule 8 (comment policy, `# LOW:` handling) is a
  main-thread/`/spec-reverse` concern — the subagent emits no comments beyond the
  `#:schema` directive. The main thread confirms the Candidate with the user,
  writes the file, and runs `specdev-analyzer validate`.

When `/spec-design` or `/spec-edit` runs the draft inline (no subagent — e.g. a
trivial one-field edit), the same rules apply; the division is about *where* the
work happens, not *which* rules hold.

## When editing existing TOML (/spec-edit specifics)

- Preserve existing indentation and key order where not semantically changed.
- Do not drop required metadata sub-objects (firmware-layered).
- Keep inline-table vs array-of-tables form consistent with the rest of the file.
- For an item-surface edit, keep the per-method shape: adding a public method to
  a type means adding one `<Type>::<member>` `kind = "fn"` item, not expanding an
  existing item into a signature. Removing a method removes its item.
- If the existing file carries disallowed comments (file-header blurbs, section dividers, prose that restates a nearby field), strip them as part of your edit. Carve-out: leave `# LOW:` markers from `/spec-reverse` alone until the user has confirmed or cleared each one.
