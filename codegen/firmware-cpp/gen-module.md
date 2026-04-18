# Firmware Module Concepts → C++ Implementation

This file defines how firmware module spec concepts map to C++ language constructs. Every rule couples a firmware domain concept with a specific C++ pattern.

## Extraction YAML Schema

See `codegen/firmware-rust/gen-module.md` for the full extraction YAML schema — field names are identical across all languages.

---

## Per-Action Metadata → Function Attributes

Each `actions[]` entry may carry `blocking: true/false` and `isr_safe: true/false`. Use these to:
- Generate Doxygen `@note Blocking` or `@note ISR-safe` comments on each method
- Verify ISR-safe methods do not call blocking dependencies or acquire mutexes
- Determine whether a method should use blocking I/O patterns or non-blocking patterns

---

## ISR-Shared Variables → volatile or std::atomic<T>

When `metadata.ConcurrencyModel.shared_state` is non-empty, each variable in that list maps based on type and access pattern:

**Integral types with atomic hardware support:**
```
#include <atomic>
static std::atomic<uint32_t> sample_count{0};
static std::atomic<bool> data_ready{false};
```
- Use `std::atomic<T>` for types where the platform supports lock-free atomics (typically up to pointer-width)
- Read: `sample_count.load(std::memory_order_seq_cst)`
- Write: `sample_count.store(value, std::memory_order_seq_cst)`
- `std::memory_order_seq_cst` is the safe default for ISR-shared data — relaxed ordering only when spec explicitly permits

**Types without atomic support or compound types:**
```
static volatile SensorReading last_reading;
```
- Use `volatile` qualifier when `std::atomic` is not available for the type
- Always access within a critical section — `volatile` alone does not guarantee atomicity for multi-word types

## ISR Protection → RAII Critical Section Wrapper

When `metadata.ConcurrencyModel.protection` is `disable_interrupts` or `critical_section`:
```
class CriticalSection {
public:
    CriticalSection() { __disable_irq(); }
    ~CriticalSection() { __enable_irq(); }
    CriticalSection(const CriticalSection&) = delete;
    CriticalSection& operator=(const CriticalSection&) = delete;
};

// Usage:
{
    CriticalSection cs;
    local_copy = sample_count;
}  // interrupts re-enabled at scope exit
```
- RAII ensures interrupts are always re-enabled, even if an exception or early return occurs
- Delete copy/move to prevent accidental scope extension
- For ARM Cortex-M: `__disable_irq()` / `__enable_irq()` in constructor/destructor
- For ESP-IDF: `portENTER_CRITICAL(&mux)` / `portEXIT_CRITICAL(&mux)` with a `portMUX_TYPE` member
- For nested critical sections: save and restore PRIMASK instead of unconditional enable:
  ```
  class CriticalSection {
      uint32_t primask_;
  public:
      CriticalSection() : primask_(__get_PRIMASK()) { __disable_irq(); }
      ~CriticalSection() { __set_PRIMASK(primask_); }
  };
  ```

## Mutex Protection → std::lock_guard or RTOS Wrapper

When `metadata.ConcurrencyModel.protection` is `mutex`:
```
#include <mutex>

class Module {
    std::mutex mutex_;
    int shared_data_;

public:
    int read_data() {
        std::lock_guard<std::mutex> lock(mutex_);
        return shared_data_;
    }
};
```
- Use `std::lock_guard<std::mutex>` for simple lock/unlock scope
- Use `std::unique_lock<std::mutex>` when conditional locking or early unlock is needed
- For FreeRTOS without `<mutex>`: create a RAII wrapper class around `xSemaphoreTake`/`xSemaphoreGive`:
  ```
  class FreeRtosMutex {
      SemaphoreHandle_t handle_;
  public:
      FreeRtosMutex() : handle_(xSemaphoreCreateMutex()) {}
      ~FreeRtosMutex() { vSemaphoreDelete(handle_); }
      void lock() { xSemaphoreTake(handle_, portMAX_DELAY); }
      void unlock() { xSemaphoreGive(handle_); }
  };
  ```
- Never take a mutex from an ISR context — use a binary semaphore with `GiveFromISR` for ISR-to-task signaling

## Static Allocation Constraint → No new/delete, Deleted Copy/Move

When `metadata.MemoryModel.allocation` is `static_alloc`:
- Never use `new`, `delete`, `new[]`, `delete[]`
- Never use `std::make_unique`, `std::make_shared`
- Delete copy and move constructors to prevent accidental heap allocation:
  ```
  class Module {
  public:
      Module(const Module&) = delete;
      Module& operator=(const Module&) = delete;
      Module(Module&&) = delete;
      Module& operator=(Module&&) = delete;
  };
  ```
