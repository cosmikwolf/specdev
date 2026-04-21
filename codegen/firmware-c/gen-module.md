# Firmware Module Concepts → C Implementation

This file defines how firmware module spec concepts map to C language constructs. Every rule couples a firmware domain concept with a specific C pattern.

## Source Directory Placeholder

Every path of the form `src/...` or `src/include/...` in this file is shorthand for `<source_dir>/...`, resolved by `/spec-gen` per target:

- If the module YAML carries a `package: <name>` key, `<source_dir>` is `<packages.<name>.source_path>/src/` — for a module `drivers::Led` in a CMake project with `source_path = "components/drivers"`, files land under `components/drivers/src/`.
- Otherwise `<source_dir>` is `<source.root>/` from `spec.toml` (typically `src/`), and substitution is a no-op.

Never hard-code the project root for a packaged module — always route through its package's `source_path`.

## Extraction YAML Schema

See `codegen/firmware-rust/gen-module.md` for the full extraction YAML schema — field names are identical across all languages.

---

## Per-Action Metadata → Function Attributes

Each `actions[]` entry may carry `blocking: true/false` and `isr_safe: true/false`. Use these to:
- Generate Doxygen `@note Blocking` or `@note ISR-safe` comments on each function
- Verify ISR-safe functions do not call blocking dependencies or acquire mutexes
- Determine whether a function should use blocking I/O patterns or non-blocking patterns

---

## ISR-Shared Variables → volatile Qualifier

When `metadata.ConcurrencyModel.shared_state` is non-empty, each variable in that list must be declared with the `volatile` qualifier:
```
static volatile uint32_t sample_count;
static volatile bool data_ready;
static volatile uint8_t buffer_index;
```
- `volatile` prevents the compiler from optimizing away reads/writes that the ISR modifies
- Every ISR-shared variable — whether scalar, struct member, or array element — must be `volatile`
- Compound types: `static volatile struct { uint16_t head; uint16_t tail; } ring_meta;`
- Arrays shared with ISRs: `static volatile uint8_t dma_buffer[DMA_BUF_SIZE];`
- Never cache a volatile value in a local without re-reading: `uint32_t local = sample_count;` is a snapshot — do not assume it stays valid

## ISR Protection Wrapping → Disable/Enable or RTOS Critical Sections

When `metadata.ConcurrencyModel.protection` is `disable_interrupts` or `critical_section`:
```
__disable_irq();
/* access volatile shared variable */
local_copy = sample_count;
__enable_irq();
```
- Always disable before access, enable after — never leave interrupts disabled across a function call
- For ARM Cortex-M: `__disable_irq()` / `__enable_irq()` (CMSIS)
- For ESP-IDF: `portDISABLE_INTERRUPTS()` / `portENABLE_INTERRUPTS()` or `taskENTER_CRITICAL()` / `taskEXIT_CRITICAL(&mux)` with a `portMUX_TYPE` spinlock
- For AVR: `cli()` / `sei()`
- Keep the critical section as short as possible — copy to local, then release

When `metadata.ConcurrencyModel.protection` is `mutex` (FreeRTOS):
```
xSemaphoreTake(module_mutex, portMAX_DELAY);
/* access shared resource */
shared_data.field = new_value;
xSemaphoreGive(module_mutex);
```
- Declare mutex handle: `static SemaphoreHandle_t module_mutex;`
- Create in init: `module_mutex = xSemaphoreCreateMutex();`
- Never take a mutex from an ISR — use `xSemaphoreGiveFromISR()` for ISR-to-task signaling
- Use `xSemaphoreTake(mutex, pdMS_TO_TICKS(timeout_ms))` when blocking timeout is specified

## Static Allocation Constraint → No malloc/free

When `metadata.MemoryModel.allocation` is `static_alloc` (the default):
- Never use `malloc()`, `calloc()`, `realloc()`, `free()`, or `strdup()`
- Declare all instances at file scope: `static module_t instance;`
- Buffers are fixed-size arrays: `static uint8_t rx_buffer[RX_BUF_SIZE];`
- Strings use fixed-size char arrays: `static char name[MAX_NAME_LEN];`
- No variable-length arrays (VLAs) on the stack — use fixed-size arrays or pass caller-owned buffers

When `metadata.MemoryModel.allocation` is `heap_alloc`:
- `malloc()` / `free()` are permitted, but check return values: `if (ptr == NULL) return MODULE_ERR_ALLOC;`
- FreeRTOS: prefer `pvPortMalloc()` / `vPortFree()` for heap-aware allocation
- Always pair allocations with a free path — document ownership in comments

## Instance Limiting → Static Counter with Init Check

When `metadata.MemoryModel.max_instances` is specified:
```
#define MAX_INSTANCES 2
static int instance_count = 0;
static module_t instances[MAX_INSTANCES];

int module_init(module_t **out) {
    if (instance_count >= MAX_INSTANCES) {
        return MODULE_ERR_MAX_INSTANCES;
    }
    module_t *inst = &instances[instance_count];
    /* initialize fields */
    instance_count++;
    *out = inst;
    return MODULE_OK;
}
```
- For `max_instances: 1`: use a single static instance and a `bool initialized` guard
- For `max_instances: N`: use a static array of instances with a counter
- The init function returns a pointer to the allocated instance slot

## Error Handling: return_code → Error Enum with int Return

When `metadata.ErrorHandling.strategy` is `return_code`:
```
typedef enum {
    MODULE_OK = 0,
    MODULE_ERR_INVALID_PARAM = -1,
    MODULE_ERR_TIMEOUT = -2,
    MODULE_ERR_BUSY = -3,
    MODULE_ERR_NOT_INIT = -4,
} module_error_t;
```
- Success is always `0` (`MODULE_OK`)
- Errors are negative integers — each variant from the spec maps to a unique negative value
- All public functions return `int` (or `module_error_t`) — never `void` for fallible operations
- Check return values at every call site: `if (ret != MODULE_OK) { return ret; }` — propagate errors upward

