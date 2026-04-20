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

Run validate. Report file count, hardware/input/screen/nav counts, validation result.

---

## Step 6 — Full project reverse (`--all`)

Do these in order, pausing for confirmation between passes:

1. **Architecture pass** — run Step 3, wait for user confirmation of `specs/architecture.toml`.
2. **Module discovery** — list discovered modules grouped by layer, ask the user to confirm the set (default: all). Allow exclusions.
3. **Batch module reverse** — for each confirmed module, in dependency order (`pac` → `driver` → `middleware` → `application`), run Step 4 extraction logic. Use **batch confirmation**: present all candidate TOMLs together, then the user confirms / edits / skips each. Write in one pass.
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
