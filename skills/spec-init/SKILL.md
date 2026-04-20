---
name: spec-init
description: Scaffold a new spec-driven project — writes spec.toml and the specs/ directory structure.
allowed-tools: Read, Write, Edit, Bash, Glob, Grep
---

!`${CLAUDE_PLUGIN_ROOT}/scripts/resolve-context`

# /spec-init — Project scaffolding

You are initializing a new spec-driven project. Follow these steps in order. Do not skip interview questions; every field lands in `spec.toml` or derives layout that later skills depend on.

If `specdev-analyzer` is not on PATH (the context header above will say so), tell the user to install it and stop. Do not attempt to write files without the analyzer.

The context header will either show a resolved `spec.toml` (the project was already initialized) or report **Status: no `spec.toml` found**. Both are valid starting states for this skill.

---

## Step 1 — refuse to clobber

If the context header shows a resolved `spec.toml`, read it and show the current contents. Ask:

- **Keep** — leave it untouched, stop here.
- **Rewrite** — continue, confirm each field one by one, overwrite at the end.

If the context header reports no `spec.toml`, continue — creating one is the whole point of this skill.

## Step 2 — detect existing build metadata

Scan the project root for build manifests. For each present file, extract fields:

| File | Extract |
|---|---|
| `Cargo.toml` | `[package] name`, `[package] edition`, `[package] rust-version`, `[dependencies]` names, `[workspace] members` |
| `.cargo/config.toml` | `[build] target` triple, `[env]` keys (MCU, RTOS hints), `build-std` presence |
| `rust-toolchain.toml` | `channel` |
| `CMakeLists.txt` | `project(...)` name, `set(CMAKE_SYSTEM_PROCESSOR ...)`, linked libraries |
| `sdkconfig.defaults` | all `CONFIG_*` lines (ESP-IDF) |
| `platformio.ini` | `board`, `framework`, `lib_deps` |

Use extracted values as defaults in Step 4's interview. Do not silently overwrite — always show the defaulted value and let the user confirm or change.

## Step 3 — list available domains

List the subdirectories of the resolved `Domains` path (from the context header). Each directory is one installable domain (`firmware`, plus any the user added under `.specdev/domains/`).

## Step 4 — interview

Ask all of these in one message. Show defaults from Step 2 in parentheses:

1. **Project name** (default: detected name)
2. **Domain** (default: `firmware` if available, else first from Step 3)
3. **Language** (`rust`, `c`, or `cpp` — default: detected from build files)
4. **Target MCU / architecture** — only if domain is `firmware` (free-form, e.g. `esp32-s3`)
5. **RTOS** — `none`, `freertos`, `zephyr`, `embassy`, `rtic` (only for firmware; default: inferred from build files, e.g. esp-idf-svc → freertos)
6. **Environment** — `std` or `no_std` (firmware only; default: inferred from RTOS + `build-std`)
7. **Source layout** — `flat` (default) or `workspace` (infer `workspace` from `[workspace] members` in root `Cargo.toml`)

Do not invent fields that are not in the project schema. If an interview answer has no home in `spec.toml` today, record it in a `## Notes` section of `specs/architecture.toml` instead.

## Step 5 — write spec.toml

Write `spec.toml` at the project root with this exact structure. Keys in this exact order:

```toml
spec_version = "1.0"

[project]
name = "<from interview>"
domain = "<from interview>"
language = "<from interview>"

[source]
root = "src"
layout = "<flat or workspace>"

# Project-local overrides: uncomment to point specific kinds at a path under
# this project, overriding .specdev/ and plugin defaults (see D13).
[paths.overrides]
# schema = "./.specdev/schema/firmware/v1"
# domain = "./.specdev/domains/firmware"
# language = "./.specdev/languages/rust"
# codegen = "./.specdev/codegen/firmware-rust"
```

Do not add `target`, `rtos`, `environment`, or similar fields to `spec.toml` — those are firmware metadata and belong in `specs/architecture.toml`.

## Step 6 — create the specs/ tree

Create these directories under the project root (only if they do not exist):

- `specs/modules/`
- `specs/requirements/`

Write `specs/architecture.toml` with this skeleton. Populate `target`, `rtos`, `environment`, `external_packages` from the interview and build-file detection:

```toml
# Project-wide architecture — one per project.
# Validated by: specdev-analyzer validate

[target]
mcu = "<from interview, e.g. esp32-s3>"
architecture = "<xtensa | arm | riscv | ...>"
toolchain = "<e.g. esp, nightly, stable>"

[runtime]
rtos = "<none | freertos | zephyr | embassy | rtic>"
environment = "<std | no_std>"

# Modules are auto-discovered under specs/modules/ — do not list them here.

# External packages referenced by modules via external_deps[].
# For each dependency your modules will use, declare how to pull it in.
[[external_packages]]
name = "example_pkg"
source = "crates.io"            # or git, path, esp_idf, etc.
build_integration = { cargo = 'example_pkg = "0.1"' }

[connections]
# Top-level connections between modules (wiring / flows). Optional for minimal projects.
# [[connections.flows]]
# name = "audio_pcm_flow"
# from = "BtA2dpSink.pcm_out"
# to   = "AudioPipeline.pcm_in"
```

Write `specs/requirements/README.md` with a one-paragraph explanation: "Each `.toml` file in this directory declares one or more requirements. Modules reference requirements via their `satisfies = [...]` field. Run `specdev-analyzer validate` to check that every satisfied id is declared and every requirement is satisfied by at least one module."

Do NOT create any module TOMLs yet. Leave `specs/modules/` empty — that is the job of `/spec-design`.

## Step 7 — verify

Run `specdev-analyzer validate --project <project_root>` and show the output. Expect a clean report (there are no modules yet, so there is nothing to invalidate). If the analyzer complains about `architecture.toml`, fix the fields you wrote in Step 6 based on its messages and re-run until clean.

## Step 8 — report

Summarize:
- What files you created (one line each, with paths).
- What the next skill is: `/spec-design <one-line module description>` to add the first module, or open `specs/architecture.toml` to refine wiring.
- Any interview answer that did not land in a file (record them in a `Notes` section at the end of `architecture.toml` so they are not lost).
