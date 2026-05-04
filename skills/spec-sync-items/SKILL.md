---
name: spec-sync-items
description: This skill should be used when the user asks to "sync items", "reconcile spec items with source", "run items propose", "audit module items", "what's drifted between my spec and source", or wants to align a module's `[[items]]` array with what tree-sitter actually finds in the source file. Three stages: deterministic detect → Claude-advisory propose with rationale → human-approved apply. Refuses to skip stages. For schema/graph-check errors, use /spec-validate. For inlined-block-vs-plugin drift, use /spec-drift.
allowed-tools: Read, Edit, Write, Bash, Glob, Grep
---

# /spec-sync-items — Three-stage items[] sync orchestrator

Reconcile a module's spec `[[items]]` array with what its source file actually exposes. Three stages, one per CLI subcommand, with a human approval gate between proposal and apply:

1. **Detect** — `specdev-analyzer items diff` (deterministic; no AI).
2. **Propose** — `specdev-analyzer items propose` writes a scaffold; Claude (this skill) fills in `rationale` and `action` per entry; human reviews and signs off.
3. **Apply** — `specdev-analyzer items apply` mutates `specs/modules/<Module>.toml` with `toml_edit` (preserves comments outside `[[items]]`).

The audit trail is the `<module>.applied.yaml` file under `specs/specdev/upstream/items-sync/`. Each apply leaves one — review-approve-apply happened, here's the record.

If `specdev-analyzer` is not on PATH, tell the user to install it (`cargo install --git https://github.com/cosmikwolf/specdev-analyzer`) and stop.

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

Read `specs/specdev/upstream/items-sync/<Module>.proposal.yaml` and walk each entry. For each `pending` entry, decide which of the four actions fits:

- **`skip`** — leave both spec and source as-is. Pick this when the drift represents a deliberate gap. Common cases:
  - `in_source_only` for a private helper that the spec deliberately doesn't enumerate (sparseness rule per D42 — items[] is a sparse subset, not a full mirror).
  - `signature_drift` where the spec's `signature_hint` was a documentation aid that's drifted and the user prefers to drop the hint over chasing the source. (In that case, `update-spec` with the hint removed is also a valid choice — pick whichever the entry's prose suggests.)

- **`add-to-spec`** — the source item is contractual and should be enumerated in the spec. Picks for `in_source_only` entries where the function/struct/etc. is part of the module's public surface AND traces to a requirement. Fill `proposed_spec` with the new [[items]] mapping (`kind`, `name`, `description`, `satisfies`, optional `signature_hint`). At minimum carry `satisfies` (D42 sparseness invariant — every item except `re_export` must satisfy ≥1 requirement).

- **`remove-from-spec`** — the spec entry is stale; the source has moved on without it. Picks for `in_spec_only` entries where the function was renamed or deleted in source and the spec wasn't updated. No `proposed_spec` needed.

- **`update-spec`** — the spec entry exists but its shape needs to change. Picks for `signature_drift` entries (most commonly) where the source signature is the right one and the spec hint is stale. Fill `proposed_spec` with the full new mapping (`update-spec` REPLACES the matched entry; partial merges are not supported — carry the full intended shape).

For each entry, write:
- `rationale` — one or two sentences naming WHY this action. Refer to specifics: which requirement, which signature mismatch, which sparseness call. Generic prose ("looks fine") is not useful audit material.
- `action` — one of the four above.
- `proposed_spec` — required for `add-to-spec` and `update-spec`; absent for `skip` and `remove-from-spec`.

When you're done, also flip the top-level `status` from `needs-rationale` to `awaiting-approval`.

**Do NOT set `approved`, `approver`, or `approved_at`.** That's the human's job.

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

- **0** — clean apply. The analyzer rewrites `specs/modules/<Module>.toml` (via `toml_edit` so non-`[[items]]` content round-trips), writes the audit log to `specs/specdev/upstream/items-sync/<Module>.applied.yaml` with `status: applied` + `applied_at`, and removes the `.proposal.yaml`. The CLI prints a one-block summary (`added`/`removed`/`updated`/`skipped` per entry id).
- **1** — gate failure (`ITEM-301` schema mismatch, `ITEM-302` not approved, `ITEM-303` missing approver, `ITEM-304` missing `approved_at`, `ITEM-305` an entry still on `pending`, `ITEM-306` add/update without `proposed_spec`, `ITEM-309` target item not found in spec, etc.). Surface stderr verbatim and stop. The spec is unchanged on a gate failure.
- **2** — environmental error (missing layout, proposal file missing). Surface and stop.

After a clean apply, suggest the user run `/spec-validate` to confirm the new items[] shape parses against the schema.

## Step 6 — close out

End with one line:

- Clean: "Synced `<Module>`: `<a>` added, `<r>` removed, `<u>` updated, `<s>` skipped. Audit: `specs/specdev/upstream/items-sync/<Module>.applied.yaml`."
- Gated: name the gate (`ITEM-302` etc.) and the next step.

---

## What this skill REFUSES to do

- **Skip stages.** The deterministic / advisory / authoritative split is the whole point. Don't run `apply` without a proposal file. Don't write a proposal file straight from `diff` without going through `propose`.
- **Set `approved: true` itself.** That flag is the human's signal. Anyone can run apply once the flag is set; only the user should be the one to set it.
- **Mass-fan-out across modules.** One module per invocation. The proposal artifact lives per-module for a reason.
- **Bypass `--clobber` silently.** If a proposal already exists, ask before overwriting.
- **Edit the spec module file directly.** Always go through `items apply` so the audit trail is preserved.

## On `[ITEM-309]` (target not found)

If apply fires `ITEM-309` for a `remove-from-spec` or `update-spec` action, the spec moved between `propose` and `apply` (the user manually edited `[[items]]` in the meantime). Re-run `propose --clobber` and the new diff will reflect the current state.

