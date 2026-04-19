---
name: spec-gen-ui
description: Generate UI rendering code from UI specs, or emit Mermaid interaction diagrams.
allowed-tools: Read, Write, Edit, Bash, Glob, Grep
argument-hint: [--diagram]
---

!`${CLAUDE_PLUGIN_ROOT}/scripts/resolve-context`

# /spec-gen-ui — UI code generation + diagrams

You turn `specs/ui/*.toml` into either rendering code or interaction diagrams. Never invent functionality not present in the specs. If a field is ambiguous, stop and ask.

Modes, decided from `$ARGUMENTS`:

| Input | Mode |
|-------|------|
| `--diagram` | Diagram mode (Step 5) |
| empty or anything else | Code generation mode (Step 2) |

---

## Step 0 — prerequisites

If the context header reports the analyzer is not on PATH, or reports no `spec.toml`, stop and tell the user to run `/spec-init`. If `specs/ui/` is empty or absent, stop and tell the user to run `/spec-design-ui` first.

## Step 1 — read the authoritative guides

1. `specs/architecture.toml` — language, RTOS, build system, external packages.
2. `<Schema dir>/ui.schema.yaml` — the contract.
3. `<Language dir>/syntax.md` — file naming, types, import style for the project's language.
4. `<Codegen dir>/gen-ui.md` — the domain + language UI implementation patterns (this is the authoritative playbook).
5. `<Domain dir>/author.md` and any `<Codegen dir>/gen-module.md` — so UI code matches module style.

---

## Step 2 — validate and extract (code gen mode)

1. Run `specdev-analyzer validate --project <project_root> --no-color`. If it fails, print the output and stop.
2. Run `specdev-analyzer extract --project <project_root> --output /tmp/specdev-extract-<pid>`. Read `/tmp/specdev-extract-<pid>/` for structured UI data: displays, input_devices, indicators, screens, navigation.

Hold the extracted YAML in mind as authoritative. Do not re-read the TOML for fields that already appear in the extract.

## Step 3 — freshness check

For each file to generate, compute SHA-256 over its dependent spec files. Check existing generated files for `@spec-hash` header comments.

Per file, classify as **GENERATE** (new), **SKIP** (hash matches), **REGENERATE** (hash differs), or **WARN** (no hash — ask the user: overwrite / skip / diff). If everything is up-to-date, report and stop.

Dependency mapping (adjust based on codegen `gen-ui.md`):

- `src/ui/assets.*` ← `specs/ui/assets.toml` (or inline assets in screen files).
- `src/ui/screens/<name>.*` ← `specs/ui/screens/<name>.toml` (or the corresponding section of a single UI TOML) + assets + display driver.
- `src/ui/navigation.*` ← `specs/ui/navigation.toml` + input devices + all screen specs.
- `src/ui/input.*` ← input devices + hardware.
- `src/ui/indicators.*` ← indicators + hardware.
- `src/ui/mod.*` ← every UI spec file.

## Step 4 — generate

Follow `<Codegen dir>/gen-ui.md` verbatim for element types, timing primitives, and file structure. Each generated file must begin with a `@spec-hash: <sha256>` comment so `/spec-gen-ui` can detect drift on re-runs.

Emit, in this order, skipping any marked SKIP:

1. **Assets** — compiled font/icon data.
2. **Screens** — one draw function per screen, calling the display interface.
3. **Navigation** — screen enum, transition state machine, timeout handling.
4. **Input dispatch** — gesture detection, debounce, combo/sequence logic.
5. **Indicators** — LED state bindings and pattern timing.
6. **UI module root** — init, update, re-exports, context struct.
7. **Test stubs** — one per screen, one for navigation, one per input device.

Report: files generated / skipped / regenerated, spec coverage, any missing assets, suggested next steps.

---

## Step 5 — diagrams (`--diagram`)

Do not run validate or extract — diagrams are TOML-only.

1. Read `specs/ui/navigation.toml` (or the nav section of the single UI TOML). If missing, stop: tell the user to run `/spec-design-ui` first.
2. Read every other file under `specs/ui/**/*.toml`.
3. Read every `specs/modules/**/*.toml` referenced by UI data bindings.

Generate all four diagrams and present with section headers:

### 5a — Navigation flow

Mermaid `stateDiagram-v2`. Each screen is a state, each transition is an arrow labeled by event. Timeouts render as `--> target : timeout(Xs)`. Self-transitions for non-navigate actions. Global actions rendered as a `note` beside the diagram.

### 5b — Data flow

Mermaid `flowchart LR`. Modules → screens → indicators. Arrows reversed for screens that invoke module actions.

### 5c — Indicator state map

Mermaid `stateDiagram-v2`. Per-indicator states with color/pattern. Transitions labeled by the bound field value. Skip entirely if no indicators.

### 5d — Input map table

Markdown table. Columns: `Input`, `Event`, `Screen`, `Action`, `Target screen`, `Notes`. Include global actions at the top.

Suggest next steps: verify navigation matches intended UX; run `/spec-validate`; run `/spec-gen-ui` (no args) when ready to emit code.
