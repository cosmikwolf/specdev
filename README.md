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

3. Verify the install: type `/` in Claude Code and confirm the specdev commands appear in the slash-command menu. The shipped surface is `/spec-init`, `/spec-design`, `/spec-edit`, `/spec-decide`, `/spec-question`, `/spec-validate`, `/spec-gen`, `/spec-reverse`, `/spec-inline`, `/spec-inline-bridge`, `/spec-drift`, `/spec-sync-items`. If they do not appear, re-run the marketplace add or check `/plugin` for installation errors.

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
| `/spec-init`           | Interview, then `specdev-analyzer bootstrap` to scaffold `spec.toml` + `specs/specdev/`. Pass `--non-interactive --profile <name> --language <lang> --name <project>` to skip the interview. |
| `/spec-design`         | Author one module TOML interactively against the project's aggregated schema.      |
| `/spec-edit`           | Modify an existing module with downstream impact analysis.                         |
| `/spec-decide`         | Record a decision in `decisions.toml`, or supersede an existing one (the chain is kept). |
| `/spec-question`       | Raise an open question in `open_questions.toml`, downgrade it to a committed default, or close it. |
| `/spec-validate`       | Run `specdev-analyzer validate` and translate findings into actionable fixes.      |
| `/spec-gen`            | Assemble a deterministic codegen prompt, emit Rust source, then emit Cargo.toml. Refuses on incomplete specs (no stub bodies). |
| `/spec-reverse`        | One-shot migration: reverse-engineer module specs from an existing codebase. Single-module or `--all`. |
| `/spec-inline`         | Inline a canonical block from `${CLAUDE_PLUGIN_ROOT}/blocks/<name>/` into `specs/specdev/inlined/blocks/<name>/`. |
| `/spec-inline-bridge`  | Inline a canonical bridge between two already-inlined blocks. Bridge name is `<a>+<b>` with `a` and `b` lex-ordered. |
| `/spec-drift`          | Read-only report on which inlined files diverge from the plugin tree or have been hand-edited. |
| `/spec-sync-items`     | Three-stage sync (deterministic diff → AI-advisory rationale → human approval → spec mutation) reconciling a module's `[[items]]` with what tree-sitter finds in source. |

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
    ├── decisions.toml                   # accepted/superseded decisions with rationale
    ├── open_questions.toml              # known unknowns; open questions gate codegen
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

## Decisions and open questions

Specs carry rationale, not just structure. Alongside `requirements.toml`, every project carries two sibling artifacts — together the three form the spec triad:

- **`decisions.toml`** records the how-and-why: each decision is `accepted` or `superseded`, and superseding a decision links it into a chain so the history of *why* survives.
- **`open_questions.toml`** records known unknowns: each question is `open`, `default_committed` (a provisional default committed to unblock work), or `closed`.

Both feed code generation directly:

- **The codegen gate.** An `open` question blocks code generation for every module it affects — resolve it (`closed`), or commit a provisional default (`default_committed`) to unblock, in which case the placeholder and a verify-this note are carried into the generated code's prompt. `/spec-validate` warns about open questions; `/spec-gen` refuses.
- **Rationale in prompts.** Decision rationale is inlined into the assembled codegen prompt, so recorded decisions shape the generated code itself, not just the spec.

Two commands manage the triad: `/spec-decide` records or supersedes a decision; `/spec-question` raises an open question, downgrades it to a committed default, or closes it.

See your profile's README for the full triad model in profile-specific terms.

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

Drop it under `blocks/<name>/` or `bridges/<a>+<b>/` or `profiles/<name>/` in the plugin tree (or any git repo / local path), then pull it into a project via `specdev-analyzer inline <block> --from <url|path>` or its `/spec-inline` wrapper. Plugin CI lints the layout via `specdev-analyzer plugin-lint`.

User-contributed blocks and bridges are first-class: `--from <url|path>` works against any git repo or local checkout, lineage records the origin, and `/spec-drift` / `/spec-inline --merge` handle updates with 3-way merge against `specs/specdev/upstream/`.

## Scope

- **v1 is single-package** for `firmware-layered`, `library-api`, and `audio-plugin` profiles. The `rust-workspace` profile coordinates multi-member projects; per-member skills (`/spec-design`, `/spec-edit`, `/spec-gen`) run from the member's directory.
- **Rust only.** C-language support is on the roadmap.
- **`/spec-reverse`** is a one-shot migration tool — moves an existing codebase into the spec-driven world. It is a person-in-the-loop authoring flow against the inlined-block schemas, not a continuous source-vs-spec audit. For ongoing source-vs-spec drift on a module's `[[items]]`, use `/spec-sync-items`.

## License

MIT.
