# Firmware UI Concepts → Rust Implementation

This file defines how firmware UI spec concepts map to Rust language constructs. Every rule couples a UI domain concept with a specific Rust pattern.

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

## Display Interface → Trait with Drawing Primitives

The display interface spec maps to a Rust trait that screen rendering code calls:
```
pub trait DisplayDriver {
    type Error;
    fn clear(&mut self) -> Result<(), Self::Error>;
    fn flush(&mut self) -> Result<(), Self::Error>;
    fn draw_text(&mut self, x: i32, y: i32, text: &str, font: FontId, align: Align) -> Result<(), Self::Error>;
    fn draw_icon(&mut self, x: i32, y: i32, icon: &Icon) -> Result<(), Self::Error>;
    fn draw_rect(&mut self, x: i32, y: i32, w: u32, h: u32, style: DrawStyle) -> Result<(), Self::Error>;
    fn draw_line(&mut self, x1: i32, y1: i32, x2: i32, y2: i32) -> Result<(), Self::Error>;
    fn draw_progress_bar(&mut self, x: i32, y: i32, w: u32, h: u32, value: u32, max: u32) -> Result<(), Self::Error>;
}
```
- The trait uses an associated `Error` type so each display driver can define its own error
- All drawing methods return `Result` — never panic on draw failure
- The concrete implementation wraps the project's display library (embedded-graphics, ssd1306, u8g2 FFI)
- Generated UI code uses `impl DisplayDriver` or `&mut dyn DisplayDriver` — never concrete types

## Screen Rendering → Free Functions with Generic Display Parameter

Each screen spec generates a draw function:
```
pub fn draw_home<D: DisplayDriver>(display: &mut D, ctx: &UiContext) -> Result<(), D::Error> {
    display.clear()?;

    // Element: title text (from spec layout.elements[0])
    display.draw_text(0, 0, "Home", FontId::Font6x10, Align::Left)?;

    // Element: status text with data binding (from spec layout.elements[1])
    let status_text = ctx.bt_status.as_str();
    display.draw_text(0, 16, status_text, FontId::Font6x10, Align::Left)?;

    // Element: volume bar (from spec layout.elements[2])
    display.draw_progress_bar(0, 48, 128, 8, ctx.volume as u32, 100)?;

    display.flush()?;
    Ok(())
}
```
- Each `layout.elements` entry in the screen spec becomes a drawing call in order
- Data bindings resolve through the `UiContext` struct — never call module APIs directly from draw functions
- `visible_when` conditions map to `if ctx.field == value { ... }` guards around draw calls
- `format_map` entries map to `match` expressions converting values to display strings
- Scrolling text uses a mutable offset tracked in the navigation state, incremented per frame

## Font Embedding → embedded-graphics MonoFont or ID Enum

Fonts from the extracted `fonts[]` data map to Rust based on source type:

**Builtin fonts:**
```
use embedded_graphics::mono_font::{ascii::FONT_6X10, MonoTextStyle};
```
- Map spec font names to `embedded_graphics::mono_font::ascii::*` constants
- The display driver implementation translates `FontId` enum variants to concrete font references

**Custom font data:**
```
pub const FONT_CUSTOM: &[u8] = include_bytes!("../assets/custom_font.raw");
```
- File-source fonts embed raw bytes via `include_bytes!`
- If the font file does not exist, generate a placeholder: `pub const FONT_CUSTOM: &[u8] = &[]; // TODO: provide font file`

**Font ID enum:**
```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FontId {
    Font6x10,
    Font8x13,
    Custom,
}
```

## Icon Embedding → const Byte Arrays or include_bytes

Icons from the extracted `icons[]` data map to Rust const data:

**Inline icons (from spec `data` field):**
```
pub const ICON_PLAY: Icon = Icon {
    width: 8,
    height: 8,
    data: &[0x00, 0x04, 0x0C, 0x1C, 0x3C, 0x1C, 0x0C, 0x04],
};
```

**File-source icons:**
```
pub const ICON_LOGO: Icon = Icon {
    width: 32,
    height: 32,
    data: include_bytes!("../assets/logo.raw"),
};
```

**Icon struct:**
```
pub struct Icon {
    pub width: u32,
    pub height: u32,
    pub data: &'static [u8],
}
```

## Navigation State Machine → Screen Enum with Match Dispatch

The navigation spec maps to a screen enum and transition logic:
```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    Home,
    Settings,
    Volume,
}

pub struct Navigation {
    current: Screen,
    last_input_time: u64,
}

impl Navigation {
    pub fn new() -> Self {
        Self {
            current: Screen::Home, // initial_screen from spec
            last_input_time: 0,
        }
    }

    pub fn handle_input<D: DisplayDriver>(
        &mut self,
        gesture: Gesture,
        ctx: &mut UiContext,
        display: &mut D,
    ) -> Result<(), D::Error> {
        self.last_input_time = ctx.now_ms;

        // Global actions (checked before per-screen)
        match gesture {
            Gesture::BothPress => { /* global action from spec */ }
            _ => {}
        }

        // Per-screen actions
        match (self.current, gesture) {
            (Screen::Home, Gesture::Press) => {
                self.current = Screen::Settings;
                draw_settings(display, ctx)?;
            }
            (Screen::Settings, Gesture::RotateCw) => {
                ctx.menu_index = ctx.menu_index.saturating_add(1);
                draw_settings(display, ctx)?;
            }
            _ => {} // unhandled gesture for this screen
        }
        Ok(())
    }
}
```
- Each screen variant corresponds to a screen in the extracted `screens[]` data
- `initial_screen` from the extracted `navigation` data sets the default in `new()`
- Per-screen actions from the navigation spec map to `(Screen, Gesture)` match arms
- `navigate` actions change `self.current` and call the target screen's draw function
- `adjust` actions modify `ctx` fields: `ctx.volume = (ctx.volume as i16 + delta).clamp(0, 100) as u8;`
- `invoke` actions call through the context: `(ctx.invoke_fn)(ctx)` or via a callback

