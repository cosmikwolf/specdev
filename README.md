# specdev

A Claude Code plugin for spec-driven development. Describe your firmware or GUI app as TOML specs, validate them against a machine-checkable schema, then generate Rust / C / C++ code — or reverse-engineer specs from code you already have.

Pairs with the [`specdev-analyzer`](https://github.com/tenkai/specdev-analyzer) CLI, which does the actual validation, extraction, and audit. The plugin provides the interactive authoring workflow; the analyzer provides the checks.

## Install

1. Install the analyzer:

   ```
   cargo install --git https://github.com/tenkai/specdev-analyzer
   ```

2. Install the plugin:

   ```
   /plugin install specdev@tenkai
   ```

   Or point Claude Code at a local checkout for development:

   ```
   claude --plugin-dir /path/to/specdev
   ```

## Quickstart

From an empty directory:

```
/spec-init                    # scaffold spec.toml + specs/
/spec-design "driver for an SSD1306 OLED over I2C"
/spec-design "status LED with solid/blinking/breathing"
/spec-validate                # check schema + graph invariants
/spec-gen --all               # write source code for every module
```

From an existing codebase:

```
/spec-reverse --all           # extract specs from src/
/spec-validate
/spec-edit StatusLed          # refine anything with LOW-confidence fields
```

## Commands

| Command          | Purpose                                                            |
| ---------------- | ------------------------------------------------------------------ |
| `/spec-init`     | Scaffold `spec.toml` and the `specs/` directory tree.              |
| `/spec-design`   | Author one module TOML interactively.                              |
| `/spec-edit`     | Modify an existing module with downstream impact analysis.         |
| `/spec-design-ui`| Design UI specs (displays, inputs, indicators, screens, nav).      |
| `/spec-validate` | Run the analyzer and translate findings into actionable fixes.     |
| `/spec-gen`      | Generate source code from validated specs.                         |
| `/spec-gen-ui`   | Generate UI rendering code, or emit Mermaid interaction diagrams.  |
| `/spec-reverse`  | Reverse-engineer specs from existing Rust / C / C++ source.        |
| `/fetch-datasheet` | Download, parse, and summarize a hardware datasheet PDF.         |

Every skill reads project configuration from `spec.toml` at invocation time, so the same commands work across firmware and GUI (egui) projects without changes.

## Project layout

Single-crate project:

```
my-project/
├── spec.toml                  # domain, language, spec_version, paths
├── src/                       # source code
└── specs/
    ├── architecture.toml      # cross-module wiring
    ├── modules/*.toml         # one module per file
    ├── requirements/*.toml    # requirements referenced by modules
    └── ui/*.toml              # optional UI specs
```

Multi-crate Cargo workspace (or CMake multi-subproject):

```
my-workspace/
├── spec.toml                  # includes [[packages]] name + source_path
├── crates/core/     src/…
├── crates/drivers/  src/…
└── specs/
    ├── architecture.toml      # endpoints may be <pkg>::<Module>.<port>
    ├── modules/
    │   ├── core/Clock.toml
    │   └── drivers/Led.toml   # depends_on = ["core::Clock"]
    └── requirements/*.toml    # requirements are not packaged
```

Package membership is derived from the filesystem path — never written inside a module TOML. Cross-package references use `<pkg>::<Module>` syntax; unqualified names resolve within the same package first.

## Customization

Every schema, domain guide, language guide, and codegen guide can be overridden per-project by dropping a copy under `.specdev/`:

```
my-project/
└── .specdev/
    ├── schema/firmware/v1/module.schema.yaml   # project-specific schema extension
    ├── domains/firmware/author.md              # house style for module TOMLs
    ├── languages/rust/syntax.md                # team formatting conventions
    └── codegen/firmware-rust/gen-module.md     # customized codegen rules
```

Resolution order: explicit path in `spec.toml` `[paths.overrides]` → `.specdev/<kind>/<name>/` → plugin defaults. Skills embed the analyzer's `resolve` output as their context header, so every skill automatically picks up overrides.

## Examples

- [`examples/blinky-a2dp/`](examples/blinky-a2dp/) — ESP32 Bluetooth A2DP audio sink with a DotStar status LED. Four modules across `driver` and `middleware` layers, a requirements file, and a wired architecture.
- [`examples/counter-egui/`](examples/counter-egui/) — minimal egui desktop GUI: one view, one persisted store, two buttons. Shows the same workflow on a non-firmware domain.

Each example validates and extracts cleanly. Run the analyzer against either directory to see the output.

## Domains and languages

Shipped out of the box:

- **Domains:** `firmware`, `egui`
- **Languages:** `rust`, `c`, `cpp`

Add a new domain by dropping a `schema/<name>/v1/` tree, a `domains/<name>/` directory (with `domain.toml`, `author.md`, `validate.md`), and codegen guides under `codegen/<name>-<language>/`. The analyzer discovers them automatically.

Add a new language by writing a tree-sitter adapter (grammar + `.scm` queries + a small Rust adapter impl in the analyzer crate) plus a `languages/<name>/` guide.

## License

MIT.
