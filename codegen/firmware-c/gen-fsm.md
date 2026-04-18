# Firmware FSM Concepts → C Implementation

This file defines how firmware finite state machine spec concepts map to C language constructs. Every rule couples an FSM domain concept with a specific C pattern.

## Extraction YAML Schema

See `codegen/firmware-rust/gen-fsm.md` for the full FSM extraction schema — field names are identical across all languages.

---

## States → typedef enum with COUNT Sentinel

Each state in the FSM spec's `states` list maps to a C enum constant:
```
typedef enum {
    BT_FSM_STATE_IDLE = 0,
    BT_FSM_STATE_CONNECTING,
    BT_FSM_STATE_CONNECTED,
    BT_FSM_STATE_STREAMING,
    BT_FSM_STATE_ERROR,
    BT_FSM_STATE_COUNT       /* sentinel — must be last */
} bt_fsm_state_t;
```
- Use a `<FSM_PREFIX>_STATE_` naming convention with UPPER_SNAKE_CASE
- Start at `0` to enable array indexing
- `STATE_COUNT` sentinel enables compile-time table sizing: `[BT_FSM_STATE_COUNT][BT_FSM_EVENT_COUNT]`
- The `error_state` from the spec is a normal enum value — it is checked after transitions

## Events → typedef enum with COUNT Sentinel

Each event in the FSM spec's `events` list maps to a C enum constant:
```
typedef enum {
    BT_FSM_EVENT_CONNECT = 0,
    BT_FSM_EVENT_DISCONNECT,
    BT_FSM_EVENT_DATA_RECEIVED,
    BT_FSM_EVENT_TIMEOUT,
    BT_FSM_EVENT_COUNT       /* sentinel — must be last */
} bt_fsm_event_t;
```
- Same naming convention and zero-start as states
- `EVENT_COUNT` sentinel for table sizing

## Transition Table → 2D const Array Indexed by [state][event]

The spec's `transitions` list maps to a static const 2D array:
```
static const bt_fsm_state_t transition_table[BT_FSM_STATE_COUNT][BT_FSM_EVENT_COUNT] = {
    /* IDLE */
    [BT_FSM_STATE_IDLE] = {
        [BT_FSM_EVENT_CONNECT]       = BT_FSM_STATE_CONNECTING,
        [BT_FSM_EVENT_DISCONNECT]    = BT_FSM_STATE_IDLE,        /* no-op */
        [BT_FSM_EVENT_DATA_RECEIVED] = BT_FSM_STATE_ERROR,       /* invalid */
        [BT_FSM_EVENT_TIMEOUT]       = BT_FSM_STATE_IDLE,        /* no-op */
    },
    /* CONNECTING */
    [BT_FSM_STATE_CONNECTING] = {
        [BT_FSM_EVENT_CONNECT]       = BT_FSM_STATE_CONNECTING,  /* ignore */
        [BT_FSM_EVENT_DISCONNECT]    = BT_FSM_STATE_IDLE,
        [BT_FSM_EVENT_DATA_RECEIVED] = BT_FSM_STATE_CONNECTED,
        [BT_FSM_EVENT_TIMEOUT]       = BT_FSM_STATE_ERROR,
    },
    /* ... remaining states ... */
};
```
- Use C99 designated initializers `[STATE] = { [EVENT] = NEXT_STATE }` for readability and safety
- Undefined transitions (not in the spec) map to `error_state` — never leave a cell uninitialized
- The table is `static const` — placed in flash/ROM, not RAM, saving memory on constrained targets

## Action Table → 2D const Array of Function Pointers

Each transition's `action` maps to a function pointer in a parallel 2D array:
```
typedef void (*bt_fsm_action_fn_t)(void *context);

static const bt_fsm_action_fn_t action_table[BT_FSM_STATE_COUNT][BT_FSM_EVENT_COUNT] = {
    /* IDLE */
    [BT_FSM_STATE_IDLE] = {
        [BT_FSM_EVENT_CONNECT]       = on_start_connect,
        [BT_FSM_EVENT_DISCONNECT]    = NULL,                /* action: "none" */
        [BT_FSM_EVENT_DATA_RECEIVED] = NULL,
        [BT_FSM_EVENT_TIMEOUT]       = NULL,
    },
    /* ... */
};
```
- `NULL` for transitions with `action: "none"` or undefined transitions
- The `void *context` parameter allows actions to access the owner module's state without globals
- Action function signatures must all match `bt_fsm_action_fn_t` — use the context pointer to pass per-action data

## Dispatch Function → Table Lookup with Error Check

