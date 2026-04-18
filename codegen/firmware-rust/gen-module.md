# Firmware Module Concepts → Rust Implementation

This file defines how firmware module spec concepts map to Rust language constructs. Every rule couples a firmware domain concept with a specific Rust pattern.

## Extraction YAML Schema

Code generation reads the per-module YAML produced by `specdev-analyzer extract`. Key fields:

| YAML path | Description |
|---|---|
| `name` | PascalCase module name |
| `layer` | `application`, `middleware`, `driver`, `hal`, or `pac` |
| `description` | one-line module purpose |
| `depends_on[]` | names of other modules this module depends on |
| `satisfies[]` | requirement ids this module satisfies |
| `external_deps[]` | external package names (resolved via `architecture.yaml`) |
| `metadata.MemoryModel.allocation` | `static_alloc`, `heap_alloc`, or `stack_alloc` |
| `metadata.MemoryModel.max_instances` | integer (omit for unbounded) |
| `metadata.MemoryModel.owner` | `owned`, `borrowed`, or `shared` |
| `metadata.ConcurrencyModel.protection` | `none`, `critical_section`, `mutex`, `atomic`, or `disable_interrupts` |
| `metadata.ConcurrencyModel.isr_safe` | boolean — module-level ISR safety |
| `metadata.ConcurrencyModel.reentrant` | boolean |
| `metadata.ConcurrencyModel.shared_state[]` | names of variables touched by ISR and task contexts |
| `metadata.ErrorHandling.strategy` | `return_code`, `result_type`, `callback`, or `output_pointer` |
| `metadata.ErrorHandling.error_type` | optional Rust/C error type name when strategy is `result_type` |
| `metadata.Ownership.owner` | `owned`, `borrowed`, or `shared` (ownership kind — the resources themselves live under `hardware`) |
| `hardware.pins[]` | `{number, role, direction}` — owned/borrowed GPIO pins |
| `hardware.peripherals[]` | `{name, kind}` — e.g. SPI, I2C, I2S, UART instances |
| `hardware.clocks[]` | `{name, frequency_hz}` — required clock sources |
| `ports[]` | `{name, direction, protocol}` |
| `actions[]` | `{name, params[]{name, type}, returns, blocking, isr_safe, description}` |
| `state_machines[]` | `{name, initial, states[], transitions[]{from, to, event, guard, action}}` |
| `errors[]` | `{variant, description}` — error variants for this module |

---

## Per-Action Metadata → Function Attributes

Each `actions[]` entry may carry `blocking: true/false` and `isr_safe: true/false`. Use these to:
- Generate doc comments noting blocking behavior and ISR safety on each function
- Verify ISR-safe actions do not call blocking dependencies or acquire mutexes
- Determine whether a function should use blocking I/O patterns (e.g., DMA wait) or non-blocking patterns

---

## ISR-Shared Variables → Atomic Types or Critical Section Wrappers

When `metadata.ConcurrencyModel.shared_state` is non-empty, each variable in shared state maps to a Rust type based on its width and complexity:

- **Integral types (u8, u16, u32, bool)** → `AtomicU8`, `AtomicU16`, `AtomicU32`, `AtomicBool` from `core::sync::atomic`
  - Read access: `.load(Ordering::SeqCst)` — SeqCst is the safe default for ISR-shared data
  - Write access: `.store(value, Ordering::SeqCst)`
  - Relaxed ordering is only permitted when the spec explicitly notes no ordering dependency
- **Complex types (structs, enums with data, arrays)** → `critical_section::Mutex<RefCell<T>>`
  - Access pattern: `critical_section::with(|cs| { SHARED.borrow_ref_mut(cs).field = value; })`
  - Never hold the critical section across a blocking call

## ISR Protection → critical_section Crate

When `metadata.ConcurrencyModel.protection` is `disable_interrupts` or `critical_section`:
- Use `critical_section::with(|cs| { ... })` for portable ISR protection
- For Cortex-M specific targets: `cortex_m::interrupt::free(|cs| { ... })` is acceptable
- For ESP32 targets: `esp_idf_hal::task::CriticalSection` or the `critical_section` crate with the esp-hal implementation
- Never use raw `unsafe { interrupt::disable(); }` without re-enable — RAII wrappers only

## Static Allocation Constraint → No Alloc Crate

When `metadata.MemoryModel.allocation` is `static_alloc` (the default):
- Add `#![no_std]` at crate root — never use `std`
- Never use `Box`, `Vec`, `String`, `Rc`, `Arc` from `alloc`
- Use `heapless::Vec<T, N>` for bounded collections where N is a const generic capacity
- Use `heapless::String<N>` for bounded strings
- Use fixed-size arrays `[T; N]` for static buffers
- Use `heapless::pool` or `heapless::LinearMap` for dynamic-like patterns with static backing

When `metadata.MemoryModel.allocation` is `heap_alloc`:
- `extern crate alloc;` is permitted
- A global allocator must be declared: `#[global_allocator] static ALLOCATOR: ...`
- Prefer bounded types even with heap — `Vec::with_capacity(N)` over unbounded growth

## Memory Model: Static Instances → Mutex<RefCell<Option<T>>> Pattern

