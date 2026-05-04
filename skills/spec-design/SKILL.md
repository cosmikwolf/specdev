---
name: spec-design
description: This skill should be used when the user asks to "design a module", "add a module spec", "author a driver spec", "design <ModuleName>", or needs to write a single module TOML under specs/modules/. Authors one module TOML per invocation, then validates it. Use for modules that do not yet exist; to modify an existing module, use /spec-edit.
allowed-tools: Read, Write, Edit, Bash, Glob, Grep
argument-hint: <one-line module description>
---

# /spec-design — Module authoring

Design one module per invocation. User input: `$ARGUMENTS` (a one-line description of what the module does). If `$ARGUMENTS` is empty, ask first.

Produce a TOML file at `specs/modules/<ModuleName>.toml` that:

1. Validates against the project's aggregated `specs/specdev/schemas/module.schema.yaml` (which includes profile-shipped fields plus any inlined-block contributions).
2. Fits the existing project: consistent naming, uses peripherals declared in `specs/architecture.toml`, depends only on modules that exist (or modules explicitly noted as follow-up work).

If `specdev-analyzer` is not on PATH, tell the user to install it and stop.

Before any analyzer invocation, run the env preamble from `${CLAUDE_PLUGIN_ROOT}/shared/plugin-root-preamble.md`:

```sh
export SPECDEV_PLUGIN_ROOT="${CLAUDE_PLUGIN_ROOT}"
```

Skip if `SPECDEV_PLUGIN_ROOT` is already set or every invocation passes `--plugin-root`.

---

## Step 0 — prerequisites

If `specs/specdev/` is absent at the project root, stop and tell the user to run `/spec-init` first. Do not author blind.

Read `spec.toml` to learn the project's `[project].profile` and `[project].language`. The profile shapes which fields the module schema accepts.

## Step 1 — read the authoritative guides

Run the analyzer's context cone for the proposed module name. The cone tells the skill which files to read; you read only those, instead of the whole `specs/modules/` tree:

```sh
specdev-analyzer context --module <ProposedName> --output /tmp/specdev-ctx-$$.yaml
```

For a module that does not yet exist, the cone returns `status: module-not-found`; `files.schema`, `files.block_readmes`, `files.bridge_readmes`, `files.profile_readme`, `files.module_conventions`, and `files.architecture` are still populated (they don't depend on M). `related_modules` is empty in this path; `excluded_module_count` carries the total existing-module count for orientation.

Then read every file path listed under `files:` and (when the module exists) under `related_files:`. The schemas establish the contract; the READMEs are the authoring view (distinct from the codegen prose `/spec-gen` consumes). The architecture file carries project-wide picks. `${CLAUDE_PLUGIN_ROOT}/profiles/<profile>/codegen/<language>/module.conventions.md` is *codegen* prose — read but do not over-apply; it informs which spec choices have semantic weight downstream.

When the module already exists or a sibling-shape inference is needed, the cone's `related_files` gives the bounded sibling set to read (lex-sorted, project-relative). The cone is one hop deep — if a sibling references something farther out, re-invoke `context --module <Sibling>` for that next-hop view.

## Step 2 — interview

Ask the user for fields not reasonably inferable from `$ARGUMENTS`. The canonical interview shape is whatever the profile's `module.schema.yaml` declares as required + the inlined-block fields the project intends to use. The lists below are **illustrative examples per shipped profile**, not the authoritative source — read the profile schema for the binding contract.

### firmware-layered (illustrative)

- **Name** — PascalCase, matches `Identifier` pattern. Default: derived from `$ARGUMENTS`.
- **Layer** — one of `application`, `middleware`, `driver`, `HAL`, `PAC` (from `enums.schema.yaml#/$defs/LayerKind`). Default from description (e.g., "driver for an I2C sensor" → `driver`).
- **depends_on** — list of existing module names. Default empty.
- **Hardware** — pins / peripherals / clocks the module owns or borrows. Cross-check against `specs/architecture.toml` and the inlined MCU block's schema (e.g., for `stm32h5`: `peripherals[].kind` must be one of the block's enum).
- **Ports** — name + `port_type` + `direction` for each port. These appear as endpoints in `architecture.toml [[connections]]`.
- **Actions** — public action surface. One line per action, with `params`, `direction`, `blocking`, `isr_safe`.
- **State machine** — if the module has one, ask for initial state, state names, triggering events. Defer transition details to the TOML.
- **Errors** — one short variant name + one-line description each.
- **Metadata** — `MemoryModel`, `ConcurrencyModel`, `ErrorHandling`, `Ownership`. Pick explicit, well-justified values; do not default through.
- **Bindings** — if both `embassy` and `stm32h5-rs` are inlined and a bridge is present, ask whether this module declares any `bindings."embassy+stm32h5-rs".peripheral_tasks[]` entries (each entry: `task`, `peripheral`, optional `handoff` ∈ `{owned, shared}`).
- **satisfies** — requirement ids if `specs/requirements.toml` declares any.

