# Firmware UI Concepts → C Implementation

This file defines how firmware UI spec concepts map to C language constructs. Every rule couples a UI domain concept with a specific C pattern.

## Input Data Source

The UI data comes from `specdev-analyzer extract` output (YAML). The extracted `ui` object contains:
- `displays[]` — name, display_type, driver, width, height, color_depth, interface, module
- `input_devices[]` — name, input_type, active, has_button, detents, module
- `leds[]` — name, led_type, colors, module
- `gestures[]` — name, device, trigger, window_ms
- `timing_defaults` — debounce_ms, long_press_ms, double_tap_ms, combo_window_ms, sequence_timeout_ms
- `fonts[]` — name, family, size, source, file
- `icons[]` — name, width, height, source, format, file
- `screens[]` — name, display, refresh_mode, poll_interval_ms, elements[]
- `screens[].elements[]` — id, element_type, x, y, width, height, font, icon, align, binding_module, binding_field, visible_when
- `indicators[]` — name, led, module, field, states[] (each with name, color, pattern, period_ms)
- `navigation` — initial_screen, screens[], transitions[] (each with name, from_state, to_state, event)

Use this extracted data as input for code generation. Do not parse .toml files directly.
Spec-hash fingerprints should hash the source .toml files for incremental generation tracking.

---

## Display Interface → Struct of Function Pointers

The display interface spec maps to a C struct containing function pointers for each drawing primitive:
```
typedef enum {
    ALIGN_LEFT = 0,
    ALIGN_CENTER,
    ALIGN_RIGHT,
} ui_align_t;

typedef enum {
    DRAW_STYLE_OUTLINE = 0,
    DRAW_STYLE_FILLED,
} ui_draw_style_t;

typedef struct {
    void (*clear)(void *driver_ctx);
    void (*flush)(void *driver_ctx);
    void (*draw_text)(void *driver_ctx, int x, int y, const char *text, font_id_t font, ui_align_t align);
    void (*draw_icon)(void *driver_ctx, int x, int y, const ui_icon_t *icon);
    void (*draw_rect)(void *driver_ctx, int x, int y, int w, int h, ui_draw_style_t style);
    void (*draw_line)(void *driver_ctx, int x1, int y1, int x2, int y2);
    void (*draw_progress_bar)(void *driver_ctx, int x, int y, int w, int h, int value, int max);
    void *driver_ctx;  /* opaque pointer to concrete driver instance */
} display_if_t;
```
- Every function pointer takes `void *driver_ctx` as first argument — this is the concrete driver handle
- The concrete display driver fills in this struct at init time, mapping to its library calls (u8g2, SSD1306, LVGL)
- Generated UI code only calls through this interface — never includes display library headers directly
- The `driver_ctx` pattern avoids global display state

## Screen Rendering → Draw Function per Screen

Each screen spec generates a draw function:
```
void ui_screen_home_draw(const display_if_t *disp, const ui_context_t *ctx) {
    disp->clear(disp->driver_ctx);

    /* Element: title text (layout.elements[0]) */
    disp->draw_text(disp->driver_ctx, 0, 0, "Home", FONT_6X10, ALIGN_LEFT);

    /* Element: status text with data binding (layout.elements[1]) */
    const char *status_str;
    switch (ctx->bt_status) {
        case BT_STATUS_DISCONNECTED: status_str = "Disconnected"; break;
        case BT_STATUS_CONNECTING:   status_str = "Connecting..."; break;
        case BT_STATUS_CONNECTED:    status_str = "Connected"; break;
        case BT_STATUS_STREAMING:    status_str = "Playing"; break;
        default:                     status_str = "Unknown"; break;
    }
    disp->draw_text(disp->driver_ctx, 0, 16, status_str, FONT_6X10, ALIGN_LEFT);

    /* Element: volume bar (layout.elements[2]) */
    disp->draw_progress_bar(disp->driver_ctx, 0, 48, 128, 8, ctx->volume, 100);

    disp->flush(disp->driver_ctx);
}
```
- Each `layout.elements` entry in the screen spec becomes a drawing call in declaration order
- `format_map` entries map to `switch` statements converting bound values to display strings
- `visible_when` conditions map to `if` guards around element draw calls
- Scrolling text elements use `ctx->scroll_offset` that the navigation update function increments
- Function signature is always `void ui_screen_<name>_draw(const display_if_t *, const ui_context_t *)`

## Font Data → External Arrays or Library Font References

