# egui domain — authoring guidance

This file is the reference the `/spec-design` and `/spec-edit` skills load
when authoring or editing view and store specs for native Rust GUI apps
built on `egui` + `eframe`. It tells the LLM how to produce TOML that
validates cleanly against `schema/egui/v1/*.schema.yaml` on the first try.

The schema is the contract. This doc explains the reasoning behind it.

---

## File layout

An egui project has one of two shapes depending on whether it declares
`[[packages]]` in `spec.toml` (D24).

**Flat (single-crate, the typical egui app):**

```
my-app/
├── spec.toml                           # no [[packages]] section
└── specs/
    ├── architecture.toml               # panel mounts + navigation FSM
    ├── modules/
    │   └── <ViewName>.toml             # one view per file, PascalCase filename
    ├── stores/
    │   └── <StoreName>.toml            # one store per file, PascalCase filename
    └── requirements/
        └── <topic>.toml                # requirements grouped by topic
```

**Packaged (multi-crate workspace):**

```
my-app/
├── spec.toml                           # declares [[packages]] for each crate
├── crates/
│   ├── ui/          src/...
│   └── core/        src/...
└── specs/
    ├── architecture.toml
    ├── modules/
    │   ├── ui/
    │   │   └── CounterView.toml        # depends_on = ["core::CounterStore"]
    │   └── core/
    │       └── CounterStore.toml
    └── requirements/
        └── <topic>.toml                # requirements are NOT packaged
```

One view per file. One store per file. Filename matches the `name` field
(`CounterView.toml` for `name = "CounterView"`, `CounterStore.toml` for
`name = "CounterStore"`). The `package` is derived from the filesystem path —
never write a `package` field inside the TOML.

Cross-package references are qualified as `<pkg>::<Module>`; unqualified names
resolve within the same package first.

Unlike firmware, this domain has no `specs/ui/` directory — the view IS
the UI. Unlike firmware, there is no layer hierarchy — views compose
freely and cycles are caught by the analyzer's `acyclic` check only.

---

## View kinds

Every view declares a `kind`. The kind shapes where it can appear in
composition but does not constrain what it may depend on.

| Kind         | Purpose                                                  | Typical consumer                 |
|--------------|----------------------------------------------------------|----------------------------------|
| `root_view`  | Mountable into a panel slot; may be a navigation target. | `architecture.toml` mounts + nav |
| `panel_view` | Renders inside a parent view's layout region.            | Another view's `sub_views`       |
| `widget_view`| Small reusable composition (a toolbar, a status chip).   | Any view's `sub_views`           |
| `dialog_view`| Shown modally via `egui::Window`.                        | Triggered by a parent action     |

Infer `kind` from the view's role. If it is addressable by navigation or
mounted by the top-level app, it is `root_view`. If it renders inside a
parent's layout with a fixed place, it is `panel_view`. If it is reused
across views, it is `widget_view`. If it pops up and is dismissible, it
is `dialog_view`.

---

## View fields

Required:
- `name` — PascalCase identifier matching the filename.
- `kind` — one of `root_view | panel_view | widget_view | dialog_view`.

Optional (but usually present):
- `description` — one sentence. Read first by both humans and LLMs.
- `depends_on` — names of other views this one composes or references.
  Must reference declared views.
- `sub_views` — subset of `depends_on` that are rendered inside this view
  (vs. merely referenced, e.g. navigation targets). Codegen uses this to
  decide which deps become child-render calls.
- `reads` — `Store.field` or `View.field` bindings this view displays.
- `writes` — `Store.field` bindings this view mutates.
- `satisfies` — requirement ids (`REQ-<NAMESPACE>-NNN`) this view
  implements. Every id here must exist in `specs/requirements/`.
- `metadata` — closed object; see next section.
- `widgets` — egui widget calls rendered by this view.
- `actions` — handler functions invoked by widget events.
- `state_machines` — view-internal modes (loading/loaded/error etc.).
- `errors` — error variants the view's async actions may raise.

A simple view typically needs only `name`, `kind`, `description`,
`metadata`, `widgets`, and `actions`.

