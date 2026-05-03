---
name: spec-edit
description: This skill should be used when the user asks to "edit <ModuleName>", "update this module spec", "rename this action", "change the pins for <module>", "remove a field", "refactor a module spec", or needs to modify an existing module TOML with downstream impact analysis. Edits one existing module spec per invocation, analyzes impact across the project, then validates. Use for modules that already exist; to author a new module, use /spec-design.
allowed-tools: Read, Write, Edit, Bash, Glob, Grep
argument-hint: <ModuleName> [change description]
---

# /spec-edit — Edit a module spec

Edit exactly one existing module TOML per invocation. First positional arg is the module name (e.g., `StatusLed`); remaining text is a free-form change description.

If `$ARGUMENTS` is empty, ask the user which module to edit and what to change. Never guess.

In v1 specdev is single-package — module names are unqualified. Multi-package qualification lands in Phase 8.5 (library) and Phase 12 (firmware).

If `specdev-analyzer` is not on PATH, tell the user to install it and stop.

Before any analyzer invocation, run the env preamble from `${CLAUDE_PLUGIN_ROOT}/shared/plugin-root-preamble.md`:

```sh
export SPECDEV_PLUGIN_ROOT="${CLAUDE_PLUGIN_ROOT}"
```

Skip if `SPECDEV_PLUGIN_ROOT` is already set or every invocation passes `--plugin-root`.

---

## Step 0 — prerequisites

If `specs/specdev/` is absent at the project root, stop and tell the user to run `/spec-init`. Do not edit blind.

Read `spec.toml` to learn the project's `[project].profile`.

## Step 1 — locate the module

Look at `specs/modules/<Module>.toml`.

- If exactly one match: use it.
- If absent: try case-insensitive match; try matching by inner `name = "..."`. If still unfound, stop: "No module named `<Module>`. Use `/spec-design <name>` to create it, or list existing modules."

## Step 2 — read context

Read, in this order:

1. The target module file.
2. **`specs/architecture.toml`** — for the project's platform/runtime/logger picks and `[[connections]]` list.
3. **Every other file under `specs/modules/*.toml`** — for downstream impact analysis (this is the file set that may reference the target via `depends_on`, ports, hardware ownership, requirement satisfaction).
4. **`specs/specdev/schemas/module.schema.yaml`** — the project-managed aggregator. Authoritative for the field list.
5. **`specs/specdev/schemas/inlined/blocks/*.schema.yaml`** + **`bridges/*.schema.yaml`** — for block- and bridge-contributed fields the change might touch.
6. **`${CLAUDE_PLUGIN_ROOT}/profiles/<profile>/codegen/<language>/module.conventions.md`** — only if the change touches semantically-loaded fields (state machines, metadata, hardware, bindings) and you need to confirm what they drive at codegen time.

Do not read codegen prose for unrelated fields — this skill does not generate code.

## Step 3 — display the current state

Summarize the module grouped by topic, not raw TOML:

- **Identity** — `name`, `layer` (firmware-layered) or `visibility` (library-api), `depends_on`.
- **Actions** — one line per action with `params`, `direction`, `blocking`, `isr_safe`.
- **State machines** — initial state, state names, event names; skip transition detail unless asked.
- **Ports** — name + `port_type` + `direction`.
- **Hardware** (firmware-layered) — `pins`, `peripherals`, `clocks`.
- **Errors** — one line per variant.
- **Metadata** (firmware-layered) — `MemoryModel`, `ConcurrencyModel`, `ErrorHandling`, `Ownership`.
- **Bindings** — any `bindings.<a>+<b>` entries (Phase 7+).
- **External deps** — module-scoped crate deps.
- **Visibility / feature_gates** (library-api). Re-exports live as `[[items]]` with `kind = "re_export"`.
- **items[]** (Phase 14) — public-API surface entries; re-exports use `kind = "re_export"`.
- **satisfies** — requirement ids if present.

## Step 4 — interview, if needed

If the request is ambiguous, ask focused questions — batch them in one message. Otherwise proceed straight to Step 5.

## Step 5 — impact analysis

Before writing, audit downstream effects. Surface them grouped by category:

- **Renaming/removing an action** — grep `depends_on` and source under `[paths].source` to find call sites. List broken call sites.
- **Renaming/removing a port** — grep `specs/architecture.toml` `[[connections]]` for `from = "<Module>.<port>"` or `to = "<Module>.<port>"`. List broken wires.
- **Changing `layer`** (firmware-layered) — check every module that `depends_on` this one; if any caller's layer is now strictly above the target's per the layered ordering (`application > middleware > driver > HAL > PAC`), flag a future `layer_dependency` violation. Same in reverse: if this module now depends on a higher layer, flag.
- **Changing `depends_on`** — check for cycles by building the new graph mentally.
- **Changing `hardware.peripherals` / `hardware.pins`** (firmware-layered) — grep every module for the same resource. Two modules owning the same pin/peripheral surfaces as a hardware-ownership violation. Cross-check `architecture.toml [[connections]]` for endpoints that name the renamed/removed peripheral.
- **Changing `metadata.ConcurrencyModel`** — local check: if any action is `isr_safe = true`, `protection` cannot be `none`.
- **Changing `bindings.<a>+<b>`** — confirm referenced `task` / `peripheral` names still exist on the target module after the edit; cross-reference any other bindings that may reference removed entries.
- **Changing `visibility`** (library-api) — modules whose `[[items]]` `kind = "re_export"` references this module may need their re-export `visibility` adjusted.
- **Changing `feature_gates`** (library-api) — `module_feature_gates_resolve` will fail if a referenced feature is removed.

If any issue surfaces, present them grouped and **wait for confirmation** before editing.

## Step 6 — edit

Apply the change using `Edit`. Obey the rules from `/spec-design` Step 3:

1. Field names must match the project's `module.schema.yaml`. No invented keys.
2. Preserve existing indentation and key order where not semantically changed.
3. Do not drop required metadata sub-objects (firmware-layered).
4. Keep inline-table vs array-of-tables form consistent with the rest of the file.
5. **Comment policy** — follow `${CLAUDE_PLUGIN_ROOT}/shared/comment-policy.md`. If the existing file carries disallowed comments (file-header blurbs, section dividers, prose that restates a nearby field), strip them as part of your edit. Carve-out: leave `# LOW:` markers from `/spec-reverse` alone until the user has confirmed or cleared each one.

Show a focused diff of the change. For non-trivial edits, wait for user confirmation before running validate.

## Step 7 — validate

Run from the project root:

```sh
specdev-analyzer validate --no-color
```

On errors:
1. Map each rule name to a fix by reading `specs/specdev/domain.toml` `[validation.rules]` + `[[project.graph_checks]]`. The rule's `kind` (`unique`, `acyclic`, `layer_dependency`, `endpoints_resolve`, etc.) tells you what was checked.
2. Fix the modeling problem — do not delete fields to silence the checker.
3. Re-run until clean or until the user stops.

## Step 8 — report

One paragraph:

- What changed (fields + summary).
- Validation result.
- Downstream modules that likely need a follow-up edit, with their bare names.
- Suggested next step: `/spec-edit <Module>` for each affected dependent, or `/spec-gen <Module>` to regenerate code once the change is locked.

## Additional resources

- `examples/edit-rename-action.md` — worked walkthrough of a rename-action edit with downstream impact analysis.