Fonts from the extracted `fonts[]` data:

**Builtin fonts (display library provides):**
```
typedef enum {
    FONT_6X10 = 0,
    FONT_8X13,
    FONT_COUNT,
} font_id_t;
```
- The display driver implementation maps `font_id_t` values to its library's font data
- For u8g2: `u8g2_font_6x10_tr`, `u8g2_font_8x13_mr`
- For custom: `extern const uint8_t font_6x10_data[];`

**Custom font data:**
```
/* Placeholder — replace with actual font data */
const uint8_t font_custom_data[] = { 0 }; /* TODO: provide font file */
const size_t font_custom_size = sizeof(font_custom_data);
```

## Icon Data → Static const Byte Arrays

Icons from the extracted `icons[]` data map to C const arrays:

**Inline icons (from spec `data` field):**
```
static const ui_icon_t icon_play = {
    .width = 8,
    .height = 8,
    .data = (const uint8_t[]){ 0x00, 0x04, 0x0C, 0x1C, 0x3C, 0x1C, 0x0C, 0x04 },
};
```

**File-source icons:**
```
/* Generated from assets/logo.xbm */
static const uint8_t icon_logo_data[] = { 0x00, 0xFF, /* ... */ };
static const ui_icon_t icon_logo = {
    .width = 32,
    .height = 32,
    .data = icon_logo_data,
};
```

**Icon struct:**
```
typedef struct {
    uint16_t width;
    uint16_t height;
    const uint8_t *data;
} ui_icon_t;
```

## Navigation State Machine → Screen Enum with Switch Dispatch

The navigation spec maps to a screen enum and transition logic:
```
typedef enum {
    SCREEN_HOME = 0,
    SCREEN_SETTINGS,
    SCREEN_VOLUME,
    SCREEN_COUNT,
} ui_screen_t;

static ui_screen_t current_screen = SCREEN_HOME;
static uint32_t last_input_time_ms = 0;

void ui_nav_handle_input(ui_gesture_t gesture, const display_if_t *disp, ui_context_t *ctx) {
    last_input_time_ms = ctx->now_ms;

    /* Global actions (checked before per-screen) */
    switch (gesture) {
        case GESTURE_BOTH_PRESS:
            /* global action from spec */
            break;
        default:
            break;
    }

    /* Per-screen actions */
    switch (current_screen) {
        case SCREEN_HOME:
            switch (gesture) {
                case GESTURE_PRESS:
                    current_screen = SCREEN_SETTINGS;
                    ui_screen_settings_draw(disp, ctx);
                    break;
                case GESTURE_ROTATE_CW:
                    if (ctx->volume < 100) ctx->volume++;
                    ui_screen_home_draw(disp, ctx);
                    break;
                default:
                    break;
            }
            break;
        case SCREEN_SETTINGS:
            /* ... */
            break;
        default:
            break;
    }
}
```
- `navigate` actions change `current_screen` and call the target screen's draw function
- `adjust` actions modify `ctx` fields with bounds checking (clamp to min/max)
- `invoke` actions call module functions through function pointers in the context or direct calls
- Nested `switch(current_screen) { switch(gesture) { ... } }` maps the spec's per-screen action tables

## Timeout Handling → Elapsed Time Check

From the navigation spec's `global_timeout_s` and per-screen `timeout_s`:
```
void ui_nav_check_timeout(const display_if_t *disp, ui_context_t *ctx) {
    uint32_t elapsed = ctx->now_ms - last_input_time_ms;

    /* Per-screen timeout */
    if (current_screen == SCREEN_VOLUME && elapsed > (VOLUME_TIMEOUT_S * 1000)) {
        current_screen = SCREEN_HOME;
        ui_screen_home_draw(disp, ctx);
        return;
    }

    /* Global timeout */
    if (elapsed > (GLOBAL_TIMEOUT_S * 1000)) {
        current_screen = SCREEN_HOME;
        ui_screen_home_draw(disp, ctx);
    }
}
```
- Per-screen timeouts override the global timeout for that specific screen
- Timeout resets on every input event (`last_input_time_ms = ctx->now_ms`)
- Timeout target screen defaults to `initial_screen` unless spec says otherwise

## Input Dispatch → Gesture Enum and Detection Functions

