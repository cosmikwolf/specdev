# egui domain — validation reasoning

Checks beyond what the JSON Schema and graph-check primitives can express.
The `/spec-validate` skill runs through these after
`specdev-analyzer validate` finishes, producing extra findings the analyzer
cannot catch without semantic understanding.

Each check lists:
- **Trigger** — when it runs
- **Finding** — what to report
- **Severity** — `error`, `warning`, or `info`

If a check cannot run (missing data, external file unavailable), note it
and move on; do not block other checks.

---

## 1. Binding references resolve (BIND010)

**Trigger:** every `reads[]` and `writes[]` entry on every view.

**Check:** each entry has the form `Owner.field`. Resolve `Owner` in the
project — it must be either a declared view name (from `modules[].name`)
or a declared store name (from `stores[].name`). Then `field` must exist:

- If `Owner` is a store → `field` must be in that store's `fields[].name`.
- If `Owner` is a view → `field` must be either (a) an action name on
  that view (valid for some read patterns) or (b) explicitly exported via
  a future `exports[]` field (not modeled in v1). For v1, view-to-view
  bindings are an advisory warning only.

**Finding:** "View `<ViewName>` reads `<Owner>.<field>` but `<Owner>` is
not a declared view or store" / "... `<Owner>.<field>` does not exist on
store `<Owner>`".

**Severity:** error for unknown owner / unknown store field; warning for
view-to-view bindings in v1.

**Note:** This is a semantic check rather than an analyzer graph-check
because cross-collection `reference_exists` against a joined
`StoreName.fieldname` target requires qualified-form synthesis the
analyzer doesn't yet express declaratively. Track with the recipe doc
(`specdev-next/09-new-domain-recipe.md` section D) and promote to a
real primitive once the pattern recurs.

---

## 2. Navigation state must be a root view (NAV_KIND)

**Trigger:** every entry in `architecture.navigation.states[]`.

**Check:** the state id must match a view with `kind = "root_view"`. The
analyzer enforces the id exists in `modules[].name` but cannot check the
kind without new primitive support.

**Finding:** "Navigation state `<name>` maps to view `<name>` which is
declared kind=`<kind>`, not `root_view`. Only root views can be
navigation targets."

**Severity:** error.

---

## 3. Mounted view must be a root view (MOUNT_KIND)

**Trigger:** every entry in `architecture.mounts[]`.

**Check:** `view` must be a view with `kind = "root_view"`. Same
rationale as NAV_KIND.

**Finding:** "Mounted view `<name>` is declared kind=`<kind>`, not
`root_view`. Panel slots accept only root views."

**Severity:** error.

---

## 4. Central slot contention

**Trigger:** `architecture.toml` is present.

**Check:** if `navigation` is declared and `mounts[]` also contains a
static entry with `slot = "central"`, fail. Navigation owns the central
slot.

**Finding:** "Navigation is declared, so the central slot is reserved
for the active navigation state. Remove the static mount of
`<view>` in `central`, or remove navigation."

**Severity:** error.

---

## 5. State ownership vs bindings consistency

**Trigger:** every view with `StateOwnership` metadata.

**Checks:**

- `location = "local"` AND `reads[]` or `writes[]` non-empty → warning.
  Local state should not produce cross-owner bindings.
- `location = "store"` AND `reads[]` and `writes[]` both empty → warning.
  Declared store ownership with no bindings is inconsistent.
- `location = "parent"` AND no `depends_on` entry resolves to a parent
  view that reads this view in its `sub_views` → warning. The parent
  relationship is asymmetric; the child should appear in the parent's
  `sub_views`.

**Severity:** warning.

---

## 6. Async action vs update trigger

**Trigger:** any action with `async = true`.

**Check:** the view's `UpdateTrigger.kind` should be `on_request`. A
`per_frame` view already self-repaints. An `on_event` view will not
re-render when the async future completes unless the completion path
explicitly requests a repaint.

**Finding:** "View `<view>` has async action `<action>` but
`UpdateTrigger.kind = <kind>`. Async completions will not trigger a
repaint; set kind to `on_request` and configure `repaint_after_ms`."

**Severity:** warning.

---

## 7. Persisted store serializability

**Trigger:** store with `persisted = true`.

**Check:** every `fields[].type` must be serde-serializable. Types the
LLM can safely assume are serializable: all domain aliases in `type_map`,
standard library containers (Vec, Option, String, BTreeMap, HashMap),
and tuples/arrays of the above. Flag any other type literal for
confirmation.

**Finding:** "Store `<name>` is persisted but field `<field>` has type
`<type>` which is not known to implement `serde::Serialize` /
`Deserialize`. Either derive them, wrap in a serde-compatible type, or
unset `persisted`."

**Severity:** warning.

---

## 8. Derived field writes

**Trigger:** any store field with `derived = true`.

**Check:** no entry in any view's `writes[]` list references this field.
Derived fields are computed; external writes bypass the derivation and
cause drift.

**Finding:** "View `<view>` writes `<StoreName>.<field>` but this field
is declared `derived = true`. Derived fields are read-only; write the
source fields this value is derived from instead."

**Severity:** error.

---

## 9. Widget requires binding or label

**Trigger:** every `widgets[]` entry.

**Check:** widgets must have renderable content. Specifically:

- `button` / `heading` / `hyperlink` without `label` and without
  `binding` → error (nothing to render).
- `slider` / `drag_value` / `text_edit_singleline` /
  `text_edit_multiline` / `checkbox` / `combo_box` / `color_picker`
  without `binding` → error (no state to manipulate).
- `separator` / `spinner` / `progress_bar` without any content is OK
  (they have no textual content).

**Severity:** error.

---

## 10. Requirement coverage

**Trigger:** project validate, every requirement id in
`specs/requirements/**`.

**Checks:**

- Requirement id appears in zero view's `satisfies[]` and zero store's
  `satisfies[]` → REQ001 warning.
- Requirement's `source[]` lists ids that do not resolve to any
  `[[source]]` entry across the requirements files → REQ003 error
  (analyzer-enforced; this line restates for readability).
- Requirement's `verified_by[]` lists free-form names that do not match
  any declared action — warning.

**Severity:** warning unless otherwise noted.

---

## 11. Drift detection (audit pipeline)

See `/spec-validate` for spec↔source drift. The audit pipeline compares
extracted YAML against the Rust tree-sitter adapter's view of the source
tree. Drift categories relevant to egui:

- **Widget drift** — generated view function contains `ui.*` calls the
  spec does not declare, or omits ones it does.
- **Action drift** — declared actions missing as methods on the view
  struct or as free functions in the view module.
- **Store field drift** — store struct has fields the spec does not
  declare, or omits declared fields.
- **Panel mount drift** — `architecture.toml` mounts do not match the
  `eframe::App::update` impl's panel routing.

Tree-sitter queries live in `languages/rust/audit.scm`; egui-specific
recognition patterns (e.g. detecting `impl eframe::App for`) may need
minor additions if the default rust adapter misses them.

If any drift is found: update the spec, update the code, or accept the
drift via a future `# drift: accepted` marker. Generated code is never
edited directly — the spec is authoritative.

---

## How to report findings

For each finding:

```
<rule id or "reasoning">  <severity>  <view_or_store>[:<action|widget|field>]
  <one-sentence summary>
  <one-sentence fix hint>
```

Group by severity, then by spec file. End with a one-line summary
(`N errors, M warnings, K info`). If any error is present, report the
project as FAILED.
