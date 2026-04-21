# egui View Concepts → Rust Implementation

This file defines how egui view spec concepts map to Rust language
constructs in an `eframe` application. Every rule couples a view domain
concept with a specific Rust pattern.

## Source Directory Placeholder

Every `src/views/...` path in this file is shorthand for `<source_dir>/views/...`, resolved by `/spec-gen`: `<packages.<name>.source_path>/src/views/` when the module carries a `package` key, else `<source.root>/views/`.

## Extraction YAML Schema

Code generation reads the per-view YAML produced by `specdev-analyzer
extract`. Key fields:

| YAML path | Description |
|---|---|
| `name` | PascalCase view name (also the struct/function name) |
| `kind` | `root_view`, `panel_view`, `widget_view`, `dialog_view` |
| `description` | one-line view purpose (becomes doc comment) |
| `depends_on[]` | names of other views this view composes or references |
| `sub_views[]` | subset of `depends_on` that render inside this view |
| `reads[]` | `Owner.field` bindings this view displays |
| `writes[]` | `Owner.field` bindings this view mutates |
| `satisfies[]` | requirement ids this view satisfies |
| `metadata.StateOwnership.location` | `local`, `store`, or `parent` |
| `metadata.UpdateTrigger.kind` | `on_event`, `per_frame`, `on_request` |
| `metadata.UpdateTrigger.repaint_after_ms` | optional integer |
| `metadata.ErrorHandling.strategy` | `result_type`, `option`, `panic`, `enum_variant` |
| `widgets[]` | `{name, kind, label?, binding?, on_event?, enabled_when?}` |
| `actions[]` | `{name, params[], returns?, async?, description}` |
| `state_machines[]` | view-internal FSMs (see `gen-fsm.md`) |
| `errors[]` | `{variant, description}` — error variants this view may raise |

---

## View Kind → Rust Shape

| Kind | Generated shape |
|---|---|
| `root_view` | `pub struct <Name> { ... }` with `impl <Name>` and `pub fn show(&mut self, ctx: &egui::Context, store: &mut AppStore)` |
| `panel_view` | Same struct pattern; `show` takes the parent `ui: &mut egui::Ui` instead of `ctx` |
| `widget_view` | Same as panel_view; may also be generated as a free function when it holds no local state |
| `dialog_view` | `pub struct <Name> { open: bool, ... }` with `show` that wraps body in `egui::Window::new(...).open(&mut self.open).show(ctx, |ui| { ... })` |

---

## State Ownership → Constructor & Show Signature

`metadata.StateOwnership.location` controls how state is threaded:

- **`local`** — view struct holds its own fields; `show` does not take
  any store reference. `reads[]` and `writes[]` should be empty or
  reference only the view's own fields.
- **`store`** — `show` takes `store: &mut <StoreName>` (or multiple
  stores when `reads[]`/`writes[]` span several). Derive the store
  parameter list from the set of `Owner` values in `reads[]` + `writes[]`.
  If only `reads[]` references a store, take `&<StoreName>`; if any
  `writes[]` entry references it, take `&mut <StoreName>`.
- **`parent`** — `show` takes a reference to the parent's struct or a
  field subset: `show(&mut self, ui: &mut egui::Ui, parent: &mut <ParentView>)`.

Multiple store parameters are passed in alphabetical order by store name
to keep signatures stable.

---

## Widgets → `ui.*` Calls

Widget list order is the rendering order. Emit one `ui.*` call per
entry, in order, inside the `show` body (or inside a layout wrapper for
panel-specific placement).

| `kind` | Emitted call |
|---|---|
| `label` | `ui.label(<label-or-binding>);` |
| `heading` | `ui.heading(<label>);` |
| `button` | `if ui.button(<label>).clicked() { self.<on_event>(...); }` |
| `hyperlink` | `ui.hyperlink_to(<label>, <binding>);` |
| `separator` | `ui.separator();` |
| `checkbox` | `ui.checkbox(&mut <binding>, <label>);` |
| `radio_button` | `ui.radio_value(&mut <binding>, <value>, <label>);` |
| `selectable_label` | `ui.selectable_value(&mut <binding>, <value>, <label>);` |
| `slider` | `ui.add(egui::Slider::new(&mut <binding>, <range>).text(<label>));` |
| `drag_value` | `ui.add(egui::DragValue::new(&mut <binding>));` |
| `text_edit_singleline` | `ui.text_edit_singleline(&mut <binding>);` |
| `text_edit_multiline` | `ui.text_edit_multiline(&mut <binding>);` |
| `combo_box` | `egui::ComboBox::from_label(<label>).selected_text(...).show_ui(ui, |ui| { ... });` |
| `color_picker` | `ui.color_edit_button_srgba(&mut <binding>);` |
| `progress_bar` | `ui.add(egui::ProgressBar::new(<binding>));` |
| `spinner` | `ui.spinner();` |
| `image` | `ui.image(<binding>);` |
| `custom` | Emit `// TODO: custom widget <name>` and a function stub |

When `enabled_when` is present, wrap the call in
`ui.add_enabled(<expr>, ...)` — use `ui.add_enabled_ui(<expr>, |ui| { ... })`
for widgets that don't return a `Response` from a direct `ui.add(...)`.