When a module must be a statically-allocated singleton:
```
static INSTANCE: critical_section::Mutex<RefCell<Option<ModuleName>>> =
    critical_section::Mutex::new(RefCell::new(None));
```
- Initialize via `critical_section::with(|cs| { INSTANCE.borrow_ref_mut(cs).replace(ModuleName::new()); })`
- Access via `critical_section::with(|cs| { let m = INSTANCE.borrow_ref(cs); ... })`
- This pattern avoids `unsafe` and provides ISR-safe access

## Instance Limiting → Static Counter or Option<&'static Self>

When `metadata.MemoryModel.max_instances` is specified:
- For `max_instances: 1`: use `Option<&'static Self>` singleton pattern or the `Mutex<RefCell<Option<T>>>` static pattern above
- For `max_instances: N > 1`: use a static `AtomicU8` counter, check in `new()`:
  ```
  static COUNT: AtomicU8 = AtomicU8::new(0);
  if COUNT.load(Ordering::SeqCst) >= MAX_INSTANCES { return Err(ModuleError::MaxInstances); }
  COUNT.fetch_add(1, Ordering::SeqCst);
  ```

## Error Handling → Result<T, ModuleError>

Regardless of `metadata.ErrorHandling.strategy`, Rust always uses `Result<T, ModuleError>`:
- Define `#[derive(Debug, Clone, Copy, PartialEq, Eq)] pub enum ModuleError { ... }` with variants from the `errors[]` list in the extracted YAML
- All fallible interface functions return `Result<T, ModuleError>`
- When strategy is `result_type`: use `Result` with `?` everywhere — no `unwrap()`, `expect()`, `panic!()`
- When strategy is `return_code`: map to `Result` — Rust does not use integer return codes
- When strategy is `output_pointer` or `callback`: keep the Rust API as `Result<T, ModuleError>`; the pointer/callback shape is only relevant at FFI boundaries
- Implement `From<DependencyError> for ModuleError` for each dependency's error type to enable `?` propagation across module boundaries

## Ownership Model → Struct Fields with Lifetimes

The extracted `metadata.Ownership.owner` kind combined with the `hardware` section maps to Rust's ownership system:
- **owned** resources (pins/peripherals declared in `hardware`) → owned types as struct fields: `pub struct Module { peripheral: SpiDevice, buffer: heapless::Vec<u8, 256> }`
- **borrowed** resources → reference fields with explicit lifetimes: `pub struct Module<'a> { shared_bus: &'a Mutex<RefCell<SpiBus>> }`
- **shared** resources → reference-counted or statically-shared handle types (e.g. `&'static Mutex<...>`)
- When a module borrows from another module, the borrowing is expressed through constructor parameters, not global access
- Peripheral ownership follows the type-state pattern from embedded-hal: once a peripheral is moved into a module, it cannot be used elsewhere — this enforces exclusive access at compile time

## External Dependencies → use Statements with Crate Paths

For each `external_deps` entry in the module spec:
- Map `used_modules` to `use` statements: `use esp_idf_hal::gpio::{PinDriver, Input, Output};`
- Comment the source package: `// from esp-idf-hal (external_packages.esp_idf)`
- Group external imports before internal: external crate `use` statements first, then `use crate::other_module`
- Resolve crate names from `external_packages` in `architecture.yaml` — the spec package name may differ from the Rust crate name (e.g., `esp_idf` package → `esp-idf-hal`, `esp-idf-svc` crates)

## no_std Enforcement → Crate-Level Attributes

When the target is bare-metal or embedded:
- `#![no_std]` at crate root
- `#![no_main]` if the module is the entry point
- Use `core::` instead of `std::` for all core library types
- Use `core::fmt::Write` instead of `std::io::Write`
- Never import `std::` anything — compiler errors are the enforcement mechanism

## Buffer Ownership → Slices and heapless Collections

When a function in `actions[]` accepts caller-owned buffers:
- Functions accept `&[u8]` for read-only buffers and `&mut [u8]` for write buffers
- Return the number of bytes written via `Result<usize, ModuleError>`
- Never store references to caller buffers beyond the function call

When the module owns its internal buffers (declared in the module struct):
- Internal buffers are `heapless::Vec<u8, N>` fields in the module struct
- Provide accessor methods returning `&[u8]` slices for callers to read

## Mutex Protection → RTOS-Specific Wrappers

When `metadata.ConcurrencyModel.protection` is `mutex`:
- For FreeRTOS (via esp-idf): `esp_idf_hal::task::Mutex<T>` or `std::sync::Mutex<T>` (if using esp-idf-hal with std feature)
- For Embassy: `embassy_sync::mutex::Mutex<NoopRawMutex, T>` or `CriticalSectionRawMutex`
- For RTIC: shared resources are declared in `#[shared]` struct and accessed via lock proxies
- Mark the module as `thread_safe: true` when all public methods acquire the mutex before accessing shared state

## Module File Structure → mod.rs with Submodules

Each module generates:
- `src/<module_name>/mod.rs` — public API, struct definition, impl block, error enum
- `src/<module_name>/tests.rs` — `#[cfg(test)] mod tests { ... }` with one `#[test]` per interface function
- Register in `src/main.rs` or `src/lib.rs` with `mod <module_name>;`
- If the module has an FSM, it may have an additional `src/<module_name>/fsm.rs` submodule

## Trait Implementations → Interface Spec Contracts

When `spec/interfaces/` contains a trait contract that this module must implement:
- Generate `impl TraitName for ModuleName { ... }` block
- Each trait method must match the interface spec signature exactly
- Use the trait's associated types for error types where specified
- Import the trait: `use crate::interfaces::TraitName;`
