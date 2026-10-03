# C Syntax Patterns

Pure language syntax reference for C code generation. These patterns apply to any domain.

---

## File Layout

Each module produces three files:

```
src/include/<module>.h           # Public header (declarations)
src/<module>/<module>.c          # Implementation
tests/<module>_test.c            # Test stubs
```

In multi-subproject CMake builds (ESP-IDF components, Zephyr modules, etc.),
`src/` above is `<source_dir>/`: the `source_path` of the spec module's
package. E.g. `drivers::led` with
`source_path = "components/drivers"` lands at
`components/drivers/src/led/led.c`.

---

## Source-to-spec mapping

C projects map to spec layout as follows:

- **CMake `add_subdirectory(components/<name>)` target** (ESP-IDF component,
  Zephyr module, CMake subproject) → one package in `spec.toml` `[[packages]]`.
  The subdirectory path is the `source_path`.
- **Single-binary CMake project** → no `[[packages]]`; the project is flat.
- **`.c` + matching `.h` pair** → one spec module. File stem is snake_case;
  the spec `name` is PascalCase (e.g. `status_led.c`/`status_led.h` →
  `StatusLed.toml`). A `.c` file that spans layers becomes multiple spec
  modules.
- **Types-only component** (`include/` with headers but no `.c`) → no spec
  modules; its structs/enums become shared ports in `architecture.toml`.
- **`main.c`** → not a spec module; it wires modules together per
  `architecture.toml`.

---

## File Header Comment

Every generated file begins with a doxygen-style comment block citing the spec and generation origin:

```c
/**
 * @file <module>.h
 * @brief Public API for the <module> module
 * @spec spec/modules/<module>.yaml
 *
 * AUTO-GENERATED from spec. Do not edit directly.
 * Regenerate with: /fw-gen-module <module>
 */
```

The same format applies to `.c` and `_test.c` files, with the appropriate filename.

---

## Include Guard

Every `.h` file uses a preprocessor include guard:

```c
#ifndef <MODULE_NAME_UPPER>_H
#define <MODULE_NAME_UPPER>_H

/* ... declarations ... */

#endif /* <MODULE_NAME_UPPER>_H */
```

The guard macro is the module name in SCREAMING_SNAKE_CASE followed by `_H`.

---

## Include Grouping

Includes are grouped in a specific order, separated by blank lines and annotated with comments:

```c
/* Standard library */
#include <stdint.h>
#include <stdbool.h>
#include <string.h>

/* External dependencies */
#include "some_external_lib.h"    /* package_name */
#include "another_lib/api.h"     /* another_package */

/* Internal dependencies */
#include "other_module.h"
#include "another_module.h"
```

1. **Standard library** headers (`<stdint.h>`, `<stdbool.h>`, `<string.h>`, `<stdlib.h>`, etc.)
2. **External dependency** headers — each with a trailing comment naming the package
3. **Internal project** headers — other modules from `depends_on`

---

## Error Enum

Define a module-level error enum using `typedef enum`. The first value is the success code at 0, and error codes are negative:

```c
typedef enum {
    <MODULE>_OK = 0,
    <MODULE>_ERR_TIMEOUT = -1,
    <MODULE>_ERR_INVALID_PARAM = -2,
    <MODULE>_ERR_NOT_INITIALIZED = -3,
} <module>_error_t;
```

- Enum tag: `<MODULE_UPPER>_OK`, `<MODULE_UPPER>_ERR_<NAME>`
- Typedef name: `<module_lower>_error_t`
- Only include error variants that are explicitly described or clearly implied by the spec

---

## Opaque Struct Pattern

In the public header (`.h`), declare a forward (opaque) type:

```c
typedef struct <module>_s <module>_t;
```

In the implementation file (`.c`), provide the full struct definition:

```c
struct <module>_s {
    uint32_t field_a;
    bool field_b;
    /* fields derived from spec owns/borrows */
};
```

This pattern hides internal details from consumers of the module. Only use a public struct definition if the spec explicitly states the struct should be public.

---

## Function Declarations

