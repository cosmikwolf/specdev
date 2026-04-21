# specdev

A Claude Code plugin for spec-driven firmware development. Author architecture and module specs in TOML, validate them against a machine-checkable JSON Schema, then generate or audit Rust / C / C++ firmware from those specs.

Successor to [`spec-driven-dev`](../spec-driven-dev/). Pairs with the [`specdev-analyzer`](../specdev-analyzer/) Rust CLI.

## Status

Phase 1 — scaffolding. Not yet usable. See [`../specdev-next/00-status.md`](../specdev-next/00-status.md) for implementation progress.

## What this plugin gives you

- Skills (`/spec-init`, `/spec-design`, `/spec-validate`, `/spec-gen`, and more) that drive the spec-authoring and codegen workflow.
- Agents under `agents/` — purpose-built subagents with their own model routing (e.g. `module-reverser` on Sonnet) that skills fan out for parallel, bounded work. Used today by `/spec-reverse --all` to extract modules concurrently.
- YAML-authored JSON Schema for the firmware domain under `schema/firmware/v1/`.
- Domain guidance (`domains/firmware/`) and language guidance (`languages/{rust,c,cpp}/`) used by the skills.
- Codegen rules per domain-language pair under `codegen/`.
- A `scripts/resolve-context` helper that reads `spec.toml`, resolves project-local overrides from `.specdev/`, and returns the effective paths each skill should use.

## How specs are laid out

Single-crate project:

```
my-firmware/
├── spec.toml                  # project config (domain, language, paths, spec_version)
├── specs/
│   ├── architecture.toml      # cross-module connections
│   ├── modules/*.toml         # one module per file
│   ├── requirements/*.toml    # requirement files, referenced by id
│   └── ui/*.toml              # optional UI specs
└── .specdev/                  # optional: project-local overrides
    ├── schema/…
    ├── domains/…
    ├── languages/…
    └── codegen/…
```

Multi-crate / multi-subproject workspace (D24 — declare `[[packages]]` in `spec.toml`):

```
my-workspace/
├── spec.toml                          # includes [[packages]] name + source_path per crate
├── crates/core/     src/…
├── crates/drivers/  src/…
└── specs/
    ├── architecture.toml              # endpoints may be <pkg>::<Module>.<port>
    ├── modules/
    │   ├── core/Clock.toml
    │   └── drivers/Led.toml           # depends_on = ["core::Clock"]
    └── requirements/*.toml            # requirements are NOT packaged
```

The `package` is derived from the filesystem path — never written inside module TOMLs. Cross-package references use `<pkg>::<Module>` syntax; unqualified names resolve within the same package first.

## Install

Once Phase 11 ships, this plugin will be installable through the Claude Code plugin mechanism. The analyzer binary installs separately via:

```
cargo install --git https://github.com/tenkai/specdev-analyzer
```

## License

MIT.
