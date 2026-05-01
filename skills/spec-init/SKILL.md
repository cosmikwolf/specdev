---
name: spec-init
description: This skill should be used when the user asks to "init specdev", "set up a new spec project", "scaffold a spec-driven project", "bootstrap a profile", "create spec.toml", "initialize specs/", or starts a new specdev project from scratch. Wraps `specdev-analyzer bootstrap`, picks a profile (firmware-layered / library-api), and seeds the `specs/specdev/` tree. Use for empty or greenfield projects; to bootstrap specs from an existing source tree, use /spec-reverse.
allowed-tools: Read, Write, Edit, Bash, Glob, Grep
---

# /spec-init — Project scaffolding

Initialize a new spec-driven project by interviewing the user, then delegating file emission to `specdev-analyzer bootstrap`. The analyzer is the source of truth for what a freshly-bootstrapped project looks like; this skill is a thin wrapper that picks the profile, fills the architecture's required fields, and verifies the result.

The current model is **profiles + blocks + bridges** (doc 15). The legacy "domains" model is retired — profiles own project shape; blocks contribute concrete frameworks/libraries/hardware; bridges wire pairs of blocks.

If `specdev-analyzer` is not on PATH, tell the user to install it (`cargo install --git https://github.com/tenkai/specdev-analyzer`) and stop. Do not attempt to write files without the analyzer.

---

## Step 1 — refuse to clobber

If `<project_root>/specs/specdev/` already exists, the project is already bootstrapped. Read `spec.toml` and show the current `[project]` block. Ask:

- **Keep** — leave it untouched, stop here.
- **Update one field** — point the user at a specific file (`spec.toml` for name/license/description; `architecture.toml` for `[platform]`/`[runtime]`/etc.). This skill does not edit existing projects.
- **Re-bootstrap into a fresh dir** — confirm a new path, treat that as the target.

If `specs/specdev/` is absent, continue — bootstrapping it is the whole point of this skill.

## Step 2 — detect existing build metadata

Scan the project root for build manifests and extract fields per `references/build-manifest-detection.md`. Use extracted values as defaults for the Step 4 interview — show every defaulted value and let the user confirm or change it. This step is read-only; nothing is written until Step 5.

## Step 3 — list available profiles

Profiles ship in the plugin tree. List them:

```sh
ls "${CLAUDE_PLUGIN_ROOT}/profiles/"
```

For each profile name, read `${CLAUDE_PLUGIN_ROOT}/profiles/<name>/README.md` and summarize in one line. As of v1 the canonical profiles are:

- **`firmware-layered`** — embedded Rust on Cortex-M, layered architecture (PAC → HAL → driver → middleware → application), ports + connections wiring, hardware ownership.
- **`library-api`** — pure Rust library, visibility-driven module boundaries, feature gates, no hardware concepts.

If the user added a custom profile under their plugin tree, surface it here as well — do not hardcode the list.

## Step 4 — interview

Ask all of these in one message. Show defaults from Step 2 in parentheses:

1. **Project name** — lowercase, digits, underscores, hyphens; must start with a letter; ≤64 chars. (default: detected `[package].name` or directory name)
2. **Profile** — one of the names listed in Step 3. (default: `firmware-layered` if a `Cargo.toml` with `cortex-m`/`embassy`/`stm32*` deps was detected; `library-api` if a non-bin `Cargo.toml` was detected; otherwise ask)
3. **Language** — `rust` is the only option in v1. C lands in Phase 9. (default: `rust`)
4. **Bootstrap directory** — usually the current working directory, but the analyzer creates it if absent and refuses if `specs/specdev/` is already present. (default: `.`)

These four questions are all the schema requires. Profile-specific decisions (target triple, MCU part, license, MSRV, …) live in `architecture.toml` / optional `[project]` fields and are filled in Step 6, after the analyzer has emitted templates.

## Step 5 — run bootstrap

Run from the project root:

```sh
specdev-analyzer bootstrap \
  --profile <profile> \
  --language <language> \
  --name <project_name> \
  <bootstrap_dir>
```

This emits the full `specs/specdev/` machinery (schemas, reductions, codegen prose for the chosen profile), plus templates: `spec.toml`, `specs/architecture.toml`, `specs/requirements.toml`, and the empty `specs/modules/` directory. Templates carry profile-specific commented hints.