Binding resolution: `Owner.field` becomes either `self.<field>` (when
Owner is this view and location=local), `store.<field>` /
`<owner_snake>.<field>` (when Owner is a store), or `parent.<field>`
(when Owner is the parent view). The codegen uses the view's `reads[]`
and `writes[]` to decide `&` vs `&mut` borrow of the store.

---

## Actions → impl Methods

Each `actions[]` entry becomes one method on the view struct:

```rust
impl <ViewName> {
    pub fn <action_name>(&mut self, /* params */) /* -> returns */ {
        // TODO: implement
    }
}
```

- `params[]` translate directly. `direction: in` → `&T` or `T`;
  `direction: out` → `&mut T`; `direction: inout` → `&mut T`.
- Omit `-> Type` when `returns` is absent (unit return).
- `async: true` actions get the `async` keyword and a TODO stub that
  schedules work on a background runtime (`tokio::spawn` / `std::thread`)
  and emits `ctx.request_repaint()` from the completion path.

When an action needs store access, add the store parameter to the method
signature to match whatever state the action must read or write —
`&mut <StoreName>` or `&<StoreName>`. The widget call site passes it
through: `self.<action>(store, ...)`.

Constructor action named `new` → `pub fn new(...) -> Self`. Constructor
action named `init` → `pub fn init(&mut self, ...)` on an already-default
view.

---

## Update Trigger → Repaint Strategy

`metadata.UpdateTrigger.kind`:

- **`on_event`** (default) — no explicit repaint calls. egui handles
  input-driven repaints automatically.
- **`per_frame`** — emit `ctx.request_repaint();` at the top of `show`.
  This requests the next frame regardless of input, producing the
  continuous repaint loop animations and live plots need.
- **`on_request`** — emit `ctx.request_repaint_after(Duration::from_millis(<repaint_after_ms>));`
  at the top of `show`. Default `repaint_after_ms` to `100` when absent.
  Async action completions should additionally call
  `ctx.request_repaint()` via a stored `egui::Context` clone.

---

## Reads/Writes → Store Binding Discipline

- Each `writes[]` binding `<Store>.<field>` implies the view receives
  `&mut <Store>` in `show` and passes `&mut store.<field>` to any widget
  that mutates it.
- Each `reads[]`-only binding (not in `writes[]`) implies at most a
  shared borrow. If the same store appears in `writes[]`, the single
  `&mut <Store>` parameter covers both.
- An empty widget `binding` on a stateful widget (slider, text_edit,
  etc.) is an authoring error caught by validate.md check #9; codegen
  aborts with a TODO comment rather than guessing.

---

## Errors → View Error Enum

When `errors[]` is non-empty and
`metadata.ErrorHandling.strategy = result_type`, emit:

```rust
#[derive(Debug, Clone)]
pub enum <ViewName>Error {
    <Variant1>,
    <Variant2>,
    // ...
}
```

Async actions that may fail return `Result<T, <ViewName>Error>`. When
`strategy = option`, use `Option<T>` instead. When `strategy = panic`,
omit the error enum and use `.expect()` internally. When
`strategy = enum_variant`, emit a status enum named
`<ViewName>Status` with one variant per `errors[].variant`.

---

## Sub-Views → Child Render Calls

For each entry in `sub_views[]`, the view holds the child view as a
field and invokes its `show` method during render:

```rust
pub struct <ViewName> {
    <child_snake>: <ChildView>,
    // ...
}

// inside show body:
self.<child_snake>.show(ui /* or ctx */, store);
```

Entries in `depends_on[]` but NOT in `sub_views[]` are referenced only
by name (e.g., as navigation targets). They become `use` statements at
the top of the file but no field or call.

---

## File Layout → `src/views/<snake>.rs`

Each view generates:

- `src/views/<view_snake>.rs` — struct, `impl`, `show`, action methods,
  optional error enum.
- Re-export in `src/views/mod.rs`: `pub mod <view_snake>; pub use <view_snake>::<ViewName>;`

The root `AppState` owns one instance of each `root_view`; panel_views
and widget_views are owned by their parent view struct.

---

## Dialog Views → `egui::Window` Wrapper

When `kind = dialog_view`:

```rust
pub struct <Name> {
    pub open: bool,
    // other fields
}

impl <Name> {
    pub fn show(&mut self, ctx: &egui::Context, store: &mut AppStore) {
        if !self.open { return; }
        let mut still_open = self.open;
        egui::Window::new(<name-as-title>)
            .open(&mut still_open)
            .show(ctx, |ui| {
                // widgets rendered here
            });
        self.open = still_open;
    }
}
```

The parent view flips `self.<dialog_snake>.open = true;` to open the
dialog and reads `self.<dialog_snake>.open` to see if it is dismissed.

---

## Testing Views → Compile Check + Smoke

egui views resist unit testing because they require a live `egui::Context`.
Generated tests are limited to:

- `#[test] fn <view_snake>_new_has_default_state() { let _ = <ViewName>::default(); }`
  when `#[derive(Default)]` can be applied.
- Action methods that do not require a `ui` or `ctx` get direct-call
  tests when feasible.

Integration-level behavior is covered by running the app with
`cargo run` — the spec's requirements drive manual verification.