### library-api (illustrative)

- **Name** — PascalCase.
- **visibility** — `pub` / `pub(crate)` / `pub(super)`.
- **depends_on** — list of existing module names.
- **items** — `[[items]]` entries for the module's contractual surface (Phase 14). Re-exports use `kind = "re_export"` with `{target, visibility, alias?}`; other public-API items (`fn`, `struct`, `enum`, `trait`, `impl`, ...) require `satisfies` (≥1 requirement id).
- **feature_gates** — list of feature names; ANY-of `cfg(feature = "...")` semantics.
- **external_deps** — module-scoped crates (CrateReference shape: `{crate, version, source?, features?, default_features?}`).
- **satisfies** — requirement ids if any.

### rust-workspace

`/spec-design` operates on a **single member crate**, not the workspace. If the project root is a rust-workspace, `cd` into the target member crate's directory (which has its own `specs/specdev/`) and run the skill there — the member's profile (typically `library-api` or `firmware-layered`) shapes the interview. If the user names a workspace member by name, ask which member to design under and route to its `specs/`.

Batch all relevant questions in one message. Show defaults so a fluent user can accept them all.

## Step 3 — write the TOML

Write `specs/modules/<ModuleName>.toml`. Obey these rules, in priority order:

1. **Schema is the contract.** Every field name must appear somewhere in the aggregator's reachable schemas. No invented keys. The schema uses `unevaluatedProperties: false`, which is `allOf`-aware — block-contributed properties pass through.
2. **Required fields by profile.** firmware-layered requires `name` + `layer`; library-api requires `name` + `visibility`. Whatever else the profile's `module.schema.yaml` lists as required must be present.
3. **Required metadata (firmware-layered).** `metadata.MemoryModel`, `metadata.ConcurrencyModel`, `metadata.ErrorHandling`, `metadata.Ownership` must all be present. Pick explicit values.
4. **Prefer inline tables for ≤3 params.** `params = [{name = "...", type = "u32", direction = "in"}]` is easier to read than an array-of-tables for short lists. Flip to `[[actions.params]]` when an action has >3 params.
5. **Naming conventions.**
   - Module names: PascalCase (`StatusLed`, `BtA2dpSink`).
   - Action names: `snake_case`.
   - Event names: `snake_case`, suffixed `_evt` (`set_solid_evt`, not `setSolid`).
   - State names: `snake_case`.
   - Error variants: `PascalCase` (they become Rust enum variants).
6. **Layer discipline (firmware-layered).** A module at layer `application` must not directly depend on a `PAC` module — walk through `middleware` / `driver` / `HAL`. The `layer_dependency` graph check enforces this.
7. **One module per file.** Each TOML holds exactly one module.
8. **Comment policy** — follow `${CLAUDE_PLUGIN_ROOT}/shared/comment-policy.md`. `/spec-design` authors from interview, not inference, so no `# LOW:` markers — those are unique to `/spec-reverse`.

## Step 4 — validate

Run from the project root:

```sh
specdev-analyzer validate --no-color
```

Show the output. If errors:

1. Read each error. Schema errors point at a TOML line; graph-check failures point at a rule name. Map rule names by reading `specs/specdev/domain.toml` `[validation.rules]` + `[[project.graph_checks]]`.
2. Fix the modeling problem. Do not delete fields to silence the checker.
3. Re-validate until clean.

If the user raises a structural issue ("actually this should be two modules"), stop, write a short plan, and wait for confirmation before rewriting.

## Step 5 — optional: extract and show

Once validate is clean, run:

```sh
specdev-analyzer extract --output /tmp/specdev-extract-<pid>
```

Read the module's YAML and show the user the first ~30 lines. This previews what `/spec-gen` will consume and surfaces any data-shape surprises early.

## Step 6 — report

One paragraph:
- Module name and path written.
- Key decisions in one line each (layer / visibility, ownership kind, error strategy, bindings).
- Anything the user asked for that wasn't placed in the file, and where it was recorded instead.
- Suggested next step: `/spec-design <next module>`, or `/spec-gen <ModuleName>` once at least one module exists and a code-path exists for the inlined blocks/bridges.

## What this skill does NOT do

- **Edit existing modules.** That's `/spec-edit`'s job. If the target file exists, stop and tell the user.
- **Inline blocks or bridges.** That's `specdev-analyzer inline <block>` / `inline-bridge <a>+<b>`. If the user wants `peripherals` / `tasks` / `bindings.<a>+<b>` fields and the corresponding block/bridge isn't inlined, the schema will reject them; tell the user to inline first.
- **Generate code.** That's `/spec-gen`'s job.
