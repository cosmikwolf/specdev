---
name: spec-reverse
description: This skill should be used when the user asks to "reverse-engineer specs", "generate specs from code", "document this existing codebase", "reverse <ModuleName>", "spec-reverse --all", or "spec-reverse --ui". Extracts architecture, module, or UI TOML specs from an existing Rust/C/C++ codebase.
allowed-tools: Read, Write, Edit, Bash, Glob, Grep
argument-hint: [<ModuleName> | --all | --ui]
---

!`${CLAUDE_PLUGIN_ROOT}/scripts/resolve-context`

# /spec-reverse — Extract specs from source

Reverse-engineer TOML specs from existing source. Never invent fields the code does not support. Every claim written must be traceable to a file and line.

Modes, decided from `$ARGUMENTS`:

| Input | Mode | Procedure |
|-------|------|-----------|
| empty or missing | Architecture reverse | `references/mode-architecture.md` |
| `--all` | Full project reverse | `references/mode-full-project.md` |
| `--ui` | UI reverse | `references/mode-ui.md` |
| any other value | Single-module reverse for `<ModuleName>` | `references/mode-single-module.md` |

---

## Step 0 — prerequisites and bootstrap

If the context header reports the analyzer is not on PATH, stop and tell the user to install it.

If the context header reports **Status: no `spec.toml` found**, this is expected for a greenfield reverse on an existing codebase. Do **not** stop — instead, bootstrap inline:

1. Produce: `spec.toml` with `[project]` populated (name, domain, languages); a `specs/` skeleton with `modules/`, `requirements/`, and `ui/` (if the domain uses UI) subdirectories; a top-level `architecture.toml` with `[[layers]]` and `[[packages]]`; and a stub `specs/requirements/README.md`. Infer defaults aggressively from source-tree evidence (`Cargo.toml`, `CMakeLists.txt`, `platformio.ini`, etc.) so the user only confirms, not types. For the full interactive interview, invoke `/spec-init` as a separate step first.
2. Once `spec.toml` and the `specs/` skeleton exist, continue with Step 1 below. Do not re-run the resolve-context header — the project layout is already known.

## Step 1 — read the authoritative guides

Read these, in order, from the paths in the context header:

1. `<Schema dir>/architecture.schema.yaml`, `<Schema dir>/module.schema.yaml`, `<Schema dir>/metadata.schema.yaml`, `<Schema dir>/ui.schema.yaml` — the authoritative field list for every spec file to be written.
2. `<Domain dir>/author.md` — structural conventions for this domain.
3. `<Language dir>/syntax.md` — file conventions for the project's language (Rust / C / C++).

Also read `specdev/shared/comment-policy.md` — the TOML comment policy applies to every file this skill writes (including the LOW-marker carve-out unique to this skill).

## Step 2 — route by mode

From the mode table above, load the matching reference file and follow it end to end. Only one mode runs per invocation (except `--all`, which composes the others via `mode-full-project.md`).

When fanning out parallel per-module work for `--all`, each subagent is the `module-reverser` agent described in `references/module-reverser-subagent.md`.

## Rules of the road

- **Evidence over invention.** Every field must trace to source. If evidence is missing, leave the field out or mark LOW and ask.
- **Schema is the contract.** Every key written must appear in `module.schema.yaml`, `architecture.schema.yaml`, or `ui.schema.yaml`.
- **One module per file.** Do not fold multiple concerns into a single TOML.
- **Stop on structural ambiguity.** If a source file plausibly represents two modules, ask before splitting.
- **Comment policy** — follow `specdev/shared/comment-policy.md`. This skill's carve-out: inline `# LOW: inferred from <source>:<line>, please confirm` markers are permitted on inferred fields and are removed once the user confirms each one.
- **Rule ids** — when validate surfaces an unfamiliar rule id, read `specdev/shared/rule-ids.md` (firmware) or the loaded `<Domain dir>/domain.toml` `[validation.rules]` table.

## Additional resources

- `references/mode-architecture.md` — procedure for the no-args architecture pass.
- `references/mode-single-module.md` — procedure for `<ModuleName>` single-module reversal.
- `references/mode-ui.md` — procedure for `--ui` UI reversal.
- `references/mode-full-project.md` — procedure for `--all` end-to-end reversal, including the project-shape survey and parallel fan-out.
- `references/module-reverser-subagent.md` — invocation contract for the `module-reverser` subagent dispatched during `--all`.
