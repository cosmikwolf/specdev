# Rust Build System Patterns

Pure build system reference for Rust projects using Cargo. These patterns apply to any domain.

---

## Workspace Cargo.toml

When a project contains multiple modules, use a workspace at the project root:

```toml
[workspace]
members = [
    "src/module_a",
    "src/module_b",
    "src/module_c",
]
```

Each module directory listed in `members` must contain its own `Cargo.toml`.

---

## Single-Crate Cargo.toml

For projects that compile as a single crate (all modules in one package):

```toml
[package]
name = "<project_name>"
version = "0.1.0"
edition = "<edition>"     # e.g. "2021" or "2024"

[dependencies]
# Internal and external dependencies listed here
```

---

## Per-Module Cargo.toml

Each module directory has its own `Cargo.toml`:

```toml
[package]
name = "<module_name>"
version = "0.1.0"
edition = "<edition>"

[dependencies]
# From depends_on: internal workspace crates
other_module = { path = "../other_module" }

# From external_deps: external crates
some_crate = "1.2"
another_crate = { version = "0.5", features = ["feature_a", "feature_b"] }
```

---

## Mapping External Packages to Cargo Dependencies

External packages from the architecture spec map to `[dependencies]` entries. The `build_integration.cargo` field in the architecture spec provides the exact Cargo dependency string.

### Source Types

| Source Type | Cargo Mapping |
|-------------|---------------|
| `crate` | Standard crates.io dependency: `name = "version"` |
| `git` | Git dependency: `name = { git = "url", rev = "ref" }` |
| `sdk` | SDK-specific crate: follows `build_integration.cargo` exactly |
| `component` | Framework registry crate: follows `build_integration.cargo` exactly |

### Version and Features

```toml
[dependencies]
# Simple version constraint
simple_dep = "1.0"

# With features enabled
featured_dep = { version = "2.3", features = ["async", "derive"] }

# Git source with specific revision
git_dep = { git = "https://github.com/org/repo", rev = "abc123" }

# Git source with branch
branch_dep = { git = "https://github.com/org/repo", branch = "main" }

# Path dependency (workspace member)
local_dep = { path = "../local_dep" }

# With default-features disabled
minimal_dep = { version = "1.0", default-features = false, features = ["needed_feature"] }
```

---

## Build Target Configuration

For embedded or cross-compilation targets, configure the target triple in `.cargo/config.toml`:

```toml
[build]
target = "<target_triple>"       # e.g. "thumbv7em-none-eabihf", "xtensa-esp32-none-elf"

[target.<target_triple>]
runner = "<flash_tool_command>"   # Optional: tool to flash/run on target

rustflags = [
    "-C", "link-arg=-Tlink.x",   # Linker script (if needed)
]
```

### Common Target Triples

| Architecture | Triple Example |
|-------------|----------------|
| ARM Cortex-M0/M0+ | `thumbv6m-none-eabi` |
| ARM Cortex-M3 | `thumbv7m-none-eabi` |
| ARM Cortex-M4/M7 (no FPU) | `thumbv7em-none-eabi` |
| ARM Cortex-M4/M7 (hard float) | `thumbv7em-none-eabihf` |
| RISC-V 32-bit | `riscv32imc-unknown-none-elf` |
| Xtensa | `xtensa-esp32-none-elf` |
| Native (host) | Omit `[build] target` |

---

## Test Configuration

### Unit Tests (in-crate)

Unit tests are built with `cargo test`. The `#[cfg(test)]` attribute ensures test code is only compiled during testing:

```toml
# No special Cargo.toml configuration needed for unit tests.
# Tests annotated with #[cfg(test)] are discovered automatically.
```

### Integration Tests

Integration tests live in a top-level `tests/` directory:

```
project/
├── src/
├── tests/
│   ├── integration_test_a.rs
│   └── integration_test_b.rs
└── Cargo.toml
```

Each file in `tests/` is compiled as a separate binary. They import the crate as an external dependency:

```rust
use project_name::module_name::SomeType;
```

### Test Dependencies

Dependencies used only during testing:

```toml
[dev-dependencies]
test_helper = "1.0"
mock_crate = "0.3"
```

---

## Profile Configuration

```toml
[profile.dev]
opt-level = 0          # No optimization during development
debug = true

[profile.release]
opt-level = "s"        # Optimize for size (common for constrained targets)
lto = true             # Link-time optimization
debug = false
panic = "abort"        # No unwinding (saves code size)
```

---

## Feature Flags

Define optional features for conditional compilation:

```toml
[features]
default = ["feature_a"]
feature_a = []
feature_b = ["dep:optional_crate"]
full = ["feature_a", "feature_b"]

[dependencies]
optional_crate = { version = "1.0", optional = true }
```

Use in code with `#[cfg(feature = "feature_name")]`.

---

## Build Scripts

When a module requires build-time code generation or configuration:

```toml
[package]
# ...
build = "build.rs"
```

The `build.rs` file runs at compile time and can emit:

```rust
fn main() {
    // Tell Cargo to re-run if a file changes
    println!("cargo:rerun-if-changed=some_file");

    // Pass configuration to the compiler
    println!("cargo:rustc-cfg=some_config");

    // Add a linker search path
    println!("cargo:rustc-link-search=native=/path/to/libs");

    // Link a native library
    println!("cargo:rustc-link-lib=static=library_name");
}
```

---

## Dependency Graph Verification

The `depends_on` list from each module spec maps directly to `[dependencies]` entries — either as path dependencies (internal modules) or crate dependencies (external packages). The build system enforces that:

- Every `depends_on` entry resolves to a declared dependency
- No circular dependencies exist (Cargo rejects these at build time)
- External packages listed in `external_deps` have corresponding entries in `[dependencies]`

---

## Additive Dependency Management

When a project already has a Cargo.toml (e.g., from a template like `esp-rs/esp-idf-template` or manual setup), spec-gen updates it additively rather than regenerating it.

### Rules

- Only add dependencies that are missing. Never modify existing entries.
- Use the exact `build_integration.cargo` string from architecture.yaml's `external_packages`.
- Append new entries at the end of the `[dependencies]` section.
- If a dependency exists but with a different version than architecture.yaml specifies, do NOT change it — the existing Cargo.toml is authoritative for versions. Report the discrepancy in the generation report.
- For `sdkconfig.defaults`: append missing `CONFIG_*` entries at the end of the file. Never modify or remove existing entries.

### Files never overwritten

These files are platform-specific boilerplate typically provided by a project template. spec-gen must never overwrite or regenerate them if they already exist:

- `build.rs`
- `rust-toolchain.toml`
- `.cargo/config.toml`
- `.gitignore`

### Dependency resolution

To check if a dependency is already satisfied:

1. Parse `[dependencies]` from Cargo.toml
2. Match by crate name (the TOML key, e.g., `esp-idf-svc`, `log`, `heapless`)
3. If present under any version or source, consider it satisfied
4. If absent, add it using `build_integration.cargo` from architecture.yaml