- Use `std::array<T, N>` instead of `std::vector<T>` for fixed-size collections
- Use `std::string_view` instead of `std::string` for non-owning string references
- Static instances: `static Module instance;` at file scope or `static Module& getInstance()` pattern
- Placement new is acceptable for initializing static storage: `new (&static_storage) Module(args);`

When `metadata.MemoryModel.allocation` is `heap_alloc`:
- `std::make_unique<T>()` and `std::make_shared<T>()` are preferred over raw `new`
- RTOS-specific allocators: override `operator new`/`delete` to use `pvPortMalloc`/`vPortFree`

## Instance Limiting → Static Counter or Meyers Singleton

When `metadata.MemoryModel.max_instances` is specified:

**For `max_instances: 1` (singleton):**
```
class Module {
public:
    static Module& instance() {
        static Module inst;
        return inst;
    }
private:
    Module() = default;
    Module(const Module&) = delete;
    Module& operator=(const Module&) = delete;
};
```

**For `max_instances: N > 1`:**
```
class Module {
    static inline int instance_count_ = 0;
    static constexpr int kMaxInstances = 4;

public:
    static std::expected<Module*, ModuleError> create() {
        if (instance_count_ >= kMaxInstances) {
            return std::unexpected(ModuleError::MaxInstances);
        }
        instance_count_++;
        return &instances_[instance_count_ - 1];
    }

private:
    static Module instances_[kMaxInstances];
};
```

## Error Handling: result_type → std::expected<T, E> or Custom Result

When `metadata.ErrorHandling.strategy` is `result_type` (preferred for modern C++):
```
enum class ModuleError {
    Ok = 0,
    InvalidParam,
    Timeout,
    Busy,
    NotInitialized,
};

// C++23:
std::expected<SensorData, ModuleError> read_sensor();

// Pre-C++23 fallback:
template<typename T, typename E>
class Result {
    // ... custom implementation
};
```
- All fallible methods return `std::expected<T, ModuleError>` (C++23) or a project-level `Result<T, E>`
- Success: return the value directly — `return sensor_data;`
- Failure: `return std::unexpected(ModuleError::Timeout);`
- Error variants come from the spec's error definitions

## Error Handling: callback → throw with enum class

When `metadata.ErrorHandling.strategy` is `callback` (mapped to C++ exceptions when the project opts in):
```
class ModuleException : public std::runtime_error {
public:
    ModuleError code;
    explicit ModuleException(ModuleError c, const char* msg = "")
        : std::runtime_error(msg), code(c) {}
};

void Module::init() {
    if (!hardware_ready()) {
        throw ModuleException(ModuleError::NotInitialized, "Hardware not ready");
    }
}
```
- Only use exceptions when the project's C++ configuration explicitly enables them — many embedded toolchains disable RTTI/exceptions
- Boundary functions (public API) catch and convert: `try { ... } catch (const ModuleException& e) { return e.code; }`
- Never throw from destructors or ISR-callable functions

## Error Handling: return_code → enum class with int-like API

When `metadata.ErrorHandling.strategy` is `return_code` (C-style but type-safe):
```
enum class ModuleError : int {
    Ok = 0,
    InvalidParam = -1,
    Timeout = -2,
    Busy = -3,
};

ModuleError Module::init() {
    // ...
    return ModuleError::Ok;
}
```
- `enum class` provides type safety over C's plain enum
- Check with `if (ret != ModuleError::Ok) { return ret; }`

## Ownership Model → RAII with Constructor Acquisition

The extracted `metadata.Ownership.owner` kind combined with `hardware.pins[]`, `hardware.peripherals[]`, and `depends_on[]` maps to C++ RAII and reference semantics:

**owned** — resources acquired in constructor, released in destructor:
```
class Module {
    GpioPin led_pin_;           // owns: LED GPIO
    SpiDevice spi_;             // owns: SPI peripheral
    std::array<uint8_t, 256> buffer_;  // owns: internal buffer

public:
    Module(GpioPin pin, SpiDevice spi)
        : led_pin_(std::move(pin))
        , spi_(std::move(spi))
        , buffer_{} {}

    ~Module() {
        // RAII: resources released in reverse order of acquisition
    }
};
```
- Use move semantics (`std::move`) for transferring peripheral ownership into the module
- Destructor releases hardware resources — follows C++ RAII idiom

