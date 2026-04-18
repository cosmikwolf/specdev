# Firmware Patterns to Detect in Rust Source (Reverse-Engineering)

This file defines what Rust language patterns to look for when reverse-engineering firmware source code into spec YAML. Every rule couples a firmware domain concept with the Rust constructs that indicate it.

---

## ISR Involvement → Interrupt Attributes and ISR Macros

Look for these patterns to detect ISR-related code and set `concurrency.isr_shared: true`:

- `#[interrupt]` attribute on functions → Cortex-M ISR handler
- `#[exception]` attribute → Cortex-M exception handler (HardFault, SysTick, etc.)
- `#[ram]` or `IRAM_ATTR` → ESP32 ISR function placed in IRAM for fast execution
- `embassy_executor` macros: `#[embassy_executor::task]`, `#[embassy_executor::main]` → async task-based ISR handling
- RTIC macros: `#[task(binds = UART0)]`, `#[shared]`, `#[local]` → RTIC interrupt binding
- Functions named `*_isr`, `*_handler`, `*_interrupt` → naming convention for ISR handlers
- `unsafe extern "C" fn` with interrupt-related names → raw ISR handler FFI

## Atomic Usage → core::sync::atomic Imports

Presence of atomic types indicates ISR-shared or concurrent data:

- `use core::sync::atomic::{AtomicU8, AtomicU16, AtomicU32, AtomicBool, AtomicUsize}` → ISR-safe shared variables
- `Ordering::SeqCst` / `Ordering::Relaxed` / `Ordering::Release` / `Ordering::Acquire` → memory ordering for shared data
- `.load(...)` / `.store(...)` / `.fetch_add(...)` / `.compare_exchange(...)` → atomic access patterns
- Static atomics: `static FLAG: AtomicBool = AtomicBool::new(false);` → ISR-shared flag variable
- Map each static atomic to an entry in `isr_shared_vars` with its type

## Critical Sections → critical_section Crate or Platform-Specific

Detect ISR protection strategy for `concurrency.protection`:

- `critical_section::with(|cs| { ... })` → portable critical section → `protection: disable_irq`
- `cortex_m::interrupt::free(|cs| { ... })` → Cortex-M specific → `protection: disable_irq`
- `interrupt::disable()` / `interrupt::enable()` → raw interrupt disable → `protection: disable_irq`
- `esp_idf_hal::task::CriticalSection` → ESP-IDF critical section → `protection: disable_irq`
- `Mutex<RefCell<T>>` combined with critical section → complex ISR-shared data pattern
- Absence of all critical section patterns → `protection: none`

## Static Allocation → Absence of alloc Crate

Determine `memory.allocation` from heap usage signals:

- `#![no_std]` without `extern crate alloc` → `allocation: static` (HIGH confidence)
- Absence of `Box::new`, `Vec::new`, `String::new`, `vec![]`, `format!()` → `allocation: static`
- Presence of `heapless::Vec<T, N>`, `heapless::String<N>`, `heapless::LinearMap` → `allocation: static` with bounded collections
- `extern crate alloc;` or `use alloc::{...}` → `allocation: heap`
- `#[global_allocator]` declaration → `allocation: heap` with explicit allocator
- `Box`, `Vec`, `String`, `Rc`, `Arc` usage → `allocation: heap`

## Error Handling → Result<T, E> Patterns

Detect `error_handling` style from return types and error patterns:

- `Result<T, ModuleError>` return types on public functions → `error_handling: result` (always for Rust)
- `pub enum ModuleError { ... }` → extract exact error variant names and doc comments for the spec. Map each variant to the specific init step or operation that produces it. Implementation-specific variants (e.g., BtInit, A2dpInit, GapInit) are more useful than generic categories because they identify exactly which step failed.
- `impl From<OtherError> for ModuleError` → cross-module error propagation, indicates dependency
- `?` operator usage → Result-based error propagation
- Absence of `unwrap()`, `expect()`, `panic!()` throughout module → `panic_never: true`
- Presence of `unwrap()` or `expect()` → `panic_never: false`, flag as potential issue
- `todo!()`, `unimplemented!()` → stub code, note in spec as incomplete

