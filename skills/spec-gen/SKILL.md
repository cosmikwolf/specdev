---
name: spec-gen
description: This skill should be used when the user asks to "generate code", "codegen", "run spec-gen", "generate <ModuleName>", "codegen all modules", "implement this spec", or needs source code generated from validated specdev specs. Handles single modules, packaged modules (`pkg::Module`), and `--all` batch generation.
allowed-tools: Read, Write, Edit, Bash, Glob, Grep
argument-hint: <ModuleName | --all>
---

!`${CLAUDE_PLUGIN_ROOT}/scripts/resolve-context`

# /spec-gen — Code generation

Generate source code for one named module, or for every module when invoked with `--all`. User input: `$ARGUMENTS`.

Act as an implementer, not an improviser. The codegen guides at the resolved `Codegen` path are the rules; the extracted YAML is the data; the language guides at the resolved `Languages` path are the style. If the three disagree, the schema/codegen guides win.

---

## Step 1 — gate on validation

Run `specdev-analyzer validate --project <project_root> --no-color`. If the exit code is non-zero, stop and tell the user to run `/spec-validate` first. Do not generate code from invalid specs — the output will be wrong in subtle ways.

## Step 2 — extract

Run `specdev-analyzer extract --project <project_root> --output /tmp/specdev-extract-<pid>` to produce canonical YAML. Note the output directory — every subsequent step reads from it.

Each module becomes `<ModuleName>.yaml`. There is also one `architecture.yaml` (project-wide summary) and one `requirements.yaml` (flattened).

## Step 3 — resolve the targets

Extract output is nested for multi-package projects: `<out>/<ModuleName>.yaml` for unpackaged modules, `<out>/<package>/<ModuleName>.yaml` for packaged ones. Treat packaged modules as `<pkg>::<Module>` when referring to them in logs and disambiguation.

- If `$ARGUMENTS` is `--all`: targets are every module YAML under the extract output (recursive), excluding `architecture.yaml` and `requirements.yaml`.
- If `$ARGUMENTS` is `<pkg>::<Module>`: the single target is `<out>/<pkg>/<Module>.yaml`.
- If `$ARGUMENTS` is a bare `<Module>`: search all packages. If exactly one match, use it. If multiple, stop and ask the user to qualify it. If zero, fail with the list of available modules.

For every target, note whether it has a `state_machines[]` array (→ also consult `gen-fsm.md`) and whether any action targets a UI element (→ also consult `gen-ui.md`, only if the project has a `ui/` spec section).

## Step 3.5 — resolve `<source_dir>` per target

Every codegen guide writes files under a placeholder `<source_dir>` (effectively what the guide calls `src/`). Resolve it per target:

1. If the target module's YAML carries a top-level `package: <name>` key, look up that package in the project's `spec.toml` `[[packages]]` list and use its `source_path` joined with `src/` — e.g. `crates/drivers/src/`.
2. Otherwise, use the project's `[source] root` from `spec.toml` (typically `src/`).

Record `<source_dir>` for each target. Wherever a codegen guide says `src/<module_name>/mod.rs`, substitute `<source_dir>/<module_name>/mod.rs`. Single-package projects stay unchanged (source root is `src/`, substitution is a no-op).

## Step 4 — read the rules

For each target module, read — from the paths in the context header, not hard-coded plugin paths:

1. `<Codegen dir>/gen-module.md` — the field-to-code mapping rules. This file is the contract.
2. `<Codegen dir>/gen-fsm.md` — if the module has `state_machines[]`.
3. `<Codegen dir>/gen-ui.md` — if the module participates in a UI.
4. `<Language dir>/syntax.md` — project-wide style rules (naming, formatting, import ordering).
5. `<Language dir>/build.md` — where to place files, how to register them with the build system.

If the target source file already exists, read it too. Preserve user edits outside the auto-generated regions where the language's conventions allow.

## Step 5 — generate

Output source code that satisfies, in priority order:

1. **The codegen guide's mapping rules** (e.g. "`ConcurrencyModel.protection = mutex` ⇒ the module struct holds a `SemaphoreHandle_t`").
2. **The language's syntax conventions** (naming, file placement, header layout).
3. **The existing project's observable conventions.** Skim other modules' source under the resolved `<source_dir>` (see Step 3.5) and match what they do — indent width, use of `#pragma once` vs include guards, etc.
4. **Style-only defaults when the guide is silent.** For *formatter-level* details the codegen guide does not prescribe (indent width, trailing commas, comment placement), match the project's observable conventions from priority 3. For *semantic* choices the guide does not prescribe (error handling strategy, concurrency primitives, module boundaries), stop and ask the user — do not improvise.

For each file written:
- At the top, emit the auto-generation header the codegen guide specifies (file name, `@spec` pointer to the module TOML, regeneration command).
- Under the header, import order and visibility as per the language guide.
- No placeholder bodies. If a function cannot be implemented because the spec lacks information, stop and ask before writing a `todo!()` / `return -1;` stub.

Do **not** modify files outside the auto-generated regions. If hand-written code exists in the target file, preserve it and warn in the report.

## Step 6 — compile gate (best effort)

If the project's build system is detected (`Cargo.toml`, `CMakeLists.txt`, `platformio.ini`), run a build. For Rust: `cargo check --quiet` from the project root. For C/C++: whatever the `build.md` prescribes.

Report the first error verbatim if the build fails. Do not attempt to fix build errors that stem from missing scaffolding outside the generation scope (e.g. "main.rs does not register this module") — surface the problem to the user with a one-line action item.

## Step 7 — clean up

Delete the `/tmp/specdev-extract-<pid>` directory once generation is complete. The extracted YAML is a cache, not output.

## Step 8 — report

One paragraph per module generated:
- Source file(s) written.
- Any codegen guide rule that required a choice and the choice made.
- Build result (pass/fail; first error if fail).
- Suggested follow-ups: `/spec-gen <OtherModule>`, wire this module into `main.rs`, or regenerate after editing the spec.

## Additional resources

- `examples/source-dir-resolution.md` — worked cases for resolving `<source_dir>` per target (flat, multi-package, edge cases).
