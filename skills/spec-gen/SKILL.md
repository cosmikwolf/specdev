---
name: spec-gen
description: This skill should be used when the user asks to "generate code", "codegen", "run spec-gen", "generate <ModuleName>", "codegen all modules", or "implement this spec". Generates Rust source code AND Cargo.toml from validated specdev specs (blocks + bridges + profiles model).
allowed-tools: Read, Write, Edit, Bash, Glob, Grep
argument-hint: <ModuleName | --all>
---

# /spec-gen — Code generation (blocks + bridges + profiles)

Generate source code for one named module, or for every module when invoked with `--all`. User input: `$ARGUMENTS`.

The spec is the source of truth for **every Cargo dependency**. Cargo.toml is OUTPUT, not input — emitted deterministically from `architecture.toml` + module-level `external_deps[]`. Source files (`.rs`) are emitted by following the assembled codegen prompt; Cargo.toml is emitted by the analyzer (no Claude judgment involved).

Act as an implementer. The codegen prose at `specs/specdev/codegen/<lang>/...` is the contract; the assembled prompt assembles those prose files in deterministic order. Emit code that follows the prose; do not improvise design choices the prose doesn't prescribe.

Before any analyzer invocation, run the env preamble from `${CLAUDE_PLUGIN_ROOT}/shared/plugin-root-preamble.md`:

```sh
export SPECDEV_PLUGIN_ROOT="${CLAUDE_PLUGIN_ROOT}"
```

Skip if `SPECDEV_PLUGIN_ROOT` is already set or every invocation passes `--plugin-root`.

---

## Step 1 — detect workspace vs single-package

Read `spec.toml` from the project root.

- If `[project].profile == "rust-workspace"`, this is a **workspace root**. Skip ahead to "Workspace mode" below — the single-package flow does not apply.
- Otherwise, this is a **single-package project** (firmware-layered, library-api, or any future single-crate profile). Continue with Steps 2–8 below.

