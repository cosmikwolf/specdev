---
name: spec-reverse
description: This skill should be used when the user asks to "reverse-engineer specs", "generate specs from code", "document this existing codebase", "reverse <ModuleName>", or "spec-reverse --all". Extracts architecture and module TOML specs from an existing Rust codebase against a chosen profile (firmware-layered or library-api).
allowed-tools: Read, Write, Edit, Bash, Glob, Grep
argument-hint: [<ModuleName> | --all]
---

# /spec-reverse — Extract specs from source

Reverse-engineer TOML specs from existing source. Never invent fields the code does not support. Every claim written must be traceable to a file and line.

**Phase 10 status (2026-04-30):** the analyzer's tree-sitter-backed reverse primitives are not yet shipped. This skill currently performs *evidence-based manual authoring against the inlined-block schemas* — a person-in-the-loop workflow, not an automated extractor. When Phase 10 lands, the skill will get a `--into <Block>` flag that auto-extracts block-shaped fields (peripherals, pins, tasks, etc.) from source. Until then, the skill walks the code with the user and authors specs by hand.

If `specdev-analyzer` is not on PATH, tell the user to install it and stop.

Before any analyzer invocation, run the env preamble from `${CLAUDE_PLUGIN_ROOT}/shared/plugin-root-preamble.md`:

```sh
export SPECDEV_PLUGIN_ROOT="${CLAUDE_PLUGIN_ROOT}"
```

Skip if `SPECDEV_PLUGIN_ROOT` is already set or every invocation passes `--plugin-root`.

Modes, decided from `$ARGUMENTS`:

| Input | Mode | Procedure |
|-------|------|-----------|
| empty or missing | Architecture reverse | `references/mode-architecture.md` |
| `--all` | Full project reverse | `references/mode-full-project.md` |
| any other value | Single-module reverse for `<ModuleName>` | `references/mode-single-module.md` |

The legacy `--ui` mode (firmware UI specs) is removed in v1 — the UI surface is not represented in current profiles.

---

## Step 0 — prerequisites and bootstrap

If the project has no `specs/specdev/` directory, this is expected for a greenfield reverse. **Stop and tell the user to run `/spec-init` first** to bootstrap a profile-aware tree. `/spec-reverse` is not a substitute for `/spec-init` in v1; the inlined block schemas drive what fields the reversed module specs may use, and those schemas only exist after `/spec-init` + `specdev-analyzer inline <block>`.

If `specs/specdev/` is present, continue. Read `spec.toml` to learn the project's `[project].profile` and `[project].language`.

For firmware-layered projects, ask the user to confirm which blocks should be inlined before reversing:
- An MCU block (e.g., `stm32h5-rs`) — needed to put `peripherals` / `pins` into module specs.
- A runtime block (e.g., `embassy`) — needed to put `tasks` / `channels` into module specs.
- Any relevant bridge (e.g., `embassy+stm32h5-rs`) — needed for `bindings.<a>+<b>` entries.

If those blocks aren't inlined yet, tell the user the relevant `specdev-analyzer inline <block>` commands and stop. Reversing into a project that lacks the right block schemas will produce module TOMLs the user can't validate.

## Step 1 — read the authoritative guides

Read these from the project (not the plugin tree):

1. **`specs/specdev/schemas/module.schema.yaml`** — the project's aggregated module schema. Every field written must be valid against it.
2. **`specs/specdev/schemas/inlined/blocks/*.schema.yaml`** + **`bridges/*.schema.yaml`** — the block-contributed shape (peripheral kinds, pin ports, task fields, binding shapes).
3. **`specs/architecture.toml`** — the existing architecture if present, for cross-referencing.
4. **`${CLAUDE_PLUGIN_ROOT}/shared/comment-policy.md`** — TOML comment policy (with the `# LOW:` carve-out unique to this skill).

## Step 2 — route by mode

From the mode table above, load the matching reference file and follow it end to end. Only one mode runs per invocation (except `--all`, which composes the others via `mode-full-project.md`).

When fanning out parallel per-module work for `--all`, each subagent is the `module-reverser` agent described in `references/module-reverser-subagent.md`.

## Rules of the road

- **Evidence over invention.** Every field must trace to source. If evidence is missing, leave the field out or mark `# LOW: inferred from <file>:<line>, please confirm` and ask.
- **Schema is the contract.** Every key written must validate against `specs/specdev/schemas/module.schema.yaml` (the aggregator). If the code uses a concept the inlined blocks don't model (e.g., raw register pokes that bypass the HAL), capture it as a comment for the user, do not invent a schema field.
- **One module per file.** Do not fold multiple concerns into a single TOML.
- **Stop on structural ambiguity.** If a source file plausibly represents two modules, ask before splitting.
- **Comment policy** — follow `${CLAUDE_PLUGIN_ROOT}/shared/comment-policy.md`. This skill's carve-out: inline `# LOW: inferred from <source>:<line>, please confirm` markers are permitted on inferred fields and are removed once the user confirms each one.
- **Profile-shaped reverse.** firmware-layered's required fields differ from library-api's. Reverse against the project's actual profile, not a generic shape.

## Validate after each module

After writing any module TOML, run `specdev-analyzer validate --no-color` and fix schema/graph-check failures before moving to the next module. Do not batch a `--all` run without iterating validate per module — drift compounds.

## Additional resources

- `references/mode-architecture.md` — procedure for the no-args architecture pass.
- `references/mode-single-module.md` — procedure for `<ModuleName>` single-module reversal.
- `references/mode-full-project.md` — procedure for `--all` end-to-end reversal, including the project-shape survey and parallel fan-out.
- `references/module-reverser-subagent.md` — invocation contract for the `module-reverser` subagent dispatched during `--all`.
- The legacy `references/mode-ui.md` is retained for historical reference but is unused — the UI surface is not represented in current profiles.
