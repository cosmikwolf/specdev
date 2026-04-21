# egui Architecture → `eframe::App` Implementation

This file defines how `specs/architecture.toml` maps to the top-level
`eframe::App` impl and `main.rs`. The architecture spec declares panel
mounts and an optional navigation FSM; code generation composes those
into the `update()` method egui calls every frame.

## Source Directory Placeholder

Every path of the form `src/...` in this file is shorthand for `<source_dir>/...`, resolved by `/spec-gen` per target: `<packages.<name>.source_path>/src/` when the module YAML carries a `package` key, else `<source.root>/` from `spec.toml`. Single-package projects (no `[[packages]]`) substitute as a no-op.

## Extraction YAML Schema

Code generation reads `<out>/architecture.yaml` produced by extract:

| YAML path | Description |
|---|---|
| `mounts[]` | `{view, slot, resizable?, default_size_px?}` |
| `navigation.initial` | Root view shown at startup |
| `navigation.states[]` | Root view names reachable via navigation |
| `navigation.transitions[]` | `{from, to, event}` |

Plus the flat `modules[]` summary (every declared view) and the
per-view extraction YAMLs (for metadata and actions).

---

## `AppState` Struct → One Field per Root View + Stores

```rust
pub struct AppState {
    // all stores, alphabetical by name
    pub <store_snake>: <StoreName>,
    // ...

    // all root views, alphabetical by name
    pub <root_view_snake>: <RootViewName>,
    // ...

    // navigation state (only when navigation is declared)
    pub current_screen: NavScreen,
}
```

- Stores appear as direct fields so `update` can borrow them mutably and
  pass `&mut` references to view `show` methods.
- Every declared `root_view` gets a field (not just ones in `mounts[]`),
  because navigation may target a root view that is not the
  default-mounted one.
- `current_screen` is the navigation FSM's current state — omitted when
  `navigation` is absent.

---

## `Default` Impl → Load Stores + Construct Views

```rust
impl AppState {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let <store_snake> = match cc.storage {
            Some(storage) => <StoreName>::load(storage),
            None => <StoreName>::default(),
        };
        // ... per store

        Self {
            <store_snake>,
            // ...
            <root_view_snake>: <RootViewName>::default(),
            // ...
            current_screen: NavScreen::<Initial>,
        }
    }
}
```

- Persisted stores call `<StoreName>::load(storage)`; non-persisted
  stores use `<StoreName>::default()`.
- Root views always start at `default()`; views are ephemeral view
  state, not durable app state.

---

## `update()` Body → Panel Slot Routing

Panel slots map one-to-one to egui panel constructors:

| Slot | Panel |
|---|---|
| `top` | `egui::TopBottomPanel::top("top_panel")` |
| `bottom` | `egui::TopBottomPanel::bottom("bottom_panel")` |
| `left` | `egui::SidePanel::left("left_panel")` |
| `right` | `egui::SidePanel::right("right_panel")` |
| `central` | `egui::CentralPanel::default()` |

Emit one panel call per `mounts[]` entry. The `central` slot is always
last (egui requires central panels to be added after side panels). When
`navigation` is declared, the central panel dispatches on
`current_screen` instead of showing a fixed view:

```rust
impl eframe::App for AppState {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            self.<top_view_snake>.show(ui, /* store refs */);
        });
        // ... other non-central slots

        egui::CentralPanel::default().show(ctx, |ui| {
            match self.current_screen {
                NavScreen::<ViewA> => self.<view_a_snake>.show(ui, /* store refs */),
                NavScreen::<ViewB> => self.<view_b_snake>.show(ui, /* store refs */),
                // ... one arm per navigation.states[]
            }
        });
    }

    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        self.<store_snake>.save(storage);
        // ... one per persisted store
    }
}
```

When there is no navigation and the central slot has a static mount,
the central panel directly calls that view's `show`.

Resizable side panels use `.resizable(true).default_width(<size>)` when
`resizable = true` / `default_size_px` is set.

---

## Navigation FSM → `NavScreen` Enum + Transition Dispatch

When `navigation` is declared, emit:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NavScreen {
    <View1>,
    <View2>,
    // ...
}

impl AppState {
    pub fn navigate(&mut self, event: &str) {
        self.current_screen = match (self.current_screen, event) {
            (NavScreen::<From>, "<event>") => NavScreen::<To>,
            // ... one arm per navigation.transitions[]
            _ => self.current_screen,  // unknown event → no-op
        };
    }
}
```

- `navigate` is called by root views when one of their actions triggers
  a navigation transition. The action name in `transitions[].event`
  matches the action name declared on the originating view.
- Generated views whose actions appear as navigation events include a
  call-site comment noting the nav target, and optionally take an
  `&mut AppState` (or a narrower navigation handle) to trigger
  `navigate`.

---

## Store Borrow Threading → Per-View `reads[]`/`writes[]`

For each view `show` call inside `update`, compute the set of store
parameters by unioning the view's `reads[]` and `writes[]`. Pass `&mut`
when the store appears in `writes[]` on that view or any of its
sub-views; pass `&` otherwise.

When two views mutating the same store appear in different panels, egui's
per-frame sequential rendering makes them safe — the borrow is released
between panel closures because each `.show(ctx, |ui| { ... })` call
returns before the next panel begins.

---

## `main.rs` → eframe Launch

```rust
fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([800.0, 600.0])
            .with_title("<app-name from spec.toml>"),
        ..Default::default()
    };
    eframe::run_native(
        "<app-name>",
        options,
        Box::new(|cc| Box::new(AppState::new(cc))),
    )
}
```

- App title comes from `spec.toml`'s project `name` or an explicit
  `app_name` field.
- `run_native` is the canonical eframe entry point; use
  `run_simple_native` only for single-function demos (not generated code).

---

## `Cargo.toml` → Dependencies

Generated `Cargo.toml` includes:

```toml
[package]
name = "<app-name>"
version = "0.1.0"
edition = "2021"

[dependencies]
eframe = { version = "0.28", features = ["persistence"] }
egui = "0.28"
serde = { version = "1", features = ["derive"] }  # only if any store is persisted
```

Pin minor versions; egui releases frequently with breaking API changes.

---

## File Layout

```
src/
├── main.rs              # eframe::run_native entry point
├── app.rs               # AppState + impl eframe::App for AppState
├── views/
│   ├── mod.rs
│   └── <view_snake>.rs  # one per view
└── stores/
    ├── mod.rs
    └── <store_snake>.rs # one per store
```

`main.rs` declares `mod app; mod views; mod stores;` and calls
`app::AppState::new` via the eframe launch closure.

---

## Anti-patterns to Avoid

- **Two central panel calls.** egui allows only one central panel per
  frame. If both navigation and a static `central` mount are declared,
  MOUNT061 and validate.md check #4 will reject it; codegen refuses.
- **Holding a store reference across panel closures.** The borrow
  checker prevents it, but spec-driven authors sometimes try to
  pre-compute. Don't — re-borrow inside each closure.
- **Skipping `save()`.** When any store is `persisted = true`, the
  generated `save()` impl must fire for each one. Missing one = silent
  data loss on shutdown.
