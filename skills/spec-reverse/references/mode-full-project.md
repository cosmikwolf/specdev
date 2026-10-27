# spec-reverse — full-project mode

Entered from the main SKILL.md when `$ARGUMENTS` is `--all`. Runs an end-to-end reversal: architecture, project-shape survey, per-module loop, validate.

## Step A — project shape survey

Before any module reversal, detect how the source is organized and propose a list of spec modules. Goal: one spec module per coherent unit (one layer, one peripheral concern) while preserving the visual grouping users expect from the project's source-tree structure.

1. **Detect project shape.** Check these in order and stop at the first match:
   - **Workspace** — a root manifest that lists members: `Cargo.toml` `[workspace] members`, `package.json` `workspaces` (or `pnpm-workspace.yaml`), a `melos.yaml` or `pubspec.yaml` `workspace:`, a `pyproject.toml` workspace table (`[tool.uv.workspace]`). Each member is a spec root with its own `specs/specdev/`; reverse runs in each member.
   - **Single package** — one `Cargo.toml` `[package]`, `package.json`, `pubspec.yaml` or `pyproject.toml`. One spec root.
   - **CMake multi-subproject** — root `CMakeLists.txt` with `add_subdirectory(...)` calls. Not reversed today (no items query for C/C++).
   - **Flat / unknown** — treat the whole tree as one spec root and ask the user.

   **Fix `spec.toml` from the tree before any module.** Bootstrap writes defaults; an existing tree may differ:
   - `[paths] source` must name the folder the code is in: the Python import package (`src/<package>/`, `_` not `-`), `lib/src` or `lib` for Dart, `src` or the real code folders for TypeScript. When the code is in several top-level folders, use the common root (`.`) and give the modules outside the default file names a `source_file`.
   - `toolchain` must match the lockfile in the tree (`package-lock.json` → npm, `pnpm-lock.yaml` → pnpm, `uv.lock` → uv, `poetry.lock` → poetry). The comment above `toolchain` in `spec.toml` lists the tools the profile accepts; when the tree's tool is not among them, tell the user before going on. `check` warns on a mismatch (`toolchain_matches_tree`).

   Run `specdev-analyzer check` after the fix: `source_resolves` and `toolchain_matches_tree` must be quiet before the loop starts.

2. **Classify each source file** by reading it:
   - **Coherent** — spans one layer and one peripheral/concern. → 1 spec module.
   - **Composite** — spans multiple layers or peripherals (e.g., a file with both a driver and its middleware consumers). → N spec modules, one per coherent unit.
   - **Infrastructure** — types-only (no actions, no state). → 0 spec modules; its types become shared port types referenced from other modules' specs.
   - **Generated** — written by a tool, not a person: `*.g.dart`, `*.freezed.dart`, `frb_generated*`, `*_pb2.py`, `*.pb.go`-style protobuf output, files with a "generated, do not edit" header. → 0 spec modules; the module that owns the generator's input is the spec.
   - **Entry point** — `main.rs`, `bin/*.dart`, `__main__.py`, a CLI script. → 1 module when it holds logic, 0 when it only wires other modules.

3. **Propose module list.** Present as a tree with one-line rationale per classification and the concrete file evidence that drove it. Example:

   ```
   src/timer.rs (coherent → 1 module)
     → Clock (HAL)  [evidence: Peripherals::TIMG0 init]
   src/led.rs (coherent → 1 module)
     → Led (driver)  [evidence: pub fn set_pattern]
   src/button.rs (coherent → 1 module)
     → Button (driver)  [evidence: pub fn poll]
   src/types.rs (infrastructure → 0 modules, ports only)
   ```

4. **User confirms / edits.** Record the confirmed list of `(module_name, source_path)` tuples for the loop in Step B.

For workspace projects, run the survey once per member (each member has its own `specs/specdev/`); a workspace `--all` is shorthand for "iterate every member and run --all in each."

## Step B — sequenced passes

Run in order, pausing for confirmation between passes:

1. **Architecture pass** — follow `mode-architecture.md`. Wait for user confirmation of `specs/architecture.toml`.
2. **Project shape survey** — Step A above.
3. **Per-module reverse loop** — for each confirmed `(module_name, source_path)` tuple, run the procedure from `mode-single-module.md`. Sequential walk preferred (cleaner audit trail; no `validate` race). **Parallel fan-out is not available.** The `module-reverser` subagent is staged, not shipped (`module-reverser-subagent.md` has the contract and the current status), and this skill's `allowed-tools` cannot dispatch a subagent in any case. Walk the modules sequentially; do not offer the user a choice that cannot be honored.
4. **Validate everything** — run `specdev-analyzer validate --no-color` from the project root (or each workspace member root). Surface graph-check failures and missing cross-references. Use `check`, not `validate`, when the code is present: it also compares the specs with the code. For unfamiliar rule names, read the project's `specs/specdev/domain.toml` `[[project.graph_checks]]` blocks for the rule's `kind` and meaning.

Project-wide source-vs-spec checking is **out of scope** for `/spec-reverse`: that is `specdev-analyzer check` (`/spec-validate`), which runs on every change. `/spec-reverse` is the adoption tool for code that has no spec yet. Per-module `[[items]]` drift is handled by `/spec-sync-items`.

## Summary report

- **Architecture**: language, target, RTOS, build system.
- **Modules**: table — name, the profile's required fields, status (OK / SKIPPED / PARTIAL), LOW-confidence field count, source files.
- **Requirements**: proposed / confirmed / dropped; tested vs `test_pending`; the source id of the run.
- **Validation**: overall PASS/FAIL with per-module issues.
- **Next steps**: review LOW fields, `/spec-edit <Module>` to refine any, then `/spec-validate` — `check` compares every reversed module with its code.
