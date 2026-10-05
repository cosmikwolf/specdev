# Rust Build System Patterns

Pure build system reference for Rust projects using Cargo. These patterns apply to any domain.

---

## Toolchain Commands

| Purpose | Command |
|---|---|
| Type-check (the usual compile gate) | `cargo check --quiet` |
| Type-check for a cross target | `cargo check --quiet --target <triple>` |
| Run tests | `cargo test` |
| Lint | `cargo clippy --all-targets` |
| Format | `cargo fmt` |
| Build release | `cargo build --release` |
| Fail when `Cargo.lock` would change (CI) | add `--locked` to any of the above |

- `cargo check` exiting 0 is the minimum bar: a type-clean tree. It does not run the code; a profile that needs more
  adds a second gate.
- **None of these commands is "the compile gate".** The gate is the profile's `gates:` registry in
  `specs/specdev/profile.yaml` — a `label` and `steps` as argv lists — rendered into `scripts/specdev-gates.sh`, which
  CI runs after `specdev-analyzer check`. This table is the menu a profile author picks steps from.

---

## Cargo.toml

`Cargo.toml` is the project's, and the only source of truth for dependencies, features and build profiles. The spec
copies none of it.

- A single crate holds every module of a package; modules are files under `src/`, not crates.
- A workspace lists its member crates under `[workspace] members`; each member has its own `Cargo.toml` and, in
  specdev, its own spec root (see the `rust-workspace` profile).
- Dependency shapes:

```toml
[dependencies]
simple_dep = "1.0"
featured_dep = { version = "2.3", features = ["async", "derive"] }
git_dep = { git = "https://github.com/org/repo", rev = "abc123" }
local_dep = { path = "../local_dep" }
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

When a crate needs a build-time step (a generated table, a linker script, a native library):

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

`Cargo.toml` is the only source of truth for dependencies. The spec's `depends_on` states which project modules a
module may use; `specdev-analyzer check` compares it with the module's actual imports (`internal_edges_declared`), and
Cargo itself refuses an external crate the manifest does not declare. Cargo also rejects circular crate dependencies.

### Files a project template owns

`build.rs`, `rust-toolchain.toml`, `.cargo/config.toml` and `.gitignore` are platform boilerplate the project or its
template owns. A firmware project's `.cargo/config.toml` names the build target that `[platform].target_triple`
repeats for the gate step; keep the two equal.