Input devices from the extracted `input_devices[]` and `gestures[]` data produce gestures:
```
typedef enum {
    GESTURE_NONE = 0,
    GESTURE_PRESS,
    GESTURE_LONG_PRESS,
    GESTURE_DOUBLE_PRESS,
    GESTURE_ROTATE_CW,
    GESTURE_ROTATE_CCW,
    /* Combinations */
    GESTURE_BOTH_PRESS,
    /* Sequences */
    GESTURE_TRIPLE_CLICK,
    GESTURE_COUNT,
} ui_gesture_t;

ui_gesture_t ui_input_poll(void);
```
- The poll function reads from the input HAL module — it does NOT implement debounce (HAL does that)
- Combination detection: check multiple device states within `combo_window_ms`
- Sequence detection: internal state machine tracking ordered inputs within `sequence_timeout_ms`
- Return `GESTURE_NONE` when no gesture is detected

## LED Indicator Binding → Status-to-Pattern Mapping Function

Indicator specs map to an update function:
```
void ui_indicators_update(const ui_context_t *ctx) {
    /* status_led indicator */
    switch (ctx->bt_status) {
        case BT_STATUS_DISCONNECTED:
            led_set_pattern(LED_PATTERN_BREATHE, 0x0000FF, 2000);  /* blue breathe */
            break;
        case BT_STATUS_CONNECTING:
            led_set_pattern(LED_PATTERN_BLINK, 0x0000FF, 200);     /* blue fast blink */
            break;
        case BT_STATUS_CONNECTED:
            led_set_pattern(LED_PATTERN_SOLID, 0x0000FF, 0);       /* blue solid */
            break;
        case BT_STATUS_STREAMING:
            led_set_pattern(LED_PATTERN_SOLID, 0x00FF00, 0);       /* green solid */
            break;
    }
}
```
- Each `mapping` entry in the indicator spec becomes a `case` in the switch
- `bound_to` field determines which `ctx` field to switch on
- LED pattern types map to an enum: `LED_PATTERN_SOLID`, `LED_PATTERN_BLINK`, `LED_PATTERN_BREATHE`, `LED_PATTERN_PULSE`
- Color values are 24-bit RGB packed in uint32_t: `0xRRGGBB`
- The function calls the LED module's public interface — never drives GPIO directly

## Spec-Hash Fingerprint → File Header Comment

Every generated UI file begins with:
```
/**
 * @generated from ui/screens/home.toml + ui/assets.toml
 * @spec-hash a1b2c3d4e5f6...
 *
 * DO NOT EDIT — regenerate with /fw-gen-ui
 */
```
- The hash is SHA-256 of the concatenated content of all source .toml files this file depends on
- `/fw-gen-ui` reads the `@spec-hash` line to determine if regeneration is needed
- Files without a `@spec-hash` line are flagged as potentially hand-written

## Data Context Struct → Aggregated Bindings from All Screens

All data bindings aggregate into a single context struct:
```
typedef struct {
    /* From home screen bindings */
    bt_status_t bt_status;
    char track_title[MAX_TITLE_LEN];
    uint8_t volume;

    /* From settings screen bindings */
    uint8_t menu_index;

    /* Timing */
    uint32_t now_ms;

    /* Scroll state */
    int16_t title_scroll_offset;
} ui_context_t;
```
- Field types come from the bound module's interface types
- String fields use fixed-size char arrays: `char field[MAX_LEN]` — never `char *` with malloc
- The application main loop populates this struct before calling `ui_nav_update()`
- All screens share one context instance — no per-screen dynamic allocation

## UI Module Root → Init and Update Functions

The top-level UI module provides:
```
/* In ui.h */
int ui_init(const display_if_t *disp, const ui_context_t *ctx);
void ui_update(const display_if_t *disp, ui_context_t *ctx);

/* In ui.c */
int ui_init(const display_if_t *disp, const ui_context_t *ctx) {
    current_screen = SCREEN_HOME;
    ui_screen_home_draw(disp, ctx);
    return 0;
}

void ui_update(const display_if_t *disp, ui_context_t *ctx) {
    ui_gesture_t gesture = ui_input_poll();
    if (gesture != GESTURE_NONE) {
        ui_nav_handle_input(gesture, disp, ctx);
    }
    ui_nav_check_timeout(disp, ctx);
    ui_indicators_update(ctx);
}
```
- `ui_init()` sets the initial screen and draws it
- `ui_update()` is called from the application main loop — it polls input, handles navigation, checks timeouts, and updates indicators
- The application loop owns the `display_if_t` and `ui_context_t` instances
