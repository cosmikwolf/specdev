# Firmware UI Concepts → C++ Implementation

This file defines how firmware UI spec concepts map to C++ language constructs. Every rule couples a UI domain concept with a specific C++ pattern.

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

## Display Interface → Abstract Base Class with Pure Virtual Methods

The display interface spec maps to an abstract C++ class:
```
#pragma once
#include <cstdint>
#include <string_view>

enum class FontId { Font6x10, Font8x13, Custom, Count };
enum class Align { Left, Center, Right };
enum class DrawStyle { Outline, Filled };

class IDisplay {
public:
    virtual ~IDisplay() = default;

    virtual void clear() = 0;
    virtual void flush() = 0;
    virtual void drawText(int x, int y, std::string_view text, FontId font, Align align = Align::Left) = 0;
    virtual void drawIcon(int x, int y, const uint8_t* data, uint16_t w, uint16_t h) = 0;
    virtual void drawRect(int x, int y, int w, int h, DrawStyle style = DrawStyle::Outline) = 0;
    virtual void drawLine(int x1, int y1, int x2, int y2) = 0;
    virtual void drawProgressBar(int x, int y, int w, int h, int value, int max) = 0;
};
```
- Pure virtual methods define the drawing contract — the project implements a concrete subclass for its display library
- `std::string_view` for text parameters — zero-copy, no allocation
- Virtual destructor ensures proper cleanup through base pointer
- Generated UI code accepts `IDisplay&` — never a concrete type
- For embedded targets where vtable overhead matters, use CRTP instead:
  ```
  template<typename Derived>
  class DisplayBase {
  public:
      void clear() { static_cast<Derived*>(this)->clearImpl(); }
      // ... compile-time polymorphism, no vtable
  };
  ```

## Screen Rendering → IScreen Interface with Per-Screen Classes

Each screen spec generates a class implementing a common screen interface:
```
class IScreen {
public:
    virtual ~IScreen() = default;
    virtual void draw(IDisplay& display, const UiContext& ctx) = 0;
    virtual void onEnter(IDisplay& display, const UiContext& ctx) { draw(display, ctx); }
    virtual void onExit() {}
};

class ScreenHome : public IScreen {
public:
    void draw(IDisplay& display, const UiContext& ctx) override {
        display.clear();

        // Element: title text (layout.elements[0])
        display.drawText(0, 0, "Home", FontId::Font6x10, Align::Left);

        // Element: status text with data binding (layout.elements[1])
        std::string_view statusText;
        switch (ctx.btStatus) {
            case BtStatus::Disconnected: statusText = "Disconnected"; break;
            case BtStatus::Connecting:   statusText = "Connecting..."; break;
            case BtStatus::Connected:    statusText = "Connected"; break;
            case BtStatus::Streaming:    statusText = "Playing"; break;
        }
        display.drawText(0, 16, statusText, FontId::Font6x10, Align::Left);

        // Element: volume bar (layout.elements[2])
        display.drawProgressBar(0, 48, 128, 8, ctx.volume, 100);

        display.flush();
    }
};
```
- Each `layout.elements` entry becomes a drawing call in the `draw()` method
- `format_map` entries map to `switch` statements converting bound values to display strings
- `visible_when` conditions map to `if` guards around element draw calls
- `onEnter()` default implementation calls `draw()` — override for entry animations or initialization
- `onExit()` for cleanup when navigating away from the screen

## Font Data → Static Arrays or Library Font References

Fonts from the extracted `fonts[]` data:

**Builtin fonts (mapped by display implementation):**
```
// The display implementation maps FontId to its library's font data
// e.g., for u8g2: FontId::Font6x10 → u8g2_font_6x10_tr
```

**Custom font data:**
```
namespace ui::assets {
    // Generated from font file, or placeholder if file not found
    static const uint8_t kFontCustomData[] = { 0 };  // TODO: provide font file
    static constexpr size_t kFontCustomSize = sizeof(kFontCustomData);
}
```

