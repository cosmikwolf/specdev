---
name: spec-edit
description: This skill should be used when the user asks to "edit <ModuleName>", "update this module spec", "rename this action", "change the pins for <module>", "remove a field", "refactor a module spec", or needs to modify an existing module TOML with downstream impact analysis. Edits one existing module spec per invocation, analyzes impact across the project, then validates. Use for modules that already exist; to author a new module, use /spec-design.
allowed-tools: Read, Write, Edit, Bash, Glob, Grep, Task
argument-hint: <ModuleName> [change description]
---

# /spec-edit — Edit a module spec

Edit exactly one existing module TOML per invocation. First positional arg is the module name (e.g., `StatusLed`); remaining text is a free-form change description.

If `$ARGUMENTS` is empty, ask the user which module to edit and what to change. Never guess.

In v1 specdev is single-package — module names are unqualified. Multi-package qualification is on the roadmap.

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

Run the analyzer's context cone for the target module — it returns the bounded sibling set + schemas + READMEs in one call:

```sh
specdev-analyzer context --module <Module> --output /tmp/specdev-ctx-$$.yaml
```

Read every file path listed under `files:` and `related_files:`. The cone replaces the "every other file under `specs/modules/*.toml`" sweep — Step 5's impact analysis runs against `related_modules` (upstream / downstream / port_endpoints / binding_endpoints / hardware_owners / re_export_targets) instead of the full module tree, with the same logic.

Order to read:

1. The target module file (in `files.target_module`).
2. `files.architecture` — for `[[connections]]` and project-wide picks.
3. Each file in `related_files` — bounded sibling set; this is the impact-analysis input.
4. `files.schema` (aggregator + inlined block + inlined bridge schemas) — authoritative for the field list.
5. `${CLAUDE_PLUGIN_ROOT}/profiles/<profile>/codegen/<language>/module.conventions.md` — only if the change touches semantically-loaded fields (state machines, metadata, hardware, bindings) and you need to confirm what they drive at codegen time.

Do not read codegen prose for unrelated fields — this skill does not generate code.

The cone is one hop deep. If impact analysis surfaces a transitive concern (e.g. a sibling that itself owns a downstream chain), re-invoke `context --module <Sibling>` for the next-hop view rather than reading siblings of siblings preemptively.

## Step 3 — display the current state

Summarize the module grouped by topic, not raw TOML:

- **Identity** — `name`, `layer` (firmware-layered) or `visibility` (library-api), `depends_on`.
- **Actions** — one line per action with `params`, `direction`, `blocking`, `isr_safe`.
- **State machines** — initial state, state names, event names; skip transition detail unless asked.
- **Ports** — name + `port_type` + `direction`.
- **Hardware** (firmware-layered) — `pins`, `peripherals`, `clocks`.
- **Errors** — one line per variant.
- **Metadata** (firmware-layered) — `MemoryModel`, `ConcurrencyModel`, `ErrorHandling`, `Ownership`.
- **Bindings** — any `bindings.<a>+<b>` entries.
- **External deps** — module-scoped crate deps.
- **Visibility / feature_gates** (library-api). Re-exports live as `[[items]]` with `kind = "re_export"`.
- **items[]** — public-API surface entries; re-exports use `kind = "re_export"`.
- **satisfies** — requirement ids if present.

## Step 4 — interview, if needed

If the request is ambiguous, ask focused questions — batch them in one message. Otherwise proceed straight to Step 5.

## Step 5 — impact analysis

Audit downstream effects per `references/impact-analysis-matrix.md`. The cone's `related_modules` block (`upstream` / `downstream` / `port_endpoints` / `binding_endpoints` / `hardware_owners` / `re_export_targets`) is the bounded input set; the matrix enumerates what to grep / cross-check for each change type (action rename, port rename, layer change, `depends_on` change, hardware reassignment, concurrency-model change, binding edit, visibility change, feature-gate edit).

If any issue surfaces, present findings grouped by change type and **wait for confirmation** before editing.

## Step 6 — edit

Two paths, by change size:

- **Trivial edits** (a single-field change, a rename, removing one entry): apply directly with `Edit`. You already have the file open from Step 3; dispatching a subagent would cost more than it saves.
- **Substantive redrafts** (adding or reshaping `[[items]]`, restructuring metadata, multi-field changes, anything touching the per-method item surface): dispatch the `module-designer` subagent with `existing_toml` set to the target file's project-relative path. The subagent re-reads the schema, profile prose, and siblings in an *isolated* context and returns the **full redrafted TOML** (its `/spec-edit` mode applies only the requested change and keeps everything else byte-stable in intent). This keeps edits on the same D70 per-method, prose-first drafting discipline as `/spec-design` without re-implementing the read path here. Pass `module_name`, `project_root`, `profile`, `language`, the change description as `description`, the Step 4 answers as `interview_answers`, `existing_toml`, and `package` (multi-package only). On a blocking error from the subagent (missing requirement ids, malformed schema), relay it and stop.

Either path follows the priority-ordered TOML authoring rules in `${CLAUDE_PLUGIN_ROOT}/skills/spec-design/references/toml-authoring-rules.md` (schema-as-contract, required fields, the D70 per-method item model, inline-table preference, naming conventions, layer discipline, one-module-per-file, comment policy). The "When editing existing TOML" section at the bottom is the spec-edit-specific carve-out: preserve existing indentation/key order where not semantically changed; an item-surface change adds/removes whole `<Type>::<member>` items rather than expanding one into a signature; strip disallowed comments as part of the edit; leave `# LOW:` markers from `/spec-reverse` alone until the user has confirmed each one.

Show a focused diff of the change (for the subagent path, diff its redrafted Candidate against the original — it should show only the intended change). For non-trivial edits, wait for user confirmation before running validate.

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

- `${CLAUDE_PLUGIN_ROOT}/agents/module-designer.md` — the subagent dispatched for substantive redrafts in Step 6 (with `existing_toml` set). Single source of truth for its input/output contract; shared with `/spec-design`.
- `references/impact-analysis-matrix.md` — change-type → downstream-check mapping used in Step 5 (action rename, port rename, layer change, `depends_on` change, hardware reassignment, concurrency-model change, binding edit, visibility change, feature-gate edit).
- `examples/edit-rename-action.md` — worked walkthrough of a rename-action edit with downstream impact analysis.