The dispatch function performs both table lookups and updates the state:
```
int bt_fsm_dispatch(bt_fsm_state_t *current_state, bt_fsm_event_t event, void *context) {
    if (*current_state >= BT_FSM_STATE_COUNT || event >= BT_FSM_EVENT_COUNT) {
        return -1;  /* invalid input */
    }

    if (*current_state == BT_FSM_STATE_ERROR) {
        return -2;  /* already in error state */
    }

    bt_fsm_state_t next = transition_table[*current_state][event];
    bt_fsm_action_fn_t action = action_table[*current_state][event];

    if (action != NULL) {
        action(context);
    }

    *current_state = next;

    if (next == BT_FSM_STATE_ERROR) {
        return -3;  /* entered error state */
    }

    return 0;  /* success */
}
```
- Bounds-check state and event against `_COUNT` sentinels before array access — prevents out-of-bounds
- Check for error state before processing — refuse events when stuck in error (unless spec defines recovery transitions)
- Call action before updating state — if the action fails, the caller can detect it
- Return negative error codes for different failure modes, `0` for success

## Error State → Dispatch Rejection or Recovery

When the FSM enters `error_state`:
- Default behavior: `dispatch()` returns negative error code and refuses further events
- If the spec defines transitions from `error_state` (recovery paths): allow those specific events through
- To support recovery, modify the error check:
  ```
  if (*current_state == BT_FSM_STATE_ERROR) {
      /* Only allow reset event when in error state */
      if (event != BT_FSM_EVENT_RESET) {
          return -2;
      }
  }
  ```

## Guards → Pre-Transition Condition Checks

When a transition has a `guard` condition:
```
/* Guard function type */
typedef bool (*bt_fsm_guard_fn_t)(void *context);

static const bt_fsm_guard_fn_t guard_table[BT_FSM_STATE_COUNT][BT_FSM_EVENT_COUNT] = {
    [BT_FSM_STATE_CONNECTED] = {
        [BT_FSM_EVENT_DATA_RECEIVED] = is_buffer_ready,  /* guard check */
    },
    /* ... all others NULL ... */
};
```
- Guard functions return `bool`: `true` allows the transition, `false` blocks it
- Check guard before applying transition in dispatch:
  ```
  bt_fsm_guard_fn_t guard = guard_table[*current_state][event];
  if (guard != NULL && !guard(context)) {
      return 0;  /* guard blocked transition — not an error, state unchanged */
  }
  ```
- Guard-blocked transitions are silent (return success) — the state simply does not change

## Action Stubs → Functions with TODO Comments

Each unique `action` value in the spec generates a stub function:
```
/**
 * @brief Action: start Bluetooth connection attempt
 * @param context Owner module context (cast to module_t*)
 *
 * Called on transition: IDLE --[CONNECT]--> CONNECTING
 */
static void on_start_connect(void *context) {
    module_t *self = (module_t *)context;
    (void)self;
    /* TODO: implement — initiate Bluetooth connection */
}
```
- Document which transition triggers the action
- Cast `void *context` to the owner module type
- Use `(void)self;` to suppress unused variable warnings in stubs
- Actions are `static` functions in the FSM implementation file — only the FSM dispatch calls them

## Integration with Owner Module → State as Struct Field

The FSM state variable lives in the owner module's struct:
```
typedef struct module_s {
    bt_fsm_state_t fsm_state;
    /* ... other fields ... */
} module_t;

int module_handle_event(module_t *self, bt_fsm_event_t event) {
    return bt_fsm_dispatch(&self->fsm_state, event, self);
}
```
- The owner module passes `&self->fsm_state` and `self` (as context) to dispatch
- The owner module wraps dispatch in its own public API — callers never call `bt_fsm_dispatch` directly
- Initialize FSM state in `module_init()`: `self->fsm_state = BT_FSM_STATE_IDLE;`

## File Placement → Header + Implementation in Owner Module Directory

Generated files:
- `src/include/<fsm_name>_fsm.h` — state/event enums, dispatch declaration, action type
- `src/<owner_module>/<fsm_name>_fsm.c` — transition table, action table, dispatch implementation, action stubs

The header includes:
```
#ifndef BT_FSM_H
#define BT_FSM_H

/* State and event enums */
/* Dispatch function declaration */
/* Action function type */

#endif /* BT_FSM_H */
```

## Testing FSMs → State Transition Verification Functions

Test stubs verify transition correctness:
```
void test_idle_to_connecting(void) {
    bt_fsm_state_t state = BT_FSM_STATE_IDLE;
    int ret = bt_fsm_dispatch(&state, BT_FSM_EVENT_CONNECT, NULL);
    assert(ret == 0);
    assert(state == BT_FSM_STATE_CONNECTING);
}

void test_invalid_transition_to_error(void) {
    bt_fsm_state_t state = BT_FSM_STATE_IDLE;
    int ret = bt_fsm_dispatch(&state, BT_FSM_EVENT_DATA_RECEIVED, NULL);
    assert(ret < 0);  /* entered error state */
    assert(state == BT_FSM_STATE_ERROR);
}
```
- One test per defined transition in the spec
- One test for undefined transitions to verify error state entry
- One test for dispatch rejection when already in error state
- Pass `NULL` context in unit tests where actions are stubs
