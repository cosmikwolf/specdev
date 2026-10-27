---
name: spec-reverse
description: "This skill should be used when the user asks to \"reverse-engineer specs\", \"generate specs from code\", \"document this existing codebase\", \"reverse <ModuleName>\", or \"spec-reverse --all\". One-shot migration tool — moves an existing codebase (Rust, TypeScript, Dart or Python) into the spec-driven world; not a continuous source-vs-spec audit (use /spec-sync-items for that). Authors module specs against the project's own profile schema, using the analyzer's tree-sitter `extract-items` and `report imports` surfaces, and proposes requirements from doc comments and test names for the user to confirm."
allowed-tools: Read, Write, Edit, Bash, Glob, Grep
argument-hint: "[<ModuleName> | --all] [--into <a,b,...>]"
---

# /spec-reverse — Extract specs from source

Reverse-engineer TOML specs from existing source. Never invent fields the code does not support. Every claim written must be traceable to a file and line. Works for every language that has an items query (`${CLAUDE_PLUGIN_ROOT}/languages/<lang>/items.scm`): Rust, TypeScript, Dart, Python.

`/spec-reverse` is the **migration tool** for v1 — one-shot for moving an existing codebase into the spec-driven world. Continuous source-vs-spec drift detection (project-wide audit) is not in v1; this skill does not replace audit. For ongoing source-vs-spec drift on a module's `[[items]]`, use `/spec-sync-items`.

If `specdev-analyzer` is not on PATH, tell the user to install it and stop.

Before any analyzer invocation, run the env preamble from `${CLAUDE_PLUGIN_ROOT}/shared/plugin-root-preamble.md`:

```sh
export SPECDEV_PLUGIN_ROOT="${CLAUDE_PLUGIN_ROOT}"
```

Skip if `SPECDEV_PLUGIN_ROOT` is already set or every invocation passes `--plugin-root`.

Modes, decided from `$ARGUMENTS`:

| Input | Mode | Procedure |
|-------|------|-----------|
| empty or missing | Architecture reverse | `references/mode-architecture.md` |
| `--all` | Full project reverse | `references/mode-full-project.md` |
| any other value | Single-module reverse for `<ModuleName>` | `references/mode-single-module.md` |

Optional `--into <block,...>` scopes the reverse vocabulary to specific inlined blocks (e.g. `--into embassy,stm32h5-rs`). Without it, all inlined blocks contribute their READMEs to the prompt context.

---

## Step 0 — prerequisites and bootstrap

If the project has no `specs/specdev/` directory, this is expected for a greenfield reverse. **Stop and tell the user to run `/spec-init` first** to bootstrap a profile-aware tree. `/spec-reverse` is not a substitute for `/spec-init`: the inlined profile schemas (and, for a profile that needs them, the inlined block schemas) drive what fields the reversed module specs may use, and they exist only after `/spec-init`.

If `specs/specdev/` is present, continue. Read `spec.toml` to learn the project's `[project].profile` and `[project].language`.

**Check that the profile fits the tree.** Read the profile README's "When to use" section and compare it with what the tree is: a published library, an application, a server, or a folder of scripts. Compare the profile's gates (test runner, build) with what the tree uses. When they differ, tell the user before any module is written: `check` cannot see a profile that does not fit, and the gates will fail. When no shipped profile fits, the user chooses: go on with the nearest one (and accept its gates), or stop and author a profile with `/spec-create-profile`.

**If `${CLAUDE_PLUGIN_ROOT}/languages/<language>/items.scm` does not exist, stop here** (C and C++ today): there is no `extract-items` for the language and no profile to author against. Tell the user to author the specs with `/spec-init` + `/spec-design`.

Read `specs/specdev/profile.yaml` too. If it declares `reverse.vocabulary: profile`, the profile's own module schema is the reverse vocabulary and no block needs to be inlined; the analyzer reads the same key in Step 1 and reports `[REVERSE-004]` as info instead of stopping on `[REVERSE-002]`. Every profile whose modules name no block field declares it.