**Font ID enum** (defined in the display interface header, see above).

## Icon Data → Static const Arrays with Metadata

Icons from the extracted `icons[]` data:

**Inline icons:**
```
namespace ui::assets {
    struct Icon {
        uint16_t width;
        uint16_t height;
        const uint8_t* data;
    };

    static const uint8_t kIconPlayData[] = { 0x00, 0x04, 0x0C, 0x1C, 0x3C, 0x1C, 0x0C, 0x04 };
    static constexpr Icon kIconPlay = { 8, 8, kIconPlayData };
}
```

**File-source icons:**
```
namespace ui::assets {
    // TODO: generate from assets/logo.raw
    static const uint8_t kIconLogoData[] = { 0 };
    static constexpr Icon kIconLogo = { 32, 32, kIconLogoData };
}
```

## Navigation → Screen Enum with Variant-Based or Table-Based Dispatch

The navigation spec maps to a screen management class:

**Enum + switch dispatch (simpler, preferred for small screen counts):**
```
enum class Screen {
    Home = 0,
    Settings,
    Volume,
    Count,
};

class Navigation {
public:
    Navigation() : current_(Screen::Home) {}

    void handleInput(Gesture gesture, IDisplay& display, UiContext& ctx) {
        lastInputTimeMs_ = ctx.nowMs;

        // Global actions
        if (gesture == Gesture::BothPress) {
            // global action from spec
        }

        // Per-screen dispatch
        switch (current_) {
            case Screen::Home:
                handleHomeInput(gesture, display, ctx);
                break;
            case Screen::Settings:
                handleSettingsInput(gesture, display, ctx);
                break;
            // ...
        }
    }

    Screen currentScreen() const { return current_; }

private:
    Screen current_;
    uint32_t lastInputTimeMs_ = 0;

    void handleHomeInput(Gesture gesture, IDisplay& display, UiContext& ctx);
    void handleSettingsInput(Gesture gesture, IDisplay& display, UiContext& ctx);
};
```

**std::variant dispatch (type-safe, for complex screens with state):**
```
using ScreenVariant = std::variant<ScreenHome, ScreenSettings, ScreenVolume>;

class Navigation {
    ScreenVariant current_{ScreenHome{}};

public:
    void handleInput(Gesture gesture, IDisplay& display, UiContext& ctx) {
        std::visit([&](auto& screen) {
            handleScreenInput(screen, gesture, display, ctx);
        }, current_);
    }
};
```
- `navigate` actions change `current_` and call the new screen's `onEnter()`
- `adjust` actions modify context fields: `ctx.volume = std::clamp(ctx.volume + delta, 0, 100);`
- `invoke` actions call through function pointers or direct module interface calls

## Input Dispatch → Gesture Enum from HAL

Input devices produce gestures:
```
enum class Gesture {
    None = 0,
    Press,
    LongPress,
    DoublePress,
    RotateCw,
    RotateCcw,
    // Combinations
    BothPress,
    // Sequences
    TripleClick,
    Count,
};

class InputDispatch {
public:
    Gesture poll();

private:
    // Combination and sequence detection state
    uint32_t lastGestureTimeMs_ = 0;
    int sequenceStep_ = 0;
};
```
- `poll()` reads from the input HAL module — does NOT implement debounce
- Combination detection: check multiple device states simultaneously within `combo_window_ms`
- Sequence detection: internal step counter reset after `sequence_timeout_ms`
- Return `Gesture::None` when no gesture detected

## LED Indicator Binding → Status-to-Pattern Mapping Method