---

## Metadata — what to declare, what to omit

The `metadata` object is **closed**: only `StateOwnership`, `UpdateTrigger`,
`ErrorHandling` are accepted. Any other top-level key is rejected by the
schema.

`domain.toml` lists all three in `required_metadata.module`. Declare them
on every view even if the values match the defaults.

### Defaults (from `domain.toml`)

| Category       | Field              | Default         |
|----------------|--------------------|-----------------|
| StateOwnership | location           | `local`         |
| UpdateTrigger  | kind               | `on_event`      |
| ErrorHandling  | strategy           | `result_type`   |

### StateOwnership

- `local` — view owns its state as fields on its own struct. `reads[]`
  and `writes[]` should be empty or reference only this view's own state.
  Choose this when state is ephemeral and never needs to outlive the view.
- `store` — view reads from / writes to an app-level store. `reads[]`
  and `writes[]` must list at least one `Store.field` binding. Choose this
  when state is shared across views or must persist across restarts.
- `parent` — view receives state by reference from a parent view (prop
  drilling). `reads[]`/`writes[]` reference the parent view by
  `ParentView.field`. Choose this for small widget_views whose state is
  logically owned by a specific parent.

### UpdateTrigger

- `on_event` — repaint driven by user input (egui's default — the ctx
  automatically requests repaints when input changes). Most views are
  `on_event`.
- `per_frame` — repaint every frame. Choose for live plots, animations,
  real-time counters.
- `on_request` — view schedules its own repaints (async task completion,
  external event loop). Set `repaint_after_ms` to cap the max delay
  between self-repaints. Codegen emits `ctx.request_repaint_after()`.

### ErrorHandling

- `result_type` — actions that can fail return `Result<T, ViewError>`.
  The view defines a `ViewError` enum from its `errors[]` list.
- `option` — actions return `Option<T>`; used for lookups and may-miss
  paths.
- `panic` — actions cannot fail in the user's model; if they do, the app
  crashes. Acceptable only for invariants guaranteed by the type system.
- `enum_variant` — multi-variant status return used when `Result` would
  paper over meaningful classification.

---

## Widgets

Each entry under `widgets[]` maps to one `ui.*` call in the generated
update() body. The list ordering is significant — widgets render
top-to-bottom in a default vertical layout.

- `name` — snake_case identifier unique within this view.
- `kind` — from the closed `WidgetKind` enum (button, label, slider, etc.).
  Use `custom` only when no canonical widget fits; codegen emits a TODO
  with the name.
- `label` — static text. Required for `button`, `label`, `heading`,
  `checkbox`, `radio_button`, `selectable_label`. Omit when text is
  data-bound via `binding`.
- `binding` — `Owner.field` this widget displays or mutates. Must also
  appear in the view's `reads[]` or `writes[]` list. Required for
  stateful widgets (slider, text_edit, checkbox when wired to state,
  drag_value, combo_box).
- `on_event` — name of an action on this view invoked when the widget
  fires. Required for button; optional for widgets whose event is
  implicit in binding (checkbox, slider release).
- `enabled_when` — free-form boolean expression pasted into the codegen
  `ui.add_enabled(<expr>, ...)` call. Keep short; prefer derived fields
  on the store when the expression grows.

---

## Actions

One action per named handler. Actions are the view's public vocabulary;
widgets reference them by name via `on_event`.

- `name` in snake_case (`increment`, `submit`, `confirm_delete`).
- `description` — one line.
- `params[]` — each `{ name, type, direction }`. Inline form for ≤3
  params; expanded `[[actions.params]]` for more.
- `returns` — a type literal. Omit for actions that return `()`.
- `async` — `true` if the handler spawns async work. Codegen emits a
  `tokio::spawn` / channel pattern and adds a repaint request when the
  future completes; the view should declare `UpdateTrigger.kind = "on_request"`.

Constructor actions are conventionally named `new` or `init`.

---

## Stores

Stores hold app-level state that outlives any single view. The `eframe::App`
impl owns one instance of each store; views receive `&` or `&mut`
references.

