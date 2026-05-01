# spec-reverse — full-project mode

Entered from the main SKILL.md when `$ARGUMENTS` is `--all`. Runs an end-to-end reversal: architecture, project-shape survey, parallel per-module fan-out, optional UI, validate, and audit.

## Step A — project shape survey

Before any module reversal, detect how the source is organized and propose a per-package modularization plan. Goal: one spec module per coherent unit (one layer, one peripheral concern) while preserving the visual grouping users expect from the project's package structure (Cargo workspace, CMake subprojects, etc.).

1. **Detect project shape.** Check these in order and stop at the first match:
   - **Cargo workspace** — root `Cargo.toml` has `[workspace] members = [...]`. Each member is a candidate package.
   - **Single Cargo crate** — root `Cargo.toml` has `[package]`. Treat the crate as one implicit package.
   - **CMake multi-subproject** — root `CMakeLists.txt` contains one or more `add_subdirectory(...)` calls. Each subdirectory is a candidate package.
   - **Flat / unknown** — no recognized shape. Treat the whole project as one implicit package.

2. **Classify each candidate package** by reading its source files:
   - **Coherent** — spans one layer and one peripheral/concern. → 1 spec module.
   - **Composite** — spans multiple layers or peripherals (e.g., a crate with both a driver and its middleware consumers). → N spec modules, one per coherent unit.
   - **Infrastructure** — types-only (no actions, no state). → 0 spec modules; its types become shared port types referenced from other modules' specs.

3. **Read `Packages` from the context header.** If `spec.toml` already declares `[[packages]]`, honor that list and skip to step 5 — only re-survey to classify each declared package. Do not propose new packages.

4. **Propose modularization plan.** Present as a tree with one-line rationale per classification and the concrete file evidence that drove it. Example:

   ```
   crates/core (coherent → 1 module)
     → Clock (HAL)  [evidence: src/timer.rs Peripherals::TIMG0 init]
   crates/drivers (composite → 2 modules)
     → Led (driver)  [evidence: src/led.rs pub fn set_pattern]
     → Button (driver)  [evidence: src/button.rs pub fn poll]
   crates/types (infrastructure → 0 modules, ports only)
   ```

5. **User confirms / edits.** Record two outputs:
   - `[[packages]]` entries to append to `spec.toml` (only for shapes that declared them implicitly; skip if already present).
   - A confirmed list of `(package, module_name, source_path_glob)` tuples used by the fan-out below.

If the user selects "no packages" for a workspace project (rare; only sensible when the workspace has just one member that contributes spec modules), emit a single implicit package and proceed with a flat `specs/modules/` layout.

## Step B — sequenced passes

Run in order, pausing for confirmation between passes:

1. **Architecture pass** — follow `mode-architecture.md`. Wait for user confirmation of `specs/architecture.toml`.
2. **Project shape survey** — Step A above. Output: confirmed `[[packages]]` list (if any) and the `(package, module_name, source_path_glob)` tuples. If packages are being introduced now, append them to `spec.toml` and create `specs/modules/<pkg>/` dirs before continuing.
3. **Parallel module reverse** — fan out one `module-reverser` subagent per confirmed `(package, module)` tuple in a **single assistant message** (one Agent tool use per tuple). Each subagent receives `module_name`, `project_root`, the `schema_dir` / `domain_dir` / `language_dir` paths from the context header, plus — when the tuple is packaged — `package` (snake_case name) and `source_path` (restrict evidence scan to this subtree). See `module-reverser-subagent.md` for the full input contract. Do NOT write files during fan-out; subagents are read-only by design. When all subagents return, collect their Candidate TOML blocks and present them grouped by package, then in layer order (`pac` → `driver` → `middleware` → `application`) for **batch confirmation** — confirm / edit / skip each. Write confirmed files to `specs/modules/<package>/<Module>.toml` (or `specs/modules/<Module>.toml` for single-package projects) in one pass. If a subagent returned an error (source not found, ambiguous split), surface it alongside the candidates and let the user resolve before writing.
4. **UI detection** — UI mode (`mode-ui.md`) is removed in v1; the UI surface is not represented in current profiles. If UI code is present in the project, note it in the report and ask whether the user wants to capture it as plain modules with documentation.
5. **Validate everything** — run `specdev-analyzer validate --no-color`. Surface graph-check failures and missing cross-references. For unfamiliar rule names, read the project's `specs/specdev/domain.toml` `[[project.graph_checks]]` blocks for the rule's `kind` and meaning.
6. **Audit cross-check** — run `specdev-analyzer audit`. This reports any source symbols not captured in the specs; for each Uncovered item, either add it to the module spec or record an intentional omission.

## Summary report

- **Architecture**: language, target, RTOS, build system, package list.
- **Modules**: table — name, layer, status (OK / SKIPPED / PARTIAL), LOW-confidence field count, source files.
- **UI**: included / not detected / skipped, file count.
- **Validation**: overall PASS/FAIL with per-module issues.
- **Audit**: Uncovered items by module.
- **Next steps**: review LOW fields, `/spec-edit <Module>` to refine any, `/spec-gen <Module>` when ready to regenerate code.