Indicator specs map to an update method:
```
struct LedPattern {
    enum class Type { Off, Solid, Blink, Breathe, Pulse };
    Type type = Type::Off;
    uint32_t color = 0;       // 0xRRGGBB
    uint32_t periodMs = 0;    // for non-solid patterns
};

class Indicators {
public:
    void update(const UiContext& ctx, ILedDriver& led) {
        // status_led indicator from spec
        LedPattern pattern;
        switch (ctx.btStatus) {
            case BtStatus::Disconnected:
                pattern = { LedPattern::Type::Breathe, 0x0000FF, 2000 };
                break;
            case BtStatus::Connecting:
                pattern = { LedPattern::Type::Blink, 0x0000FF, 200 };
                break;
            case BtStatus::Connected:
                pattern = { LedPattern::Type::Solid, 0x0000FF, 0 };
                break;
            case BtStatus::Streaming:
                pattern = { LedPattern::Type::Solid, 0x00FF00, 0 };
                break;
        }
        led.setPattern(pattern);
    }
};
```
- Each `mapping` entry in the indicator spec becomes a switch case
- `bound_to` field determines which context field to switch on
- `ILedDriver` is an abstract interface to the LED hardware module
- Pattern types carry timing information — the LED driver handles the animation

## Spec-Hash Fingerprint → File Header Comment

Every generated file begins with:
```
/**
 * @generated from ui/screens/home.toml + ui/assets.toml
 * @spec-hash a1b2c3d4e5f6...
 *
 * DO NOT EDIT — regenerate with /fw-gen-ui
 */
```
- Same format as C — `/fw-gen-ui` parses this to determine regeneration freshness
- The hash is SHA-256 of the concatenated content of all source .toml files this file depends on

## Data Context Struct → Aggregated Bindings

All data bindings aggregate into a context struct:
```
struct UiContext {
    // From home screen bindings
    BtStatus btStatus = BtStatus::Disconnected;
    char trackTitle[64] = {};  // fixed-size when memory.allocation is static
    uint8_t volume = 0;

    // From settings screen bindings
    uint8_t menuIndex = 0;

    // Timing
    uint32_t nowMs = 0;

    // Scroll state
    int16_t titleScrollOffset = 0;
};
```
- camelCase member names following C++ convention
- Default member initializers for safety
- Fixed-size char arrays when `memory.allocation: static` — `std::string` only when heap is allowed
- Alternatively, use `std::string_view` for non-owning references to module-owned strings

## Timeout Handling → Elapsed Time Check

```
void Navigation::checkTimeout(IDisplay& display, UiContext& ctx) {
    uint32_t elapsed = ctx.nowMs - lastInputTimeMs_;

    // Per-screen timeout
    if (current_ == Screen::Volume && elapsed > kVolumeTimeoutMs) {
        navigateTo(Screen::Home, display, ctx);
        return;
    }

    // Global timeout
    if (elapsed > kGlobalTimeoutMs) {
        navigateTo(Screen::Home, display, ctx);
    }
}
```
- `navigateTo()` handles `onExit` / `onEnter` calls and screen redraw
- Timeout constants as `static constexpr uint32_t` class members

## UI Module Root → Namespace with Init/Update Functions

The top-level UI module:
```
namespace ui {

class Ui {
public:
    void init(IDisplay& display, const UiContext& ctx) {
        nav_ = Navigation{};
        screens_[static_cast<int>(Screen::Home)]->draw(display, ctx);
    }

    void update(IDisplay& display, UiContext& ctx) {
        Gesture gesture = input_.poll();
        if (gesture != Gesture::None) {
            nav_.handleInput(gesture, display, ctx);
        }
        nav_.checkTimeout(display, ctx);
        indicators_.update(ctx, led_);
    }

private:
    Navigation nav_;
    InputDispatch input_;
    Indicators indicators_;
    ILedDriver& led_;
};

}  // namespace ui
```
- Single `Ui` class owns the navigation, input dispatch, and indicator subsystems
- `init()` sets initial screen and draws it
- `update()` is called from the application loop — handles everything
- The application owns the `IDisplay` implementation and populates the `UiContext`
