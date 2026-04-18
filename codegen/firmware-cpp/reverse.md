# Firmware Patterns to Detect in C++ Source (Reverse-Engineering)

This file defines what C++ language patterns to look for when reverse-engineering firmware source code into spec YAML. Every rule couples a firmware domain concept with the C++ constructs that indicate it.

---

## ISR Involvement → Interrupt Attributes and Handler Functions

Look for these patterns to detect ISR-related code and set `concurrency.isr_shared: true`:

- Functions ending in `_IRQHandler` → ARM Cortex-M NVIC interrupt handler (same as C)
- Functions ending in `_isr` or `_Handler` → generic ISR naming convention
- `IRAM_ATTR` attribute → ESP32 ISR function placed in IRAM
- `__attribute__((interrupt))` → GCC interrupt attribute
- `extern "C"` wrapper around ISR functions → C linkage for interrupt vector table
- ISR registration calls: `NVIC_EnableIRQ()`, `esp_intr_alloc()`, `gpio_isr_handler_add()`
- Static member functions used as ISR callbacks: `static void IRAM_ATTR onInterrupt(void* arg)` — often a class pattern for ISR-to-object bridging

## Atomic Usage → std::atomic<T> and C++ Memory Ordering

Presence of atomic types indicates ISR-shared or concurrent data:

- `std::atomic<uint32_t>`, `std::atomic<bool>`, `std::atomic<int>` → ISR-safe shared variables
- `std::atomic_flag` → lock-free boolean flag for ISR signaling
- `.load(std::memory_order_seq_cst)` / `.store(value, ...)` → atomic access operations
- `.fetch_add()`, `.compare_exchange_weak()`, `.compare_exchange_strong()` → read-modify-write atomics
- `std::atomic_signal_fence()` → memory fence for signal/ISR handlers
- Static atomic members: `static std::atomic<bool> dataReady_;` → class-level ISR-shared state
- Map each `std::atomic` member to `isr_shared_vars` with its underlying type

## Mutex Patterns → Standard Library and RTOS Wrappers

Detect `concurrency.protection: mutex` and `thread_safe: true`:

**Standard library:**
- `#include <mutex>` → mutex usage
- `std::mutex` member variable → module uses mutex protection
- `std::lock_guard<std::mutex>` → RAII lock/unlock pattern
- `std::unique_lock<std::mutex>` → conditional locking or condition variable usage
- `std::scoped_lock` → multi-mutex locking (C++17)
- `std::recursive_mutex` → re-entrant locking (flag as design concern)

**RTOS wrappers:**
- Custom `Mutex` class wrapping `xSemaphoreTake`/`xSemaphoreGive` → FreeRTOS C++ wrapper
- `FreeRtosMutex`, `OsMutex`, `CriticalGuard` → named wrapper classes
- Template RAII wrappers: `template<typename M> class LockGuard { ... }` → generic lock wrapper

**Condition variables:**
- `std::condition_variable` → waiting pattern, indicates `blocking: true` for associated functions

## Critical Sections → RAII Wrappers Around Interrupt Disable

Detect `concurrency.protection: disable_irq`:

- Class named `CriticalSection`, `InterruptGuard`, `IrqLock` → RAII critical section wrapper
- Constructor calls `__disable_irq()` or `taskENTER_CRITICAL()` → interrupt disable on scope entry
- Destructor calls `__enable_irq()` or `taskEXIT_CRITICAL()` → interrupt enable on scope exit
- Deleted copy constructor/assignment → prevents scope extension (good practice indicator)
- Scoped usage: `{ CriticalSection cs; /* access shared data */ }` → RAII pattern
- PRIMASK save/restore in constructor/destructor → nested-safe critical section

## Static Allocation → Absence of Dynamic Memory

Determine `memory.allocation`:

- Absence of `new`, `delete`, `new[]`, `delete[]` → `allocation: static` (HIGH confidence)
- Absence of `std::make_unique`, `std::make_shared`, `std::unique_ptr`, `std::shared_ptr` → reinforces static
- `= delete` on copy/move constructors → designed for static allocation
- `std::array<T, N>` usage instead of `std::vector<T>` → bounded, static
- `std::string_view` instead of `std::string` → non-owning, no allocation
- Placement new: `new (&storage) Type(args)` → static storage with explicit construction (still static allocation)
- Presence of `new`/`delete` or smart pointers → `allocation: heap`
- Overridden `operator new`/`delete` using `pvPortMalloc`/`vPortFree` → heap with RTOS allocator

## Error Handling → Return Types and Exception Patterns

Detect `error_handling` style:

**result (C++23 std::expected or custom):**
- `std::expected<T, E>` return types → `error_handling: result`
- `std::optional<T>` return types → lightweight result (no error info)
- Custom `Result<T, E>` template → project-specific result type
- `std::unexpected(error)` → error construction

**exception:**
- `throw` statements → exception-based error handling
- `try { ... } catch (...) { ... }` blocks → exception handling
- Custom exception classes inheriting `std::exception` or `std::runtime_error`
- `-fno-exceptions` compiler flag → exceptions disabled, cannot be `exception` style
- `-fno-rtti` → RTTI disabled, affects `dynamic_cast` in exception handling

**return_code:**
- `enum class Error : int { Ok = 0, ... }` → typed return code
- Functions returning `int` with 0=success pattern → C-style return code
- `esp_err_t` return type → ESP-IDF convention

**Mixed patterns** (common in C++ firmware):
- Public API uses `std::expected`, internal implementation uses exceptions → note both in spec
- C API wrappers converting exceptions to error codes at boundary

Extract error variant names from `enum class` definitions.

## RTOS Detection → Framework Includes and C++ Wrappers

Identify RTOS/framework:

