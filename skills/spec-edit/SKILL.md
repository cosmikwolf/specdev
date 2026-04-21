---
name: spec-edit
description: This skill should be used when the user asks to "edit <ModuleName>", "update this module spec", "rename this action", "change the pins for <module>", "remove a field", "refactor a module spec", or needs to modify an existing module TOML with downstream impact analysis. Edits one existing module spec per invocation, analyzes same-package/cross-package/project-wide impact, then validates.
allowed-tools: Read, Write, Edit, Bash, Glob, Grep
argument-hint: <ModuleName> [change description]
---

!`${CLAUDE_PLUGIN_ROOT}/scripts/resolve-context`

# /spec-edit — Edit a module spec

Edit exactly one existing module TOML per invocation. First positional arg is the module name. It may be bare (`StatusLed`) or qualified as `<pkg>::<Module>` (`drivers::StatusLed`). Qualified form is required when two packages declare modules with the same name. Any remaining text is a free-form change description.

If `$ARGUMENTS` is empty, ask the user which module to edit and what to change. Never guess.

---

## Step 0 — prerequisites

If the context header reports the analyzer is not on PATH, or reports no `spec.toml`, stop and tell the user to run `/spec-init`. Do not edit blind.

## Step 1 — locate the module

Parse `$ARGUMENTS` for a `<pkg>::<Module>` split.

- **Qualified (`<pkg>::<Module>`)** — look at `specs/modules/<pkg>/<Module>.toml`. If absent, stop with an error.
- **Bare (`<Module>`)** — glob `specs/modules/**/<Module>.toml`:
  1. If exactly one match, use it. Record its package as the parent directory name (or `None` if directly under `specs/modules/`).
  2. If multiple matches (same name in multiple packages), stop and list them: "Ambiguous — qualify as `<pkg>::<Module>`."
  3. If zero matches, try case-insensitive; try matching by inner `name = "..."`. If still unfound, stop: "No module named `<Module>`. Use `/spec-design <name>` to create it, or list existing modules."

Record the resolved package (if any) — Step 5 uses it to scope searches.

## Step 2 — read context

Read, in this order:

1. The target module file.
2. `specs/architecture.toml` — for the project's peripherals and connection list.
3. Every other file under `specs/modules/**/*.toml` — for downstream impact analysis.
4. `<Schema dir>/module.schema.yaml` and `<Schema dir>/metadata.schema.yaml` — the authoritative field list for edits.
5. `<Domain dir>/author.md` — if the change touches unfamiliar structure.

Do not read codegen guides — this skill does not generate code.

## Step 3 — display the current state

Summarize the module grouped by topic, not raw TOML:

- **Identity** — `name`, `layer`, `depends_on`.
- **Actions** — one line per action with `params`, `direction`, `blocking`, `isr_safe`.
- **State machines** — initial state, state names, event names; skip transition detail unless asked.
- **Ports** — name + `port_type`.
- **Hardware** — `pins`, `peripherals`, `clocks`.
- **Errors** — one line per variant.
- **Metadata** — `MemoryModel`, `ConcurrencyModel`, `ErrorHandling`, `Ownership`.
- **Requirements** — `requirements_satisfied` ids if present.

## Step 4 — interview, if needed

If the request is ambiguous, ask focused questions — batch them in one message. Otherwise proceed straight to Step 5.

## Step 5 — impact analysis

Before writing, audit downstream effects. Partition findings into three buckets when the project declares `[[packages]]`:

- **Same-package impact** — callers or resource conflicts within the target module's package.
- **Cross-package impact** — callers or references from other packages (expected for public cross-crate APIs; flag so the user knows the blast radius).
- **Project-wide impact** — concerns that ignore package boundaries (hardware ownership, wiring in `specs/architecture.toml`).

For unpackaged projects, collapse to a single flat report.

Do each that applies:

- **Renaming/removing an action** — grep `depends_on` and source under the target's `package.source_path` first (same-package bucket), then every other package's `source_path` (cross-package bucket). Unqualified references inside the same package resolve locally; qualified `<pkg>::<Module>.action` references from other packages must also be matched.
- **Renaming/removing a port** — grep `specs/architecture.toml` `[[connections]]` entries; a connection endpoint may appear qualified (`<pkg>::<Module>.port`) or unqualified. List broken wires as **project-wide**.
- **Changing `layer`** — check every module that `depends_on` this one across *all* packages; if their layer is now higher per `<Domain dir>/domain.toml`, flag `LAYER001`. Report in the Cross-package bucket if the caller's package differs.
- **Changing `depends_on`** — check for cycles and layer rules both ways. Qualified references cross package boundaries; resolve them before the graph check.
- **Changing `hardware.peripherals` / `hardware.pins`** — always **project-wide**. Pin conflicts ignore package boundaries. Cross-check `specs/architecture.toml` and grep every module for the same resource.
- **Changing `metadata.ConcurrencyModel`** — local check: if any action is `isr_safe = true`, `protection` cannot be `none`.
- **Changing `metadata.Ownership.owner`** — verify the owner module exists; accept unqualified same-package name or qualified `<pkg>::<Module>`.

If any issue surfaces, present them grouped by bucket and **wait for confirmation** before editing.

## Step 6 — edit

Apply the change using `Edit`. Obey the rules from `/spec-design` step 3:

1. Field names must match `module.schema.yaml`. No invented keys.
2. Preserve existing indentation and key order where not semantically changed.
3. Do not drop required metadata sub-objects.
4. Keep inline-table vs array-of-tables form consistent with the rest of the file.

Show a focused diff of the change. For non-trivial edits, wait for user confirmation before running validate.

## Step 7 — validate

Run `specdev-analyzer validate --project <project_root> --no-color`. Present the output.

On errors:
1. Map each rule id to a fix (`LAYER001`, `FSM020/21/22`, `PORT030`, `ISR060`, `OWN050`, `REQ001/002`, `META010`).
2. Fix the modeling problem — do not delete fields to silence the checker.
3. Re-run until clean or until the user stops.

## Step 8 — report

One paragraph:

- What changed (fields + summary).
- Validation result.
- Downstream modules that likely need a follow-up edit (list by qualified name `<pkg>::<Module>` when packaged, else bare name).
- Suggested next step: `/spec-edit <qualified-name>` for each affected dependent, or `/spec-gen <qualified-name>` to regenerate code.
