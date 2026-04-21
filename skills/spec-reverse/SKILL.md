---
name: spec-reverse
description: Reverse-engineer specs from existing source code (architecture, modules, or UI).
allowed-tools: Read, Write, Edit, Bash, Glob, Grep
argument-hint: [<ModuleName> | --all | --ui]
---

!`${CLAUDE_PLUGIN_ROOT}/scripts/resolve-context`

# /spec-reverse — Extract specs from source

You reverse-engineer TOML specs from existing source. Never invent fields the code doesn't support. Every claim you write must be traceable to a file and line.

Modes, decided from `$ARGUMENTS`:

| Input | Mode |
|-------|------|
| empty or missing | Architecture reverse (Step 3) |
| `--all` | Full project reverse (Step 6) |
| `--ui` | UI reverse (Step 5) |
| any other value | Single-module reverse for `<ModuleName>` (Step 4) |

---

## Step 0 — prerequisites and bootstrap

If the context header reports the analyzer is not on PATH, stop and tell the user to install it.

If the context header reports **Status: no `spec.toml` found**, this is expected for a greenfield reverse on an existing codebase. Do **not** stop — instead, run the `/spec-init` flow inline first:

1. Follow every step in `specdev/skills/spec-init/SKILL.md` (Steps 1–7), reading that skill's instructions verbatim. Defaults for the interview should be aggressively inferred from the source tree (Cargo.toml, CMakeLists.txt, etc.) so the user only confirms, not types.
2. Once `spec.toml` and the `specs/` skeleton exist, continue with Step 1 below. Do not re-run the resolve-context header — you already know the project layout.

## Step 1 — read the authoritative guides

Read these, in order, from the paths in the context header:

1. `<Schema dir>/architecture.schema.yaml`, `<Schema dir>/module.schema.yaml`, `<Schema dir>/metadata.schema.yaml`, `<Schema dir>/ui.schema.yaml` — the authoritative field list for every spec file you may write.
2. `<Domain dir>/author.md` — structural conventions for this domain.
3. `<Language dir>/syntax.md` — file conventions for the project's language (Rust / C / C++).

## Step 2 — route by mode

Jump to the appropriate step below.

---

## Step 2.5 — Project shape survey (used by `--all`)

Skip this section for single-module and `--ui` modes.

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
   - A confirmed list of `(package, module_name, source_path_glob)` tuples used by Step 6 fan-out.

If the user selects "no packages" for a workspace project (rare; only sensible when the workspace has just one member that contributes spec modules), emit a single implicit package and proceed with a flat `specs/modules/` layout.

---

## Step 3 — Architecture reverse (no args)

If `specs/architecture.toml` exists, ask: **Compare (dry-run diff) / Replace / Cancel**.

Scan the project root:

