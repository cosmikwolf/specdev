---
name: spec-inline
description: This skill should be used when the user asks to "inline a block", "add embassy/stm32h5/lvgl/etc to the spec", "run specdev inline", "import a block", or wants to copy a canonical block from the plugin tree into the project's `specs/specdev/`. Thin wrapper over `specdev-analyzer inline`. Use `--merge` when re-inlining an already-inlined block to update it.
allowed-tools: Read, Bash, Glob, Grep
argument-hint: <block-name> [--merge]
---

# /spec-inline — Thin wrapper over `specdev-analyzer inline`

Inline a canonical block from the plugin tree (`${CLAUDE_PLUGIN_ROOT}/blocks/<block>/`) into the project's `specs/specdev/inlined/blocks/<block>/`. The analyzer is the source of truth for what a block contributes (schema, reduction, codegen prose); this skill runs the command, surfaces output, and reports.

If `specdev-analyzer` is not on PATH, tell the user to install it (`cargo install --git https://github.com/cosmikwolf/specdev-analyzer`) and stop.

Before any analyzer invocation, run the env preamble from `${CLAUDE_PLUGIN_ROOT}/shared/plugin-root-preamble.md`:

```sh
export SPECDEV_PLUGIN_ROOT="${CLAUDE_PLUGIN_ROOT}"
```

Skip if `SPECDEV_PLUGIN_ROOT` is already set or every invocation passes `--plugin-root`.

---

## Step 1 — parse arguments

User input: `$ARGUMENTS`. Expected: `<block-name> [--merge]`.

- `<block-name>` (required) — must match a directory under `${CLAUDE_PLUGIN_ROOT}/blocks/`. v1 ships `embassy`, `stm32h5-rs` (and historically `stm32f4-rs`). List them with `ls "${CLAUDE_PLUGIN_ROOT}/blocks/"` if the user asks.
- `--merge` (optional) — required to re-inline an already-inlined block; performs a 3-way merge from the recorded upstream snapshot to the current plugin version. Without it, re-inline is refused.

If `<block-name>` is missing, ask the user once and stop.

## Step 2 — run inline

From the project root (cwd):

```sh
specdev-analyzer inline <block-name> [--merge]
```

Capture stdout, stderr, and exit code.

Exit codes:
- `0` — clean. Go to Step 3.
- non-zero — surface stderr verbatim. Common error families:
  - `INLINE-001` — block already inlined; rerun with `--merge` to update, or remove `specs/specdev/inlined/blocks/<block>/` if the user wants a fresh inline (rare; use `--merge` instead).
  - `INLINE-002` — block name not found under the plugin's `blocks/` tree. Verify spelling; list available blocks with `ls "${CLAUDE_PLUGIN_ROOT}/blocks/"`.
  - `INLINE-003` — file collision in the destination tree (a hand-authored file at the same path as a block-emitted file). Surface the path; let the user resolve.
  - `MERGE-001..003`, `MERGE-FILESET` — `--merge` conflict or fileset evolution. Show verbatim; the conflict markers are left in place for retry.
  - `LINEAGE-001` — lineage file inconsistent with the inlined tree. Plugin-author bug or local hand-edit; surface for the user.
  - `PLUGIN-001` — plugin root not resolvable. Re-run the preamble.

Do not hand-edit inlined files to silence errors.

## Step 3 — validate and report

After a successful inline (or merge), run:

```sh
specdev-analyzer validate --no-color
```

A clean inline should leave the project validating. If new findings appear, they're typically schema-shape issues introduced by the block — surface them and suggest `/spec-validate` for the full report.

Report:

- Block inlined (and whether `--merge` was used).
- Files written / changed (count or short list under `specs/specdev/inlined/blocks/<block>/`).
- New `lineage.toml` entry confirmed (or merged with existing).
- Validate result.
- If a paired bridge is the obvious next step (e.g. user just inlined `embassy` after `stm32h5-rs`, or vice versa), suggest `/spec-inline-bridge embassy+stm32h5-rs`. Otherwise no follow-up suggestion.

## On merge conflicts

`--merge` shells out to `git merge-file`. If conflicts occur:

1. The analyzer leaves conflict markers in the affected files and preserves the upstream snapshot + lineage entry.
2. The exit code is non-zero.
3. Surface the conflicting file paths verbatim. Tell the user to resolve markers, then re-run `/spec-validate` to confirm the result is schema-valid.

Do not attempt to auto-resolve conflict markers from within this skill.

## What this skill does NOT do

- **Inline bridges.** Use `/spec-inline-bridge`.
- **Bootstrap a project.** Use `/spec-bootstrap` or `/spec-init` first; `inline` requires `specs/specdev/` to exist.
- **Author module specs that consume the block.** Use `/spec-design` after inline.
- **Edit the inlined files by hand.** They're project-managed but plugin-owned; hand-edits surface as drift in `/spec-drift`.