- Same C patterns apply: FreeRTOS includes, Zephyr includes, etc.
- C++ wrapper classes for FreeRTOS: `Task`, `Queue<T>`, `Mutex`, `Timer` wrapping C API
- `std::thread` usage (with POSIX threading underneath) → Linux-targeted or POSIX RTOS
- `std::async`, `std::future` → standard async patterns (uncommon in embedded)
- Template-based task wrappers: `template<size_t StackSize> class Task { ... }` → statically-sized RTOS tasks
- No RTOS includes and bare `main()` with `while(1)` → `rtos: none`

## Module Discovery → Class Declarations and Namespace Structure

Identify module boundaries:

- `.hpp` files in `src/include/` → module public API headers
- `.cpp` files in `src/<module>/` → module implementation
- `class ModuleName { ... }` with public methods → primary module type
- `namespace module_name { ... }` → module namespace scoping
- `namespace detail { ... }` inside module namespace → private implementation details
- Forward declarations: `class ModuleName;` in other headers → dependency indicator
- Separate interface: `class IModuleName { virtual ... }` + `class ModuleName : public IModuleName` → interface/implementation split

## Dependency Detection → #include and using Declarations

Map module dependencies:

- `#include "other_module.hpp"` → `depends_on: [other_module]` (HIGH confidence)
- `#include "other_module.h"` → dependency on C module (mixed-language project)
- `using namespace other_module;` in `.cpp` file → implementation dependency
- Forward declarations in header → lighter dependency (compile-time only)
- Template parameter constraints: `requires SomeConcept<T>` (C++20) → interface dependency
- Constructor parameter types from other modules → ownership/borrow dependency

## Display Detection → Graphics Library Classes and Draw Calls

Identify display hardware for UI spec:

**Same C library patterns (used via C linkage):**
- `u8g2_DrawStr()`, `ssd1306_*()` → C display library used from C++

**C++ wrappers and libraries:**
- Custom `Display` class with `drawText()`, `drawRect()`, `clear()`, `flush()` methods → display abstraction
- `IDisplay` or `DisplayDriver` abstract class → display interface
- Inheritance: `class OledDisplay : public IDisplay` → concrete display implementation
- Template display drivers: `template<typename Bus> class SSD1306 { ... }` → parameterized driver
- embedded-graphics equivalent in C++: drawing primitive methods on a canvas class

## Namespace Usage → Module Scoping and Encapsulation

Detect module organization:

- `namespace firmware { ... }` → project-level namespace
- `namespace firmware::module_name { ... }` → C++17 nested namespace for module
- `namespace { ... }` (anonymous) in `.cpp` → internal linkage (module-private implementations)
- `using` aliases: `using Error = module_name::ModuleError;` → type aliasing
- `inline namespace v1 { ... }` → API versioning pattern

## Template Patterns → Generic HAL and Driver Abstractions

Detect abstraction patterns relevant to module specs:

- `template<typename T> class Driver { ... }` → generic driver parameterized by hardware type
- `template<typename SpiDriver, typename GpioPin> class Sensor { ... }` → hardware-abstracted sensor
- CRTP: `class Module : public Base<Module>` → compile-time polymorphism (avoids vtable)
- `template<size_t BufferSize>` → compile-time buffer sizing → extract for `memory` spec
- Concepts (C++20): `template<typename T> requires Writable<T>` → constrained interface
- `std::enable_if`, `if constexpr` → conditional compilation based on hardware features

## FSM Detection → State Machine Classes and Patterns

Identify state machines:

- `enum class State { Idle, Connected, ... }` → FSM state enum
- `enum class Event { Connect, Timeout, ... }` → FSM event enum
- `switch (state_) { case State::Idle: ... }` → switch-based dispatch
- Member: `State state_;` → state stored in class
- `dispatch(Event)` or `handleEvent(Event)` method → FSM entry point
- Nested classes `struct StateIdle {}; struct StateConnected {};` → type-state pattern
- `std::variant<StateIdle, StateConnected, ...>` → variant-based FSM
- `std::visit` with lambda/overloaded → variant dispatch
- Transition tables as `constexpr std::array` → table-driven FSM
- Function pointer arrays: `using ActionFn = void (*)(); static const ActionFn actions[N][M];` → action dispatch table

## Blocking Detection → Function Signatures and Async Patterns

Determine `blocking` attribute:

- Method names containing `_blocking`, `_wait`, `_poll`, `_sync` → `blocking: true`
- `std::this_thread::sleep_for()` → blocking delay → `blocking: true`
- `std::future::wait()`, `std::future::get()` → blocking wait → `blocking: true`
- `std::condition_variable::wait()` → blocking wait → `blocking: true`
- `while (!ready_) { ... }` → busy wait → `blocking: true`
- `std::async(std::launch::async, ...)` → async operation → caller may block on `.get()`
- Callback-based: method returns immediately, result via callback/lambda → `blocking: false`
- Fire-and-forget: method enqueues work and returns → `blocking: false`
- Return type `std::future<T>` → non-blocking if caller does not immediately `.get()`

## RAII Resource Management → Ownership Detection

Detect `owns` vs `borrows` from C++ ownership patterns:

- Move-constructed member: `member_(std::move(param))` → owns (resource transferred in)
- Reference member: `Type& ref_;` → borrows (lifetime managed by caller)
- `const Type&` member → borrows read-only
- `std::unique_ptr<T>` member → exclusive ownership
- `std::shared_ptr<T>` member → shared ownership
- Raw pointer member: `Type* ptr_;` → ambiguous, check if `delete` is called in destructor (owns) or not (borrows)
- Constructor that creates hardware handles → owns the hardware
- Constructor that receives handles by reference → borrows