- `name` — PascalCase matching the filename.
- `fields[]` — each `{ name, type, default?, description?, derived? }`.
  Field names are snake_case and unique within the store.
- `persisted` — if `true`, codegen emits `eframe::Storage` load/save. All
  fields must be serde-serializable.
- `derived: true` on a field means the value is computed from other
  fields. Codegen emits a read-only accessor; external `writes[]`
  bindings pointing at a derived field are an authoring error.

Prefer one store per coherent data subject — not one giant AppState.
Example: a `SessionStore` (auth, current user) separate from a
`DocumentStore` (loaded document, unsaved changes).

---

## Architecture file

`specs/architecture.toml` declares two things: panel mounts and
navigation.

- `mounts[]` — which root views mount in which eframe panel slots. The
  `central` slot is special: if `navigation` is declared, the currently
  active navigation state is mounted there (do not also declare a static
  mount for `central`). At most one view per non-central slot.
- `navigation` — FSM over root views (same shape as a view-internal FSM).
  `initial` is the view shown at startup. `states[]` lists all navigable
  root views. `transitions[].event` is the name of an action that, when
  invoked on the current view, triggers the transition.

If the app has only one screen, omit `navigation` entirely and mount the
single root view in `central`.

---

## Naming conventions

| Element            | Style                       | Example             |
|--------------------|-----------------------------|---------------------|
| View name          | PascalCase                  | `CounterView`       |
| Store name         | PascalCase                  | `CounterStore`      |
| Filename           | `<Name>.toml`               | `CounterView.toml`  |
| Action name        | snake_case                  | `increment`         |
| Widget name        | snake_case                  | `inc_button`        |
| Field name         | snake_case                  | `current_count`     |
| State name         | snake_case                  | `loading`           |
| Event name         | snake_case                  | `submit_clicked`    |
| Error variant      | PascalCase                  | `NetworkUnreachable`|
| Requirement id     | `REQ-<NAMESPACE>-NNN`       | `REQ-COUNTER-001`   |
| Metadata key       | CamelCase                   | `StateOwnership`    |
| Binding format     | `Owner.field`               | `CounterStore.count`|
| Type alias         | PascalCase (via `type_map`) | `Integer` → `i32`   |

---

## Common anti-patterns

1. **Mutating state inside render without declaring a write.** If
   `widgets[]` renders a slider bound to `CounterStore.count`, that
   binding must appear in `writes[]`. The analyzer will catch missing
   reads via BIND010-style validation in `validate.md`; authors should
   still list it explicitly because codegen uses `writes[]` to decide
   whether to pass `&mut store` vs `&store`.
2. **Retained widget thinking.** egui is immediate-mode: a widget does
   not "exist" between frames, it is a *call* each frame. Do not author
   widgets with lifecycle concepts (`on_mount`, `on_unmount`). The
   widget vocabulary is the view's render vocabulary.
3. **Fat views.** A view that manages five panes, a modal, and five
   async flows is not one view. Split into `RootView` + panel_views +
   a dialog_view and compose via `sub_views`.
4. **Unused reads.** A binding in `reads[]` that no widget consumes is
   dead weight. Remove it or wire a widget.
5. **Two mounts claiming one panel slot.** Enforced by MOUNT061. If two
   root views both want the `right` slot, one of them is actually a
   panel_view that should render inside the other.
6. **Modeling app-level state as `local`.** If the value outlives this
   view's lifetime (e.g. the logged-in user), it belongs in a store, not
   as a field on the view struct.
7. **Per-frame updates by default.** `on_event` is right for almost all
   views. Reach for `per_frame` only for genuinely animated UI — live
   plots, video, timers.

---

## When in doubt

- Read the existing views in the project before authoring a new one.
- Prefer the narrowest description that answers "what does this render?".
- When a field's default is fine, omit it.
- When a value outlives the current render, it belongs in a store; when
  it does not, it belongs as local view state.
- Ask the user rather than guess when the answer cannot be inferred from
  `spec.toml` + existing specs + the current description.