In the header file, declare public functions with doxygen-style comments:

```c
/**
 * @brief Initialize the module
 *
 * @param[in]  config  Configuration parameters
 * @param[out] handle  Pointer to module handle
 * @return <module>_error_t  <MODULE>_OK on success
 */
<module>_error_t <module>_init(const <module>_config_t *config, <module>_t **handle);

/**
 * @brief Perform an operation
 *
 * @param[in] handle  Module handle
 * @param[in] param   Input parameter
 * @return <module>_error_t  <MODULE>_OK on success
 */
<module>_error_t <module>_do_something(<module>_t *handle, uint32_t param);
```

### Naming Convention

- All public symbols are prefixed with `<module>_`
- Function names use `snake_case`
- Type names use `snake_case` with `_t` suffix
- Enum values use `SCREAMING_SNAKE_CASE`

### Parameter Annotations

- `@param[in]` — input parameter (read-only)
- `@param[out]` — output parameter (written by function)
- `@param[in,out]` — bidirectional parameter

---

## Function Stubs

In the implementation file, each function body contains a stub:

```c
<module>_error_t <module>_init(const <module>_config_t *config, <module>_t **handle)
{
    /* TODO: implement */
    return <MODULE>_OK;
}
```

- Each function body contains a `/* TODO: implement */` comment
- Returns the appropriate success value (`<MODULE>_OK` or `0`)
- Parameter validation stubs may return `<MODULE>_ERR_INVALID_PARAM`

---

## Static Instance Pattern

When `memory.allocation` is `static`, declare module-level static instances in the `.c` file:

```c
static struct <module>_s instance;
static bool initialized = false;
```

If `max_instances` is specified:

```c
#define <MODULE>_MAX_INSTANCES 4

static struct <module>_s instances[<MODULE>_MAX_INSTANCES];
static uint32_t instance_count = 0;
```

Enforce the limit in the init function:

```c
<module>_error_t <module>_init(/* ... */)
{
    if (instance_count >= <MODULE>_MAX_INSTANCES) {
        return <MODULE>_ERR_NO_RESOURCE;
    }
    /* TODO: implement */
    instance_count++;
    return <MODULE>_OK;
}
```

---

## Const Correctness

- Use `const` for pointer parameters that should not be modified
- Use `const` for configuration structs passed by pointer:

```c
<module>_error_t <module>_configure(const <module>_config_t *config);
```

---

## Test Stub File

`tests/<module>_test.c` follows this structure:

```c
/**
 * @file <module>_test.c
 * @brief Test stubs for the <module> module
 * @spec spec/modules/<module>.yaml
 *
 * AUTO-GENERATED from spec. Do not edit directly.
 * Regenerate with: /fw-gen-module <module>
 */

#include "<module>.h"

void test_<function_name>(void)
{
    /* TODO: implement test */
}

void test_<another_function>(void)
{
    /* TODO: implement test */
}

int main(void)
{
    test_<function_name>();
    test_<another_function>();
    return 0;
}
```

- One test stub function per interface entry: `void test_<function_name>(void)`
- Each test body contains `/* TODO: implement test */`
- A `main` function calls all test functions sequentially and returns 0

---

## Typedef Patterns

### Configuration Struct

```c
typedef struct {
    uint32_t setting_a;
    uint16_t setting_b;
    bool enable_feature;
} <module>_config_t;
```

### Callback Function Pointer

```c
typedef void (*<module>_callback_t)(void *context, <module>_event_t event);
```

### Enum Typedef

```c
typedef enum {
    <MODULE>_STATE_IDLE,
    <MODULE>_STATE_ACTIVE,
    <MODULE>_STATE_ERROR,
} <module>_state_t;
```

---

## Documentation Comments

- Use `/** ... */` doxygen-style for public API documentation
- Use `/* ... */` for implementation comments
- Use `// ...` for inline comments (C99+)

```c
/**
 * @brief Brief description of the function
 *
 * Detailed description spanning multiple lines
 * if needed.
 *
 * @param[in] param  Description of parameter
 * @return Error code
 */
```
