---
name: spec-gen
description: This skill should be used when the user asks to "generate code", "codegen", "run spec-gen", "generate <ModuleName>", "codegen all modules", "implement this spec", "build my source files from spec", or names any module to materialize as source. Generates source code (Rust or Dart, per the project's language) AND the build manifest (Cargo.toml / pubspec.yaml) deterministically from validated specdev specs (blocks + bridges + profiles model). Refuses on incomplete specs — no stub bodies (`todo!()` / `throw UnimplementedError()`); if the spec is missing information needed to implement a function, the skill stops and asks rather than improvising.
allowed-tools: Read, Write, Edit, Bash, Glob, Grep
argument-hint: <ModuleName | --all [--force]>
---

# /spec-gen — Code generation (blocks + bridges + profiles)

Generate source code for one named module, or for every module when invoked with `--all`. `--all` regenerates only the modules whose inputs changed or whose output is stale/missing (Step 3.5); add `--force` to regenerate every module regardless. User input: `$ARGUMENTS`.

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

## Step 3.5 — incremental plan (`--all` only)

A single-module invocation always regenerates that module — **skip this step**. For `--all`, don't re-author modules whose generation inputs are unchanged and whose output is still clean; regenerating them only churns the files against the minimize-diff intent (Step 5).

If `$ARGUMENTS` includes `--force`, **skip the plan** — every module is a target; go to Step 4. (Use `--force` when an upstream codegen-prose change hasn't been re-inlined into `specs/specdev/codegen/`: the project's inputs are then correctly unchanged, so the plan would skip, but you want a blanket regenerate. Re-inlining stale plugin prose is `/spec-drift`'s job; `--force` is the override.)

Otherwise run:

```sh
specdev-analyzer report gen-plan
```

It emits one deterministic verdict per module under `specs/modules/` — no judgment from you:

- `unchanged` — output clean and inputs match the last gen. **Skip.**
- `edited` — a generated output file was hand-edited (carries the `files`). **Skip** — regenerating would clobber the edit. Surface it and tell the user to reconcile via `/spec-drift`.
- `stale` — output clean but the spec or assembled prompt changed since the last gen. **Regenerate.**
- `missing` — a recorded output file is gone. **Regenerate.**
- `new` — never generated. **Regenerate.**

Your **regenerate set** is every module whose category is `new`, `stale`, or `missing`. The rest are skipped on this axis (carry the `unchanged` and `edited` counts into the Step 9 report).

Each module also carries a second, independent field — **`verified`** — which is `true` only when a **passing** compile-gate result is recorded for it at the current inputs *and* the current lockfile (Step 8.5 writes those). It answers a different question from `category`: not "would regeneration produce something new?" but "did what is there ever compile, against the dependencies that are there now?". A module that failed its gate last run, or whose lockfile moved since (`cargo update`, a new dependency), is `unchanged` **and** `verified: false`.

Your **re-gate set** is every module with `category: unchanged` and `verified: false`. These modules are **not regenerated** — nothing about their inputs changed — but they are **not skipped as done** either: run the compile gate (Step 8) for them and record the result (Step 8.5). A re-gate costs one compiler run; a regenerate costs a session. `edited` modules are never in the re-gate set (a hand-edit is `/spec-drift`'s finding).

The report may also carry a `batch:` list — the whole-tree result of the last gate per label, with `current: true` when it was recorded against today's lockfile. A `pass: false` batch row means the tree did not build the last time a gate ran, even if every module you are about to skip reads `verified: true` (a later module broke it). Surface it in Step 9.

**Exit rule.** If the regenerate set is empty, skip Steps 4–7. If the re-gate set is *also* empty, skip Step 8 too and go straight to Step 9 to report what was skipped. Otherwise run Step 8 (and 8.5) for the re-gate set alone.

## Step 4 — assemble the prompt: shared preamble once, then per-module tails

The codegen prompt is split into a **project-invariant preamble** (read once, applies to every module in this run) and a **per-module tail**. Follow this single path for both single-module and `--all` invocations — do not re-assemble the preamble per module.

**Step 4a — assemble the shared preamble once.** Pass one `--module <Name>` per module in your **target set** — the regenerate set from Step 3.5 for an incremental `--all`, every module for `--force`, or the single named module — so the preamble carries exactly the blocks/bridges those targets use:

```sh
# single target (or one --module per module in the target set):
specdev-analyzer gen --preamble --module <Name>
# every module (only when the target set is the full module list, e.g. --force):
specdev-analyzer gen --preamble
```

This is gate-free shared prose, a deterministic lex-ordered concatenation of:

1. Profile module conventions (`specs/specdev/codegen/<lang>/module.conventions.md`).
2. Per relevant block (lex-sorted): `codegen/<lang>/blocks/<name>/README.md`.
3. Per relevant bridge (lex-sorted): `codegen/<lang>/bridges/<name>/README.md`.
4. Architecture (`specs/architecture.toml`), TOML-fenced.

Read the preamble once. It is the contract for **every** module you generate this run — keep it in mind across the per-module loop; do not re-fetch it per module.

**Step 4b — assemble the per-module tail.** For each module `<Name>` in the target set (Step 3.5's regenerate set, or every module under `--force` / a single named module):

```sh
specdev-analyzer gen --module <Name> --tail
```

`--tail` enforces the **codegen gate** before assembling: if the module is reachable from an open question (`status = "open"` in `specs/open_questions.toml`), it aborts non-zero with a refusal naming the question, the module, and the exit options. Present the refusal verbatim and stop for that module — do not fall back to `prompt --module` or the whole-prompt `gen --module` to sidestep the gate. The user resolves via `/spec-question` (close the question, or commit a provisional default to downgrade it to `default_committed`); then re-run this step.

When the gate passes, the tail is a deterministic concatenation of:

5. The decision/open-question context in scope (see below).
6. Project context (`module.context_files`), if any.
7. Test-item verification context, if the module declares test items.
8. Module spec, TOML-fenced.

When the project authors decisions / open questions, the tail inlines the decision rationale reachable along the module → requirement → `decided_by` walk, and the placeholder + verify text of any `default_committed` question affecting the module — generated code reflects recorded decisions, and committed defaults arrive flagged for verification.

Read preamble + tail together. They tell you:
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
- No stub bodies. Rust: no `todo!()` / `unimplemented!()`. Dart: no `throw UnimplementedError()` (in any dress). If the spec lacks information needed to implement a function body, **stop and ask the user** — do not improvise. (The comment syntax above is `//` in both languages; for Dart use `// @spec ...` the same way.)

Do not modify files outside the auto-generated regions. If hand-written code already exists in the target file, preserve it and warn in the report.

When the target file already exists from a prior generation run, read it before emitting and **minimize the diff**: preserve code that is still spec-equivalent verbatim, and change only what the spec change actually requires. Do not reorder, rename, or rephrase code the spec doesn't force to change — gratuitous churn makes the gen-lineage record (Step 6.5) and the user's VCS history useless for spotting real changes.

## Step 6 — emit the build manifest

```sh
specdev-analyzer manifest --write
```

The analyzer renders the project's build manifest deterministically from the spec, dispatching on `[project].language` and writing the language's canonical filename in the project root — `Cargo.toml` for rust (from `spec.toml [project]` + `architecture.toml` `[platform]`/`[runtime]`/`[logger]`/`[[external_deps]]`/`[msrv]`/`[[features]]` + per-module `external_deps[]`), `pubspec.yaml` for dart (from `[project]` + `architecture.toml` `[environment]`/`[[external_deps]]`/`[[dependency_overrides]]` + per-module `external_deps[]`). Lex-sorted; byte-stable.

If stderr has `warning:` lines (typically version conflicts when two spec locations declare the same dependency at different versions), surface them in the report. The user reconciles by editing the spec, not the generated manifest.

**Do not edit the generated manifest by hand.** If you need a dep that isn't there, add it to the spec — `[[external_deps]]` in `architecture.toml` for project-wide infra, or `external_deps[]` in a module file for module-scoped deps — and re-run this step.

## Step 6.5 — record generation lineage

Record every file this run emitted, so later runs can detect hand-edits:

```sh
specdev-analyzer gen-record --accept-edits --module <Name> <source files written in Step 5>
specdev-analyzer gen-record --accept-edits Cargo.toml   # or pubspec.yaml — whichever manifest Step 6 wrote
```

**Why `--accept-edits` here, and only here.** On a re-run the emitted bytes differ from the previous recording, and byte-difference alone cannot tell a fresh emission from a hand-edit — so `gen-record` refuses to move the baseline unless the caller states that the current bytes are the intended one. This step is that statement: you just wrote these files, so their bytes are the new baseline by definition.

**Never pass it anywhere else.** In particular, if `/spec-drift` reports a file as `edited`, do **not** re-record to clear it. Recording is what makes a file `clean`, so re-recording makes the hand-edit the new truth and the drift report can never surface it again — the file silently stops matching what the spec produces. Fix the cause instead: fold the change into the spec and re-run `/spec-gen`, or discard the edit.

`gen-record` writes a SHA-256 per file into `specs/specdev/gen-lineage.toml`. The flow from spec to code is one-way; a generated file that later gets hand-edited is where that flow silently breaks, and the hash record is what makes the break visible — `/spec-drift` runs `specdev-analyzer report gen-drift` and flags any recorded file whose bytes have diverged (`edited`) or that has disappeared (`missing`).

For a module record (`--module <Name>`), `gen-record` also captures the SHA-256 of that module's whole assembled prompt — the *input* hash that the next `--all` compares in Step 3.5 to decide skip-vs-regenerate. Nothing extra to pass; recording the emitted files is what makes the module skippable next time. (Project-level records like `Cargo.toml` carry no input hash.)

Re-running gen and re-recording replaces the entries; there is nothing else to manage. Do not hand-edit `gen-lineage.toml`.

**Ordering trap (formatters).** Run any formatter — `dart format`, `rustfmt` — on the emitted files *before* this step, not after. A formatter pass after recording changes the bytes, so `/spec-drift` reports gen-owned files as `edited`, and the only clean-looking fix is exactly the re-record this section forbids. Formatting is part of emission; record what you are going to keep.

Do **not** record `memory.x` — firmware post-processing (Step 7) may preserve a hand-authored version, and recording it would flag legitimate hand-authorship as drift.

Do **not** record externally-owned files in a Dart/Flutter tree: the platform directories (`android/`, `ios/`, `web/`, `macos/`, `windows/`, `linux/`), `analysis_options.yaml`, `.metadata`, and any third-party codegen output (`*.g.dart`, `*.freezed.dart`, FFI-bridge glue). Those files belong to `flutter create` or their own generators; recording them reports perpetual `edited` drift the moment their generator reruns. Record only what this skill emitted: `lib/main.dart`, `lib/src/*.dart`, test files, and `pubspec.yaml`.

## Step 7 — firmware-layered post-processing (memory.x)

If `spec.toml [project].profile != "firmware-layered"`, skip this step entirely. For firmware-layered, follow `references/firmware-layered-postprocess.md` to emit (or preserve a hand-authored) `memory.x`.

## Step 8 — compile gate

Dispatch on `spec.toml [project].language`.

For Rust projects:

```sh
cargo check --quiet
```

For firmware-layered, `cargo check` against the host won't pass without the target triple. Use:

```sh
cargo check --quiet --target <[platform].target_triple from architecture.toml>
```

If the target isn't installed (`rustup target add ...` missing), surface the install command in the report; don't try to install it automatically.

For Dart projects (Flutter profiles):

```sh
flutter pub get && flutter analyze
```

The analyze gate must come back **clean** (no errors; treat analyzer errors like compile errors). If the Flutter SDK is not on PATH, report that the gate could not run and stop — do not mark generation verified without it. A Flutter app also needs its platform scaffolding once per project (`flutter create . --platforms <matrix>` from the project root, platforms from `architecture.toml`); if the platform dirs are absent, surface that command as an action item rather than running it implicitly — it writes a large externally-owned tree the user should see created. Include in the same action item: delete the scaffold's `test/widget_test.dart` afterwards — it references template symbols that don't exist in a spec-generated app and breaks both gates; the spec's test items supersede it.

In every language: if the gate fails, **record the failure (Step 8.5) before you stop**, then report the first error verbatim and stop. Do not auto-edit code to fix build errors stemming from missing scaffolding outside the per-module emission scope (e.g. the entry point doesn't register this module yet) — surface the gap as a one-line action item.

For an incremental `--all`, the gate runs for the **regenerate set and the re-gate set together** (Step 3.5). One `cargo check` / `flutter analyze` covers the whole tree; that is fine — the recording step below is what scopes the result to the modules of this run.

## Step 8.5 — record the gate result

Record the outcome of Step 8 for every module in this run — the regenerate set plus the re-gate set — under a label that names the gate:

```sh
# pass:
specdev-analyzer gen-record-gate --label cargo-check --pass --module <A> --module <B> ...
# fail:
specdev-analyzer gen-record-gate --label cargo-check --fail --module <A> --module <B> ...
```

Use `--label flutter-analyze` for the Dart gate. One `--module` per module in the run; the command also writes one **batch** row (the whole-tree result) under the same label. It hashes the project's lockfile *at this moment* — which is why this step comes **after** Step 8 and never before it: `cargo check` creates or rewrites `Cargo.lock`, and a pass recorded against the pre-gate lockfile would be invalid the instant it was written.

**Record every module in the run on a failure, not just the one the error names.** `cargo check` compiles the whole crate, so a failure proves nothing about any module in the run; under-claiming is the safe direction. Modules *outside* the run are untouched — their earlier pass rows stand, and the batch row is what tells Step 9 the tree does not build.

**Never record a pass you did not observe.** If the gate could not run (SDK missing, target not installed), record nothing: an unrecorded gate reads `verified: false` next time and simply re-gates, which is the correct outcome. A recorded pass means exactly "one environment compiled these inputs against this lockfile, once" — the toolchain version is deliberately not part of the key, so do not read `verified: true` as "compiles everywhere".

Do not hand-edit the `[[gates]]` rows in `gen-lineage.toml`.

## Step 9 — report

For an incremental `--all`, open with a one-line summary of the plan from Step 3.5: `N unchanged + verified (skipped), R re-gated (unchanged, gate re-run), M regenerated, K edited (skipped — reconcile via /spec-drift)`, naming the `edited` modules and their files so the user can act on them. If the plan carried a `batch:` row with `pass: false`, say so on its own line: the tree did not build the last time a gate ran. A `--force` or single-module run has no plan summary.

"Verified" in this report is the **recorded** fact from Step 8.5 — a module is verified when `gen-plan` says `verified: true`, and not otherwise. Do not call a module verified from memory of having seen the gate pass; if you did not record it, it is not verified.

Then one paragraph per module **generated** (skipped modules need no paragraph; re-gated modules get one line each with the gate result):
- Source file(s) written, with paths relative to project root.
- Any prose rule that required a choice and the choice made (e.g., "embassy bridge `handoff = "shared"` chose `Mutex<CriticalSectionRawMutex, ...>`").
- Build result (pass/fail; first error if fail) — and that it was recorded (Step 8.5).
- Any `manifest` warnings (version conflicts, missing typed picks).
- Confirmation that emitted files were recorded in `gen-lineage.toml` (Step 6.5).
- Suggested follow-ups: `/spec-gen <OtherModule>`, wire this module into `main.rs`, regenerate after editing the spec, etc.

---

## What this skill does NOT do

- **Multi-package coordination outside `rust-workspace`.** If members are independent (no `[workspace]` in spec.toml), invoke `/spec-gen` per member from the member's directory.
- **Manifest editing.** The analyzer's `manifest` subcommand is the only path to write the build manifest (Cargo.toml / pubspec.yaml). If a dep is missing, edit the spec.
- **Auto-fix build errors.** Report and stop; the user fixes (likely by editing the spec).
- **Generate placeholder bodies.** No `todo!()` / stubs — stop and ask if the spec is incomplete. The audio-plugin profile's audio-thread methods are the one structured exception, and the silence-and-log emit template for those methods lives in the audio-plugin codegen prose (`codegen/rust/module.conventions.md` § "Audio-thread method bodies"); the prompt assembled in Step 4 already includes it for audio-plugin modules.

## Additional resources

- `references/workspace-mode.md` — full procedure when `[project].profile == "rust-workspace"` (W1–W6: workspace manifest read, validation gate, member dep graph, per-member walk, workspace-root Cargo.toml emit, summary report).
- `references/firmware-layered-postprocess.md` — `memory.x` emission rules for `firmware-layered` projects (placeholder when `mcu_part` is unset, preserve hand-authored versions, MCU-block-driven layout).