## RTOS Detection → Framework-Specific Imports and Macros

Identify RTOS/framework for `architecture.yaml`:

- `embassy_executor::Spawner`, `embassy_time::Timer`, `#[embassy_executor::main]` → `rtos: embassy`
- `embassy_sync::mutex::Mutex`, `embassy_sync::channel::Channel` → embassy concurrency primitives
- RTIC attributes: `#[rtic::app(device = ...)]`, `#[task]`, `#[shared]`, `#[local]` → `rtos: rtic`
- `esp_idf_hal::task::*`, `esp_idf_svc::*` → ESP-IDF with Rust bindings (may use FreeRTOS underneath)
- `freertos_rust` crate imports → `rtos: freertos` via Rust bindings
- `tokio::`, `async-std::` → not typical for firmware, but indicates async runtime
- No framework imports, bare `#[entry]` or `#[cortex_m_rt::entry]` → `rtos: none` (bare-metal)

## Module Discovery → mod Declarations and Directory Structure

Identify module boundaries:

- `mod module_name;` declarations in `src/lib.rs` or `src/main.rs` → module exists
- `pub mod module_name;` → module has public API, is a spec candidate
- `src/<module_name>/mod.rs` files → module with submodule structure
- `src/<module_name>.rs` files → single-file module
- `pub struct ModuleName { ... }` → primary module type, maps to spec `module` name
- `impl ModuleName { pub fn new(...) -> ... }` → constructor, indicates owned resources from parameters
- Private `mod tests { ... }` blocks → existing tests, can extract interface coverage

## Dependency Detection → use Statements and Cargo.toml

Map module dependencies for `depends_on` and `external_deps`:

- `use crate::other_module::*` → `depends_on: [other_module]` (HIGH confidence)
- `use crate::other_module::OtherStruct` → specific dependency on another module's type
- `[dependencies]` section in `Cargo.toml` → external crate dependencies
- `use esp_idf_hal::*` → external dep on esp-idf-hal crate
- `use embedded_hal::*` → external dep on embedded-hal trait abstractions
- Feature flags in `Cargo.toml`: `[features]` → may affect available functionality
- `#[cfg(feature = "...")]` guards → conditional compilation affecting module interface

## Peripheral Ownership → Struct Fields with HAL Types

Detect `owns` resources from struct field types:

- `gpio::PinDriver<'_, impl InputPin>` → owns a GPIO input pin
- `gpio::PinDriver<'_, impl OutputPin, Output>` → owns a GPIO output pin
- `spi::SpiDeviceDriver<'_>` → owns an SPI device
- `i2c::I2cDriver<'_>` → owns an I2C bus
- `uart::UartDriver<'_>` → owns a UART peripheral
- `timer::TimerDriver<'_>` → owns a hardware timer
- `adc::AdcChannelDriver<'_>` → owns an ADC channel
- Struct fields with `&'a` references to HAL types → `borrows` rather than `owns`
- Constructor parameters that take ownership (`pin: PinDriver`) vs borrow (`pin: &'a PinDriver`)

## Layer Inference → Import Patterns and Abstraction Level

Determine module `layer` from code structure:

- Direct PAC register access: `unsafe { (*pac::GPIOA::ptr()).odr.write(...) }` → `layer: hal`
- PAC crate imports: `use stm32f4::stm32f411::*` → `layer: pac`
- HAL trait implementations: `impl embedded_hal::digital::OutputPin for ...` → `layer: hal`
- Wrapping HAL drivers: struct fields are HAL types, public API is domain-specific → `layer: driver`
- Protocol logic: Bluetooth, WiFi, USB, I2S codec management → `layer: middleware`
- Application orchestration: `main()`, task spawning, module initialization sequence → `layer: application`
- Calling only other module APIs, no hardware types → `layer: application` or `layer: middleware`

## Display Detection → Graphics Library Imports

