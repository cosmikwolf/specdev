# specdev

A Claude Code plugin for spec-driven development. Describe your project's intent as TOML specs — what each module is for, what it may depend on, which requirements it satisfies, which decisions guide it — validate them against a machine-checkable schema, and check them against the code you and the agent write, in Rust, Dart, TypeScript, C or C++. specdev does not generate code; it keeps the code honest to the intent. Specs for code you already have can be reverse-engineered.

The plugin models a project as a **profile** (project shape) composed with **blocks** (concrete frameworks, libraries, hardware families) and **bridges** (pairwise wirings between blocks). All three ship in the plugin and are inlined into the project's own `specs/specdev/` tree by deterministic CLI operations; once inlined, the project owns its content.

Pairs with the `specdev-analyzer` CLI — a private repository alongside this one, in the same organization — which does validation, the checks against the code (`check`), extraction, inline, bootstrap, migration, and rendering the compile gates into the script CI runs. The plugin provides the interactive authoring workflow; the analyzer provides the mechanical operations.

## Install

**Both repositories are private, and they are one unit.** `specdev` (this
plugin) and `specdev-analyzer` (the CLI) ship together and must be installed
as a matched pair — see "Keeping it current" below for why.

Replace `<org>` in the commands below with the GitHub organization you were
granted access to (the upstream is `cosmikwolf`). If `git` cannot read the URL, you do not have
access yet — ask for it rather than working around it, because the two
repositories must come from the *same* org or the analyzer will validate your
project against schemas the plugin does not ship.

1. Install the analyzer (requires a stable Rust toolchain, at least 1.86):

   ```
   cargo install --git https://github.com/<org>/specdev-analyzer --tag v0.3.3
   ```

   Already have a checkout? `cargo install --path <analyzer-checkout>`
   needs no network and no org access, and is the supported path for
   anyone developing against the pair.

   **Keeping it current.** The analyzer embeds the JSON Schemas it
   validates the plugin's files against, so the two are one unit: pull the
   plugin and you must re-install the analyzer. Re-run the same command with
   `--force` (or `cargo install --path <analyzer-checkout> --force`) — an
   install at an unchanged version still replaces the binary.

   A stale analyzer announces itself two ways:

   - `[MANIFEST-002] … declares contract_version N; this specdev-analyzer
     understands M` — the plugin is ahead. Re-install.
   - `[MANIFEST-001]` naming a file **under the plugin tree** — an analyzer
     older than the `contract_version` handshake. Re-install.

   Neither means anything is wrong with your project.

   **Do not use the version numbers to answer this.** The two repositories
   version independently (the plugin moves faster than the analyzer), so the
   numbers are not meant to match — and neither number says anything about
   the binary you actually have installed, which can be any older release.
   `specdev-analyzer --version` reports what is on your PATH, not what the
   plugin needs, and nothing compares them for you.
   `contract_version` is the real handshake, and it is the one that fires.

2. Add the marketplace and install the plugin in Claude Code:

   ```
   /plugin marketplace add <org>/specdev
   /plugin install specdev@specdev
   ```

   For local development against a checkout:

   ```
   claude --plugin-dir /path/to/specdev
   ```

3. Verify the install: type `/` in Claude Code and confirm the specdev commands appear in the slash-command menu. **Sixteen** commands ship; the Commands table below lists them all. Rather than counting them by eye, compare the menu against the install: `ls "${CLAUDE_PLUGIN_ROOT}/skills/"`. If none appear, re-run the marketplace add or check `/plugin` for installation errors.

## Quickstart

From an empty directory:

```
/spec-init                    # interview → analyzer bootstrap → specs/specdev/ + spec.toml
                              # then inline the blocks and bridges your project needs:
/spec-inline stm32h5-rs
/spec-inline embassy
/spec-inline-bridge embassy+stm32h5-rs

/spec-design "status LED with solid/blinking/breathing"   # writes planned = true
# write the code: read `specdev-analyzer context --module StatusLed` first
/spec-validate                # specdev-analyzer check: the spec, and the spec against the code
```