## Error Handling: callback → Function Pointer Parameter

When `metadata.ErrorHandling.strategy` is `callback`:
```
typedef void (*module_error_cb_t)(module_error_t err, void *user_data);

int module_init(module_t *self, module_error_cb_t error_cb, void *user_data);
```
- The callback function pointer is stored in the module struct
- Errors invoke the callback instead of (or in addition to) returning an error code
- Always provide a `user_data` void pointer for context — never use globals for callback context

## Error Handling: output_pointer → Results via Pointer Parameters

When `metadata.ErrorHandling.strategy` is `output_pointer`:
```
int module_read(module_t *self, uint8_t *out_data, size_t *out_len);
```
- The return value is still the error code (success/failure)
- Output data is written through pointer parameters prefixed with `out_`
- Always check output pointers for NULL before writing: `if (out_data == NULL) return MODULE_ERR_INVALID_PARAM;`
- Length parameters are in/out: input specifies buffer capacity, output specifies bytes written

## Ownership Model → Struct Fields and Init Parameters

The extracted `metadata.Ownership.owner` kind combined with `hardware.pins[]`, `hardware.peripherals[]`, and `depends_on[]` maps to C struct composition and pointer storage:

**owned** — resources the module initializes and controls (declared under `hardware`):
```
typedef struct module_s {
    gpio_num_t led_pin;         /* owns: LED GPIO pin */
    spi_device_handle_t spi;    /* owns: SPI device handle */
    uint8_t buffer[BUF_SIZE];   /* owns: internal buffer */
} module_t;
```

**borrowed** — resources received from caller, stored as pointers:
```
typedef struct module_s {
    i2c_port_t i2c_port;        /* borrows: I2C bus (shared) */
    const config_t *config;     /* borrows: configuration (read-only) */
} module_t;
```
- Owned resources are initialized in `module_init()` and cleaned up in `module_deinit()`
- Borrowed resources are passed as parameters to `module_init()` and stored as pointers
- Borrowed resources must outlive the module — document this in the header comment

## Buffer Ownership → Caller vs Module Patterns

When buffer parameters in `actions[]` are caller-owned (the default pattern):
```
int module_read(module_t *self, uint8_t *buf, size_t buf_len, size_t *bytes_read);
int module_write(module_t *self, const uint8_t *data, size_t data_len);
```
- Read functions: caller provides buffer and capacity, module fills and reports bytes written
- Write functions: caller provides data pointer and length, module consumes
- Use `const` qualifier on input buffers — `const uint8_t *data`

When the module owns its internal buffers (declared in the module struct):
```
typedef struct module_s {
    uint8_t internal_buf[INTERNAL_BUF_SIZE];
    size_t buf_len;
} module_t;

const uint8_t *module_get_buffer(const module_t *self, size_t *out_len);
```
- Module manages its own internal buffer
- Expose read access through a const pointer accessor
- Caller must not free or modify the returned pointer

## External Dependencies → Grouped #include Directives

For each `external_deps` entry, generate grouped includes:
```
/* External dependencies */
#include "driver/gpio.h"         /* esp_idf: GPIO driver */
#include "driver/spi_master.h"   /* esp_idf: SPI master driver */
#include "esp_log.h"             /* esp_idf: logging */

/* Internal dependencies */
#include "bt_hal.h"              /* depends_on: bt_hal module */
#include "audio_pipeline.h"      /* depends_on: audio_pipeline module */
```
- External includes first, then internal module includes
- Comment each include with its source package and purpose
- Never include internal module `.c` files — only `.h` headers
- Use `""` quotes for project headers, `<>` for system/standard headers

## Opaque Type Pattern → Forward Declaration with Pointer API

When the spec does not explicitly say the struct should be public:
```
/* In header (.h) */
typedef struct module_s module_t;

/* Public functions take module_t pointer */
int module_init(module_t **out);
int module_read(module_t *self, uint8_t *buf, size_t len);
void module_deinit(module_t *self);
```
```
/* In implementation (.c) */
struct module_s {
    /* private fields */
    gpio_num_t pin;
    uint32_t count;
};
```
- The struct definition is only in the `.c` file — callers cannot access fields
- Public functions accept `module_t *self` as the first parameter (object-oriented style in C)
- This enforces module encapsulation — no module may access another's internal struct fields

## File Header → Spec Citation and Auto-Generation Notice

Every generated file begins with:
```
/**
 * @file module_name.h
 * @brief Public API for the module_name module
 * @spec spec/modules/module_name.yaml
 *
 * AUTO-GENERATED from spec. Do not edit directly.
 * Regenerate with: /fw-gen-module module_name
 */
```
- Always cite the spec file path
- Include the regeneration command for maintainability
- The include guard follows: `#ifndef MODULE_NAME_H` / `#define MODULE_NAME_H` / `#endif`

## Function Documentation → Doxygen Comments from Spec Interface

Each public function gets a Doxygen comment derived from the spec:
```
/**
 * @brief Initialize the module
 * @param[out] out Pointer to receive the module instance
 * @param[in] config Configuration parameters
 * @return MODULE_OK on success, negative error code on failure
 * @note blocking — may take up to 100ms for hardware init
 * @note NOT isr_safe — must not be called from ISR context
 */
int module_init(module_t **out, const module_config_t *config);
```
- `@note blocking` / `@note non-blocking` from spec's `blocking` field
- `@note isr_safe` / `@note NOT isr_safe` from spec's `isr_safe` field
- `@param[in]` / `@param[out]` / `@param[in,out]` based on parameter direction