## Input Dispatch → Gesture Enum from HAL Module Output

Input devices from the extracted `input_devices[]` and `gestures[]` data produce gestures:
```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Gesture {
    Press,
    LongPress,
    DoublePress,
    RotateCw,
    RotateCcw,
    // Combinations
    BothPress,
    // Sequences
    TripleClick,
}
```
- Each gesture name from the spec becomes an enum variant
- The input dispatch module reads from the input HAL module's interface — it does NOT implement debounce or timing
- Combination detection: check multiple device states within `combo_window_ms`
- Sequence detection: track ordered inputs within `sequence_timeout_ms` using a small state machine
- The gesture enum is the contract between input dispatch and navigation

## LED Indicator Binding → Match on Module Status

Indicator specs map to an update function that reads module state and drives LEDs:
```
pub fn update_indicators(ctx: &UiContext, led: &mut impl LedDriver) {
    // status_led indicator from spec
    match ctx.bt_status {
        BtStatus::Disconnected => led.set_pattern(LedPattern::Breathe {
            color: Color::BLUE,
            period_ms: 2000,
        }),
        BtStatus::Connecting => led.set_pattern(LedPattern::Blink {
            color: Color::BLUE,
            on_ms: 100,
            off_ms: 100,
        }),
        BtStatus::Connected => led.set_pattern(LedPattern::Solid {
            color: Color::BLUE,
        }),
        BtStatus::Streaming => led.set_pattern(LedPattern::Solid {
            color: Color::GREEN,
        }),
    }
}
```
- Each `mapping` entry in the indicator spec becomes a match arm
- `bound_to` field determines which `ctx` field to match on
- Pattern types (solid, blink, breathe, pulse) map to `LedPattern` enum variants with timing parameters
- The `LedDriver` trait abstracts the LED hardware module — generated code never calls hardware directly

## Spec-Hash Fingerprint → File Header Comment

Every generated UI file begins with:
```
// @generated from ui/screens/home.toml + ui/assets.toml + interfaces/display.toml
// @spec-hash: a1b2c3d4e5f6...
// DO NOT EDIT — regenerate with /fw-gen-ui
```
- The hash is SHA-256 of the concatenated content of all source .toml files this file depends on
- `/fw-gen-ui` reads the `@spec-hash` line to determine if regeneration is needed
- Files without a `@spec-hash` line are flagged as potentially hand-written

## Data Context Struct → Aggregated Bindings from All Screens

All data bindings across all screens aggregate into a single context struct:
```
pub struct UiContext {
    // From home screen bindings
    pub bt_status: BtStatus,
    pub track_title: heapless::String<64>,
    pub volume: u8,

    // From settings screen bindings
    pub menu_index: u8,

    // Timing
    pub now_ms: u64,

    // Scroll state (for scrolling text elements)
    pub title_scroll_offset: i32,
}
```
- Field types come from the bound module's interface return types
- String fields use `heapless::String<N>` when `memory.allocation` is `static`
- The context is populated by the application main loop before calling `navigation.update()`
- Scroll offsets for `scroll: true` text elements are tracked here

## Refresh Strategy → Event-Driven or Timer-Based Redraw

From the screen spec's `refresh` field:

**Event-driven (`on_change`):**
- Redraw only when `handle_input()` is called or when a data binding changes
- The application loop calls `navigation.update()` only when it detects a change

**Polling (`interval_ms: N`):**
- The application loop calls `navigation.update()` every N milliseconds
- A timer or `embassy_time::Timer::after(Duration::from_millis(N))` drives the interval
- Scrolling text increments its offset on each polling tick

## UI Module Root → mod.rs Re-exports

The top-level UI module ties everything together:
```
pub mod assets;
pub mod input;
pub mod navigation;
pub mod indicators;
pub mod screens;

pub use navigation::{Navigation, Screen};
pub use input::Gesture;

pub fn init<D: DisplayDriver>(display: &mut D, ctx: &UiContext) -> Result<Navigation, D::Error> {
    let nav = Navigation::new();
    screens::home::draw(display, ctx)?;
    Ok(nav)
}
```
- Re-exports the key types callers need: `Navigation`, `Screen`, `Gesture`, `UiContext`
- `init()` creates the navigation state and draws the initial screen
- The application main loop owns the `Navigation` instance and calls its `handle_input()` and `check_timeout()` methods
