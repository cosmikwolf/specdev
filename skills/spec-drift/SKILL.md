---
name: spec-drift
description: This skill should be used when the user asks "are my inlined blocks stale", "check drift", "run spec-drift", "see what's changed in inlined files", "are my inlined blocks up to date", "is the plugin newer than my project", "check for plugin updates", "show drift report", "did I hand-edit generated code", "check generated files against the spec", "which generated files were edited", or mentions `gen-lineage.toml` or a `gen-drift` report — i.e. they want to know which inlined block/bridge files are out-of-sync with the plugin tree or hand-edited locally, or which generated source files were hand-edited after `/spec-gen` emitted them. Thin wrapper over `specdev-analyzer drift` + `specdev-analyzer report gen-drift`; pure reporter (no edits). For schema/graph-check errors, use /spec-validate. For source-vs-spec drift on a module's `[[items]]` array, use /spec-sync-items.
allowed-tools: Read, Bash, Glob, Grep
---

# /spec-drift — Thin wrapper over `specdev-analyzer drift` + `report gen-drift`

Run the analyzer's drift reporters — inlined-unit drift and generated-file drift — and present the results grouped by drift class. This skill never edits files — drift is observational. Acting on drift (`--merge` to update from plugin, hand-edits to lock in local divergence, etc.) is a separate user decision driven by `/spec-inline --merge`, `/spec-inline-bridge --merge`, or hand-edit + acceptance.

