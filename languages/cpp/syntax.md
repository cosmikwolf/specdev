# C++ Syntax Patterns

Pure language syntax reference for C++ code generation. These patterns apply to any domain.

---

## File Layout

Each module produces three files:

```
src/include/<module>.hpp         # Public header (class declarations)
src/<module>/<module>.cpp        # Implementation
tests/<module>_test.cpp          # Test stubs
```

---

## File Header Comment

Every generated file begins with a doxygen-style comment block citing the spec and generation origin:

```cpp
/**
 * @file <module>.hpp
 * @brief Public API for the <module> module
 * @spec spec/modules/<module>.yaml
 *
 * AUTO-GENERATED from spec. Do not edit directly.
 * Regenerate with: /fw-gen-module <module>
 */
```

The same format applies to `.cpp` and `_test.cpp` files, with the appropriate filename.

---

## Include Guard

Use `#pragma once` (preferred) or traditional include guards:

### Preferred

```cpp
#pragma once
```

### Alternative

```cpp
#ifndef <MODULE_NAME_UPPER>_HPP
#define <MODULE_NAME_UPPER>_HPP

/* ... declarations ... */

#endif /* <MODULE_NAME_UPPER>_HPP */
```

The guard macro is the module name in SCREAMING_SNAKE_CASE followed by `_HPP`.

---

## Include Grouping

Includes are grouped in a specific order, separated by blank lines and annotated with comments:

```cpp
/* Standard library */
#include <cstdint>
#include <cstddef>
#include <expected>       // C++23 — for result-style error handling
#include <memory>
#include <optional>

/* External dependencies */
#include "some_external_lib.hpp"    /* package_name */
#include "another_lib/api.hpp"     /* another_package */

/* Internal dependencies */
#include "other_module.hpp"
#include "another_module.hpp"
```

1. **Standard library** headers (`<cstdint>`, `<expected>`, `<memory>`, `<string>`, `<optional>`, etc.)
2. **External dependency** headers — each with a trailing comment naming the package
3. **Internal project** headers — other modules from `depends_on`

---

## Error Enum Class

Define a scoped enum class for module errors:

```cpp
enum class <Module>Error {
    Ok = 0,
    Timeout,
    InvalidParam,
    NotInitialized,
};
```

- Enum name: `<ModulePascalCase>Error`
- `Ok = 0` is always the first value
- Only include error variants that are explicitly described or clearly implied by the spec

---

## Class Declaration

The module's primary type is a class with public methods and private members:

```cpp
class <ModuleName> {
public:
    /// Constructor
    explicit <ModuleName>(ParamType param);

    /// Destructor
    ~<ModuleName>();

    /// Public method matching interface spec entry
    std::expected<ReturnType, <Module>Error> do_something(ParamType param);

    /// Another public method
    <Module>Error configure(const ConfigType& config);

private:
    // Owned resources (from spec owns list)
    OwnedType owned_field_;

    // Borrowed resources (from spec borrows list) — stored as references or pointers
    const BorrowedType& borrowed_field_;

    // Internal state
    bool initialized_;
};
```

### Member Naming Convention

- Private member variables use trailing underscore: `field_name_`
- Public methods use `snake_case`
- Class names use `PascalCase`

---

## Deleted Copy/Move Constructors

When `memory.allocation` is `static` or the type manages a unique resource, delete copy and move operations:

```cpp
class <ModuleName> {
public:
    <ModuleName>(const <ModuleName>&) = delete;
    <ModuleName>& operator=(const <ModuleName>&) = delete;
    <ModuleName>(<ModuleName>&&) = delete;
    <ModuleName>& operator=(<ModuleName>&&) = delete;

    // ... rest of class
};
```

---

## Error Handling Patterns

### Result-Style (std::expected)

When `error_handling` is `result`, fallible methods return `std::expected<T, Error>` (C++23):

```cpp
std::expected<uint32_t, <Module>Error> read_value();
std::expected<void, <Module>Error> write_value(uint32_t val);
```

For pre-C++23, use a project-level `Result<T, E>` type or return the error enum directly:

```cpp
<Module>Error write_value(uint32_t val);
<Module>Error read_value(uint32_t* out_val);
```

### Exception-Based

When `error_handling` is `exception`, methods throw on failure:

```cpp
/// @throws <Module>Exception on failure
uint32_t read_value();

/// @throws <Module>Exception on failure
void write_value(uint32_t val);
```

Define a corresponding exception class:

```cpp
#include <stdexcept>

class <Module>Exception : public std::runtime_error {
public:
    explicit <Module>Exception(<Module>Error code, const char* message)
        : std::runtime_error(message), error_code_(code) {}

    <Module>Error code() const noexcept { return error_code_; }

private:
    <Module>Error error_code_;
};
```

Exceptions must not cross module boundaries unless the spec explicitly allows it.

### Return Code

