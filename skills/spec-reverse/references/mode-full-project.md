# spec-reverse — full-project mode

Entered from the main SKILL.md when `$ARGUMENTS` is `--all`. Runs an end-to-end reversal in this order: survey and `spec.toml` fix (Step A), architecture, the per-module loop, `check` (Step B).

## Step A — project shape survey

Before any module reversal, detect how the source is organized, fix `spec.toml` to match the tree, and propose a list of spec modules. Goal: one spec module per coherent unit while preserving the grouping users expect from the source tree. Module names are PascalCase (`toml-authoring-rules.md` rule 5), also where a profile README example shows a lower-case name; a module in a subfolder takes the folder in its name (`lib/src/admin/store.dart` → `AdminStore`, with `source_file`). All module files go flat in `specs/modules/`.

1. **Detect project shape.** Check these in order and stop at the first match:
   - **Workspace** — a root manifest that lists members: `Cargo.toml` `[workspace] members`, `package.json` `workspaces` (or `pnpm-workspace.yaml`), a `melos.yaml` or `pubspec.yaml` `workspace:`, a `pyproject.toml` workspace table (`[tool.uv.workspace]`). Each member is a spec root with its own `specs/specdev/`; reverse runs in each member.
   - **Single package** — one `Cargo.toml` `[package]`, `package.json`, `pubspec.yaml` or `pyproject.toml`. One spec root.
   - **CMake multi-subproject** — root `CMakeLists.txt` with `add_subdirectory(...)` calls. Not reversed today (no items query for C/C++).
   - **Flat / unknown** — treat the whole tree as one spec root and ask the user.

   A part of the tree in another language (a Rust crate beside a Flutter app, a native module beside a Python package) is not this spec root's: it gets its own spec root. Say so in the report, and say when the language has no specdev support (shell scripts, CI workflow files).

2. **Fix `spec.toml` from the tree.** Bootstrap writes defaults; an existing tree may differ:
   - `[paths] source` names the folder the hand-written code is in: the Python import package (`src/<package>/`), `lib/src` or `lib` for Dart, `src` or the real code folders for TypeScript. When the default folder holds only generated code, or the library code is in several top-level folders, use the folder that holds them all (`lib`, `.`) and give each module outside the default file name a `source_file`. Entry points outside the library folder (`bin/`, `scripts/`) do not move `source`: their modules set `source_file`.
   - `toolchain` — only when `spec.toml` declares one (the profile has a choice of tools; Rust and Dart profiles have none). It matches the tree's lockfile: `package-lock.json` → npm, `pnpm-lock.yaml` → pnpm, `uv.lock` → uv, `poetry.lock` → poetry. The comment above `toolchain` lists the tools the profile accepts; when the tree's tool is not among them, tell the user before going on.

   Then run `specdev-analyzer check --no-color`. Before the loop starts it must give none of: `source_root_exists` (the source folder holds no code of the language), `toolchain_matches_tree`, `GATE-SCRIPT-001`. A toolchain change makes the gate script stale: `GATE-SCRIPT-001` says so, and `specdev-analyzer gate --emit` rewrites it.

3. **Classify each source file** by reading it:
   - **Coherent** — one concern. → 1 spec module.
   - **Composite** — several concerns (a driver and its consumers in one file). → N spec modules, one per concern.
   - **Types only** — data types with little or no behavior (a conversion or mapping table is behavior: such a file is coherent). When its types are public (exported, re-exported, `pub`), it is a module and its types are items. When they are internal, it gets no module, and the modules that use them cover them. In a profile with `ports` (firmware), its traits become port types other modules reference.
   - **Re-export only** — a Python `__init__.py`, a TypeScript `index.ts` or a Dart library of `export` lines that only re-exports. → 0 spec modules. A module that imports a name through it depends on the module that defines the name: read the names and list those modules in `depends_on`. `report imports` resolves such an import to the re-export file and marks its lines `reexport: false` for Python and Rust; `check` sees through a TypeScript barrel only, so for Python and Rust these entries are yours to get right.
   - **Generated** — written by a tool: a "generated" / "do not edit" header (`@generated`), `*.g.dart`, `*.freezed.dart`, `frb_generated*`, `*_pb2.py`. → 0 spec modules. The generator's input is the spec; when it is in another language, it belongs to that language's spec root.
   - **Example or sample** — `examples/`, `samples/`, a demo script outside the library folder. → 0 spec modules; list them in the report.
   - **Entry point** — `main.rs`, `bin/*.dart`, `lib/main.dart` (and any other Flutter target), `__main__.py`, a CLI script. → 1 module when it holds logic, 0 when it only wires other modules. Its required fields (a `role`) come from the schema like any module's.
   - **Crate or package root with logic** — `lib.rs`, a Python `__init__.py` with code. → 1 module, named after the package, with `source_file`.

4. **Propose module list.** Present as a tree with one-line rationale per classification and the file evidence that drove it. Example:

   ```
   src/timer.rs (coherent → 1 module)
     → Clock  [evidence: Peripherals::TIMG0 init]
   src/types.rs (types only, internal → 0 modules)
   lib/src/rust/frb_generated.dart (generated → 0 modules; input is the Rust crate in rust/)
   ```

5. **User confirms / edits** (an unattended run: SKILL.md, "An unattended run"). Record the confirmed list of `(module_name, source_path)` tuples for the loop in Step B.

For workspace projects, run the survey once per member (each member has its own `specs/specdev/`); a workspace `--all` is shorthand for "iterate every member and run --all in each."

## Step B — sequenced passes

Run in order, pausing for confirmation between passes:

1. **Architecture pass** — follow `mode-architecture.md`.
2. **Per-module reverse loop** — for each confirmed `(module_name, source_path)` tuple, run the procedure from `mode-single-module.md`. Walk the modules one at a time. **Parallel fan-out is not available**: the `module-reverser` subagent is staged, not shipped (`module-reverser-subagent.md`), and this skill's `allowed-tools` cannot dispatch a subagent. Do not offer the user a choice that cannot be honored.
3. **Check everything** — run `specdev-analyzer check --no-color` from the project root (or each workspace member root). It validates the specs and compares them with the code. For unfamiliar rule names, read the project's `specs/specdev/domain.toml` `[[project.graph_checks]]` blocks for the rule's `kind` and meaning. Then run `specdev-analyzer items diff --module <Name>` for each module: a public name with no item shows as `in_source_only` (see `mode-single-module.md`, "Items").

Ongoing source-vs-spec checking is `check` (`/spec-validate`), on every change; per-module `[[items]]` drift is `/spec-sync-items`. `/spec-reverse` is the adoption tool for code that has no spec yet.

## Summary report

- **Architecture**: language, build system, and only the fields the project's architecture schema has.
- **Modules**: table — name, the profile's required fields, status (OK / SKIPPED / PARTIAL), `# LOW:` markers (fields and requirements), source files. Files left without a module, and why; in-project imports into a file with no module (the next modules to reverse: `specdev-analyzer report imports --root <file>` on each module's file lists them).
- **Requirements**: proposed / confirmed / dropped; tested vs `test_pending`; the source id of the run.
- **Findings**: defects the reverse found in the code (dropped from the spec), where the code breaks the profile's guidance, and a project with no tests at all (every requirement is then `test_pending`).
- **Check**: the summary line and every error or warning; `items diff` counts per module.
- **Next steps**: review LOW fields, `/spec-edit <Module>` to refine any, then `/spec-validate`.