If `specdev-analyzer` is not on PATH, tell the user to install it (follow `${CLAUDE_PLUGIN_ROOT}/shared/analyzer-install.md` — it derives the org from the plugin's own remote; both repositories are private) and stop. If it IS on PATH but reports `MANIFEST-002`, or a `MANIFEST-001` naming a file under the plugin tree, then the analyzer is older than the plugin — not a problem with any project file. Tell the user to re-run the install command with `--force` (or `cargo install --path <checkout> --force` from a local checkout), then retry.

Before any analyzer invocation, run the env preamble from `${CLAUDE_PLUGIN_ROOT}/shared/plugin-root-preamble.md`:

```sh
export SPECDEV_PLUGIN_ROOT="${CLAUDE_PLUGIN_ROOT}"
```

Skip if `SPECDEV_PLUGIN_ROOT` is already set or every invocation passes `--plugin-root`.

---

## Step 1 — run drift

From the project root:

```sh
specdev-analyzer drift
```

Capture stdout and the exit code. Three outcomes:

- **`0`** — the report is the signal, not the exit code. Drift is a reporter: a file classified `locally-edited` still exits `0`.
- **`1`** — a broken invariant (`DRIFT-INVARIANT-00N`): a corrupted tree, such as a file recorded in `lineage.toml` that is no longer on disk. Read the `invariants:` list first; it takes priority over every category below.
- **`2`** — an analyzer error (layout missing, plugin root unresolvable, …). Surface the message verbatim and stop.

## Step 2 — present the four drift classes

The analyzer classifies each tracked inlined file (under `specs/specdev/inlined/{blocks,bridges}/<unit>/`) into exactly one of four classes:

1. **clean** — local file matches the recorded upstream snapshot byte-for-byte; the plugin's current version of the same file also matches. Nothing to do.
2. **plugin-moved** — local file matches upstream snapshot, but the plugin tree has a newer version. `--merge` would fast-forward. Suggest: `/spec-inline <block> --merge` (or `/spec-inline-bridge <bridge> --merge`).
3. **locally-edited** — local file diverges from the upstream snapshot, but the plugin tree's version is unchanged. The user (or a previous skill) hand-edited the inlined file. Surface the path and a one-line hint: "hand-edits are preserved; if intentional, leave them; if accidental, copy from `specs/specdev/upstream/...` to revert."
4. **both** — local file diverges AND the plugin moved. A future `--merge` will perform a 3-way merge, possibly with conflict markers. Suggest the user run `--merge` when ready, and budget conflict-resolution time.

Group findings by class in this order: `both`, `plugin-moved`, `locally-edited`, `clean`. Show counts per class, then the affected paths under each non-empty class.

For `clean`: collapse to a single line ("`<N>` files clean.") unless the user asks for the full list.

## Step 2b — present the profile substrate

The report also carries a `profile:` section covering the files `bootstrap`
scaffolded from the profile — the schemas under `specs/specdev/schemas/`,
`profile.yaml`, `specs/specdev/templates/*`, the root TOMLs. These use the
same four classes, so read them exactly as in Step 2, with two differences.

**`status: not_recorded`.** The project was bootstrapped before the
substrate was hashed, so `files:` is empty and nothing is being claimed
either way. Say that plainly — do NOT report the substrate as clean. Tell
the user the only way to start tracking it is a fresh bootstrap into a new
tree, and that `migrate --refresh` cannot help here because no file can be
proven untouched.

**`comparison: excludes_unit_refs`.** `module.schema.yaml` is co-owned:
`inline` appends each inlined unit's `$ref` to its `allOf` array. That
array is excluded from the comparison and is reported by the `units:`
section instead. So a `clean` verdict on this file means "the profile's part
of it is current" — it says nothing about the `$ref` list.

For a `plugin-moved` profile file, the remedy is NOT `--merge` (that is for
blocks and bridges). Offer:

```sh
specdev-analyzer migrate --refresh --dry-run
```

It rewrites only files whose on-disk hash still matches the hash bootstrap
recorded — proof that nobody edited them. Anything you changed is listed as
`KEPT` and left alone, because profile files keep no `upstream/` base copy
and there is nothing to merge against. Show the dry run, then re-run
without `--dry-run` once the user agrees.

A `locally-edited` or `both` profile file is the user's call. There is no
merge path; the choice is keep the edit or re-apply the plugin's version by
hand.

## Step 3 — check generated files (gen-drift)

Inlined-unit drift is one direction; the other is the code `/spec-gen` emitted. Run:

```sh
specdev-analyzer report gen-drift
```

Exit codes: `0` clean, `1` drift present, `2` analyzer error (surface verbatim and skip this section). The YAML report classifies each file recorded in `specs/specdev/gen-lineage.toml`:

1. **clean** — bytes still match the hash recorded at generation time. Nothing to do.
2. **edited** — the file was generated, then hand-edited. This is the one-way spec→code flow breaking silently: the edit lives only in source and will be lost on the next `/spec-gen` run. Surface the path and suggest: fold the change back into the spec (then regenerate), or — if the edit was accidental — re-run `/spec-gen <module>` to restore spec-derived code. If the row carries `derived = true` in `gen-lineage.toml` (a file `specdev-analyzer derive` wrote — a route shim), the fix is `specdev-analyzer derive --write --accept-edits`, not a regenerate: a derived file is never a hand-edit target.
3. **missing** — recorded at generation time but no longer on disk. Usually a rename or delete outside the spec flow; suggest re-running `/spec-gen <module>` or, if the module was intentionally removed, removing its spec. A `derived = true` row is the exception: `specdev-analyzer derive --write` rewrites the file when a rule still produces it and removes the row when none does, so it is never `missing` for long. When `derive` itself prints `edited <path>: stale derived file kept because of a hand edit`, no rule produces that file any more; remove it or run `derive --write --accept-edits`.

**If the user needs certainty about the code inside functions.** A `clean` line means the file's bytes still match the hash recorded when it was emitted — it answers "did these bytes change since then", nothing more. `specdev-analyzer items diff --module <Name>` is a second opinion that uses no stored record (it re-reads the source with tree-sitter), but it covers only the public surface: names of types, functions and fields, not what is inside them. To check the bodies, re-run `/spec-gen <module>` and compare the output.

**Never re-record to clear an `edited` line.** Recording is what makes a file `clean`, so `gen-record` on an edited file makes the hand-edit the new baseline and this report can never surface it again. The analyzer refuses this by default; the refusal is correct — fix the cause instead, per the `edited` guidance above.

If `gen-lineage.toml` doesn't exist yet (empty report), say so in one line — the project either predates lineage recording or hasn't run `/spec-gen` since; the record starts accruing on the next gen run.

On `GEN-LINEAGE-001` (malformed `gen-lineage.toml`): surface verbatim. The recovery is re-running `/spec-gen` for the affected modules (recording rewrites the file); do not hand-edit it.

## Step 4 — report and suggest

End with one line per surface:

- Inlined units clean: "No drift. `<N>` inlined files match plugin + upstream." Otherwise: "`<both>` both, `<moved>` plugin-moved, `<edited>` locally-edited, `<clean>` clean. Use `/spec-inline <block> --merge` to fast-forward plugin-moved units; review locally-edited paths if unintentional."
- Generated files clean: "`<N>` generated files match their gen-time hashes." Otherwise: "`<edited>` generated files hand-edited, `<missing>` missing — fold edits back into the spec or regenerate."

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
- **Fold hand-edits to generated code back into the spec.** Edit the spec (for `[[items]]`-level drift, `/spec-sync-items` orchestrates it), then `/spec-gen` to regenerate and re-record.
