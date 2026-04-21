---
name: spec-design
description: Author or refine a single module TOML spec under specs/modules/, then validate it.
allowed-tools: Read, Write, Edit, Bash, Glob, Grep
argument-hint: <one-line module description>
---

!`${CLAUDE_PLUGIN_ROOT}/scripts/resolve-context`

# /spec-design — Module authoring

You design one module per invocation. User input: `$ARGUMENTS` (a one-line description of what the module does). If `$ARGUMENTS` is empty, ask first.

Your job is to produce a TOML file at `specs/modules/<ModuleName>.toml` that:

1. Validates against the resolved schema.
2. Follows the authoring guidance in the resolved domain directory.
3. Fits the existing project: consistent naming, uses actual peripherals declared in `specs/architecture.toml`, and depends only on modules that exist (or that you explicitly note as follow-up work).

---

## Step 0 — prerequisites

If the context header reports the analyzer is not on PATH, or reports no `spec.toml`, stop and tell the user to run `/spec-init` first. Do not author blind.

## Step 1 — read the authoritative guides

Read these files, in order, from the paths given in the context header (NOT hard-coded plugin paths — the resolver may have pointed you at `.specdev/` overrides):

1. `<Domain dir>/author.md` — how modules are structured for this domain, with examples.
2. `<Schema dir>/module.schema.yaml` — canonical field list and enums. This is the contract; treat it as authoritative over any other doc.
3. `<Schema dir>/metadata.schema.yaml` — `MetadataObject` sub-schema used by modules.
4. `specs/architecture.toml` in the project — for the peripherals/packages the module may use.
5. Every existing file under `specs/modules/` — for naming conventions, layer placement, and the pattern of fields the user already uses.

Do not read the codegen guides. They describe how specs turn into code, which is `/spec-gen`'s job.

## Step 2 — interview

Ask the user for the fields you cannot reasonably infer from `$ARGUMENTS`:

- **Name** — PascalCase. Default: suggest a name derived from `$ARGUMENTS`.
- **Layer** — from the enum in `module.schema.yaml`. Ask which, default from the description (e.g. "driver for an I2C sensor" → `driver`).
- **Depends_on** — list of existing module names. Default: empty; ask whether any existing module must be wired in as a dependency.
- **Public action surface** — what interface functions does this module expose? One line per action. Mark each `blocking` and `isr_safe` explicitly.
- **Hardware** — pins, peripherals, clocks the module owns or borrows. Cross-check against `specs/architecture.toml`.
- **State machine** — if the module has one, ask for initial state, state names, and the triggering events. Defer transition details to the TOML.
- **Errors** — which failure modes the module reports. One short variant name + one-line description each.
- **Requirements satisfied** — if the project has `specs/requirements/*.toml`, list the ids this module implements. Otherwise leave empty. (Requirements themselves may declare `source`/`source_ref` pointing at an external spec document — the module's `satisfies[]` still references requirement ids only, not source ids.)

Batch these questions in one message. Show reasonable defaults so a fluent user can accept them all.

## Step 3 — write the TOML

Write `specs/modules/<ModuleName>.toml`. Obey these rules, in priority order:

1. **Schema is the contract.** Every field name must appear in `module.schema.yaml`. No invented keys.
2. **Required metadata.** `metadata.MemoryModel`, `metadata.ConcurrencyModel`, `metadata.ErrorHandling`, `metadata.Ownership` must be present (they are `required: true` per domain config). Pick explicit, well-justified values — do not default through.
3. **Prefer inline tables for ≤3 params.** Arrays of tables (`[[actions.params]]`) become noisy for short lists; inline `params = [{name = "...", type = "u32"}]` is easier to read. Flip to array-of-tables form when an action has >3 params.
4. **Name conventions.**
   - Module names: PascalCase (`StatusLed`, `BtA2dpSink`).
   - Action names: `snake_case`.
   - Event names: `snake_case`, suffixed `_evt` (`set_solid_evt`, not `setSolid`).
   - State names: `snake_case`.
   - Error variants: `PascalCase` (they become `enum class` / Rust enum variants downstream).
5. **Layer discipline.** A module at layer `application` must not depend on a `pac` module directly; walk through middleware/driver. If in doubt, read `author.md`.
6. **One module per file.** Never put two `part def` siblings in one file; each TOML holds exactly one module.

## Step 4 — validate

Run `specdev-analyzer validate --project <project_root>` (do not add `--domain` — the analyzer resolves it from `spec.toml`). Show the output to the user.

If there are errors:
1. Read each error. The `rule` id points at a specific check (LAYER001 = layer dependency, FSM020 = unreachable state, META010 = required metadata, PORT030 = duplicate port name, etc.).
2. Fix the TOML. Do not mask errors by removing fields — fix the actual modeling problem.
3. Re-validate until clean.

If the user stops you to discuss a structural issue (e.g. "actually this should be two modules"), stop, write a short plan, and wait for confirmation before rewriting.

## Step 5 — optional: extract and show

Once validate is clean, run `specdev-analyzer extract --project <project_root> --output /tmp/specdev-extract-<pid>` and read the module's YAML. Show the user the first ~30 lines. This previews what `/spec-gen` will consume and surfaces any data-shape surprises early.

## Step 6 — report

One-paragraph summary:
- Module name and path written.
- Key decisions (layer, ownership kind, error strategy) in one line each.
- Anything the user asked for but was not placed in the file, and where you recorded it instead.
- Suggested next step: `/spec-design <next module>`, or `/spec-gen <ModuleName>` once at least one module exists.