If the analyzer fails:
- `BOOTSTRAP-001` — target tree already exists or profile name invalid; surface message verbatim.
- `BOOTSTRAP-002` — unknown placeholder in profile template (a plugin authoring bug); ask the user to file an issue against the plugin.
- `PLUGIN-001` — plugin root not resolvable; tell the user to set `CLAUDE_PLUGIN_ROOT` or pass `--plugin-root`.

Do not attempt to repair these errors by hand-writing files.

## Step 6 — confirm or fill architecture defaults

Read the emitted `specs/architecture.toml`. The template ships pre-filled with sane defaults; bootstrap-emitted architecture is valid as-is. This step is interview-driven refinement, not error recovery — only edit fields the user actually wants to change.

### firmware-layered

Pre-filled, confirm or change:
- `[platform].target_triple` — `thumbv8m.main-none-eabihf` (Cortex-M33 / STM32H5). Common alternatives: `thumbv7em-none-eabihf` (M4F/M7F; STM32F4/H7), `thumbv6m-none-eabi` (M0/M0+).
- `[platform.mcu_core]` / `[platform.mcu_runtime]` — `cortex-m` / `cortex-m-rt`.
- `[runtime.panic_handler]` — `panic-probe`.
- `[logger.provider]` / `[logger.backend]` — `defmt` / `defmt-rtt`.

Commented in the template; uncomment + set if the user knows the part:
- `[platform].mcu_part` (e.g., `"stm32h563"`, `"stm32f411"`, `"nrf52840"`) — drives the inlined MCU block's HAL feature flag at codegen time. Optional in the schema, but strongly recommended; without it, codegen emits a `// SPEC WARNING` comment.

Optional (offer but don't insist):
- `[runtime.allocator]` (e.g., `embedded-alloc`) — only if modules need `alloc`.
- `[runtime.critical_section_impl]` — only when overriding `cortex-m`'s bundled impl.
- `[[external_deps]]` — project-wide crates beyond the typed slots (e.g., `heapless`, `static-cell`).

### library-api

The library-api architecture template is minimal — no required fields. Offer:
- `msrv = "1.75"` — minimum supported Rust version, becomes `Cargo.toml`'s `rust-version`.
- `[[features]]` entries — Cargo features the library exposes.

### Optional `[project]` fields (both profiles)

Write to `spec.toml` only if the user asks:
- `edition` — `"2018"`, `"2021"` (default), or `"2024"`.
- `license` — SPDX identifier (e.g., `"MIT OR Apache-2.0"`).
- `description` — one-line, ≤200 chars (libraries should set this; it appears on crates.io).

## Step 7 — verify

Run from the project root:

```sh
specdev-analyzer validate --no-color
```

Expect a clean report — bootstrap emits valid templates, and Step 6's optional refinements stay within schema. If `SCHEMA-ARCH` or `SCHEMA` errors appear, the user's Step 6 edits violated a field constraint; fix the offending field based on the message — do not delete fields to silence checks. Re-run until clean.

## Step 8 — report

Summarize:
- Files created (one line each, with paths).
- The selected profile and a one-line reminder of what it shapes (firmware-layered → layered embedded; library-api → Rust library).
- Validate result.
- **Suggested next steps**:
  - **firmware-layered** — inline at least one MCU block + a runtime block before authoring modules: `specdev-analyzer inline stm32h5-rs && specdev-analyzer inline embassy && specdev-analyzer inline-bridge embassy+stm32h5-rs`. Then `/spec-design <one-line module description>`.
  - **library-api** — go straight to `/spec-design <one-line module description>` to add the first module.

## What this skill does NOT do

- **Author module specs.** That's `/spec-design`'s job. Bootstrap leaves `specs/modules/` empty.
- **Inline blocks or bridges.** The user (or a future `/spec-block` skill) drives `specdev-analyzer inline` / `inline-bridge`.
- **Edit an existing project.** Re-running `spec-init` over a bootstrapped tree is refused at Step 1; for changes, edit the relevant template-emitted file directly.
- **Pick MCU hardware.** It asks the user; it does not auto-detect from `.cargo/config.toml` target triples (the architecture template defaults are sane for the most common case but require confirmation).

## Additional resources

- `references/build-manifest-detection.md` — manifests scanned in Step 2 and fields extracted as interview defaults.
- `references/requirements-readme-template.md` — verbatim body for an optional `specs/requirements/README.md` file (write only if the project intends to track requirements).
