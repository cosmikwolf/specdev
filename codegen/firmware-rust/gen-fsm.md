# Firmware FSM Concepts → Rust Implementation

This file defines how firmware finite state machine spec concepts map to Rust language constructs. Every rule couples an FSM domain concept with a specific Rust pattern.

## Extraction YAML Schema (state_machines[])

FSMs appear in the per-module extracted YAML under `state_machines[]`:

| Field | Description |
|---|---|
| `state_machines[].name` | FSM name (e.g. `ConnectionFSM`) |
| `state_machines[].initial_state` | Name of the initial state (from `entry; then <state>;`) |
| `state_machines[].states[]` | List of state name strings |
| `state_machines[].transitions[].name` | Transition identifier |
| `state_machines[].transitions[].from_state` | Source state name |
| `state_machines[].transitions[].to_state` | Target state name |
| `state_machines[].transitions[].event` | Event name (matches an `attribute def` in the spec) |
| `state_machines[].transitions[].guard` | Optional guard expression string, or null |
| `state_machines[].transitions[].action` | Optional action name string, or null |

There is no `error_state` field in the extraction YAML. An error state is identified by having no outgoing transitions (FSM025 terminal state warning). Detect it by finding states with no `from_state` entries in the transitions list.

---

## States → Exhaustive Enum with Derive Traits

Each state in the FSM spec's `states` list maps to a variant in a Rust enum:
```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum <FsmName>State {
    Idle,
    Connected,
    Streaming,
    Error,
}
```
- Always derive `Debug, Clone, Copy, PartialEq, Eq` — states are value types compared by identity
- Variant names are PascalCase conversions of the spec state names (e.g., `waiting_for_data` → `WaitingForData`)
- The `Error` state (from `error_state` in spec) is a normal variant — it is not a Rust `Result`
- Include a `Count` constant if needed: `const STATE_COUNT: usize = 4;` — but prefer exhaustive match over array indexing

## Events → Exhaustive Enum with Derive Traits

Each event in the FSM spec's `events` list maps to a variant:
```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum <FsmName>Event {
    Connect,
    Disconnect,
    DataReceived,
    Timeout,
}
```
- Same derive traits as states — events are value types
- Events may carry data as tuple or struct variants when the spec's transition uses the event payload: `DataReceived(u16)` — but only if the spec explicitly defines event data
- Variant names are PascalCase conversions of spec event names

## FSM Error Enum → Dedicated Error Type

The FSM needs its own error type for dispatch failures:
```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum <FsmName>Error {
    EnteredErrorState,
    InvalidTransition,
}
```
- `EnteredErrorState` — returned when a transition leads to the `error_state`
- `InvalidTransition` — returned when the wildcard `_` match arm fires (no defined transition for the state+event pair)
- Additional variants only if the spec defines specific FSM error conditions

## FSM Struct → State Field with Constructor

The FSM is a struct holding the current state:
```
pub struct <FsmName> {
    state: <FsmName>State,
}

impl <FsmName> {
    pub fn new() -> Self {
        Self { state: <FsmName>State::<InitialState> }
    }

    pub fn current_state(&self) -> <FsmName>State {
        self.state
    }
}
```
- `initial_state` from the spec sets the value in `new()`
- `current_state()` returns a copy (states are `Copy`) — never expose `&mut` to the state field
- The FSM struct is a field inside the owner module's struct, not a standalone static

## Dispatch → Match on (State, Event) Tuple

The core FSM dispatch uses Rust's pattern matching on the `(state, event)` tuple:
```
pub fn dispatch(&mut self, event: <FsmName>Event) -> Result<(), <FsmName>Error> {
    let next = match (self.state, event) {
        (State::Idle, Event::Connect) => {
            self.on_connect();
            State::Connected
        }
        (State::Connected, Event::Disconnect) => {
            self.on_disconnect();
            State::Idle
        }
        // ... one arm per transition in the spec
        _ => return Err(<FsmName>Error::InvalidTransition),
    };

    self.state = next;

    if self.state == State::Error {
        return Err(<FsmName>Error::EnteredErrorState);
    }

    Ok(())
}
```
- Each `transitions` entry in the spec becomes one match arm
- The action is called before the state assignment — if the action fails, the state should not change
- The wildcard `_` arm handles undefined transitions — return `InvalidTransition` error
- The error state check happens after every transition

