---
name: spec-sync-items
argument-hint: <ModuleName>
description: This skill should be used when the user asks "is my source out of sync with the spec items", "sync items", "reconcile spec items with source", "run items propose", "audit module items", "what's drifted between my spec and source", or wants to align a module's `[[items]]` array with what tree-sitter actually finds in the source file. Three stages: deterministic detect → Claude-advisory propose with rationale → human-approved apply. Refuses to skip stages. For schema/graph-check errors, use /spec-validate. For inlined-block-vs-plugin drift, use /spec-drift.
allowed-tools: Read, Edit, Write, Bash, Glob, Grep
---

# /spec-sync-items — Three-stage items[] sync orchestrator

Reconcile a module's spec `[[items]]` array with what its source file actually exposes. Three stages, one per CLI subcommand, with a human approval gate between proposal and apply:

1. **Detect** — `specdev-analyzer items diff` (deterministic; no AI).
2. **Propose** — `specdev-analyzer items propose` writes a scaffold; Claude (this skill) fills in `rationale` and `action` per entry; human reviews and signs off.
3. **Apply** — `specdev-analyzer items apply` mutates `specs/modules/<Module>.toml` with `toml_edit` (preserves comments outside `[[items]]`).

The audit trail is the `<module>.applied.yaml` file under `specs/specdev/upstream/items-sync/`. Each apply leaves one — review-approve-apply happened, here's the record.

