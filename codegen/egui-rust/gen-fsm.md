# egui View-Internal FSM Concepts → Rust Implementation

View-internal state machines (the `state_machines[]` list on a view
spec) model modes inside a single view: loading / loaded / error,
editing / viewing, wizard step transitions. They are distinct from the
top-level navigation FSM in `architecture.toml` — that one routes
between root views; this one is local to one view.

The Rust pattern is nearly identical to the firmware FSM pattern; this
file records the egui-specific adjustments.

## Extraction YAML Schema (state_machines[])

| Field | Description |
|---|---|
| `state_machines[].name` | PascalCase FSM name (e.g. `LoadFsm`) |
| `state_machines[].initial` | Initial state name |
| `state_machines[].states[]` | List of state name strings |
| `state_machines[].transitions[].from` | Source state |
| `state_machines[].transitions[].to` | Target state |
| `state_machines[].transitions[].event` | Event name (often an action on the view) |
| `state_machines[].transitions[].guard` | Optional guard expression |

---

## States / Events → Enums

Same as firmware gen-fsm:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum <FsmName>State { /* one variant per states[] */ }

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum <FsmName>Event { /* one variant per unique event across transitions */ }
```

State and event names are PascalCase conversions of the snake_case spec
names (`load_requested` → `LoadRequested`).

---

## FSM Struct → Field on the View

Unlike firmware where the FSM may be a standalone struct with its own
owner module, egui view FSMs are always embedded as a field on the
view struct:

```rust
pub struct <ViewName> {
    fsm: <FsmName>,
    // other view state
}

pub struct <FsmName> {
    state: <FsmName>State,
}

impl <FsmName> {
    pub fn new() -> Self {
        Self { state: <FsmName>State::<Initial> }
    }
    pub fn current(&self) -> <FsmName>State { self.state }
    pub fn dispatch(&mut self, event: <FsmName>Event) {
        self.state = match (self.state, event) {
            (<FsmName>State::<From>, <FsmName>Event::<Event>) => <FsmName>State::<To>,
            // ...
            _ => self.state,  // no-op on undefined transitions
        };
    }
}
```

Note: the FSM `dispatch` returns `()` rather than
`Result<(), FsmError>`. egui views tolerate invalid transitions as
no-ops — the render path gates behavior on `current()` instead of
propagating dispatch failures. If a specific FSM needs hard failure
semantics, fall back to the firmware `Result<(), <FsmName>Error>` shape.

---

## Widget Gating on FSM State

Views commonly render different widgets based on their FSM state:

```rust
pub fn show(&mut self, ui: &mut egui::Ui, store: &mut AppStore) {
    match self.fsm.current() {
        LoadFsmState::Idle => {
            if ui.button("Load").clicked() {
                self.fsm.dispatch(LoadFsmEvent::LoadRequested);
                self.start_load();
            }
        }
        LoadFsmState::Loading => {
            ui.spinner();
            ui.label("Loading...");
        }
        LoadFsmState::Loaded => {
            // render loaded content
        }
        LoadFsmState::Error => {
            ui.colored_label(egui::Color32::RED, "Load failed");
            if ui.button("Retry").clicked() {
                self.fsm.dispatch(LoadFsmEvent::RetryRequested);
            }
        }
    }
}
```

- Each match arm corresponds to one state; widgets inside the arm are
  the state's rendered content.
- `self.fsm.dispatch(...)` fires from button handlers or async
  completion paths — never from the middle of another state's render
  body.
- Async action completions that drive transitions use the stored
  `egui::Context` clone to call `ctx.request_repaint()` after
  dispatching, because a per-frame redraw would otherwise miss the
  state change.

---

## Guards → Match-Arm `if`

Same as firmware — guards become `if` clauses:

```rust
(LoadFsmState::Loading, LoadFsmEvent::DataReceived) if self.buffer_ready() => {
    LoadFsmState::Loaded
}
```

Guard methods are `self.<guard_name>(&self) -> bool` on the view struct
(not on the FSM), because guards typically inspect view or store state.

---

## Testing

egui view FSMs test like firmware FSMs, but without requiring an
`egui::Context`:

```rust
#[test]
fn loads_after_request_then_complete() {
    let mut fsm = LoadFsm::new();
    assert_eq!(fsm.current(), LoadFsmState::Idle);
    fsm.dispatch(LoadFsmEvent::LoadRequested);
    assert_eq!(fsm.current(), LoadFsmState::Loading);
    fsm.dispatch(LoadFsmEvent::DataReceived);
    assert_eq!(fsm.current(), LoadFsmState::Loaded);
}
```

Tests exercise every transition declared in the spec plus a sample of
no-op undefined transitions. Render-path gating (match on
`current()`) is covered by manual smoke of the app.
