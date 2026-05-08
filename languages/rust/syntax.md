# Rust Syntax Patterns

Pure language syntax reference for Rust code generation. These patterns apply to any domain.

---

## File Layout

Each module produces two files:

```
src/<module>/mod.rs      # Public API and implementation
src/<module>/tests.rs    # Test stubs
```

The parent `src/lib.rs` or `src/main.rs` must declare each module with `mod <module>;`.

In multi-crate workspaces, `src/` in the paths above is `<source_dir>/`: the
`source_path` of the spec module's package, resolved by `/spec-gen`. E.g.
`drivers::Led` with `source_path = "crates/drivers"` lands at
`crates/drivers/src/led/mod.rs`.

---

## Source-to-spec mapping

Rust projects map to spec layout as follows:

- **Cargo workspace member** → one package in `spec.toml` `[[packages]]`. The
  member's directory (`crates/<name>`) is the package's `source_path`.
- **Single Cargo crate** → no `[[packages]]` needed; the project is flat.
- **`src/<name>.rs` or `src/<name>/mod.rs`** → one spec module per file, named
  PascalCase (e.g. `src/status_led.rs` → `StatusLed.toml`). A file that spans
  two layers (e.g. both HAL wrappers and middleware logic) becomes *two* spec
  modules, not one.
- **Types-only crate** (common names: `common`, `types`, `interfaces`) → no
  spec modules; its types surface as shared ports in `architecture.toml`.
- **Binary crate `src/main.rs`** → the `main` entrypoint is not a spec module;
  it wires up other modules and belongs in `architecture.toml`.

---

## File Header Comment

Every generated file begins with a doc comment block citing the spec and generation origin:

```rust
//! @file <module>/mod.rs
//! @brief Public API for the <module> module
//! @spec spec/modules/<module>.yaml
//!
//! AUTO-GENERATED from spec. Do not edit directly.
//! Regenerate with: /fw-gen-module <module>
```

---

## Module-Level Attributes

When targeting a bare-metal or `no_std` environment (i.e., the spec does not explicitly allow std):

```rust
#![no_std]
```

Place this as the very first line of `mod.rs`, before the file header comment.

When std is explicitly allowed, omit `#![no_std]`.

---

## Use Statements

Group `use` statements in this order, separated by blank lines:

1. **Core/std imports** (from `core::` or `std::`)
2. **External crate imports** — each annotated with a comment naming the package
3. **Internal module imports** — sibling modules from the same project

```rust
use core::fmt;

// External: <package_name>
use some_external_crate::SomeTrait;
use another_crate::AnotherType;

// Internal
use crate::other_module::OtherStruct;
use crate::another_module::AnotherError;
```

For `external_deps`, generate `use` statements from the `used_modules` list in the spec. Each external group gets a comment identifying the package name.

---

## Error Enum

Define a module-level error enum with standard derives. Only include variants that are explicitly described or clearly implied by the spec:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum <Module>Error {
    /// Description of variant
    VariantOne,
    /// Description of variant
    VariantTwo,
}
```

- Enum name: `<ModulePascalCase>Error`
- Each variant gets a doc comment if the spec provides a description
- Variants are unit variants unless the spec specifies payload data

### Display and Error Trait Implementations

When std is available, implement `std::fmt::Display` and `std::error::Error`:

```rust
impl core::fmt::Display for <Module>Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::VariantOne => write!(f, "description"),
            Self::VariantTwo => write!(f, "description"),
        }
    }
}
```

---

## Module Struct

Define the primary struct with fields derived from the spec's `owns` and `borrows` lists:

- `owns` fields become owned types
- `borrows` fields become references with appropriate lifetime parameters

```rust
pub struct <ModuleName><'a> {
    // Owned resources
    owned_field: OwnedType,

    // Borrowed resources
    borrowed_field: &'a BorrowedType,
}
```

If there are no borrowed references, omit the lifetime parameter:

```rust
pub struct <ModuleName> {
    owned_field: OwnedType,
}
```

### Visibility

- Fields are private by default (no `pub` on struct fields)
- The struct itself is `pub`
- Use `pub(crate)` for items that should be visible within the crate but not externally

---

## Impl Block

Methods match every entry in the spec's `interface` list:

```rust
impl<'a> <ModuleName><'a> {
    /// Brief description from spec
    pub fn function_name(&self, param: ParamType) -> Result<ReturnType, <Module>Error> {
        todo!("implement")
    }

    /// Brief description from spec
    pub fn mutable_function(&mut self, param: ParamType) -> Result<(), <Module>Error> {
        todo!("implement")
    }
}
```

### Method Signatures

- Fallible functions return `Result<T, <Module>Error>`
- Infallible functions return the value directly (no `Result` wrapper)
- Use `&self` for read-only access, `&mut self` for mutation
- Method bodies contain `todo!("implement")` as a stub

---

## Constructor Pattern

The conventional constructor is a `new` associated function:

```rust
impl<'a> <ModuleName><'a> {
    /// Create a new <ModuleName> instance
    pub fn new(param: ParamType, borrowed: &'a BorrowedType) -> Result<Self, <Module>Error> {
        todo!("implement")
    }
}
```

- Constructors are fallible by default (return `Result<Self, Error>`)
- Parameters correspond to owned and borrowed resources from the spec
- If `memory.max_instances` is specified, enforce the limit in the constructor

---

## Static Allocation

When `memory.allocation` is `static`, declare module-level static instances:

```rust
use core::sync::atomic::AtomicBool;

static INSTANCE_INITIALIZED: AtomicBool = AtomicBool::new(false);
```

No `alloc` usage (no `Box`, `Vec`, `String`, `Arc`, etc.) unless the spec explicitly allows it.

---

## Visibility Rules

| Scope | Keyword | Usage |
|-------|---------|-------|
| Public API | `pub` | Struct, error enum, interface methods |
| Crate-internal | `pub(crate)` | Helper functions, internal types |
| Module-private | (none) | Implementation details, private fields |

---

## Test Module

Tests live in a separate file `src/<module>/tests.rs`, structured as:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_function_name() {
        // TODO: implement test
    }

    #[test]
    fn test_another_function() {
        // TODO: implement test
    }
}
```

- One `#[test]` function per interface entry in the spec
- Each test body contains a `// TODO: implement test` placeholder
- Tests import all items from the parent module with `use super::*`

---

## Trait Implementations

When the module must implement a trait defined in `spec/interfaces/`:

```rust
impl SomeTrait for <ModuleName> {
    fn trait_method(&self, param: ParamType) -> ReturnType {
        todo!("implement")
    }
}
```

Traits themselves (from interface specs) are defined as:

```rust
pub trait TraitName {
    fn method_name(&self, param: ParamType) -> Result<ReturnType, ErrorType>;
}
```

---

## Documentation Comments

- Use `///` for public items (structs, enums, functions, methods)
- Use `//!` for module-level documentation (file header)
- Use `//` for inline implementation comments

```rust
/// A brief description of the struct
pub struct Example {
    /// Description of the field's purpose
    field: u32,
}
```