## Error State → Post-Transition Check

When the FSM enters the `error_state`:
- `dispatch()` returns `Err(<FsmName>Error::EnteredErrorState)` after the transition
- Once in error state, subsequent dispatches may either:
  - Allow a recovery transition (if the spec defines one from `error_state`)
  - Reject all events: add a guard at the top of `dispatch()`:
    ```
    if self.state == State::Error {
        return Err(<FsmName>Error::InErrorState);
    }
    ```
  - The spec determines which behavior applies — check if any transition has `from: error_state`

## Guards → Conditional Match Arms

When a transition has a `guard` condition in the spec:
```
(State::Connected, Event::DataReceived) if self.is_buffer_ready() => {
    self.process_data();
    State::Streaming
}
```
- Guards are expressed as `if` clauses on match arms
- Guard functions are methods on the FSM struct or the owner module
- When a guard fails, the match falls through to the next matching arm or the wildcard
- Multiple transitions from the same (state, event) pair with different guards are ordered: guarded arms first, unguarded fallback last

## Actions → Method Calls on Self or Context

Each `action` in a transition maps to a method call:
- **Self-contained actions** → methods on the FSM struct: `fn on_connect(&mut self) { ... }`
- **Context-dependent actions** → if the FSM needs access to the owner module's resources, pass context:
  ```
  pub fn dispatch(&mut self, event: Event, ctx: &mut OwnerContext) -> Result<(), FsmError> {
      match (self.state, event) {
          (State::Idle, Event::Connect) => {
              ctx.start_connection();
              State::Connected
          }
          ...
      }
  }
  ```
- Action stubs contain `// TODO: implement` placeholder
- Actions with `"none"` in the spec generate no method call — just the state transition

## Integration with Owner Module → FSM as a Struct Field

The FSM is embedded in its `owner_module`'s struct:
```
pub struct BluetoothSink {
    fsm: ConnectionFsm,
    // ... other fields
}

impl BluetoothSink {
    pub fn handle_event(&mut self, event: ConnectionEvent) -> Result<(), BluetoothError> {
        self.fsm.dispatch(event).map_err(|e| BluetoothError::FsmError(e))
    }
}
```
- The owner module wraps FSM dispatch in its own public interface
- FSM errors are converted to the owner module's error type via `From` or `map_err`
- The FSM is never exposed publicly — callers interact with the owner module's interface

## Transition Table Alternative → const Array Lookup

For FSMs with many states and events where a match block would be unwieldy, use a const lookup table:
```
const TRANSITIONS: [[Option<State>; EVENT_COUNT]; STATE_COUNT] = [
    // Idle:        [Connect → Connected, Disconnect → None, ...]
    [Some(State::Connected), None, ...],
    // Connected:   [Connect → None, Disconnect → Idle, ...]
    [None, Some(State::Idle), ...],
];
```
- `None` means no valid transition — return `InvalidTransition`
- Actions still dispatch via match since function pointers in const arrays require `unsafe`
- Prefer the match-based approach for FSMs with fewer than 20 transitions — it is more readable and the compiler verifies exhaustiveness

## Testing FSMs → State Transition Verification

Each test verifies a transition sequence:
```
#[test]
fn test_idle_to_connected() {
    let mut fsm = ConnectionFsm::new();
    assert_eq!(fsm.current_state(), State::Idle);
    assert!(fsm.dispatch(Event::Connect).is_ok());
    assert_eq!(fsm.current_state(), State::Connected);
}

#[test]
fn test_invalid_transition() {
    let mut fsm = ConnectionFsm::new();
    assert_eq!(fsm.dispatch(Event::Disconnect), Err(FsmError::InvalidTransition));
    assert_eq!(fsm.current_state(), State::Idle); // state unchanged
}
```
- One test per defined transition in the spec
- One test per undefined transition to verify rejection
- One test for the error state entry and behavior
- Tests verify both the resulting state and the return value
