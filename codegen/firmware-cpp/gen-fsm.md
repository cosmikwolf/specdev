# Firmware FSM Concepts → C++ Implementation

This file defines how firmware finite state machine spec concepts map to C++ language constructs. Every rule couples an FSM domain concept with a specific C++ pattern.

## Extraction YAML Schema

See `codegen/firmware-rust/gen-fsm.md` for the full FSM extraction schema — field names are identical across all languages.

---

## States → enum class with Count Sentinel

Each state in the FSM spec's `states` list maps to a C++ scoped enum variant:
```
enum class BtFsmState {
    Idle = 0,
    Connecting,
    Connected,
    Streaming,
    Error,
    Count,  // sentinel for array sizing
};

constexpr int kStateCount = static_cast<int>(BtFsmState::Count);
```
- `enum class` provides type safety — cannot accidentally compare states with integers
- PascalCase variant names converted from spec state names (e.g., `waiting_for_data` → `WaitingForData`)
- `Count` sentinel enables compile-time array sizing for transition/action tables
- Start at `0` for array indexing via `static_cast<int>(state)`

## Events → enum class with Count Sentinel

Each event in the FSM spec's `events` list maps to a scoped enum:
```
enum class BtFsmEvent {
    Connect = 0,
    Disconnect,
    DataReceived,
    Timeout,
    Count,
};

constexpr int kEventCount = static_cast<int>(BtFsmEvent::Count);
```
- Same pattern as states — `enum class` with `Count` sentinel
- Events may carry associated data via a separate union or variant type when the spec requires it:
  ```
  struct BtFsmEventData {
      BtFsmEvent type;
      union {
          uint16_t data_length;  // for DataReceived
          uint32_t timeout_ms;   // for Timeout
      };
  };
  ```

## FSM Class → Encapsulated State with Dispatch Method

The FSM is a class encapsulating the state and transition logic:
```
class BtFsm {
public:
    using ActionFn = void (*)(void* context);

    explicit BtFsm() : state_(BtFsmState::Idle) {}

    /**
     * @brief Process an event in the state machine
     * @param event The event to process
     * @param context Owner module context for action callbacks
     * @return 0 on success, negative on error
     */
    int dispatch(BtFsmEvent event, void* context = nullptr);

    BtFsmState current_state() const { return state_; }

private:
    BtFsmState state_;

    static const BtFsmState transition_table_[kStateCount][kEventCount];
    static const ActionFn action_table_[kStateCount][kEventCount];
};
```
- Constructor sets `initial_state` from the spec
- `current_state()` returns by value (enums are cheap to copy)
- Tables are `static const` class members — shared across instances, stored in ROM
- `ActionFn` is a plain function pointer — avoids `std::function` overhead on embedded targets
- `void* context` allows actions to access the owner module without template coupling

## Transition Table → constexpr 2D std::array

The spec's `transitions` list maps to a static const 2D array:
```
const BtFsmState BtFsm::transition_table_[kStateCount][kEventCount] = {
    // Idle
    { BtFsmState::Connecting, BtFsmState::Idle,  BtFsmState::Error, BtFsmState::Idle },
    // Connecting
    { BtFsmState::Connecting, BtFsmState::Idle,  BtFsmState::Connected, BtFsmState::Error },
    // Connected
    { BtFsmState::Connected,  BtFsmState::Idle,  BtFsmState::Streaming, BtFsmState::Error },
    // Streaming
    { BtFsmState::Streaming,  BtFsmState::Idle,  BtFsmState::Streaming, BtFsmState::Error },
    // Error
    { BtFsmState::Error,      BtFsmState::Error, BtFsmState::Error, BtFsmState::Error },
};
```
- Alternatively, use C++17 `constexpr std::array` for bounds checking:
  ```
  static constexpr std::array<std::array<BtFsmState, kEventCount>, kStateCount> transition_table_ = {{
      {{ BtFsmState::Connecting, BtFsmState::Idle, ... }},
      ...
  }};
  ```
- Undefined transitions map to `error_state` — never leave cells uninitialized
- Use comments to label which state each row represents

## Action Table → Function Pointer 2D Array

Each transition's `action` maps to a parallel function pointer table:
```
// Forward declare action stubs
static void on_start_connect(void* context);
static void on_disconnect(void* context);

const BtFsm::ActionFn BtFsm::action_table_[kStateCount][kEventCount] = {
    // Idle:       Connect           Disconnect  DataReceived  Timeout
    { on_start_connect, nullptr,     nullptr,    nullptr },
    // Connecting: ...
    { nullptr,          on_disconnect, nullptr,  nullptr },
    // ...
};
```
- `nullptr` for transitions with `action: "none"` or undefined transitions
- Action functions take `void* context` — cast to the owner module type inside
- Actions are declared `static` in the `.cpp` file — they are implementation details

## Dispatch Implementation → Table Lookup with Bounds and Error Checks

