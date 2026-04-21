---
name: spec-init
description: This skill should be used when the user asks to "init specdev", "set up a new spec project", "scaffold a spec-driven project", "create spec.toml", "initialize specs/", or starts a new specdev project from scratch. Writes spec.toml, architecture.toml, and the specs/ directory structure.
allowed-tools: Read, Write, Edit, Bash, Glob, Grep
---

!`${CLAUDE_PLUGIN_ROOT}/scripts/resolve-context`

# /spec-init — Project scaffolding

Initialize a new spec-driven project. Follow these steps in order. Do not skip interview questions; every field lands in `spec.toml` or derives layout that later skills depend on.

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

Use extracted values as defaults for the Step 4 / Step 4b interviews. Do not silently overwrite — always show the defaulted value and let the user confirm or change it. Step 2 extractions that no domain consumes are harmless (they just aren't asked about).

## Step 3 — list available domains

List the subdirectories of the resolved `Domains` path (from the context header). Each directory is one installable domain (`firmware`, plus any the user added under `.specdev/domains/`).

## Step 4 — cross-domain interview

Ask all of these in one message. Show defaults from Step 2 in parentheses:

1. **Project name** (default: detected name)
2. **Domain** (default: `firmware` if available, else first from Step 3)
3. **Language** (`rust`, `c`, or `cpp` — default: detected from build files)
4. **Source layout** — `flat` (default) or `workspace` (infer `workspace` from `[workspace] members` in root `Cargo.toml`)

These four questions are domain-agnostic — every domain needs them. Domain-specific questions (if any) come in Step 4b.

Do not invent fields outside the project schema. Every interview answer — both here and in Step 4b — must have a declared destination in `spec.toml` or a schema-valid location in `specs/architecture.toml`. No `## Notes` escape hatches.

## Step 4b — load domain init.md and run extra interview

Resolve `<Domain dir>/<chosen-domain>/init.md` using the `<Domain dir>` path from the context header. If the file is absent, print:

```
Warning: domain '<name>' has no init.md — proceeding with cross-domain scaffolding only.
```

and skip to Step 5.

Otherwise read the file. It contains five H2 sections: `## Extra interview questions`, `## architecture.toml skeleton`, `## Extra scaffold directories`, `## Extra scaffold files`, `## Post-write notes`. Parse them — they drive Steps 4b, 6, and 8.

If `## Extra interview questions` is "None.", skip to Step 5. Otherwise ask each listed question in a single batched message, using Step 2 detections for defaults per the init.md's inference rules. Record the answers keyed by destination field.

## Step 5 — write spec.toml

Write `spec.toml` at the project root with this exact structure. Keys in this exact order. Do **not** emit explanatory comments — the only comment permitted in any written TOML is a top-of-file `#:schema` directive, which `spec.toml` does not use.

```toml
spec_version = "1.0"

[project]
name = "<from interview>"
domain = "<from interview>"
language = "<from interview>"

[source]
root = "src"
layout = "<flat or workspace>"

[paths.overrides]
```

Leave `[paths.overrides]` empty. Override keys (`schema`, `domain`, `language`, `codegen`) are documented in `<Schema dir>/project.schema.yaml` and D13 — users who need them will add them. Do not emit commented-out placeholders.

**If `layout = "workspace"`:** enumerate the root `Cargo.toml`'s `[workspace] members` (or `CMakeLists.txt` `add_subdirectory(...)` calls for C/C++). Append one `[[packages]]` entry per member with `source_path` set to the member's relative path. Example:

```toml
[[packages]]
name = "core"
source_path = "crates/core"

[[packages]]
name = "drivers"
source_path = "crates/drivers"
```

Package `name` must be snake_case and unique. Omit the `[[packages]]` section entirely for single-package (flat) projects.

Do not add any fields to `spec.toml` beyond the schema above. Per-domain fields (wiring, mounts, hardware, etc.) live in `specs/architecture.toml` or per-module/view TOMLs — never in `spec.toml`.

## Step 6 — create the specs/ tree

Create these directories under the project root (only if they do not exist):

- `specs/modules/`
- `specs/requirements/`

**If `[[packages]]` is declared:** also create one empty directory per declared package under `specs/modules/<pkg>/` and drop a `.gitkeep` file inside each. Module TOMLs will land there in later skills (`/spec-reverse`, `/spec-design`).

Write `specs/architecture.toml` using the `## architecture.toml skeleton` from the loaded `init.md`. Substitute any Step 4b interview answers into their declared destination fields. If the skeleton says "Emit an empty file." — or if no `init.md` was loaded (warning path in Step 4b) — write an empty file (no content, no `#:schema` directive). Emit **no comments** in any case — not explanatory prose, not commented-out sample entries. Every written key must be a live, schema-valid entry.

For each entry in the `init.md`'s `## Extra scaffold directories`, create the directory under the project root and drop a `.gitkeep` file in it (per the init.md's per-dir policy). For each entry in `## Extra scaffold files`, write the file with the specified contents.

Write `specs/requirements/README.md` with a one-paragraph explanation: "Each `.toml` file in this directory declares one or more requirements and, optionally, the external `[[source]]` documents (protocol specs, standards, user briefs) those requirements are derived from. Each requirement has an id (`REQ-<NAMESPACE>-NNN`) and may reference one or more source ids via `source = [...]` with a free-form `source_ref` locator. Modules reference requirements via their `satisfies = [...]` field. Run `specdev-analyzer validate` to check that every satisfied id is declared, every `source` id resolves, and every requirement is satisfied by at least one module."

Do NOT create any module TOMLs yet. Leave `specs/modules/` empty — that is the job of `/spec-design`.

## Step 7 — verify

Run `specdev-analyzer validate --project <project_root>` and show the output. Expect a clean report (there are no modules yet, so there is nothing to invalidate). If the analyzer complains about `architecture.toml`, fix the fields written in Step 6 based on its messages and re-run until clean.

## Step 8 — report

Summarize:
- Files created (one line each, with paths).
- The loaded `init.md`'s `## Post-write notes` section, reproduced verbatim under a **Domain notes** heading. Skip this if no `init.md` was loaded.
- Next skill: `/spec-design <one-line module description>` to add the first module.
