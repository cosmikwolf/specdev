---
name: spec-init
description: This skill should be used when the user asks to "init specdev", "set up a new spec project", "scaffold a spec-driven project", "bootstrap a profile", "create spec.toml", "initialize specs/", or starts a new specdev project from scratch. Wraps `specdev-analyzer bootstrap`, picks a profile from whatever ships under `${CLAUDE_PLUGIN_ROOT}/profiles/` (including end-user-authored profiles), and seeds the `specs/specdev/` tree. Use for empty or greenfield projects. For non-interactive scaffolding with explicit args, use /spec-bootstrap. To bootstrap specs from an existing source tree, use /spec-reverse.
allowed-tools: Read, Write, Edit, Bash, Glob, Grep
---

# /spec-init — Project scaffolding

Initialize a new spec-driven project by interviewing the user, then delegating file emission to `specdev-analyzer bootstrap`. The analyzer is the source of truth for what a freshly-bootstrapped project looks like; this skill is a thin wrapper that picks the profile, fills the architecture's required fields, and verifies the result.

The current model is **profiles + blocks + bridges** (doc 15). The legacy "domains" model is retired — profiles own project shape; blocks contribute concrete frameworks/libraries/hardware; bridges wire pairs of blocks.

If `specdev-analyzer` is not on PATH, tell the user to install it (`cargo install --git https://github.com/cosmikwolf/specdev-analyzer`) and stop. Do not attempt to write files without the analyzer.

---

## Step 0 — session env preamble

Run the export from `${CLAUDE_PLUGIN_ROOT}/shared/plugin-root-preamble.md` before any analyzer invocation:

```sh
export SPECDEV_PLUGIN_ROOT="${CLAUDE_PLUGIN_ROOT}"
```

See the shared file for the full rationale and skip-conditions. Short version: `CLAUDE_PLUGIN_ROOT` is set in this skill body but is **not** inherited by Bash subshells, so exporting `SPECDEV_PLUGIN_ROOT` once makes plugin-root resolution work for the rest of the session.

## Step 1 — refuse to clobber

If `<project_root>/specs/specdev/` already exists, the project is already bootstrapped. Read `spec.toml` and show the current `[project]` block. Ask:

- **Keep** — leave it untouched, stop here.
- **Update one field** — point the user at a specific file (`spec.toml` for name/license/description; `architecture.toml` for `[platform]`/`[runtime]`/etc.). This skill does not edit existing projects.
- **Re-bootstrap into a fresh dir** — confirm a new path, treat that as the target.

If `specs/specdev/` is absent, continue — bootstrapping it is the whole point of this skill.

## Step 2 — detect existing build metadata

Scan the project root for build manifests and extract fields per `references/build-manifest-detection.md`. Use extracted values as defaults for the Step 4 interview — show every defaulted value and let the user confirm or change it. This step is read-only; nothing is written until Step 5.

## Step 3 — list available profiles

Profiles ship in the plugin tree. List them dynamically — never hardcode the set, since end users may author their own profiles:

```sh
ls "${CLAUDE_PLUGIN_ROOT}/profiles/"
```

For each profile name returned, read `${CLAUDE_PLUGIN_ROOT}/profiles/<name>/README.md` and surface one line per profile (the README's first paragraph or a one-sentence summary). Present the list to the user verbatim — every directory under `profiles/` is a valid choice; user-authored profiles surface here the same way as plugin-shipped ones.

## Step 4 — interview

Ask all of these in one message. Show defaults from Step 2 in parentheses:

1. **Project name** — lowercase, digits, underscores, hyphens; must start with a letter; ≤64 chars. (default: detected `[package].name` or directory name)
2. **Profile** — one of the names listed in Step 3. Default heuristics for shipped profiles: `firmware-layered` if a `Cargo.toml` with `cortex-m`/`embassy`/`stm32*`/`nrf*`/`esp*` deps was detected; `library-api` if a non-bin `Cargo.toml` was detected; `audio-plugin` if a `nih-plug` / `vst3-sys` / `clack-host` dep was detected; `rust-workspace` if `[workspace]` was detected. Otherwise ask. For user-authored profiles no default is inferred — ask.
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
- `PLUGIN-001` — plugin root not resolvable. Run Step 0's `export SPECDEV_PLUGIN_ROOT="${CLAUDE_PLUGIN_ROOT}"` if you skipped it; otherwise pass `--plugin-root <path-to-specdev-plugin>` explicitly. Do not rely on `CLAUDE_PLUGIN_ROOT` propagating into Bash subshells.

Do not attempt to repair these errors by hand-writing files.

## Step 6 — confirm or fill architecture defaults

Read the emitted `specs/architecture.toml`. The template ships pre-filled with sane defaults; bootstrap-emitted architecture is valid as-is. This step is interview-driven refinement, not error recovery — only edit fields the user actually wants to change.

The illustrative refinement shapes below cover the **shipped profiles**. For any other profile (including user-authored), read `${CLAUDE_PLUGIN_ROOT}/profiles/<profile>/README.md` and the emitted `specs/architecture.toml` together — the README enumerates which architecture-template fields the profile considers required vs optional, and the emitted template carries commented hints. Apply the same "only edit what the user actually wants to change" rule.

### firmware-layered (illustrative)

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

### library-api (illustrative)

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
- **Suggested next steps**: shape depends on the profile.
  - **firmware-layered** — inline at least one MCU block + a runtime block + the matching bridge before authoring modules. List shipped MCU/runtime blocks with `ls "${CLAUDE_PLUGIN_ROOT}/blocks/"` and shipped bridges with `ls "${CLAUDE_PLUGIN_ROOT}/bridges/"`; pick the pair matching the project's MCU and runtime. Example for STM32H5 + Embassy: `/spec-inline stm32h5-rs && /spec-inline embassy && /spec-inline-bridge embassy+stm32h5-rs`. Then `/spec-design <one-line module description>`.
  - **library-api** — go straight to `/spec-design <one-line module description>` to add the first module.
  - **audio-plugin** — inline an audio framework block (e.g. `nih-plug`) and optionally a UI block (e.g. `egui`) plus the matching bridge if the framework + UI are paired. Example: `/spec-inline nih-plug && /spec-inline egui && /spec-inline-bridge egui+nih-plug`. Then `/spec-design <plugin shell module>`.
  - **rust-workspace** — bootstrap each member crate with its own profile (`cd <member> && /spec-bootstrap --profile <profile> --language rust --name <member-name>`).
  - **other / user-authored profile** — read the profile's README at `${CLAUDE_PLUGIN_ROOT}/profiles/<profile>/README.md`; the README is the authoritative source for what to do next.

## What this skill does NOT do

- **Author module specs.** That's `/spec-design`'s job. Bootstrap leaves `specs/modules/` empty.
- **Inline blocks or bridges.** Use `/spec-inline <block>` and `/spec-inline-bridge <a>+<b>` after bootstrap.
- **Edit an existing project.** Re-running `spec-init` over a bootstrapped tree is refused at Step 1; for changes, edit the relevant template-emitted file directly.
- **Pick MCU hardware.** It asks the user; it does not auto-detect from `.cargo/config.toml` target triples (the architecture template defaults are sane for the most common case but require confirmation).

## Additional resources

- `references/build-manifest-detection.md` — manifests scanned in Step 2 and fields extracted as interview defaults.
- `references/requirements-readme-template.md` — verbatim body for an optional `specs/requirements/README.md` file (write only if the project intends to track requirements).
