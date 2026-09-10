---
name: spec-inline
description: This skill should be used when the user names any block (e.g. "inline embassy", "add stm32h5-rs", "inline nih-plug", "add egui", or any user-authored block) or asks to "inline a block", "add a block to the spec", "run specdev inline", "import a block". Thin wrapper over `specdev-analyzer inline`. Resolves block names dynamically against `${CLAUDE_PLUGIN_ROOT}/blocks/` so user-authored blocks work the same as plugin-shipped ones. Use `--merge` when re-inlining an already-inlined block to update it. To wire two inlined blocks together, use /spec-inline-bridge.
allowed-tools: Read, Bash, Glob, Grep
argument-hint: <block-name> [--merge]
---

# /spec-inline — Thin wrapper over `specdev-analyzer inline`

Inline a canonical block from the plugin tree (`${CLAUDE_PLUGIN_ROOT}/blocks/<block>/`) into the project's `specs/specdev/` tree. A block's inlined content is **split across** four locations (there is no single `inlined/blocks/<block>/` directory): `schemas/inlined/blocks/<block>.schema.yaml`, `reductions/blocks/<block>.yaml`, `codegen/<lang>/blocks/<block>/`, and an `upstream/blocks/<block>/` snapshot (used by `--merge`). The analyzer is the source of truth for what a block contributes (schema, reduction, codegen prose); this skill runs the command, surfaces output, and reports.

If `specdev-analyzer` is not on PATH, tell the user to install it (`cargo install --git https://github.com/cosmikwolf/specdev-analyzer`) and stop. If it IS on PATH but reports `MANIFEST-002`, or a `MANIFEST-001` naming a file under the plugin tree, then the analyzer is older than the plugin — not a problem with any project file. Tell the user to re-run the install command with `--force` (or `cargo install --path <checkout> --force` from a local checkout), then retry.

Before any analyzer invocation, run the env preamble from `${CLAUDE_PLUGIN_ROOT}/shared/plugin-root-preamble.md`:

```sh
export SPECDEV_PLUGIN_ROOT="${CLAUDE_PLUGIN_ROOT}"
```

Skip if `SPECDEV_PLUGIN_ROOT` is already set or every invocation passes `--plugin-root`.

---

## Step 1 — parse arguments

User input: `$ARGUMENTS`. Expected: `<block-name> [--merge]`.

- `<block-name>` (required) — must match a directory under `${CLAUDE_PLUGIN_ROOT}/blocks/`. Never hardcode the available set; list dynamically with `ls "${CLAUDE_PLUGIN_ROOT}/blocks/"` — every directory there is a valid choice, including user-authored blocks. Read `${CLAUDE_PLUGIN_ROOT}/blocks/<name>/README.md` for what the block contributes.
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
  - `INLINE-001` — block already inlined; rerun with `--merge` to update (recommended). A fresh inline would require removing the block's inlined files across `specs/specdev/` (schema, reduction, codegen, `upstream/` snapshot) — rare; prefer `--merge`.
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
- Files written / changed (count or short list under `specs/specdev/` — schema, reduction, codegen prose, `upstream/` snapshot).
- New `lineage.toml` entry confirmed (or merged with existing).
- Validate result.
- If a paired bridge exists in the plugin tree for this block + an already-inlined block, suggest the corresponding `/spec-inline-bridge <a>+<b>` (lex-ordered). Discover paired bridges with `ls "${CLAUDE_PLUGIN_ROOT}/bridges/" | grep <block-name>` — surface only bridges where the *other* endpoint is already inlined in the project. Otherwise no follow-up suggestion.

## On merge conflicts

`--merge` shells out to `git merge-file`. If conflicts occur:

1. The analyzer leaves conflict markers in the affected files and preserves the upstream snapshot + lineage entry.
2. The exit code is non-zero.
3. Surface the conflicting file paths verbatim. Tell the user to resolve markers, then re-run `/spec-validate` to confirm the result is schema-valid.

Do not attempt to auto-resolve conflict markers from within this skill.

## What this skill does NOT do

- **Inline bridges.** Use `/spec-inline-bridge`.
- **Bootstrap a project.** Use `/spec-init` first; `inline` requires `specs/specdev/` to exist.
- **Author module specs that consume the block.** Use `/spec-design` after inline.
- **Edit the inlined files by hand.** They're project-managed but plugin-owned; hand-edits surface as drift in `/spec-drift`.