When `error_handling` is `return_code` or `error_code`, methods return the error enum directly:

```cpp
<Module>Error do_something(ParamType input, ReturnType* output);
```

---

## Constructor and Destructor

```cpp
// In .hpp
class <ModuleName> {
public:
    explicit <ModuleName>(const ConfigType& config);
    ~<ModuleName>();
};

// In .cpp
<ModuleName>::<ModuleName>(const ConfigType& config)
    : owned_field_{}
    , borrowed_field_(config.some_ref)
    , initialized_(false)
{
    // TODO: implement
}

<ModuleName>::~<ModuleName>()
{
    // TODO: implement cleanup
}
```

- Use `explicit` for single-parameter constructors to prevent implicit conversion
- Use member initializer lists
- RAII: acquire resources in constructor, release in destructor

---

## Method Stubs

In the implementation file, each method body contains a stub:

```cpp
std::expected<uint32_t, <Module>Error> <ModuleName>::read_value()
{
    // TODO: implement
    return std::unexpected(<Module>Error::NotInitialized);
}

<Module>Error <ModuleName>::configure(const ConfigType& config)
{
    // TODO: implement
    return <Module>Error::Ok;
}
```

---

## Static Instance Pattern

When `memory.allocation` is `static`, declare module-level static storage:

```cpp
namespace {
    alignas(<ModuleName>) std::byte instance_storage[sizeof(<ModuleName>)];
    bool initialized = false;
}
```

Or with a simpler approach:

```cpp
namespace {
    <ModuleName>* instance = nullptr;
    // Static storage allocated at file scope
}
```

If `max_instances` is specified, use a fixed-size array:

```cpp
namespace {
    constexpr size_t kMaxInstances = 4;
    std::array<std::optional<<ModuleName>>, kMaxInstances> instances;
}
```

---

## Namespace Usage

Wrap module declarations in a project namespace:

```cpp
namespace <project_name> {

class <ModuleName> {
    // ...
};

} // namespace <project_name>
```

In the implementation file:

```cpp
namespace <project_name> {

<ModuleName>::<ModuleName>(/* ... */)
{
    // TODO: implement
}

} // namespace <project_name>
```

---

## Abstract Base Class (Interface Pattern)

Interfaces from `spec/interfaces/` map to abstract base classes with pure virtual methods:

```cpp
class I<InterfaceName> {
public:
    virtual ~I<InterfaceName>() = default;

    virtual std::expected<ReturnType, ErrorType> method_name(ParamType param) = 0;
    virtual void another_method() = 0;
};
```

The module class inherits from the interface:

```cpp
class <ModuleName> : public I<InterfaceName> {
public:
    std::expected<ReturnType, ErrorType> method_name(ParamType param) override;
    void another_method() override;
};
```

---

## Test Stub File

`tests/<module>_test.cpp` follows this structure:

```cpp
/**
 * @file <module>_test.cpp
 * @brief Test stubs for the <module> module
 * @spec spec/modules/<module>.yaml
 *
 * AUTO-GENERATED from spec. Do not edit directly.
 * Regenerate with: /fw-gen-module <module>
 */

#include "<module>.hpp"

void test_function_name()
{
    // TODO: implement test
}

void test_another_function()
{
    // TODO: implement test
}

int main()
{
    test_function_name();
    test_another_function();
    return 0;
}
```

- One test stub function per interface entry: `void test_<function_name>()`
- Each test body contains `// TODO: implement test`
- A `main` function calls all test functions and returns 0

### With Test Framework

When a test framework is available (e.g., Catch2, Google Test):

```cpp
// Google Test example
#include <gtest/gtest.h>
#include "<module>.hpp"

TEST(<ModuleName>Test, FunctionName) {
    // TODO: implement test
}

TEST(<ModuleName>Test, AnotherFunction) {
    // TODO: implement test
}
```

```cpp
// Catch2 example
#include <catch2/catch_test_macros.hpp>
#include "<module>.hpp"

TEST_CASE("<ModuleName>::function_name", "[<module>]") {
    // TODO: implement test
}
```

---

## Const Correctness

- Use `const` member functions for read-only methods:

```cpp
uint32_t get_value() const;
bool is_initialized() const noexcept;
```

- Use `const&` for input parameters that should not be copied or modified:

```cpp
<Module>Error configure(const ConfigType& config);
```

- Use `noexcept` for methods that are guaranteed not to throw:

```cpp
bool is_ready() const noexcept;
```

---

## Documentation Comments

- Use `/** ... */` doxygen-style for public API documentation
- Use `///` for single-line doc comments
- Use `//` for inline implementation comments

```cpp
/**
 * @brief Brief description of the class
 *
 * Detailed description.
 */
class Example {
public:
    /// Brief description of the method
    /// @param param  Description of parameter
    /// @return Description of return value
    std::expected<int, Error> method(int param);
};
```
