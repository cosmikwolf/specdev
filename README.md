# specdev

A Claude Code plugin for spec-driven development. Describe your project as TOML specs, validate them against a machine-checkable schema, then generate Rust code — or reverse-engineer specs from code you already have.

The plugin models a project as a **profile** (project shape) composed with **blocks** (concrete frameworks, libraries, hardware families) and **bridges** (pairwise wirings between blocks). All three ship in the plugin and are inlined into the project's own `specs/specdev/` tree by deterministic CLI operations; once inlined, the project owns its content.

Pairs with the [`specdev-analyzer`](https://github.com/cosmikwolf/specdev-analyzer) CLI, which does validation, extraction, audit, inline, bootstrap, prompt assembly, and Cargo.toml emission. The plugin provides the interactive authoring workflow; the analyzer provides the mechanical operations.

## Install

1. Install the analyzer (requires a stable Rust toolchain):

   ```
   cargo install --git https://github.com/cosmikwolf/specdev-analyzer
   ```

2. Add the marketplace and install the plugin in Claude Code:

   ```
   /plugin marketplace add cosmikwolf/specdev
   /plugin install specdev@specdev
   ```

   For local development against a checkout:

   ```
   claude --plugin-dir /path/to/specdev
   ```

3. Verify the install: type `/` in Claude Code and confirm `/spec-init`, `/spec-design`, `/spec-validate`, and `/spec-gen` appear in the slash-command menu. If they do not, re-run the marketplace add or check `/plugin` for installation errors.

## Quickstart

From an empty directory:

```
/spec-init                    # interview → analyzer bootstrap → specs/specdev/ + spec.toml
                              # then inline the blocks and bridges your project needs:
specdev-analyzer inline stm32h5-rs
specdev-analyzer inline embassy
specdev-analyzer inline-bridge embassy+stm32h5-rs

/spec-design "status LED with solid/blinking/breathing"
/spec-validate                # check schema + graph invariants
/spec-gen StatusLed           # assemble prompt → Claude emits source → analyzer emits Cargo.toml
```

From an existing codebase (single-module mode is the supported v1 path; `--all` is gated on Phase 10):

```
/spec-init                    # bootstrap a profile first
/spec-reverse StatusLed       # walk source with Claude, author one module spec
/spec-validate
```

## Commands

| Command            | Purpose                                                                            |
| ------------------ | ---------------------------------------------------------------------------------- |
| `/spec-init`       | Interview, then `specdev-analyzer bootstrap` to scaffold `spec.toml` + `specs/specdev/`. |
| `/spec-design`     | Author one module TOML interactively against the project's aggregated schema.      |
| `/spec-edit`       | Modify an existing module with downstream impact analysis.                         |
| `/spec-validate`   | Run `specdev-analyzer validate` and translate findings into actionable fixes.      |
| `/spec-gen`        | Assemble a deterministic codegen prompt, emit Rust source, then emit Cargo.toml.   |
| `/spec-reverse`    | Reverse-engineer a single module spec from existing source. (`--all` Phase 10-pending.) |
| `/fetch-datasheet` | Download, parse, and summarize a hardware datasheet PDF.                           |

Block and bridge inlining is driven directly via `specdev-analyzer inline <block>` / `inline-bridge <a>+<b>`. A dedicated `/spec-block` skill is planned but not yet shipped.

## Project layout

After `specdev-analyzer bootstrap` and a few inlines, a single-package project looks like:

```
my-project/
├── spec.toml                            # [project] name, profile, language, [paths]
├── src/                                 # source code
└── specs/
    ├── architecture.toml                # cross-module wiring (profile-shaped)
    ├── requirements.toml                # requirements referenced by modules
    ├── modules/*.toml                   # one module per file
    └── specdev/                         # machinery: schemas, reductions, codegen, lineage
        ├── schemas/
        │   ├── module.schema.yaml       # aggregator: $refs to inlined/
        │   ├── architecture.schema.yaml
        │   ├── requirements.schema.yaml
        │   └── inlined/
        │       ├── blocks/<name>.schema.yaml
        │       └── bridges/<name>.schema.yaml
        ├── reductions/
        │   ├── project.yaml
        │   ├── blocks/<name>.yaml
        │   └── bridges/<name>.yaml      # canonical; project-local under bridges/local/
        ├── codegen/<lang>/
        │   ├── module.conventions.md    # profile conventions
        │   ├── blocks/<name>/README.md
        │   └── bridges/<name>/README.md
        ├── upstream/                    # bytes-as-of-inlining; the merge base for --merge
        ├── domain.toml                  # user-edited graph-check rules
        └── lineage.toml                 # provenance + content hash for every inlined unit
```

`specs/specdev/` is committed to git. It is part of the project, not a hidden cache. Once the project is bootstrapped, no runtime path reaches into the plugin — the analyzer operates entirely from `specs/specdev/`.

## Profiles, blocks, and bridges

What ships in the plugin:

- **Profiles** — project-shape skeletons. One-shot scaffolding at bootstrap; no ongoing dependency.
  - `firmware-layered` — embedded Rust on Cortex-M, layered architecture (PAC → HAL → driver → middleware → application), ports + connections wiring, hardware ownership, Cargo.toml-emitting platform/runtime/logger sections.
  - `library-api` — pure Rust library, visibility-driven module boundaries, re-exports, Cargo features.
- **Blocks** — concrete frameworks, libraries, or hardware families. Each ships a schema fragment, reduction rules, per-language codegen prose, and README.
  - `embassy` (Rust async runtime), `stm32h5-rs` (STM32H5 HAL).
- **Bridges** — pairwise wirings between two named blocks.
  - `embassy+stm32h5-rs`.

Adding a block or bridge: drop it under `blocks/<name>/` or `bridges/<a>+<b>/` in the plugin (or any git repo / local path) and pull it in via `specdev-analyzer inline <block> --from <url|path>`. Plugin CI lints the layout via `specdev-analyzer plugin-lint`. See `15-blocks-and-bridges.md` (the authoritative composition design) in the planning workspace for the full model.

User-contributed blocks and bridges are first-class: `--from <url|path>` works against any git repo or local checkout, lineage records the origin, and `specdev drift` / `inline --merge` handle updates with 3-way merge against `specs/specdev/upstream/`.

## Scope and status

- **v1 is single-package.** Multi-package (`[[packages]]` workspace) machinery lands in Phase 8.5 (library) and Phase 12 (firmware). Skills currently refuse multi-package projects with a clear message.
- **Rust only.** A C-language stack lands in Phase 9.
- **`/spec-reverse --all`** is Phase 10-pending. Single-module reverse is the supported workflow today; it is a person-in-the-loop authoring flow against the inlined-block schemas, not an automated extractor.

## License

MIT.