If `specdev-analyzer` is not on PATH, tell the user to install it (follow `${CLAUDE_PLUGIN_ROOT}/shared/analyzer-install.md` — it derives the org from the plugin's own remote; both repositories are private) and stop. If it IS on PATH but reports `MANIFEST-002`, or a `MANIFEST-001` naming a file under the plugin tree, then the analyzer is older than the plugin — not a problem with any project file. Tell the user to re-run the install command with `--force` (or `cargo install --path <checkout> --force` from a local checkout), then retry.

Before any analyzer invocation, run the env preamble from `${CLAUDE_PLUGIN_ROOT}/shared/plugin-root-preamble.md`:

```sh
export SPECDEV_PLUGIN_ROOT="${CLAUDE_PLUGIN_ROOT}"
```

Skip if `SPECDEV_PLUGIN_ROOT` is already set or every invocation passes `--plugin-root`.

---

## Step 0 — pick the module

The user invocation should name the module (e.g. `/spec-sync-items Foo`). If absent:

- If `specs/modules/` contains exactly one TOML, use its stem.
- Otherwise list candidates and ask the user which one to sync.

Refuse to fan-out across modules in one run. The proposal artifact + approval workflow is per-module on purpose; running this against five modules silently would mean five proposal files and five approval rounds, and the user would lose track of which they've reviewed.

## Step 1 — detect drift

```sh
specdev-analyzer items diff --module <Module>
```

Capture stdout (YAML) and exit code. Three possibilities:

- **Exit 0** — no drift. Stop. Tell the user "spec items[] match source." Do not write a proposal.
- **Exit 1** — drift present. Stdout YAML carries `in_spec_only`, `in_source_only`, and `both[]` (with `signature_drift` flagged on the diverging pairs). Continue to Step 2.
- **Exit 2** — hard error (missing module spec, source-read failure, language adapter missing). Surface stderr verbatim and stop.

Show the user a one-line summary per category (counts, not full paths) so they know the shape before you pull up a proposal file.

## Step 2 — write the proposal scaffold

```sh
specdev-analyzer items propose --module <Module>
```

This writes `specs/specdev/upstream/items-sync/<Module>.proposal.yaml`. The analyzer's job is purely deterministic: each drift entry is enriched with the source snippet (sliced by tree-sitter span) and the spec [[items]] mapping. Every entry's `recommendation.action` is `pending`; this skill (next step) fills them in.

If a proposal file already exists, the analyzer refuses and emits `[ITEM-202]`. Two cases:

- **Existing file is stale (no rationale yet, no recent review).** Re-run with `--clobber` and continue.
- **Existing file carries unsubmitted rationales.** Stop and ask the user — they may want to review the existing file rather than regenerate. Surface the path; let them decide.

Default behavior when in doubt: stop and ask. Losing rationales silently is worse than asking.

## Step 3 — fill in recommendations (Claude-advisory pass)

Read `specs/specdev/upstream/items-sync/<Module>.proposal.yaml`. Walk each `pending` entry and pick one of five actions per the taxonomy in `references/action-taxonomy.md` (`skip` / `add-to-spec` / `remove-from-spec` / `update-spec` / `rename`). For each entry, write `rationale`, `action`, and (for `add-to-spec` / `update-spec`) `proposed_spec` or (for `rename`) `rename_to`. A symbol renamed in source is a `rename` on the `in_spec_only` old-name entry + a `skip` on the paired `in_source_only` new-name entry — never remove + add, which drops the `satisfies`/`verifies` trace.

Save the file. Tell the user how many entries were resolved per action category, and ask them to review.

## Step 4 — wait for human approval

The user reads the proposal file and either:

- Edits a few entries (changing your `action`, refining `rationale`, tweaking `proposed_spec`), then sets:
  ```yaml
  approved: true
  approver: <their handle or email>
  approved_at: <ISO-8601 timestamp>
  ```
- Or rejects the proposal wholesale (deletes the file, asks for a different approach).

Do not edit the proposal on the user's behalf at this stage. The approval gate is what makes Stage 3 (apply) safe.

If the user comes back with edits but no approval flags, re-read the file, summarize what changed, and ask them to set `approved: true` + `approver` + `approved_at` to proceed.

## Step 5 — apply

```sh
specdev-analyzer items apply --module <Module>
```

Exit codes:

- **0** — clean apply. The analyzer rewrites `specs/modules/<Module>.toml` (via `toml_edit` so non-`[[items]]` content round-trips), writes the audit log to `specs/specdev/upstream/items-sync/<Module>.applied.yaml` with `status: applied` + `applied_at`, and removes the `.proposal.yaml`. The CLI prints a one-block summary (`added`/`removed`/`updated`/`renamed`/`skipped` per entry id).
- **1** — gate failure (`ITEM-301` schema mismatch, `ITEM-302` not approved, `ITEM-303` missing approver, `ITEM-304` missing `approved_at`, `ITEM-305` an entry still on `pending`, `ITEM-306` add/update without `proposed_spec`, `ITEM-309` target item not found in spec, etc.). Surface stderr verbatim and stop. The spec is unchanged on a gate failure.
- **2** — environmental error (missing layout, proposal file missing). Surface and stop.

After a clean apply, suggest the user run `/spec-validate` to confirm the new items[] shape parses against the schema.

## Step 6 — close out

End with one line:

- Clean: "Synced `<Module>`: `<a>` added, `<r>` removed, `<u>` updated, `<n>` renamed, `<s>` skipped. Audit: `specs/specdev/upstream/items-sync/<Module>.applied.yaml`."
- Gated: name the gate (`ITEM-302` etc.) and the next step.

---

## What this skill REFUSES to do

- **Skip stages.** The deterministic / advisory / authoritative split is the whole point. Don't run `apply` without a proposal file. Don't write a proposal file straight from `diff` without going through `propose`.
- **Set `approved: true` itself.** That flag is the human's signal. Anyone can run apply once the flag is set; only the user should be the one to set it.
- **Mass-fan-out across modules.** One module per invocation. The proposal artifact lives per-module for a reason.
- **Bypass `--clobber` silently.** If a proposal already exists, ask before overwriting.
- **Edit the spec module file directly.** Always go through `items apply` so the audit trail is preserved.

## On `[ITEM-309]` (target not found)

If apply fires `ITEM-309` for a `remove-from-spec`, `update-spec`, or `rename` action, the spec moved between `propose` and `apply` (the user manually edited `[[items]]` in the meantime). Re-run `propose --clobber` and the new diff will reflect the current state. A `rename` with no `rename_to` fires `ITEM-315` — add the new name to the recommendation.

## Additional resources

- `references/action-taxonomy.md` — the five `recommendation.action` values, when to pick each, the `proposed_spec` shape required for `add-to-spec` and `update-spec`, and the `rename_to` field for `rename`.