**borrowed** — resources received as references, lifetime managed by caller:
```
class Module {
    I2cBus& shared_bus_;        // borrows: I2C bus (shared)
    const Config& config_;      // borrows: configuration (read-only)

public:
    Module(I2cBus& bus, const Config& config)
        : shared_bus_(bus), config_(config) {}
};
```
- Reference members are initialized in the constructor initializer list
- `const&` for read-only borrowed resources
- `&` for mutable borrowed resources
- Document that borrowed resources must outlive the module instance

## Buffer Ownership → std::span or std::array

When buffer parameters in `actions[]` are caller-owned (the default pattern):
```
// C++20:
ModuleError read(std::span<uint8_t> buffer, size_t& bytes_read);
ModuleError write(std::span<const uint8_t> data);

// Pre-C++20:
ModuleError read(uint8_t* buf, size_t buf_len, size_t& bytes_read);
ModuleError write(const uint8_t* data, size_t data_len);
```
- `std::span<uint8_t>` is preferred over raw pointer+length — carries size information
- `std::span<const uint8_t>` for read-only input buffers

When the module owns its internal buffers (declared in the module struct):
```
class Module {
    std::array<uint8_t, 512> internal_buffer_;
    size_t buffer_len_ = 0;

public:
    std::span<const uint8_t> get_buffer() const {
        return {internal_buffer_.data(), buffer_len_};
    }
};
```

## External Dependencies → Grouped #include with Namespaces

For each `external_deps` entry:
```
/* External dependencies */
#include "driver/gpio.h"         // esp_idf: GPIO driver
#include "driver/spi_master.h"   // esp_idf: SPI master

/* Internal dependencies */
#include "bt_hal.hpp"            // depends_on: bt_hal module
#include "audio_pipeline.hpp"    // depends_on: audio_pipeline
```
- Same grouping as C — external first, then internal
- If the external package provides C++ wrappers with namespaces: `using namespace hal;` only in `.cpp`, never in `.hpp`
- Prefer C++ headers over C: `<cstdint>` over `<stdint.h>`, `<cstring>` over `<string.h>`

## Module Class Structure → Single Class per Module

Each module spec generates one class:
```
/**
 * @file module_name.hpp
 * @brief Public API for the module_name module
 * @spec spec/modules/module_name.yaml
 *
 * AUTO-GENERATED from spec. Do not edit directly.
 * Regenerate with: /fw-gen-module module_name
 */

#pragma once

#include <cstdint>
#include <expected>  // C++23

namespace firmware {

enum class ModuleError { /* ... */ };

class ModuleName {
public:
    explicit ModuleName(/* constructor params from owns/borrows */);
    ~ModuleName();

    // Delete copy/move if static allocation
    ModuleName(const ModuleName&) = delete;
    ModuleName& operator=(const ModuleName&) = delete;

    /** @brief blocking — see spec for timeout behavior */
    std::expected<SensorData, ModuleError> read_sensor();

    /** @brief non-blocking, isr_safe */
    ModuleError set_output(uint8_t value);

private:
    // owned resources
    // borrowed references
    // internal state
};

}  // namespace firmware
```
- Wrap in a project namespace: `namespace firmware { ... }`
- Use `#pragma once` instead of include guards for C++
- `explicit` constructor to prevent implicit conversions
- Document `blocking` and `isr_safe` attributes in method comments

## Namespace Scoping → Module Isolation

Each module uses namespace scoping for encapsulation:
```
namespace firmware {
namespace module_name {

// internal implementation details

}  // namespace module_name
}  // namespace firmware
```
- The module class itself is in `namespace firmware`
- Internal helpers, constants, and types go in a nested `namespace module_name::detail` or an anonymous namespace in the `.cpp` file
- Never expose implementation types in the public header — use forward declarations or the pimpl idiom

## Template Patterns → Generic HAL Wrappers

When the module wraps hardware with interchangeable implementations:
```
template<typename SpiDriver>
class SensorModule {
    SpiDriver& spi_;
public:
    explicit SensorModule(SpiDriver& spi) : spi_(spi) {}
    ModuleError read(uint8_t reg, uint8_t& value) {
        return spi_.transfer(reg, value);
    }
};
```
- Template parameters allow compile-time polymorphism without virtual dispatch overhead
- Useful for HAL-layer modules that should work with different bus implementations
- Keep templates in the header — embedded linkers may not handle separate template instantiation well