If the user invoked `/spec-gen` from inside a member crate of a workspace (member's `spec.toml [project].profile` is e.g. `library-api`, but a parent dir has a `spec.toml` declaring `[workspace]`), treat the invocation as single-package on that member. The workspace-level walk is only triggered when invoked at the workspace root itself.

## Step 2 — gate on validation

Run from the project root:

```sh
specdev-analyzer validate --no-color
```

If exit code is non-zero, stop. Tell the user to run `/spec-validate` and fix findings before generating. Do not generate from invalid specs.

## Step 3 — resolve targets

- If `$ARGUMENTS` is `--all`: every `.toml` file under `specs/modules/`.
- If `$ARGUMENTS` is a name: `specs/modules/<Name>.toml` must exist.

Discover module list via `ls specs/modules/*.toml`. (Workspace mode handles cross-member coordination; see "Workspace mode" below.)

## Step 4 — assemble the prompt for each target

For each target module `<Name>`:

```sh
specdev-analyzer prompt --module <Name>
```

The output is a deterministic, lex-ordered concatenation of:

1. Profile module conventions (`specs/specdev/codegen/<lang>/module.conventions.md`).
2. Per relevant block (lex-sorted): `codegen/<lang>/blocks/<name>/README.md`.
3. Per relevant bridge (lex-sorted): `codegen/<lang>/bridges/<name>/README.md`.
4. Architecture (`specs/architecture.toml`), TOML-fenced.
5. Module spec, TOML-fenced.

Read the prompt. It tells you:
- What file path(s) to emit (the profile prose says where).
- How to map fields to types/methods (block + bridge prose).
- What infrastructure picks (panic handler, logger backend, MCU part) Cargo.toml will reflect.

## Step 5 — generate source files

Output the source files the prompt prescribes, following these priority rules when prose disagrees with itself:

1. **Bridge prose wins over block prose** for cross-block wiring.
2. **Block prose wins over profile prose** for block-specific fields.
3. **Profile prose wins** for project-shape decisions (file layout, no_std attrs, error handling strategy).

For each file written:
- File header line: `// SPDX-License-Identifier: <project license>` if the spec declares one in `spec.toml [project].license`.
- A regen-pointer comment: `// @spec specs/modules/<Name>.toml` so a reader can navigate from source to spec.
- No `todo!()` / `unimplemented!()` placeholders. If the spec lacks information needed to implement a function body, **stop and ask the user** — do not improvise.

Do not modify files outside the auto-generated regions. If hand-written code already exists in the target file, preserve it and warn in the report.

## Step 6 — emit Cargo.toml

```sh
specdev-analyzer cargo-toml --output Cargo.toml
```

The analyzer renders Cargo.toml deterministically from `spec.toml [project]` + `architecture.toml` (`[platform]`, `[runtime]`, `[logger]`, `[[external_deps]]`, `[msrv]`, `[[features]]`) + per-module `external_deps[]`. Lex-sorted; byte-stable.

If stderr has `warning:` lines (typically version conflicts when two modules declare the same crate at different versions), surface them in the report. The user reconciles by editing the spec, not the generated Cargo.toml.

**Do not edit the generated Cargo.toml by hand.** If you need a dep that isn't there, add it to the spec — `[[external_deps]]` in `architecture.toml` for project-wide infra, or `external_deps[]` in a module file for module-scoped deps — and re-run this step.

## Step 7 — emit memory.x (firmware-layered only, Rust)

**Profile gate (read first).** Read `spec.toml [project].profile`. If it is anything other than `firmware-layered`, **skip this step entirely** — `memory.x` is a firmware linker concern; library-api and rust-workspace projects do not need it. Do not emit a placeholder, do not warn. Proceed to Step 8.

If profile is `firmware-layered`, continue:

`architecture.toml [platform].mcu_part` drives the linker script. If `mcu_part` is unset, emit a placeholder comment in `memory.x` and a warning in the report. If the project has a hand-authored `memory.x`, preserve it (do not overwrite).

The MCU-specific MEMORY layout comes from the relevant MCU block's codegen prose. For STM32H5 specifically (`mcu_part = "stm32h5XX"`), see `codegen/rust/blocks/stm32h5-rs/README.md`'s memory layout section if present; otherwise emit a generic `MEMORY { FLASH ... RAM ... }` skeleton with a `// FILL IN` comment.

## Step 8 — compile gate

For Rust projects:

```sh
cargo check --quiet
```

If the build fails, report the first error verbatim and stop. Do not auto-edit code to fix build errors stemming from missing scaffolding outside the per-module emission scope (e.g. `main.rs` doesn't register this module yet) — surface the gap as a one-line action item.

For firmware-layered, `cargo check` against the host won't pass without the target triple. Use:

```sh
cargo check --quiet --target <[platform].target_triple from architecture.toml>
```

If the target isn't installed (`rustup target add ...` missing), surface the install command in the report; don't try to install it automatically.

## Step 9 — report

One paragraph per module generated:
- Source file(s) written, with paths relative to project root.
- Any prose rule that required a choice and the choice made (e.g., "embassy bridge `handoff = "shared"` chose `Mutex<CriticalSectionRawMutex, ...>`").
- Build result (pass/fail; first error if fail).
- Any `cargo-toml` warnings (version conflicts, missing typed picks).
- Suggested follow-ups: `/spec-gen <OtherModule>`, wire this module into `main.rs`, regenerate after editing the spec, etc.

---

## Workspace mode

Triggered when the project root's `spec.toml [project].profile == "rust-workspace"`. Walks every member crate in topological dep-order and emits a single workspace-level summary at the end. Per **G-001** (Phase 11b acceptance, codegen-pass dogfood 2026-05-01).

### W1 — read the workspace manifest

Read `spec.toml [workspace]` and `[[workspace.members]]`. Each entry has a `path` (relative to workspace root) and a `name` (the crate name). If `[workspace.members]` is missing or empty, stop with an error: a rust-workspace project with no members is malformed.

### W2 — gate on workspace-root validation

```sh
specdev-analyzer validate --no-color
```

Run from the workspace root. The rust-workspace profile validates the workspace manifest itself (members exist, paths point at directories with `spec.toml`, no duplicate names). If it errors, stop and tell the user to run `/spec-validate` at the workspace root first.

### W3 — build the dep graph

For each member, parse the member's `spec.toml`, `architecture.toml` (if present), and every `specs/modules/*.toml`. Collect `external_deps[]` entries. An edge `member_a → member_b` exists when:

- `external_deps[].source.path` resolves (relative to member_a's dir) to member_b's directory, OR
- `external_deps[].source == "workspace"` AND the dep's `crate` field matches member_b's `name` AND the workspace's `[workspace.dependencies]` entry for that crate uses a path-dep into member_b.

Topologically sort. If a cycle is found, stop with an error pointing at the cycle (Cargo would refuse to build it anyway, but surfacing it here saves a round-trip).

### W4 — per-member walk

For each member in topological order:

1. `cd <workspace>/<member.path>`.
2. Run Steps 2–8 of single-package mode against that member directory. Each member has its own `specs/specdev/`, its own profile (typically `library-api` or `firmware-layered`), and its own `Cargo.toml` output. Use `--project <member-dir>` if invoking the analyzer from elsewhere.
3. Capture per-member status: validate result, prompt assembly status, source emit count, cargo-toml result, cargo check outcome.

If any member fails validate, do NOT skip downstream members — record the failure and continue. The point of the workspace walk is to surface every member's status in one report. (Exception: stop the walk if member_a fails to emit and member_b structurally depends on member_a's emitted source — `cargo check` on member_b will fail to find symbols. Record the cascade clearly.)

### W5 — emit workspace-root Cargo.toml

After every member has been processed, emit the workspace-root Cargo.toml:

```sh
specdev-analyzer cargo-toml --output Cargo.toml
```

Run from the workspace root. The analyzer detects `[workspace]` and renders workspace-root mode (`[workspace]` + `[workspace.package]` + `[workspace.dependencies]`).

### W6 — workspace-level summary

Emit ONE report at the end with one line per member:

```
ym2414b-core      ✓ validate (12m, 8r, 0 graph) | prompt (4 modules) | emit (12 files) | cargo-toml ✓ | cargo check ✓
ym2414b-sysex     ✓ validate (3m, 1r, 0 graph) | prompt (3 modules) | emit (5 files)  | cargo-toml ✓ | cargo check ✓
workspace         ✓ Cargo.toml (workspace-root)
```

Then a follow-up paragraph: any per-member warnings, any cross-member coupling that surfaced, suggested next steps.

---

## What this skill does NOT do

- **Multi-package coordination outside `rust-workspace`.** If members are independent (no `[workspace]` in spec.toml), invoke `/spec-gen` per member from the member's directory.
- **Cargo.toml editing.** The analyzer's `cargo-toml` subcommand is the only path to write Cargo.toml. If a dep is missing, edit the spec.
- **Auto-fix build errors.** Report and stop; the user fixes (likely by editing the spec).
- **Generate placeholders.** No `todo!()` / stubs — stop and ask if the spec is incomplete. (Phase 13's `--skeleton` mode is the named exception, not yet shipped.)