From an existing **Rust** codebase (reverse engineering is Rust-only today):

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
| `/spec-decide`         | Record a decision under `specs/decisions/`, or supersede an existing one (the chain is kept). |
| `/spec-question`       | Raise an open question under `specs/open_questions/`, downgrade it to a committed default, or close it. |
| `/spec-validate`       | Run `specdev-analyzer check` (validate, then the spec against the code) and translate findings into actionable fixes. |
| `/spec-reverse`        | One-shot migration: reverse-engineer module specs from an existing **Rust** codebase. Single-module or `--all`. |
| `/spec-inline`         | Inline a canonical block from `${CLAUDE_PLUGIN_ROOT}/blocks/<name>/` into the project's `specs/specdev/` tree, split across schema, reduction, implementation guidance, an optional rule pack and an upstream snapshot. |
| `/spec-inline-bridge`  | Inline a canonical bridge between two already-inlined blocks. Bridge name is `<a>+<b>` with `a` and `b` lex-ordered. |
| `/spec-drift`          | Read-only report on which inlined files diverge from the plugin tree or have been edited locally. |
| `/spec-sync-items`     | Three-stage sync (deterministic diff → AI-advisory rationale → human approval → spec mutation) reconciling a module's `[[items]]` with what tree-sitter finds in source. |
| `/spec-config`         | Write the two files the project owns from then on — `.claude/rules/specdev.md` (the project rules, so they survive with hooks disabled) and `specdev-feedback.md` (an append-only log for the specdev developer). Each only when absent; never overwrites, never merges. `/spec-init` runs it for you. |
| `/spec-add-member`     | Adopt a validated single-crate spec into `crates/<member>/` under a `rust-workspace` root, preserving requirement / decision / open-question ids. For a greenfield workspace use `/spec-init --profile rust-workspace` instead. |
| `/spec-create-block`   | **Plugin authoring.** Scaffold a NEW block unit (schema + reduction + implementation guidance + README) for a framework, library or hardware family. To inline an existing block, use `/spec-inline`. |
| `/spec-create-bridge`  | **Plugin authoring.** Scaffold a NEW canonical bridge between two blocks that already exist in the plugin tree. To inline an existing bridge, use `/spec-inline-bridge`. |
| `/spec-create-profile` | **Plugin authoring.** Scaffold a NEW profile (manifest + schemas + reduction + templates + implementation guidance + CI workflow) for a project shape not already covered. To bootstrap an existing profile, use `/spec-init`. |

`/spec-inline` and `/spec-inline-bridge` are the user-facing block / bridge inlining surface; both are thin wrappers over `specdev-analyzer inline` / `inline-bridge`.

**To see how the modules fit together**, run `specdev-analyzer report graph` (Mermaid) or `report graph --format dot`. It draws every module, grouped by the layers your profile's `domain.toml` declares, with each dependency that points the wrong way through the layers in thick red — the same edges `/spec-validate` reports. The output is deterministic: commit it (`report graph -o docs/module-map.mmd`) and a pull request shows every new edge.

The last three are **plugin-authoring** commands: they extend the plugin's own `blocks/`, `bridges/` and `profiles/` trees. A project consuming specdev never needs them. To check this table against what is installed, `ls "${CLAUDE_PLUGIN_ROOT}/skills/"` — it must list the same names.

## Project layout

After `specdev-analyzer bootstrap` and a few inlines, a single-package project looks like:

```
my-project/
├── spec.toml                            # [project] name, profile, language, [paths]
├── src/                                 # source code, written by you and the agent
├── scripts/specdev-gates.sh             # check + compile gates; CI runs it
├── .github/workflows/specdev.yml        # written once at bootstrap; yours afterwards
└── specs/
    ├── architecture.toml                # cross-module wiring (profile-shaped)
    ├── requirements/<id>.toml           # requirements referenced by modules
    ├── decisions/<id>.toml              # accepted/superseded decisions with rationale
    ├── open_questions/<id>.toml         # known unknowns; an open one blocks built code
    ├── modules/*.toml                   # one module per file
    └── specdev/                         # machinery: schemas, reductions, guidance, lineage
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
        ├── guidance/                    # implementation guidance: rules for the code
        │   ├── profile.md
        │   ├── blocks/<name>.md
        │   └── bridges/<name>.md
        ├── checks/, queries/            # inlined rule packs: checks as data, tree-sitter queries
        ├── upstream/                    # bytes-as-of-inlining; the merge base for --merge
        ├── domain.toml                  # user-edited graph-check rules
        └── lineage.toml                 # provenance + content hash for every inlined unit
```

`specs/specdev/` is committed to git. It is part of the project, not a hidden cache. Once the project is bootstrapped, no runtime path reaches into the plugin — the analyzer operates entirely from `specs/specdev/`.

## Decisions and open questions

Specs carry rationale, not just structure. Alongside the requirements, every project carries two sibling ledgers, one file per entry:

- **Decisions** record the how-and-why: each decision is `accepted` or `superseded`, and superseding a decision links it into a chain so the history of *why* survives.
- **Open questions** record known unknowns: each question is `open`, `default_committed` (a provisional default committed to unblock work), or `closed`.

Both reach the code:

- **The open-question gate.** An `open` question that reaches a module with code fails `check` — resolve it (`closed`), or commit a provisional default (`default_committed`) with a line saying how it will be verified.
- **Decisions for whoever edits.** `specdev-analyzer report decisions --module <M>` lists the decisions that reach a module, so whoever edits its code follows them.

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

