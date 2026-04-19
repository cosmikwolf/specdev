---
name: spec-edit
description: Edit an existing module TOML spec with downstream impact analysis, then validate.
allowed-tools: Read, Write, Edit, Bash, Glob, Grep
argument-hint: <ModuleName> [change description]
---

!`${CLAUDE_PLUGIN_ROOT}/scripts/resolve-context`

# /spec-edit — Edit a module spec

You edit exactly one existing module TOML per invocation. First positional arg is the module name (PascalCase, matching the file stem under `specs/modules/`). Any remaining text is a free-form change description.

If `$ARGUMENTS` is empty, ask the user which module to edit and what to change. Never guess.

---

## Step 0 — prerequisites

If the context header reports the analyzer is not on PATH, or reports no `spec.toml`, stop and tell the user to run `/spec-init`. Do not edit blind.

## Step 1 — locate the module

Glob `specs/modules/**/<ModuleName>.toml`. If not found:
1. Try a case-insensitive match.
2. Try matching by `name = "..."` inside TOML files, in case the filename diverged.
3. If still unfound, stop: "No module named `<ModuleName>`. Use `/spec-design <name>` to create it, or list existing modules."

## Step 2 — read context

Read, in this order:

1. The target module file.
2. `specs/architecture.toml` — for the project's peripherals and connection list.
3. Every other file under `specs/modules/**/*.toml` — for downstream impact analysis.
4. `<Schema dir>/module.schema.yaml` and `<Schema dir>/metadata.schema.yaml` — the authoritative field list for edits.
5. `<Domain dir>/author.md` — if the user's change touches structure you haven't seen before.

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

If the user's request is ambiguous, ask focused questions — batch them in one message. Otherwise proceed straight to Step 5.

## Step 5 — impact analysis

Before writing, audit downstream effects. Do each that applies:

- **Renaming/removing an action** — grep other modules for the action name; flag callers.
- **Renaming/removing a port** — grep `specs/architecture.toml` `[[connections]]` entries; list broken wires.
- **Changing `layer`** — check every module that `depends_on` this one; if their layer is now higher per `<Domain dir>/domain.toml`, flag `LAYER001` violations.
- **Changing `depends_on`** — check for cycles; check layer rules both ways.
- **Changing `hardware.peripherals` / `hardware.pins`** — cross-check `specs/architecture.toml` and grep other modules for the same resource (two modules must not both own the same pin or peripheral).
- **Changing `metadata.ConcurrencyModel`** — if any action is `isr_safe = true`, `protection` cannot be `none`.
- **Changing `metadata.Ownership.owner`** — verify that owner module exists.

If any issue surfaces, present them clearly and **wait for confirmation** before editing.

## Step 6 — edit

Apply the change using `Edit`. Obey the rules from `/spec-design` step 3:

1. Field names must match `module.schema.yaml`. No invented keys.
2. Preserve existing indentation and key order where not semantically changed.
3. Do not drop required metadata sub-objects.
4. Keep inline-table vs array-of-tables form consistent with the rest of the file.

Show a focused diff of the change. For non-trivial edits wait for user confirmation before running validate.

## Step 7 — validate

Run `specdev-analyzer validate --project <project_root> --no-color`. Present the output.

On errors:
1. Map each rule id to a fix (`LAYER001`, `FSM020/21/22`, `PORT030`, `ISR060`, `OWN050`, `REQ001/002`, `META010`).
2. Fix the modeling problem — do not delete fields to silence the checker.
3. Re-run until clean or until the user stops you.

## Step 8 — report

One paragraph:

- What changed (fields + summary).
- Validation result.
- Downstream modules that likely need a follow-up edit (list by name).
- Suggested next step: `/spec-edit` each affected dependent, or `/spec-gen <ModuleName>` if the user wants regenerated code.
