---
name: spec-design-ui
description: This skill should be used when the user asks to "design the firmware UI", "add a screen", "set up navigation", "add input devices", or needs to author firmware UI specs (displays, inputs, indicators, screens, navigation) under specs/ui/. Firmware-domain only — for egui / web / mobile projects the UI lives inside view modules; use /spec-design instead. Authors TOML UI specs interactively.
allowed-tools: Read, Write, Edit, Bash, Glob, Grep
argument-hint: <one-line UI description>
---

!`${CLAUDE_PLUGIN_ROOT}/scripts/resolve-context`

# /spec-design-ui — UI authoring

Design the UI surface for the project. Output is a set of TOMLs under `specs/ui/`. `$ARGUMENTS` is a natural-language description of the UI; if empty, ask the user for a description with 2–3 examples of expected detail level.

The v1 UI schema is deliberately permissive — **inner items accept unknown keys**. That means authoring ergonomics wins over strictness. Still, every *known* key emitted must match `ui.schema.yaml` exactly, and required keys (`name`, `Navigation.initial`, `Navigation.states`) must be present.

---

## Step 0 — prerequisites

This skill is firmware-only. Check, in order:

1. If the context header reports the analyzer is not on PATH, or reports no `spec.toml`, stop and tell the user to run `/spec-init`.
2. Read `spec.toml`'s `[project] domain` field. If it is not `firmware`, stop with: *"This skill is firmware-only — it models physical displays, GPIO inputs, and indicator LEDs. For `<domain>` projects, UI lives inside view/screen modules — use `/spec-design` directly."* Do not proceed.
3. If `specs/architecture.toml` is missing, stop and tell the user to run `/spec-design` for at least the architecture first.

## Step 1 — read the authoritative guides

1. `<Schema dir>/ui.schema.yaml` — the contract. Note the five top-level arrays (`displays`, `input_devices`, `indicators`, `screens`, `navigation`) and their required vs optional keys.
2. `<Schema dir>/enums.schema.yaml` — `Identifier` pattern applies to `Navigation.initial`, `states[]`, and transition `from/to/event`.
3. `<Domain dir>/author.md` — whether the project has conventions for UI naming or structure.
4. `specs/architecture.toml` — for hardware that is already declared (reuse it, do not re-declare pins).
5. Every `specs/modules/**/*.toml` — data-source bindings (screen elements often read module state).
6. Any existing `specs/ui/**/*.toml` — user may be extending.

## Step 2 — multi-phase design

Present each phase as a table or layout and **wait for approval between phases**. Do not design everything in one shot.

### 2a — Hardware

Identify displays, input devices, indicators. For each display: `name` (PascalCase), `width`, `height`, plus free-form keys for `driver`, `interface`, `color_depth`. For input devices: `name`, `kind` (e.g. `button`, `rotary`, `touch`). Indicators: `name`, `kind` (e.g. `rgb_led`, `pwm_led`).

Cross-reference pins/peripherals in `specs/architecture.toml`. If a pin is not declared there yet, offer to add it in a follow-up.

### 2b — Input vocabulary

Define the gesture set usable by navigation: `press`, `long_press`, `double_press`, `rotate_cw`, `rotate_ccw`, etc. Keep these as free-form keys on each `InputDevice` entry (schema allows it via `additionalProperties: true`).

### 2c — Indicators

For each indicator, describe its states and the data bindings. Example pattern — `kind = "rgb_led"`, plus a free-form `states = [{name = "booting", color = "yellow", pattern = "blink"}, ...]`. Bind to module state where possible.

### 2d — Screens

Design **one screen at a time**. For each, present:

- Pixel-level ASCII layout at the display's resolution (scaled).
- Element list: type (`text` / `image` / `shape`), position, size, font, data binding.
- Refresh strategy (`full` / `partial` / `dirty_regions`).

Record them as free-form keys under each `Screen` entry.

### 2e — Assets

List fonts and icons referenced by screens. These are free-form — add an `assets` top-level table only if the user prefers that layout; otherwise keep them inline under each screen.

### 2f — Navigation

Build `[navigation]`:

- `initial` — the starting screen.
- `states` — every screen name that appears as `from` or `to`.
- `transitions` — `from`, `to`, `event`. Event names must match gestures defined in 2b.

Validate mentally: every transition's `from`/`to` is in `states`, every `event` is one of the declared input gestures, and there are no orphan screens unreachable from `initial`.

## Step 3 — check dependencies

Verify but do NOT block on missing dependencies:

- Data-bound modules exist under `specs/modules/**/*.toml`.
- Display/input pins appear in `specs/architecture.toml`.
- If a binding refers to a module that does not exist, list it as a follow-up for `/spec-design <Module>`.

Report status with check/cross marks for each dependency.

## Step 4 — write the TOMLs

Layout:

- `specs/ui/<UIName>.toml` for a single-screen or small UI.
- `specs/ui/hardware.toml`, `specs/ui/screens/<ScreenName>.toml`, `specs/ui/navigation.toml` for larger UIs — one concern per file. Choose the simpler layout unless there are ≥3 screens.

Each TOML must start with `name = "..."` (required). Use inline tables for short arrays; switch to `[[array_of_tables]]` when an item has ≥3 fields.

**Comment policy** — follow `specdev/shared/comment-policy.md`. This skill does not emit `# LOW:` markers (authoring, not inference).

## Step 5 — validate

Run `specdev-analyzer validate --project <project_root> --no-color`. For each error, map the rule id to a fix:

- `Identifier` pattern violations → rename (snake_case for events/states, PascalCase for names).
- Missing `name` on a display / input / indicator / screen → add it.
- `Navigation.transitions` referencing an unknown state → add the state or correct the reference.

Fix mechanical issues; ask about design-level changes.

## Step 6 — report

- Hardware counts (displays, inputs, indicators).
- Input vocabulary size.
- Screen count with element counts.
- Navigation: initial screen, state count, transition count.
- Files written.
- Dependency status.
- Validation result.
- Suggested next steps: review specs, `/spec-design <Module>` for any missing data-source modules, then `/spec-gen-ui` when ready.
