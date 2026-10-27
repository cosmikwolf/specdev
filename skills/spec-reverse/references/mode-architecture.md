# spec-reverse — architecture mode

Entered from the main SKILL.md when `$ARGUMENTS` is empty or missing. Produces `specs/architecture.toml` from source-tree evidence. Do not proceed to per-module reversal in this mode; that is `/spec-reverse <Module>` or `/spec-reverse --all`.

## Architecture reverse

**First, read the project's `architecture.schema.yaml`.** It says which fields the architecture has. When the evidence below fills none of them (a library profile whose architecture holds only `project_decisions`, and the project has no decisions), write nothing: leave `specs/architecture.toml` as bootstrap wrote it, or absent, and say so in the report. `check` reports an absent file as info only.

If `specs/architecture.toml` holds entries a person wrote (more than the profile's `architecture.toml.template`), ask: **Compare (dry-run diff) / Replace / Cancel**. When it holds only the template, edit it in place without asking, and keep the template's comments.

Scan the project root for evidence in this order:

1. **Language** — `[project].language` in `spec.toml`; confirm it against the file extensions.
2. **Build system and manifest** — `Cargo.toml`, `package.json` (+ `tsconfig.json`), `pubspec.yaml`, `pyproject.toml`; for firmware also `CMakeLists.txt` / `Makefile` / `meson.build`, `platformio.ini`.
3. **Target and runtime** — only where the architecture schema has fields for them: MCU defines, linker scripts, HAL crates, RTOS markers (RTIC, Embassy, FreeRTOS, Zephyr); the platforms of a Flutter app; the Python versions in `requires-python`.
4. **External packages** — the manifest's dependencies; present unrecognized ones for confirmation.
5. **Modules** — candidates from the source tree under `[paths] source` (see `mode-full-project.md`, Step A, for the classification and for generated files).
6. **Peripherals and pins** (firmware) — peripheral initialization (`Peripherals::take`, `gpio_init`, …), pin numbers and peripheral names.
7. **Connections** — wiring between modules: `specdev-analyzer report imports --root <source>` gives each in-project import.

Write only fields the project's `architecture.schema.yaml` defines; evidence for anything else goes to the user as a note.

**When the evidence disagrees** (a generator's metadata lists six platforms, the notes show four tested, the tree has folders for two), prefer what the project shows it builds or runs over what a tool generated, mark the field `# LOW:` and ask.

Present candidate `specs/architecture.toml` with confidence (HIGH/MEDIUM/LOW) per field. Wait for confirmation. Write clean TOML per `${CLAUDE_PLUGIN_ROOT}/shared/comment-policy.md` — in particular, LOW markers are permitted inline next to LOW-confidence fields and are removed after the user confirms each one. Scaffold `specs/modules/`, `specs/requirements/` and `specs/sources/` if absent.

Report: language, build system, the architecture fields written (or why none), packages, module candidates. Suggested next step: `/spec-reverse <Module>` or `/spec-reverse --all`.
