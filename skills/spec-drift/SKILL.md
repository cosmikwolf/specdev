---
name: spec-drift
description: This skill should be used when the user asks "are my inlined blocks stale", "check drift", "run spec-drift", "see what's changed in inlined files", "are my inlined blocks up to date", "is the plugin newer than my project", "check for plugin updates", or "show drift report" — i.e. they want to know which inlined block/bridge files are out-of-sync with the plugin tree or hand-edited locally. Thin wrapper over `specdev-analyzer drift`; pure reporter (no edits). For schema/graph-check errors, use /spec-validate. For source-vs-spec drift on a module's `[[items]]` array, use /spec-sync-items.
allowed-tools: Read, Bash, Glob, Grep
---

# /spec-drift — Thin wrapper over `specdev-analyzer drift`

Run the analyzer's drift reporter and present the results grouped by drift class. This skill never edits files — drift is observational. Acting on drift (`--merge` to update from plugin, hand-edits to lock in local divergence, etc.) is a separate user decision driven by `/spec-inline --merge`, `/spec-inline-bridge --merge`, or hand-edit + acceptance.

If `specdev-analyzer` is not on PATH, tell the user to install it (`cargo install --git https://github.com/cosmikwolf/specdev-analyzer`) and stop.

Before any analyzer invocation, run the env preamble from `${CLAUDE_PLUGIN_ROOT}/shared/plugin-root-preamble.md`:

```sh
export SPECDEV_PLUGIN_ROOT="${CLAUDE_PLUGIN_ROOT}"
```

Skip if `SPECDEV_PLUGIN_ROOT` is already set or every invocation passes `--plugin-root`.

---

## Step 1 — run drift

From the project root:

```sh
specdev-analyzer drift --no-color
```

Capture stdout and exit code. Drift always exits `0` on a successful run — the report's *content* is the signal, not the exit code. A non-zero exit indicates an analyzer error (layout missing, plugin root unresolvable, …); surface the message verbatim and stop.

## Step 2 — present the four drift classes

The analyzer classifies each tracked inlined file (under `specs/specdev/inlined/{blocks,bridges}/<unit>/`) into exactly one of four classes:

1. **clean** — local file matches the recorded upstream snapshot byte-for-byte; the plugin's current version of the same file also matches. Nothing to do.
2. **plugin-moved** — local file matches upstream snapshot, but the plugin tree has a newer version. `--merge` would fast-forward. Suggest: `/spec-inline <block> --merge` (or `/spec-inline-bridge <bridge> --merge`).
3. **locally-edited** — local file diverges from the upstream snapshot, but the plugin tree's version is unchanged. The user (or a previous skill) hand-edited the inlined file. Surface the path and a one-line hint: "hand-edits are preserved; if intentional, leave them; if accidental, copy from `specs/specdev/upstream/...` to revert."
4. **both** — local file diverges AND the plugin moved. A future `--merge` will perform a 3-way merge, possibly with conflict markers. Suggest the user run `--merge` when ready, and budget conflict-resolution time.

Group findings by class in this order: `both`, `plugin-moved`, `locally-edited`, `clean`. Show counts per class, then the affected paths under each non-empty class.

For `clean`: collapse to a single line ("`<N>` files clean.") unless the user asks for the full list.

## Step 3 — report and suggest

End with one line:

- Everything clean: "No drift. `<N>` inlined files match plugin + upstream."
- Drift present: "`<both>` both, `<moved>` plugin-moved, `<edited>` locally-edited, `<clean>` clean. Use `/spec-inline <block> --merge` to fast-forward plugin-moved units; review locally-edited paths if unintentional."

Do not edit files. Do not run `--merge` from this skill — drift is a read-only report.

## On `LINEAGE-001` errors

If drift errors with `LINEAGE-001`, the project's `lineage.toml` is inconsistent with the inlined tree (e.g., a block was hand-deleted but lineage still claims it; or a file was added under `specs/specdev/inlined/blocks/<x>/` without a matching lineage entry). Surface the message verbatim. Resolving lineage is a manual step:

- Restoring a deleted block: re-inline with `/spec-inline <block>` (refused if `specs/specdev/inlined/blocks/<block>/` exists; remove it first).
- Cleaning up an unexpected file under an inlined unit: delete it (it isn't a block-emitted file) and re-run `/spec-drift` to confirm.

Do not auto-edit `lineage.toml`.

## What this skill does NOT do

- **Update inlined files.** Use `/spec-inline --merge` or `/spec-inline-bridge --merge`.
- **Revert local edits.** Manual step: copy the file from `specs/specdev/upstream/<unit>/<path>` over the local file.
- **Resolve `--merge` conflict markers.** Hand-resolve, then `/spec-validate`.
