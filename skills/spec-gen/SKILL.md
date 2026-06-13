---
name: spec-gen
description: This skill should be used when the user asks to "generate code", "codegen", "run spec-gen", "generate <ModuleName>", "codegen all modules", "implement this spec", "build my source files from spec", or names any module to materialize as source. Generates Rust source code AND Cargo.toml deterministically from validated specdev specs (blocks + bridges + profiles model). Refuses on incomplete specs — no `todo!()` / stub bodies; if the spec is missing information needed to implement a function, the skill stops and asks rather than improvising.
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

- If `[project].profile == "rust-workspace"`, this is a **workspace root**. Follow `references/workspace-mode.md` end-to-end — the single-package flow below does not apply.
- Otherwise, this is a **single-package project** (firmware-layered, library-api, audio-plugin, or any user-authored single-crate profile). Continue with Step 2.

## Step 2 — gate on validation

Run from the project root:

```sh
specdev-analyzer validate --no-color
```

If exit code is non-zero, stop. Tell the user to run `/spec-validate` and fix findings before generating. Do not generate from invalid specs.

Validate may also emit `codegen_unblocked` **warnings** — open questions in `specs/open_questions.toml` that gate generation for the modules they affect. Warnings don't fail validate; the hard refusal happens at Step 4. If the warning names a module you're about to generate, surface it now so the user can resolve it (`/spec-question`) before the per-module walk begins.

## Step 3 — resolve targets

- If `$ARGUMENTS` is `--all`: every `.toml` file under `specs/modules/`.
- If `$ARGUMENTS` is a name: `specs/modules/<Name>.toml` must exist.

Discover module list via `ls specs/modules/*.toml`. (Workspace mode handles cross-member coordination; see "Workspace mode" below.)

## Step 4 — assemble the prompt for each target

For each target module `<Name>`:

```sh
specdev-analyzer gen --module <Name>
```

`gen` enforces the **codegen gate** before assembling: if the module is reachable from an open question (`status = "open"` in `specs/open_questions.toml`), it aborts non-zero with a refusal naming the question, the module, and the exit options. Present the refusal verbatim and stop for that module — do not fall back to `prompt --module` to sidestep the gate. The user resolves via `/spec-question` (close the question, or commit a provisional default to downgrade it to `default_committed`); then re-run this step.

When the gate passes, the output is a deterministic, lex-ordered concatenation of:

1. Profile module conventions (`specs/specdev/codegen/<lang>/module.conventions.md`).
2. Per relevant block (lex-sorted): `codegen/<lang>/blocks/<name>/README.md`.
3. Per relevant bridge (lex-sorted): `codegen/<lang>/bridges/<name>/README.md`.
4. Architecture (`specs/architecture.toml`), TOML-fenced.
5. Module spec, TOML-fenced.

When the project authors decisions / open questions, the prompt additionally inlines the decision rationale reachable along the module → requirement → `decided_by` walk, and the placeholder + verify text of any `default_committed` question affecting the module — generated code reflects recorded decisions, and committed defaults arrive flagged for verification.

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

When the target file already exists from a prior generation run, read it before emitting and **minimize the diff**: preserve code that is still spec-equivalent verbatim, and change only what the spec change actually requires. Do not reorder, rename, or rephrase code the spec doesn't force to change — gratuitous churn makes the gen-lineage record (Step 6.5) and the user's VCS history useless for spotting real changes.

## Step 6 — emit Cargo.toml

```sh
specdev-analyzer cargo-toml --output Cargo.toml
```

The analyzer renders Cargo.toml deterministically from `spec.toml [project]` + `architecture.toml` (`[platform]`, `[runtime]`, `[logger]`, `[[external_deps]]`, `[msrv]`, `[[features]]`) + per-module `external_deps[]`. Lex-sorted; byte-stable.

If stderr has `warning:` lines (typically version conflicts when two modules declare the same crate at different versions), surface them in the report. The user reconciles by editing the spec, not the generated Cargo.toml.

**Do not edit the generated Cargo.toml by hand.** If you need a dep that isn't there, add it to the spec — `[[external_deps]]` in `architecture.toml` for project-wide infra, or `external_deps[]` in a module file for module-scoped deps — and re-run this step.

## Step 6.5 — record generation lineage

Record every file this run emitted, so later runs can detect hand-edits:

```sh
specdev-analyzer gen-record --module <Name> <source files written in Step 5>
specdev-analyzer gen-record Cargo.toml
```

`gen-record` writes a SHA-256 per file into `specs/specdev/gen-lineage.toml`. The flow from spec to code is one-way; a generated file that later gets hand-edited is where that flow silently breaks, and the hash record is what makes the break visible — `/spec-drift` runs `specdev-analyzer report gen-drift` and flags any recorded file whose bytes have diverged (`edited`) or that has disappeared (`missing`).

Re-running gen and re-recording replaces the entries; there is nothing else to manage. Do not hand-edit `gen-lineage.toml`.

Do **not** record `memory.x` — firmware post-processing (Step 7) may preserve a hand-authored version, and recording it would flag legitimate hand-authorship as drift.

## Step 7 — firmware-layered post-processing (memory.x)

If `spec.toml [project].profile != "firmware-layered"`, skip this step entirely. For firmware-layered, follow `references/firmware-layered-postprocess.md` to emit (or preserve a hand-authored) `memory.x`.

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
- Confirmation that emitted files were recorded in `gen-lineage.toml` (Step 6.5).
- Suggested follow-ups: `/spec-gen <OtherModule>`, wire this module into `main.rs`, regenerate after editing the spec, etc.

---

## What this skill does NOT do

- **Multi-package coordination outside `rust-workspace`.** If members are independent (no `[workspace]` in spec.toml), invoke `/spec-gen` per member from the member's directory.
- **Cargo.toml editing.** The analyzer's `cargo-toml` subcommand is the only path to write Cargo.toml. If a dep is missing, edit the spec.
- **Auto-fix build errors.** Report and stop; the user fixes (likely by editing the spec).
- **Generate placeholder bodies.** No `todo!()` / stubs — stop and ask if the spec is incomplete. The audio-plugin profile's audio-thread methods are the one structured exception, and the silence-and-log emit template for those methods lives in the audio-plugin codegen prose (`codegen/rust/module.conventions.md` § "Audio-thread method bodies"); the prompt assembled in Step 4 already includes it for audio-plugin modules.

## Additional resources

- `references/workspace-mode.md` — full procedure when `[project].profile == "rust-workspace"` (W1–W6: workspace manifest read, validation gate, member dep graph, per-member walk, workspace-root Cargo.toml emit, summary report).
- `references/firmware-layered-postprocess.md` — `memory.x` emission rules for `firmware-layered` projects (placeholder when `mcu_part` is unset, preserve hand-authored versions, MCU-block-driven layout).