For a profile without `reverse.vocabulary: profile` (its modules name block fields), the relevant blocks should be inlined before reversing (the analyzer's pre-flight in Step 1 will tell you which are present):
- An MCU block (e.g., `stm32h5-rs`) — needed to put `peripherals` / `pins` into module specs.
- A runtime block (e.g., `embassy`) — needed to put `tasks` / `channels` into module specs.
- Any relevant bridge (e.g., `embassy+stm32h5-rs`) — needed for `bindings.<a>+<b>` entries.

If the relevant blocks aren't inlined yet, stop after Step 1 and tell the user the `specdev-analyzer inline <block>` commands they need.

## Step 1 — analyzer pre-flight

Run from the project root:

```sh
specdev-analyzer reverse [--into <a,b,...>]
```

The analyzer reports the inlined blocks, the effective vocabulary (`--into` if passed, else all inlined), any errors, and any warnings. Error/warning codes:

- `[REVERSE-001]` — `--into` named a block that's not inlined. Either inline it, drop it from `--into`, or correct the spelling.
- `[REVERSE-002]` — no blocks inlined and no `--into`, and the profile does not declare its own vocabulary. Stop and tell the user to inline at least one block or pass `--into`.
- `[REVERSE-004]` (info) — no blocks inlined, but `profile.yaml` declares `reverse.vocabulary: profile`. The vocabulary is the profile's own schema; proceed.
- `[REVERSE-003]` (warning) — 2+ blocks inlined and no `--into`; vocabulary is ambiguous. Either accept that the skill will use READMEs from every inlined block as prompt context, or restart with `--into <a,b,...>` to scope.

Do not proceed past `[REVERSE-002]` — the skill cannot author against an empty vocabulary. `[REVERSE-004]` is not empty: the profile schema is the vocabulary.

## Step 2 — read the authoritative guides

For each module, run the analyzer's context cone (it prints YAML to stdout; keep it in the conversation, never in a file outside the project):

```sh
specdev-analyzer context --module <Name>
```

For a module that does not exist yet, the cone returns `status: module-not-found` with `files.schema` (the module, architecture and requirements schemas, and the files they `$ref`), `files.block_readmes`, `files.bridge_readmes`, `files.profile_readme` and `files.guidance`, and `files.source` when a file of the default name exists. No `files.source` means the module's file has another name (a kebab-case TypeScript file, a crate root): it needs `source_file`. Read every path under `files:` (project-relative and `${CLAUDE_PLUGIN_ROOT}/...` alike); when `related_files` is non-empty (already-reversed siblings), read those for naming conventions and pattern reuse.

Plus the comment-policy file (not surfaced via the cone since it isn't a per-module concern):

- **`${CLAUDE_PLUGIN_ROOT}/shared/comment-policy.md`** — TOML comment policy (with the `# LOW:` carve-out unique to this skill).

For `--all` mode, each module's pass (`references/mode-single-module.md`, step 4) runs `context` once for that module rather than reading the whole tree per pass.

## Step 3 — route by mode

From the mode table above, load the matching reference file and follow it end to end. Only one mode runs per invocation (except `--all`, which composes the others via `mode-full-project.md`). The mode files own per-mode procedure (stub creation, `extract-items` invocation, TOML authoring, per-module validate); this SKILL.md owns only the cross-mode scaffold above and the rules below.

## Rules of the road

- **Evidence over invention.** Every field must trace to source. If evidence is missing, leave the field out or mark `# LOW: inferred from <file>:<line>, please confirm` and ask.
- **Schema is the contract.** Every key written must validate against `specs/specdev/schemas/module.schema.yaml` (the aggregator). If the code uses a concept the inlined blocks don't model (e.g., raw register pokes that bypass the HAL), capture it as a comment for the user, do not invent a schema field.
- **One module per file.** Do not fold multiple concerns into a single TOML.
- **A separate code root.** When `spec.toml` sets `[paths] code`, the code is in that folder: every `src/…` path in the references is relative to it, and so is each `source_file` you write.
- **Stop on structural ambiguity.** If a source file plausibly represents two modules, ask before splitting.
- **Write only inside the project.** No scratch copy, backup or redirect outside it (`/tmp` included). To see what a check does with an edit, make the edit in the project and undo it.
- **Never change the code.** A reverse writes specs only. Where it finds a defect, a helper that should be private, or a breach of the profile's guidance, it reports it; the user changes the code.
- **An unattended run** (no user to answer): where a step says to ask, confirm or wait, take your own proposal and go on, and list every such question with the answer you took in the report. Write requirements with their `# LOW:` markers kept; `satisfies` and `items` may point at them. A profile that does not fit (Step 0) is the report's first finding. Every requirement of the run is reported as unconfirmed. A public name the attended run would offer to make private is left out (and reported).
- **Comment policy** — follow `${CLAUDE_PLUGIN_ROOT}/shared/comment-policy.md`. This skill's carve-out: inline `# LOW: inferred from <source>:<line>, please confirm` markers are permitted on inferred fields and are removed once the user confirms each one.
- **Profile-shaped reverse.** The fields come from the project's module schema, never from a fixed list: required fields first, then every optional field the code gives evidence for. Each field's `description` (and the enum's, through `$ref`) says what it means and how to read it from the code. When the profile README has a `## Reverse` section, it adds the inference rules the schema cannot hold.
- **Requirements are proposals.** A requirement read from code states what the code does, defects included. Propose each one for the user to confirm, and tag it with the run's `derived` source (see `references/requirements.md`).
- **Vocabulary discipline.** Only use block-namespaced fields whose blocks were in Step 1's effective vocabulary. If a peripheral surfaces in source that no inlined block covers, capture it as a `# LOW:` comment and recommend inlining a covering block before recursing.

## Additional resources

- `references/mode-architecture.md` — procedure for the no-args architecture pass.
- `references/mode-single-module.md` — procedure for `<ModuleName>` single-module reversal (its `extract-items` output, step 6, is the structural basis).
- `references/mode-full-project.md` — procedure for `--all` end-to-end reversal: project-shape survey, then a sequential per-module loop using the same single-module procedure. Parallel fan-out is not available (the `module-reverser` subagent is staged, not shipped).
- `references/requirements.md` — how a reverse proposes requirements: text, `verified_by`, `test_pending`, the `derived` source.
- `references/module-reverser-subagent.md` — the contract and status of the staged `module-reverser` subagent (not shipped; nothing dispatches it).
- `${CLAUDE_PLUGIN_ROOT}/skills/spec-design/references/toml-authoring-rules.md` — priority-ordered TOML authoring rules shared with `/spec-design` and `/spec-edit`. Rule 8 carves out the `# LOW:` marker convention that's unique to `/spec-reverse`; rules 1–7 apply once the user confirms inferred fields and the markers are stripped.

## What this skill does NOT do

- **Replace `/spec-init`.** Bootstrap first; this skill needs the inlined profile (and block) schemas to author against.
- **Inline blocks or bridges.** That's `/spec-inline` / `/spec-inline-bridge`. If the source uses a peripheral or runtime construct that no inlined block covers, stop and recommend the relevant inline.
- **Continuous audit.** Project-wide source-vs-spec audit is not in v1. `/spec-reverse` is the migration tool, not the maintenance tool. For per-module `[[items]]` drift, `/spec-sync-items` is the supported path.
- **Edit existing specs.** That's `/spec-edit`. If `specs/modules/<Name>.toml` already exists, ask whether to compare/replace/cancel before overwriting.
