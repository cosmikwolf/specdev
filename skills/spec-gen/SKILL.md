---
name: spec-gen
description: This skill should be used when the user asks to "generate code", "codegen", "run spec-gen", "generate <ModuleName>", "codegen all modules", "implement this spec", "build my source files from spec", or names any module to materialize as source. Generates source code (Rust, Dart or TypeScript, per the project's language) AND the build manifest (Cargo.toml / pubspec.yaml / package.json) deterministically from validated specdev specs (blocks + bridges + profiles model). Refuses on incomplete specs — no stub bodies (`todo!()` / `throw UnimplementedError()` / `throw new Error("not implemented")`); if the spec is missing information needed to implement a function, the skill stops and asks rather than improvising.
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

## Step 3.5 — incremental plan

**Single-module invocation.** Run `specdev-analyzer report gen-plan` and read **only the target module's row**. Two verdicts stop the run before anything is generated:

- `edited` — a recorded output was changed after the last gen. Name the files and ask which the user wants: (1) **fold the change into the spec**, then regenerate; (2) **regenerate and discard the change** — right when the edit only repaired broken generated output (a syntax error, a missing import), because the spec already describes the correct code; (3) **cancel**. Do not assume (2): the file cannot tell you whether a person changed the design or repaired the output.
- `unrecorded` — hand-written source is already at the module's path. Ask with the three options under "`unrecorded` is not a slow `new`" below.

Every other verdict regenerates the module — a named module is always a target. Skip the rest of this step, including the regenerate and re-gate sets.

For `--all`, don't re-author modules whose generation inputs are unchanged and whose output is still clean; regenerating them only churns the files against the minimize-diff intent (Step 5).

If `$ARGUMENTS` includes `--force`, **skip the plan** — every module is a target; go to Step 4. (Use `--force` when an upstream codegen-prose change hasn't been re-inlined into `specs/specdev/codegen/`: the project's inputs are then correctly unchanged, so the plan would skip, but you want a blanket regenerate. Re-inlining stale plugin prose is `/spec-drift`'s job; `--force` is the override.)

Otherwise run:

```sh
specdev-analyzer report gen-plan
```

It emits one deterministic verdict per module under `specs/modules/` — no judgment from you:

- `unchanged` — output clean and inputs match the last gen. **Skip.**
- `edited` — a generated output file was hand-edited (carries the `files`). **Skip** — regenerating would clobber the edit. Surface it and tell the user to reconcile via `/spec-drift`.
- `unrecorded` — no lineage row, but a file **already exists at the module's source path** — its `source_file`, else the language's convention path under `[paths].source` (e.g. `src/<snake_name>.ts`), the same path `extract-items` reads (carries the `files`). **Stop and ask.** See below.
- `stale` — output clean but the spec or assembled prompt changed since the last gen. **Regenerate.**
- `missing` — a recorded output file is gone. **Regenerate.**
- `new` — never generated, and nothing at the module's source path. **Regenerate.**

Your **regenerate set** is every module whose category is `new`, `stale`, or `missing`. The rest are skipped on this axis (carry the `unchanged`, `edited` and `unrecorded` counts into the Step 9 report).

**`unrecorded` is not a slow `new`.** It means this project adopted specdev over code it already had: the file at the module's source path is hand-authored, and `/spec-gen` has never written it. Generating would overwrite working source that the plan never warned about. Do not put these modules in the regenerate set, and do not silently skip them either — a skipped module reads as done.

For each `unrecorded` module, name the file and ask the user which of these they want:

1. **Keep the hand-written source.** Bring the spec up to match what is there — `/spec-sync-items` reconciles the module's `items[]` against what tree-sitter finds — and leave the body alone. The module stays `unrecorded` until something records it, which is correct: nothing generated it.
2. **Adopt the file as generated output.** `specdev-analyzer gen-record --accept-edits --module <Name> <file>` declares the current bytes the baseline. From then on it is an ordinary `unchanged` module and regenerates normally. Only do this when the user confirms the file already matches what the spec describes — recording is what makes a file `clean`, so a wrong record hides the mismatch forever.
3. **Replace it.** The user moves or deletes the file, and the module becomes `new` on the next run.

Never choose for them. This is the one verdict where the analyzer cannot tell whose work is at stake.

Each module also carries a second, independent field — **`verified`** — which is `true` only when a **passing** compile-gate result is recorded for it at the current inputs, the current lockfile *and* the current profile substrate (Step 8 writes those). It answers a different question from `category`: not "would regeneration produce something new?" but "did what is there ever compile, against the dependencies that are there now?". A module that failed its gate last run, or whose lockfile moved since (`cargo update`, a new dependency), is `unchanged` **and** `verified: false`.

Your **re-gate set** is every module with `category: unchanged` and `verified: false`. These modules are **not regenerated** — nothing about their inputs changed — but they are **not skipped as done** either: run the compile gate (Step 8) for them — the runner records the result. A re-gate costs one compiler run; a regenerate costs a session. `edited` modules are never in the re-gate set (a hand-edit is `/spec-drift`'s finding).

`verified` is computed against the **gate registry** — the `gates:` list in `specs/specdev/profile.yaml` (Step 8). A module is verified only when **every** gate with `scope: module` has a passing row for it at the current key; a pass from one of two declared gates is not enough.

The report opens with **`gates_declared`**. It is `true` when the registry declares at least one `scope: module` gate. When it is **`false`**, no module can be verified — every module reads `verified: false`, and there is no gate to run in Step 8 — so **stop here**. Do not treat this as "no gates required": a project with nothing declared has nothing to verify with, and that is a missing precondition, not a pass. The fix is a `gates:` block in the inlined profile; `specdev-analyzer gen --module <any module> --tail` prints the block to paste (the plugin's registry for this profile) or, for a profile that ships none, the minimal shape to write. Run it for one module, present the message verbatim, and stop.

The report may also carry a `batch:` list — the whole-tree result of the last gate per label, with `current: true` when it was recorded against today's lockfile. A `pass: false` batch row means the tree did not build the last time a gate ran, even if every module you are about to skip reads `verified: true` (a later module broke it). Surface it in Step 9.

**Exit rule.** If the regenerate set is empty, skip Steps 4, 5 and 7. Steps 6 (manifest), 6.2 (derive) and 6.5 (record — the manifest line only, no `--module`) **always run**. Run Step 8 for the re-gate set; skip it only when that set is also empty, then go to Step 9 to report what was skipped. The manifest and the derived files are deterministic, so an unchanged tree writes identical bytes; derive needs the full module set to find a stale file, and a deleted module has no `gen-plan` row at all.

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

`--tail` enforces two gates before assembling, and it does so in every mode — a single-module run reads only its own plan row and `--force` skips the plan, but neither skips this.

- **The gate registry.** If `specs/specdev/profile.yaml` declares no gate with `scope: module` (no `gates:` key, a batch-only registry, or no inlined profile at all), `--tail` aborts non-zero. The message carries the fix: the plugin's `gates:` block for this profile to paste into the inlined profile, or — for a profile that ships none — the minimal shape to write. Present it verbatim and stop the run; nothing is generated ungated. The user pastes and re-runs.
- **The codegen gate.** If the module is reachable from an open question (`status = "open"` in `specs/open_questions.toml`), it aborts non-zero with a refusal naming the question, the module, and the exit options. Present the refusal verbatim and stop for that module. The user resolves via `/spec-question` (close the question, or commit a provisional default to downgrade it to `default_committed`); then re-run this step.

Do not fall back to `prompt --module` or the whole-prompt `gen --module` to sidestep either gate.

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
- **The header lines go above a first-line directive, not below it.** When the profile's prose calls for a directive or a guard import on "the first line of the body" (`"use client"`, `import "server-only"`), that means the first line of *code*: the SPDX and `// @spec` comments sit above it. Measured on Next 16.3.4: a leading comment does not stop the directive being recognized. Do not drop the header to get the directive onto line 1, and do not move the header below it.
- No stub bodies. Rust: no `todo!()` / `unimplemented!()`. Dart: no `throw UnimplementedError()` (in any dress). TypeScript: no `throw new Error("not implemented")`, no `return null as any`, no `// @ts-expect-error` over an unwritten body. If the spec lacks information needed to implement a function body, **stop and ask the user** — do not improvise. (The comment syntax above is `//` in all three languages; use `// @spec ...` the same way.)

Do not modify files outside the auto-generated regions. If hand-written code already exists in the target file, preserve it and warn in the report.

When the target file already exists from a prior generation run, read it before emitting and **minimize the diff**: preserve code that is still spec-equivalent verbatim, and change only what the spec change actually requires. Do not reorder, rename, or rephrase code the spec doesn't force to change — gratuitous churn makes the gen-lineage record (Step 6.5) and the user's VCS history useless for spotting real changes.

## Step 6 — emit the build manifest

```sh
specdev-analyzer manifest --write
```

**If this refuses with `[MANIFEST-003] … unrecorded file at target`,** the project has a hand-authored `Cargo.toml` / `pubspec.yaml` / `package.json` that `/spec-gen` did not write and whose bytes differ from what the spec renders — the same brownfield situation as an `unrecorded` module in Step 3.5. Do **not** reach for `--accept-edits` to get past it; that discards whatever the file declares that the spec does not yet carry. Surface the refusal verbatim, show the user what is in their file that the spec has no home for, and fold those declarations into the spec (`[[external_deps]]` in `architecture.toml` for project-wide dependencies, `external_deps[]` in a module for module-scoped ones). `--accept-edits` is for when the user has confirmed the file is disposable.

Identical bytes and already-recorded manifests are never refused, so the ordinary regenerate loop never sees this.

The analyzer renders the project's build manifest deterministically from the spec, dispatching on `[project].language` and writing the language's canonical filename in the project root — `Cargo.toml` for rust (from `spec.toml [project]` + `architecture.toml` `[platform]`/`[runtime]`/`[logger]`/`[[external_deps]]`/`[msrv]`/`[[features]]` + per-module `external_deps[]`), `pubspec.yaml` for dart (from `[project]` + `architecture.toml` `[environment]`/`[[external_deps]]`/`[[dependency_overrides]]` + per-module `external_deps[]`), `package.json` for typescript (from `[project]` + `architecture.toml` `version`/`[environment]`/`[scripts]`/`[[external_deps]]` + per-module `external_deps[]`). Lex-sorted; byte-stable. A typescript workspace root also writes `pnpm-workspace.yaml` beside `package.json`; it is gen-owned and overwritten on every write — record it in Step 6.5 with the manifest.

The same command also writes the profile's **paired files** — config files the profile ships as templates (`nextjs-app`: `tsconfig.json`, `next.config.ts`, `vitest.config.ts`, `playwright.config.ts`, `.gitignore`, `app/.gitkeep`). Each is written **only when absent**, one `wrote paired file <path>` line each; a file already on disk is left untouched. After that first write they are externally owned — never rewritten, never gen-recorded, and never listed in Step 6.5. They are in the gate key, so a user editing one re-gates every module, which is correct: the gate now proves something different.

The analyzer's progress lines (`wrote package.json …`, `kept N existing paired file(s) …`, `wrote paired file …`) go to **stderr**, as all analyzer progress does; stdout is reserved for machine output. Judge success by the exit code, not by what either stream contains — a wrapper that greps stdout for these lines sees nothing and misreads an empty stream as a failure. If stderr has `warning:` lines (typically version conflicts when two spec locations declare the same dependency at different versions), surface them in the report. The user reconciles by editing the spec, not the generated manifest.

**Do not edit the generated manifest by hand.** If you need a dep that isn't there, add it to the spec — `[[external_deps]]` in `architecture.toml` for project-wide infra, or `external_deps[]` in a module file for module-scoped deps — and re-run this step.

## Step 6.2 — derive the profile's derived files

```sh
specdev-analyzer derive --write
```

When `specs/specdev/profile.yaml` declares `derived_files:` rules (nextjs-app: the route files under `app/` that re-export each module body), the analyzer derives one file per matching module, deterministically, and records each as a `derived` row in `gen-lineage.toml`. A profile without rules prints nothing. Run it for **every module on every gen**, an incremental `--all` included — a stale file (module deleted, `app_path` changed) is found only against the full module set. Run it after Step 5's formatter pass and before Step 6.5.

One line per path on stdout:

- `wrote <path>` / `kept <path>` — normal.
- `skipped <Module>: no body` — the module has no source file yet (a Step 4 refusal, a Step 5 stop). Its derived file is not written, and an old one is deleted; it returns on the run that writes the body.
- `edited <path>` — the derived file was hand-edited. Kept, not overwritten. **Action item for Step 9.**
- `conflict <path>: unrecorded file at target` — a file the spec did not produce sits at the target. Kept, nothing recorded. **Action item for Step 9.**
- `deleted <path>` — a stale derived file and its row removed.

**Never pass `--accept-edits` from this skill.** A derived file is never a legitimate hand-edit target (route segment config belongs in the body, where `export *` carries it); the user runs `specdev-analyzer derive --write --accept-edits` by hand after an `edited` or `conflict` line. A non-zero exit is a refusal — two modules on one target, a page and a route handler in one segment, an unreachable rule set, a stale inlined profile (the message carries the block to paste) — surface it verbatim and stop.

## Step 6.5 — record generation lineage

Record every file this run emitted, so later runs can detect hand-edits:

```sh
specdev-analyzer gen-record --accept-edits --module <Name> <source files written in Step 5>
specdev-analyzer gen-record --accept-edits Cargo.toml   # or pubspec.yaml / package.json — whichever manifest Step 6 wrote
specdev-analyzer gen-record --accept-edits package.json pnpm-workspace.yaml   # typescript workspace root: both files
```

**Never list a derived file** — anything Step 6.2 printed. `gen-record` refuses such a path; a plain record would drop its `derived` flag and stale deletion would stop for that file. Derive records its own files.

**Why `--accept-edits` here, and only here.** On a re-run the emitted bytes differ from the previous recording, and byte-difference alone cannot tell a fresh emission from a hand-edit — so `gen-record` refuses to move the baseline unless the caller states that the current bytes are the intended one. This step is that statement: you just wrote these files, so their bytes are the new baseline by definition.

**Never pass it anywhere else.** In particular, if `/spec-drift` reports a file as `edited`, do **not** re-record to clear it. Recording is what makes a file `clean`, so re-recording makes the hand-edit the new truth and the drift report can never surface it again — the file silently stops matching what the spec produces. Fix the cause instead: fold the change into the spec and re-run `/spec-gen`, or discard the edit.

`gen-record` writes a SHA-256 per file into `specs/specdev/gen-lineage.toml`. The flow from spec to code is one-way; a generated file that later gets hand-edited is where that flow silently breaks, and the hash record is what makes the break visible — `/spec-drift` runs `specdev-analyzer report gen-drift` and flags any recorded file whose bytes have diverged (`edited`) or that has disappeared (`missing`).

For a module record (`--module <Name>`), `gen-record` also captures the SHA-256 of that module's whole assembled prompt — the *input* hash that the next `--all` compares in Step 3.5 to decide skip-vs-regenerate. Nothing extra to pass; recording the emitted files is what makes the module skippable next time. (Project-level records like `Cargo.toml` carry no input hash.)

Re-running gen and re-recording replaces the entries; there is nothing else to manage. Do not hand-edit `gen-lineage.toml`.

**Ordering trap (formatters).** Run any formatter — `dart format`, `rustfmt`, `prettier` — on the **Step-5 file list only**, the source files you wrote, and before this step. Never run it on `app/` or on any derived file: derived files are written in the formatter's default shape and are not this skill's output. A formatter pass after recording changes the bytes, so `/spec-drift` reports gen-owned files as `edited`, and the only clean-looking fix is exactly the re-record this section forbids. Formatting is part of emission; record what you are going to keep.

Do **not** record `memory.x` — firmware post-processing (Step 7) may preserve a hand-authored version, and recording it would flag legitimate hand-authorship as drift.

Do **not** record externally-owned files in a Dart/Flutter tree: the platform directories (`android/`, `ios/`, `web/`, `macos/`, `windows/`, `linux/`), `analysis_options.yaml`, `.metadata`, and any third-party codegen output (`*.g.dart`, `*.freezed.dart`, FFI-bridge glue). Those files belong to `flutter create` or their own generators; recording them reports perpetual `edited` drift the moment their generator reruns. Record only what this skill emitted: `lib/main.dart`, `lib/src/*.dart`, test files, and `pubspec.yaml`.

Do **not** record externally-owned files in a TypeScript/Node tree: `node_modules/`, the framework's build output (`.next/`, `dist/`, `build/`), the lockfile (`pnpm-lock.yaml` / `package-lock.json` / `yarn.lock`), tool-written declaration shims (`next-env.d.ts`), incremental-build state (`*.tsbuildinfo`), and any agent-instruction block a dev server maintains (`next dev` re-adds a block to `AGENTS.md` on every run; the same holds for a `CLAUDE.md` block if the tool writes one). The package manager and the framework own those and rewrite them whenever they run, so a record reports perpetual `edited` drift. Record only what this skill emitted: the module source files, test files, and `package.json` (plus `pnpm-workspace.yaml` at a workspace root). The profile's `paired_files` config templates (`tsconfig.json`, `next.config.ts`, `vitest.config.ts`, `playwright.config.ts`, `.gitignore`) are written by Step 6 **only when absent** and are externally owned after that — never rewritten and never gen-recorded, by the analyzer or by you. `next build` rewrites `tsconfig.json` on its first run, which is why that file can live in no other ownership category. They are still in the gate key, so editing one re-gates every module.

## Step 7 — firmware-layered post-processing (memory.x)

If `spec.toml [project].profile != "firmware-layered"`, skip this step entirely. For firmware-layered, follow `references/firmware-layered-postprocess.md` to emit (or preserve a hand-authored) `memory.x`.

## Step 8 — compile gate

The gates are **data**: the `gates:` list in `specs/specdev/profile.yaml`. Each entry has a `label` (opaque; it is the row key), a `scope` (`module` or `batch`) and `steps` — a list of argv lists. **The analyzer runs them and records the result.** You do not run a compiler command yourself, you do not dispatch on the language or the profile, and you do not record anything by hand. A project whose registry lacks a check has a profile bug, not a skill gap.

For every gate in the registry, in declared order:

```sh
# scope: module — one --module per module in the run (regenerate set + re-gate set):
specdev-analyzer gate --label <label> --module <A> --module <B> ...
# scope: batch — no --module:
specdev-analyzer gate --label <label>
```

The runner resolves any `${root.<table>.<field>}` token against `specs/architecture.toml`, runs each step as one process with exactly that argv in the project root (a workspace member's own root — the workspace walk already `cd`s per member) with the inherited environment and stdin closed, stops at the first non-zero exit, prints the output, hashes the key **after** the run, and writes the rows: always the batch row, plus one row per `--module` for a `scope: module` gate.

Read its **exit code**. The three outcomes are different facts and must not be merged:

- **`0` — the gate passed.** Rows are written. Any `stderr` the runner printed is a warning the tool emitted, not a failure; carry it into Step 9.
- **the tool's own non-zero code** (`1`, `3`, …) — **a step ran and exited non-zero**: a compile error, an analyzer error, a network failure during an install step. The gate **failed** and fail rows are written for every module named. Report the printed output verbatim and stop.
- **`2` — usually means the gate did not run.** A step's executable could not start (SDK missing, target not installed), a token resolved to nothing, the registry declares no module-scope gate, or the rows could not be written. In that case no row was written, which is correct: an unrecorded gate reads `verified: false` next run and simply re-gates. Surface the hint in the printed message as a one-line action item — do not install anything yourself.

  **Confirm it from stdout, not from the code alone.** The runner passes a failing step's own exit code through unchanged, so a gate tool that legitimately exits 2 is indistinguishable by code from the runner's own exit-2 classes — and in that case a `fail` row **was** written. The rows are what separate the two:

  ```
  gate <label> fail module <Name> key sha256:...   <- it ran, and failed
  ```

  A `gate ... fail ...` line at exit 2 means "ran and failed" (report it as a failure), and no such line means "did not run" (report the hint). No shipped gate does this today — `cargo check` fails 101, `tsc` and `flutter analyze` fail 1 — so treat it as a check to make rather than a case to expect.

A `scope: module` gate runs **once per run**; the `--module` list scopes its rows, not the number of executions. One `cargo check` covers the whole tree and that is fine, so name **every** module in the run, not just the one an error mentions — under-claiming is the safe direction. Modules *outside* the run keep their earlier pass rows; the batch row is what tells Step 9 the tree does not build.

A `pruned <label> …` line means a row survived under a label the registry no longer declares (a rename). The runner removes it. Nothing to do.

**Never claim a pass the runner did not record.** A recorded pass means exactly "one environment compiled these inputs, against this lockfile and this profile substrate, once". The toolchain version is deliberately not in the key, so `verified: true` never means "compiles everywhere".

Do not hand-edit the `[[gates]]` rows in `gen-lineage.toml`. (`specdev-analyzer gen-record-gate` still exists for recording a result by hand; the codegen flow never calls it.)

Do not auto-edit code to fix build errors stemming from missing scaffolding outside the per-module emission scope (e.g. the entry point doesn't register this module yet) — surface the gap as a one-line action item.

## Step 9 — report

For an incremental `--all`, open with a one-line summary of the plan from Step 3.5: `N unchanged + verified (skipped), R re-gated (unchanged, gate re-run), M regenerated, K edited (skipped — reconcile via /spec-drift)`, naming the `edited` modules and their files so the user can act on them. If the plan carried a `batch:` row with `pass: false`, say so on its own line: the tree did not build the last time a gate ran. A `--force` or single-module run has no plan summary.

Every `edited` and `conflict` line Step 6.2 printed is an **action item**, one line each: the path, what stands there (a hand edit in a derived file; an unrecorded file at a derived target), and the exit — `specdev-analyzer derive --write --accept-edits`, or move the file. A `conflict` records nothing, so the module reads `unchanged` and this line is the only signal that its route file is not the spec's.

"Verified" in this report is the **recorded** fact from Step 8 — a module is verified when `gen-plan` says `verified: true` (a pass row from every module-scope gate in the registry), and not otherwise. Do not call a module verified from memory of having seen the gate pass; if the runner did not record it, it is not verified.

Then one paragraph per module **generated** (skipped modules need no paragraph; re-gated modules get one line each with the gate result):
- Source file(s) written, with paths relative to project root.
- Any prose rule that required a choice and the choice made (e.g., "embassy bridge `handoff = "shared"` chose `Mutex<CriticalSectionRawMutex, ...>`").
- Build result (pass/fail; first error if fail) — and that the runner recorded it (Step 8).
- Any `manifest` warnings (version conflicts, missing typed picks).
- Confirmation that emitted files were recorded in `gen-lineage.toml` (Step 6.5).
- Suggested follow-ups: `/spec-gen <OtherModule>`, wire this module into `main.rs`, regenerate after editing the spec, etc.

---

## What this skill does NOT do

- **Multi-package coordination outside `rust-workspace`.** If members are independent (no `[workspace]` in spec.toml), invoke `/spec-gen` per member from the member's directory.
- **Manifest editing.** The analyzer's `manifest` subcommand is the only path to write the build manifest (Cargo.toml / pubspec.yaml / package.json, plus `pnpm-workspace.yaml` at a typescript workspace root). If a dep is missing, edit the spec.
- **Auto-fix build errors.** Report and stop; the user fixes (likely by editing the spec).
- **Generate placeholder bodies.** No `todo!()` / stubs — stop and ask if the spec is incomplete. The audio-plugin profile's audio-thread methods are the one structured exception, and the silence-and-log emit template for those methods lives in the audio-plugin codegen prose (`codegen/rust/module.conventions.md` § "Audio-thread method bodies"); the prompt assembled in Step 4 already includes it for audio-plugin modules.

## Additional resources

- `references/workspace-mode.md` — full procedure when `[project].profile == "rust-workspace"` (W1–W6: workspace manifest read, validation gate, member dep graph, per-member walk, workspace-root Cargo.toml emit, summary report).
- `references/firmware-layered-postprocess.md` — `memory.x` emission rules for `firmware-layered` projects (placeholder when `mcu_part` is unset, preserve hand-authored versions, MCU-block-driven layout).
