# spec-reverse — full-project mode

Entered from the main SKILL.md when `$ARGUMENTS` is `--all`. Runs an end-to-end reversal: architecture, project-shape survey, per-module loop, validate.

## Step A — project shape survey

Before any module reversal, detect how the source is organized and propose a list of spec modules. Goal: one spec module per coherent unit (one layer, one peripheral concern) while preserving the visual grouping users expect from the project's source-tree structure.

1. **Detect project shape.** Check these in order and stop at the first match:
   - **Cargo workspace** — root `Cargo.toml` has `[workspace] members = [...]`. Each member is a candidate spec root (each member has its own `specs/specdev/`; reverse runs per-member-independently).
   - **Single Cargo crate** — root `Cargo.toml` has `[package]`. Treat the crate as one spec root.
   - **CMake multi-subproject** — root `CMakeLists.txt` contains one or more `add_subdirectory(...)` calls. Each subdirectory is a candidate spec root.
   - **Flat / unknown** — no recognized shape. Treat the whole project as one implicit spec root.

2. **Classify each source file** by reading it:
   - **Coherent** — spans one layer and one peripheral/concern. → 1 spec module.
   - **Composite** — spans multiple layers or peripherals (e.g., a file with both a driver and its middleware consumers). → N spec modules, one per coherent unit.
   - **Infrastructure** — types-only (no actions, no state). → 0 spec modules; its types become shared port types referenced from other modules' specs.

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
4. **Validate everything** — run `specdev-analyzer validate --no-color` from the project root (or each workspace member root). Surface graph-check failures and missing cross-references. For unfamiliar rule names, read the project's `specs/specdev/domain.toml` `[[project.graph_checks]]` blocks for the rule's `kind` and meaning.

Project-wide source-vs-spec audit is **out of scope** for `/spec-reverse`. v1 ships one-way spec→source codegen + `/spec-reverse` as a migration tool; continuous audit is not in v1. Per-module `[[items]]` drift is handled by `/spec-sync-items`.

## Summary report

- **Architecture**: language, target, RTOS, build system.
- **Modules**: table — name, layer (firmware-layered) or visibility (library-api), status (OK / SKIPPED / PARTIAL), LOW-confidence field count, source files.
- **Validation**: overall PASS/FAIL with per-module issues.
- **Next steps**: review LOW fields, `/spec-edit <Module>` to refine any, `/spec-gen <Module>` when ready to regenerate code.
