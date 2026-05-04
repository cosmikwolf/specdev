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

3. Verify the install: type `/` in Claude Code and confirm the specdev commands appear in the slash-command menu. The shipped surface is `/spec-init`, `/spec-bootstrap`, `/spec-design`, `/spec-edit`, `/spec-validate`, `/spec-gen`, `/spec-reverse`, `/spec-inline`, `/spec-inline-bridge`, `/spec-drift`, `/spec-sync-items`, `/fetch-datasheet`. If they do not appear, re-run the marketplace add or check `/plugin` for installation errors.

## Quickstart

From an empty directory:

```
/spec-init                    # interview → analyzer bootstrap → specs/specdev/ + spec.toml
                              # then inline the blocks and bridges your project needs:
/spec-inline stm32h5-rs
/spec-inline embassy
/spec-inline-bridge embassy+stm32h5-rs

/spec-design "status LED with solid/blinking/breathing"
/spec-validate                # check schema + graph invariants
/spec-gen StatusLed           # assemble prompt → Claude emits source → analyzer emits Cargo.toml
```

From an existing codebase:

```
/spec-init                    # bootstrap a profile first
/spec-reverse StatusLed       # walk source with Claude, author one module spec
/spec-validate
```

## Commands

| Command                | Purpose                                                                            |
| ---------------------- | ---------------------------------------------------------------------------------- |
| `/spec-init`           | Interview, then `specdev-analyzer bootstrap` to scaffold `spec.toml` + `specs/specdev/`. |
| `/spec-bootstrap`      | Non-interactive wrapper over `bootstrap` for callers passing explicit `--profile`/`--language`/`--name`. |
| `/spec-design`         | Author one module TOML interactively against the project's aggregated schema.      |
| `/spec-edit`           | Modify an existing module with downstream impact analysis.                         |
| `/spec-validate`       | Run `specdev-analyzer validate` and translate findings into actionable fixes.      |
| `/spec-gen`            | Assemble a deterministic codegen prompt, emit Rust source, then emit Cargo.toml. Supports `--strict` and `--skeleton` (cargo-check-gated handoff scaffold). |
| `/spec-reverse`        | Reverse-engineer a single module spec from existing source. (`--all` Phase 10-pending.) |
| `/spec-inline`         | Inline a canonical block from `${CLAUDE_PLUGIN_ROOT}/blocks/<name>/` into `specs/specdev/inlined/blocks/<name>/`. |
| `/spec-inline-bridge`  | Inline a canonical bridge between two already-inlined blocks. Bridge name is `<a>+<b>` with `a` and `b` lex-ordered. |
| `/spec-drift`          | Read-only report on which inlined files diverge from the plugin tree or have been hand-edited. |
| `/spec-sync-items`     | Three-stage sync (deterministic diff → AI-advisory rationale → human approval → spec mutation) reconciling a module's `[[items]]` with what tree-sitter finds in source. |
| `/fetch-datasheet`     | Download, parse, and summarize a hardware datasheet PDF.                           |

`/spec-inline` and `/spec-inline-bridge` are the user-facing block / bridge inlining surface; both are thin wrappers over `specdev-analyzer inline` / `inline-bridge`.

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

The set is **dynamic** — every directory under `${CLAUDE_PLUGIN_ROOT}/profiles/`, `/blocks/`, or `/bridges/` is a valid choice, including ones authored by end users. To list the current set in your install:

```sh
ls "${CLAUDE_PLUGIN_ROOT}/profiles/"
ls "${CLAUDE_PLUGIN_ROOT}/blocks/"
ls "${CLAUDE_PLUGIN_ROOT}/bridges/"
```

Each entry has its own `README.md` describing what it contributes; read those for authoritative details rather than relying on enumerations in this README.

What this repo currently ships at HEAD (subject to change as new profiles / blocks / bridges land):

- **Profiles** — project-shape skeletons. One-shot scaffolding at bootstrap; no ongoing dependency.
  - `firmware-layered` — embedded Rust on Cortex-M, layered architecture (PAC → HAL → driver → middleware → application), ports + connections wiring, hardware ownership, Cargo.toml-emitting platform/runtime/logger sections.
  - `library-api` — pure Rust library, visibility-driven module boundaries, re-exports, Cargo features.
  - `audio-plugin` — VST3/CLAP/standalone audio plugins, four-layer (host_interface → plugin_shell → engine → dsp_node), `[plugin]`/`[audio]`/`[runtime]` architecture sections.
  - `rust-workspace` — workspace root profile that walks per-member crates each carrying their own profile (typically `library-api` or `audio-plugin` or `firmware-layered`).
- **Blocks** — concrete frameworks, libraries, or hardware families. Each ships a schema fragment, reduction rules, per-language codegen prose, and README.
  - `embassy` (Rust async runtime), `stm32h5-rs` (STM32H5 HAL), `nih-plug` (audio plugin framework), `egui` (immediate-mode UI, topology-only).
- **Bridges** — pairwise wirings between two named blocks (`<a>+<b>`, lex-ordered).
  - `embassy+stm32h5-rs`, `egui+nih-plug`.

### Authoring your own profile / block / bridge

Drop it under `blocks/<name>/` or `bridges/<a>+<b>/` or `profiles/<name>/` in the plugin tree (or any git repo / local path), then pull it into a project via `specdev-analyzer inline <block> --from <url|path>` or its `/spec-inline` wrapper. Plugin CI lints the layout via `specdev-analyzer plugin-lint`. See `15-blocks-and-bridges.md` and `18-authoring-recipes.md` (the authoritative composition design + mechanical recipe doc) in the planning workspace for the full model.

User-contributed blocks and bridges are first-class: `--from <url|path>` works against any git repo or local checkout, lineage records the origin, and `/spec-drift` / `/spec-inline --merge` handle updates with 3-way merge against `specs/specdev/upstream/`.

## Scope and status

- **v1 is single-package** for `firmware-layered` and `library-api` profiles. The `rust-workspace` profile coordinates multi-member projects; per-member skills (`/spec-design`, `/spec-edit`, `/spec-gen`) run from the member's directory.
- **Rust only.** A C-language stack lands in Phase 9.
- **`/spec-reverse --all`** is Phase 10-pending. Single-module reverse is the supported workflow today; it is a person-in-the-loop authoring flow against the inlined-block schemas, not an automated extractor.

## License

MIT.