```
int BtFsm::dispatch(BtFsmEvent event, void* context) {
    int si = static_cast<int>(state_);
    int ei = static_cast<int>(event);

    // Bounds check
    if (si < 0 || si >= kStateCount || ei < 0 || ei >= kEventCount) {
        return -1;
    }

    // Error state rejection (unless spec defines recovery transitions)
    if (state_ == BtFsmState::Error) {
        return -2;
    }

    BtFsmState next = transition_table_[si][ei];
    ActionFn action = action_table_[si][ei];

    // Call action before state transition
    if (action != nullptr) {
        action(context);
    }

    state_ = next;

    // Check if we entered error state
    if (state_ == BtFsmState::Error) {
        return -3;
    }

    return 0;
}
```
- Cast enum to int for array indexing — `static_cast<int>()` is explicit and safe
- Bounds check prevents undefined behavior from invalid enum values
- Action called before state update — consistent with C pattern
- Different negative return codes distinguish failure modes

## Error State → Configurable Rejection or Recovery

When the FSM is in `error_state`:
- Default: reject all events with return code `-2`
- If spec defines recovery transitions from error state, allow those:
  ```
  if (state_ == BtFsmState::Error) {
      // Only allow Reset event for recovery
      if (event != BtFsmEvent::Reset) {
          return -2;
      }
  }
  ```
- The owner module decides whether to expose error recovery in its public API

## Guards → Pre-Transition Predicate Functions

When a transition has a `guard` condition:
```
using GuardFn = bool (*)(void* context);

static const GuardFn guard_table_[kStateCount][kEventCount] = {
    // ... mostly nullptr, with specific guards placed
};
```
- Guard functions return `bool`: `true` allows transition, `false` blocks it
- Check guard before applying transition:
  ```
  GuardFn guard = guard_table_[si][ei];
  if (guard != nullptr && !guard(context)) {
      return 0;  // guard blocked — not an error
  }
  ```
- Alternative approach: virtual guard methods on a derived FSM class for complex guard logic

## Action Stubs → Static Functions with Context Cast

Each unique `action` in the spec generates a stub:
```
/**
 * @brief Start Bluetooth connection attempt
 *
 * Transition: Idle --[Connect]--> Connecting
 */
static void on_start_connect(void* context) {
    auto* self = static_cast<BluetoothModule*>(context);
    (void)self;
    // TODO: implement — initiate Bluetooth connection
}
```
- `static_cast` from `void*` to the owner module type — type-safe cast
- Document which transition triggers the action
- `(void)self;` suppresses unused variable warnings in stubs

## Integration with Owner Module → FSM as Private Member

The FSM is a private member of the owner module class:
```
class BluetoothModule {
public:
    int handle_event(BtFsmEvent event) {
        return fsm_.dispatch(event, this);
    }

    BtFsmState connection_state() const {
        return fsm_.current_state();
    }

private:
    BtFsm fsm_;
    // ... other members
};
```
- Pass `this` as the `void* context` so action functions can access the owner
- The owner wraps dispatch in its public API — callers never interact with the FSM directly
- FSM errors map to the owner module's error type:
  ```
  std::expected<void, BluetoothError> handle_event(BtFsmEvent event) {
      int ret = fsm_.dispatch(event, this);
      if (ret < 0) return std::unexpected(BluetoothError::FsmError);
      return {};
  }
  ```

## File Placement → Header + Implementation in Owner Module Directory

Generated files:
- `src/include/<fsm_name>_fsm.hpp` — enum classes, FSM class declaration, action type
- `src/<owner_module>/<fsm_name>_fsm.cpp` — table definitions, dispatch implementation, action stubs

Header structure:
```
#pragma once

enum class BtFsmState { /* ... */ };
enum class BtFsmEvent { /* ... */ };

class BtFsm {
    // ... declaration
};
```

## Alternative: Type-Safe FSM with std::variant (C++17)

For FSMs where states carry different data:
```
struct Idle {};
struct Connecting { uint32_t start_time; };
struct Connected { uint8_t device_id; };
struct Error { int error_code; };

using FsmState = std::variant<Idle, Connecting, Connected, Error>;

class Fsm {
    FsmState state_{Idle{}};

public:
    void dispatch(BtFsmEvent event) {
        state_ = std::visit([&](auto& s) -> FsmState {
            return handle(s, event);
        }, state_);
    }

private:
    FsmState handle(Idle&, BtFsmEvent e);
    FsmState handle(Connecting&, BtFsmEvent e);
    // ... one overload per state
};
```
- Use only when states carry distinct data — otherwise the table-driven approach is simpler
- `std::visit` dispatches to the correct handler based on current state type
- Compile-time exhaustiveness checking — missing a state overload is a compiler error
- Higher overhead than table-driven (variant has size of largest alternative + discriminant) — not suitable for memory-constrained targets

## Testing → State Transition Verification

```
void test_idle_to_connecting() {
    BtFsm fsm;
    assert(fsm.current_state() == BtFsmState::Idle);
    int ret = fsm.dispatch(BtFsmEvent::Connect, nullptr);
    assert(ret == 0);
    assert(fsm.current_state() == BtFsmState::Connecting);
}

void test_error_state_rejects_events() {
    BtFsm fsm;
    // Force into error state
    fsm.dispatch(BtFsmEvent::DataReceived, nullptr);  // invalid from Idle
    assert(fsm.current_state() == BtFsmState::Error);
    int ret = fsm.dispatch(BtFsmEvent::Connect, nullptr);
    assert(ret == -2);  // rejected — already in error state
}
```
- One test per defined transition
- One test for undefined transitions
- One test for error state behavior
- Pass `nullptr` context in unit tests where actions are stubs