Identify display hardware for UI spec reverse-engineering:

- `use embedded_graphics::*` → embedded-graphics display library
- `use ssd1306::*` → SSD1306 OLED driver (128x64 or 128x32, monochrome, I2C/SPI)
- `use st7789::*` → ST7789 TFT driver (240x240 or 240x320, color, SPI)
- `use sh1106::*` → SH1106 OLED driver (128x64, monochrome, I2C/SPI)
- `use ili9341::*` → ILI9341 TFT driver (320x240, color, SPI)
- `use display_interface::*` → generic display interface trait
- `DrawTarget` implementations → custom display driver
- `BinaryColor` → 1-bit monochrome; `Rgb565` → 16-bit color; `Rgb888` → 24-bit color

## Input Detection → GPIO Reads and Encoder Crates

Identify input devices:

- `rotary_encoder_embedded` crate → rotary encoder with A/B channels
- `PinDriver::<Input>::is_high()` / `is_low()` → button or switch GPIO read
- `gpio_isr_handler_add` or GPIO interrupt configuration → interrupt-driven input
- Debounce patterns: timestamp comparisons, `last_press_time`, `debounce_ms` constants → input timing
- State variables named `pressed`, `held`, `released`, `long_press` → button state machine
- ADC reads with threshold comparisons → analog input (slider, potentiometer)

## LED Detection → Color/Pattern Libraries and GPIO PWM

Identify LED indicators:

- `smart_leds` crate, `ws2812_esp32_rmt` → addressable RGB LED (WS2812/NeoPixel)
- `RGB8`, `RGBW`, `set_color`, `set_pixel` patterns → RGB LED control
- `PinDriver::<Output>::set_high/set_low` on LED-named pins → single-color LED
- PWM configuration on LED pins → LED brightness control
- Pattern state machines: `blink`, `pulse`, `breathe`, `fade` in function/variable names → LED animation
- Color constants: `Color::RED`, `RGB8::new(255, 0, 0)` → color definitions for state mapping
- Conditional color assignment: `if status == ... { led.set_color(...) }` → status-to-LED binding

## FSM Detection → State/Event Enum + Match Pattern

Identify state machines for potential `spec/state_machines/` extraction:

- `enum State { Idle, Connected, ... }` or `enum FsmState { ... }` → FSM state enum
- `enum Event { Connect, Timeout, ... }` or `enum FsmEvent { ... }` → FSM event enum
- `match (self.state, event) { ... }` → dispatch on (state, event) tuple → table-style FSM
- `match self.state { State::Idle => match event { ... } }` → nested match → hierarchical FSM
- `self.state = State::Connected;` → state transition assignment
- Struct with `state: State` field → FSM container
- Functions named `dispatch`, `handle_event`, `transition`, `process_event` → FSM dispatch entry point

## Blocking Detection → Function Naming and Async Patterns

Determine `blocking` attribute for interface functions:

- Function names containing `_blocking`, `_wait`, `_poll`, `_sync` → `blocking: true`
- `loop { ... }` with hardware register polling inside → blocking wait pattern
- `while !flag.load(...) { }` → busy-wait loop → `blocking: true`
- `async fn` → non-blocking (runs on async executor) → `blocking: false`
- `.await` usage → async operation → `blocking: false`
- `embassy_time::Timer::after(...)` → async delay → `blocking: false`
- `thread::sleep()` or `FreeRtos::delay_ms()` → blocking delay → `blocking: true`
- Timeout parameters (`timeout_ms: u32`) → may block up to timeout → `blocking: true`

## Trait Implementations → Interface Spec Detection

Detect shared interface contracts:

- `impl TraitName for ModuleName` → module implements an interface contract
- Trait defined in `src/interfaces/` or `src/traits/` → shared interface spec candidate
- `embedded_hal` trait implementations → HAL-level interface compliance
- Generic type parameters `<T: SomeTrait>` → dependency on a trait contract
- `dyn TraitName` or `impl TraitName` in function signatures → polymorphic interface usage