- **Profiles** — project-shape skeletons. Bootstrap copies their files into `specs/specdev/`; `drift` tracks them against the plugin from then on.
  - `firmware-layered` (rust) — embedded Rust on Cortex-M, layered architecture (PAC → HAL → driver → middleware → application), ports + connections wiring, hardware ownership.
  - `library-api` (rust) — pure Rust library, visibility-driven module boundaries, an optional `items[]` contract surface.
  - `rust-app` (rust) — standalone Rust executable: a testable core plus a thin entry point.
  - `audio-plugin` (rust) — VST3/CLAP/standalone audio plugins, four-layer (host_interface → plugin_shell → engine → dsp_node), parameter bindings.
  - `rust-workspace` (rust) — workspace root profile that walks per-member crates each carrying their own profile (typically `library-api` or `audio-plugin` or `firmware-layered`).
  - `flutter-app` (dart) — Flutter application, composition root plus screens and services, fenced by platform.
  - `nextjs-app` (typescript) — Next.js App Router application, nine module roles, the client/server boundary as two fields.
- **Blocks** — concrete frameworks, libraries, or hardware families. Each ships a schema fragment, reduction rules, implementation guidance, a README, and optionally a rule pack (checks as data).
  - Rust: `eframe`, `egui`, `egui-plot`, `elf-dwarf`, `embassy`, `iroh`, `itm`, `mk20dx`, `nih-plug`, `probe-rs`, `rtic`, `stm32h5-rs`.
  - Dart: `flutter-rust-bridge`, `universal-ble`.
  - TypeScript: `next-auth`, `prisma`.
- **Bridges** — pairwise wirings between two named blocks (`<a>+<b>`, lex-ordered).
  - `egui+egui-plot`, `egui+nih-plug`, `elf-dwarf+probe-rs`, `embassy+stm32h5-rs`, `itm+probe-rs`, `next-auth+prisma`.

### Authoring your own profile / block / bridge

Author it with `/spec-create-block`, `/spec-create-bridge` or `/spec-create-profile`, under `blocks/<name>/`, `bridges/<a>+<b>/` or `profiles/<name>/` in a plugin tree you can write to (this checkout, or a fork). A project then inlines it with `/spec-inline` / `/spec-inline-bridge` (or bootstraps it with `/spec-init`), with `--plugin-root` pointing at that tree when it is not the installed plugin. The layout is linted by `specdev-analyzer plugin-lint --plugin <tree>`, and gated in the **analyzer's** CI (`plugin_lint::live_plugin_lint_passes`, which checks this tree out as a sibling) rather than in this repository's.

User-contributed blocks and bridges are first-class: lineage records the origin, and `/spec-drift` / `/spec-inline --merge` handle updates with 3-way merge against `specs/specdev/upstream/`.

## Scope

- **v1 is single-package** for every profile except `rust-workspace`. That profile coordinates multi-member Rust projects; per-member skills (`/spec-design`, `/spec-edit`) run from the member's directory. A typescript package can sit inside a pnpm workspace, but no shipped profile drives the workspace root.
- **Checked languages: `rust`, `dart`, `typescript`, `c`, `cpp`** — test discovery and imports for all five. Shipped profiles cover `rust`, `dart` and `typescript`.
- **`/spec-reverse`** is a one-shot migration tool — moves an existing **Rust** codebase into the spec-driven world. Rust profiles only; a TypeScript or Dart project authors its modules with `/spec-design` instead. It is a person-in-the-loop authoring flow against the inlined-block schemas, not a continuous source-vs-spec audit. For ongoing source-vs-spec drift on a module's `[[items]]`, use `/spec-sync-items`.

## Permissions (optional hardening)

The skills and their drafting subagents call one external program: `specdev-analyzer`. Everything else they do is reading and writing files in your project. If you want the analyzer's calls to run without a permission prompt each time — and any *other* shell command they might attempt to surface to you instead — add a single allow rule to your `.claude/settings.json`:

```json
{
  "permissions": {
    "allow": ["Bash(specdev-analyzer *)"]
  }
}
```

Use `allow` with **no** matching `deny`. Two things to know about Claude Code's permission model:

- **`deny` always wins over `allow`.** A rule pair like `allow: ["Bash(specdev-analyzer *)"]` + `deny: ["Bash(*)"]` does **not** mean "allow only the analyzer" — the broad `deny` matches first and blocks the analyzer too. To allow only the analyzer, list just the `allow` rule; every other command then falls through to your normal prompt-on-use behavior.
- **Syntax is a space, not a colon:** `Bash(specdev-analyzer *)` (the trailing `* ` after a space matches "the binary with any arguments"). Process wrappers like `timeout`/`time` are handled automatically; pipes and env-var prefixes are not.

This is optional. Without it, analyzer calls follow your session's default permission behavior — nothing breaks, you may just be prompted.

## License

MIT.