1. **Language** — from file extensions under `src/` and build files.
2. **Build system** — `Cargo.toml` (Rust), `CMakeLists.txt` / `Makefile` / `meson.build` (C/C++), `platformio.ini`.
3. **MCU / target** — defines, linker scripts, HAL crate names, SDK includes.
4. **RTOS** — look for FreeRTOS, Zephyr, RTIC, Embassy, bare-metal markers.
5. **External packages** — dependencies in Cargo.toml or `#include <...>` matches against known SDKs; present unrecognized ones for confirmation.
6. **Modules** — discover from `src/` structure. For Rust workspaces, read `[workspace] members`. For single-crate Rust, treat each `src/**/mod.rs` or top-level `src/<name>.rs` as a candidate module.
7. **Peripherals and pins** — grep for peripheral initialization (e.g. `Peripherals::take`, `gpio_init`, `twi_init`), collect pin numbers and peripheral names.
8. **Connections** — look for wiring between modules (calls from module A into module B's public surface).

Present candidate `specs/architecture.toml` with confidence (HIGH/MEDIUM/LOW) per field. Wait for confirmation. Write clean TOML to `specs/architecture.toml`. Scaffold `specs/modules/` and `specs/requirements/` if absent.

Report: language, build system, target, RTOS, packages, modules discovered, next step `/spec-reverse <Module>` or `/spec-reverse --all`.

---

## Step 4 — Single-module reverse (`<ModuleName>`)

1. Read `specs/architecture.toml`. If missing, stop: "Run `/spec-reverse` (no args) first to produce architecture."
2. Read every existing `specs/modules/**/*.toml` for cross-module context.
3. Check if `specs/modules/**/<ModuleName>.toml` exists; if so, ask **Compare / Replace / Cancel**.
4. Locate source for the module. Heuristics:
   - Rust: `src/<snake_case>.rs`, `src/<snake_case>/mod.rs`, or a workspace member with that name.
   - C/C++: `src/<snake_case>.c`, `src/<snake_case>.cpp`, plus matching header.
   If not found at expected paths, ask the user.

### Extraction checklist

For each item, quote the source lines you used as evidence. Fields not evidenced are omitted.

- **`name`** — PascalCase derived from the file/module name.
- **`layer`** — infer from dependencies: direct register access → `pac` or `driver`; depends only on drivers → `middleware`; depends on middleware or has policy/business logic → `application`.
- **`description`** — one sentence stating the module's *purpose* at a level higher than the rest of the spec. Do not restate anything that is already captured by other fields: no listing of actions, state machines, ports, hardware, dependencies, or error variants. If the sentence could be deleted without losing information a reader can't get from scanning the rest of the file, rewrite it. Target ≤ 20 words. Example — good: *"Drives the onboard status LED with solid, blinking, and breathing patterns."* Bad: *"A driver module in the driver layer that exposes set_pattern and tick actions, owns GPIO pin PA5, uses a critical section for shared state, and implements a three-state FSM for the LED."*
- **`depends_on`** — imports/includes to other in-project modules.
- **`actions[]`** — each public function (`pub fn`, non-`static` C function, or exported C++ method). For each: `params` (inline if ≤3), `direction` from parameter mode (`in` / `out` / `inout`), `blocking` (true if it can await, loop on IO, or acquire a mutex — otherwise false), `isr_safe` (true only when the function is clearly safe from ISR context).
- **`state_machines[]`** — any enum consumed by a `match`/`switch` dispatching on current state; states are enum variants. Event enum, initial state, and transitions if the source dispatch is clear.
- **`ports[]`** — each trait the module implements (Rust) or vtable/PIMPL interface (C++) or function-pointer struct (C).
- **`hardware`** — pins, peripherals, clocks the module initializes or references. Cross-check against `specs/architecture.toml`.
- **`errors[]`** — variants of the module's error enum, or distinct return codes with stable names.
- **`metadata.MemoryModel`** — `allocation` must be one of `static_alloc` / `heap_alloc` / `stack_alloc`. Static allocations, `static mut`, `Box::leak`, fixed-size arrays → `static_alloc`. `Vec::new()`, `malloc`, `new` → `heap_alloc`.
- **`metadata.ConcurrencyModel`** — `protection` must be one of `none` / `critical_section` / `mutex` / `atomic` / `disable_interrupts`. Infer from the code's guards.
- **`metadata.ErrorHandling`** — `strategy` (panic vs return vs defer) and whether errors propagate.
- **`metadata.Ownership`** — `owner` module if any.

### Writing

Present the candidate TOML with confidence annotations **next to LOW fields only** (comments like `# LOW: inferred, please confirm`). Wait for confirmation. Write to `specs/modules/<ModuleName>.toml`.

Run `specdev-analyzer validate --project <project_root>`; if errors, show them and ask whether to iterate.

Report: source files analyzed, field confidence summary, action/state/error counts, validation result.

---

## Step 5 — UI reverse (`--ui`)

1. Read `specs/architecture.toml` and every `specs/modules/**/*.toml`.
2. If `specs/ui/*.toml` exists, ask **Compare / Replace / Merge**.

Scan for UI code:

- **Display** — driver chip, resolution, interface (SPI/I2C/parallel), color depth from init code.
- **Input devices** — GPIO pin + event kind (press/long-press/rotate) + debounce.
- **Indicators** — LEDs, per-state color, pattern.
- **Screens** — each rendering function is a candidate screen; elements are `text` / `image` / `shape` calls with data bindings.
- **Navigation** — transitions between screens, timeouts, global actions (e.g. long-press to sleep).

Read `<Schema dir>/ui.schema.yaml` for the exact field list. Present candidate `specs/ui/*.toml` with LOW-confidence comments. Write on confirmation.

Any `description` field (on the UI file, a screen, an element) is one sentence describing *purpose*. Do not summarize the widgets, inputs, or navigation defined elsewhere in the same file.

Run validate. Report file count, hardware/input/screen/nav counts, validation result.

---

## Step 6 — Full project reverse (`--all`)

Do these in order, pausing for confirmation between passes:

1. **Architecture pass** — run Step 3, wait for user confirmation of `specs/architecture.toml`.
2. **Project shape survey** — run Step 2.5. Output: confirmed `[[packages]]` list (if any) and the `(package, module_name, source_path_glob)` tuples for step 3. If packages are being introduced now, append them to `spec.toml` and create `specs/modules/<pkg>/` dirs before continuing.
3. **Parallel module reverse** — fan out one `module-reverser` subagent per confirmed `(package, module)` tuple in a **single assistant message** (one Agent tool use per tuple). Each subagent receives `module_name`, `project_root`, the `schema_dir` / `domain_dir` / `language_dir` paths from the context header, plus — when the tuple is packaged — `package` (snake_case name) and `source_path` (restrict evidence scan to this subtree). Do NOT write files during fan-out; subagents are read-only by design. When all subagents return, collect their Candidate TOML blocks and present them grouped by package, then in layer order (`pac` → `driver` → `middleware` → `application`) for **batch confirmation** — confirm / edit / skip each. Write confirmed files to `specs/modules/<package>/<Module>.toml` (or `specs/modules/<Module>.toml` for single-package projects) in one pass. If a subagent returned an error (source not found, ambiguous split), surface it alongside the candidates and let the user resolve before writing.
4. **UI detection** — if UI code is detected, ask whether to run Step 5.
5. **Validate everything** — run `specdev-analyzer validate --project <project_root>`. Surface layer violations and missing cross-references.
6. **Audit cross-check** — run `specdev-analyzer audit --project <project_root>`. This reports any source symbols not captured in the specs; for each Uncovered item, either add it to the module spec or record an intentional omission.

### Summary report

- **Architecture**: language, target, RTOS, build system, package list.
- **Modules**: table — name, layer, status (OK/SKIPPED/PARTIAL), LOW-confidence field count, source files.
- **UI**: included / not detected / skipped, file count.
- **Validation**: overall PASS/FAIL with per-module issues.
- **Audit**: Uncovered items by module.
- **Next steps**: review LOW fields, `/spec-edit <Module>` for any you want to refine, `/spec-gen <Module>` when you're ready to regenerate code.

---

## Rules of the road

- **Evidence over invention.** Every field must trace to source. If you cannot find evidence, leave the field out or mark LOW and ask.
- **Schema is the contract.** Every key you write must appear in `module.schema.yaml`, `architecture.schema.yaml`, or `ui.schema.yaml`.
- **One module per file.** Don't fold multiple concerns into a single TOML.
- **Stop on structural ambiguity.** If a source file plausibly represents two modules, ask before splitting.
