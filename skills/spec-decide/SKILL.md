---
name: spec-decide
description: This skill should be used when the user asks to "record a decision", "write a DEC", "add a design decision", "add an ADR for X", "log an ADR", "capture an ADR", "decide <X>", "we decided to use Y", "lock in <choice>", "pin the choice to use Y", "commit to <choice>", "log this as DEC-NNN", "supersede DEC-NNN with a new decision", "this is the decision we're making", or needs to author a single new decision file under `specs/decisions/`. Authors one accepted decision per invocation, then validates. **New decisions only — refuses if a decision with the proposed id already exists.** To supersede an existing decision, this skill can author the successor with `supersedes = [...]`; to resolve an open question, pair it with `/spec-question` path C. To edit the body of an existing decision, edit its file directly.
allowed-tools: Read, Write, Edit, Bash, Glob, Grep
argument-hint: <one-line title for the decision>
---

# /spec-decide — Decision authoring

Record one decision per invocation. User input: `$ARGUMENTS` (a one-line title — what was decided). If `$ARGUMENTS` is empty, ask first.

A decision pins an implementation choice over the alternatives that were considered. It carries the *how* and *why* alongside the requirements (the *what*) and modules (where the *what* is satisfied). Decisions cross-reference the rest of the spec via `requirement.decided_by[]` (added later via `/spec-edit`), `module.guided_by[]` (similarly), and the decision's own `closes_questions[]` / `defaults_for[]` / `supersedes[]` arrays. `architecture.project_decisions[]` is a declaration only: `validate` checks that its ids resolve, but it links the decision to no module.

**How a decision reaches the code.** Whoever edits a module runs `specdev-analyzer report decisions --module <M>` before editing (the project rules say so) and follows every accepted decision listed. A decision reaches a module through exactly three paths: the module's own `guided_by[]`, the `decided_by[]` of every requirement the module `satisfies`, and the workspace tier. For a cross-slice rule ("read the service, not the Db"), wire it through `decided_by[]` on the requirements the affected modules satisfy.

**A decision is not a context file.** A choice, or a known problem with a chosen answer, belongs in a decision (or an open question while unanswered), where the graph checks track it. A module's `context_files` hold reference reading only — a datasheet, a protocol note.

If `specdev-analyzer` is not on PATH, tell the user to install it and stop.

Before any analyzer invocation, run the env preamble from `${CLAUDE_PLUGIN_ROOT}/shared/plugin-root-preamble.md`:

```sh
export SPECDEV_PLUGIN_ROOT="${CLAUDE_PLUGIN_ROOT}"
```

Skip if `SPECDEV_PLUGIN_ROOT` is already set or every invocation passes `--plugin-root`.

---

## Step 0 — prerequisites

If `specs/specdev/` is absent at the project root, stop and tell the user to run `/spec-init` first. Do not author blind.

Read `spec.toml`. If `spec_version` is below `2.0`, the project still uses the combined `specs/decisions.toml`: tell the user to run `specdev-analyzer migrate` first (see `/spec-validate`, "Format migration") and stop.

## Step 1 — read the schema

Read `specs/specdev/schemas/decisions.schema.yaml` to learn the binding contract. Required fields: `id`, `title`, `status`, `text`. Optional prose: `context`, `consequences`, `alternatives`, `verification`. Optional refs: `source[]`, `source_ref`, `supersedes[]`, `closes_questions[]`, `defaults_for[]`. Optional: `codegen_include[]` (which prose fields `report decisions` lists as applying; the name is historical).

List `specs/decisions/` to see the project's id-prefix convention (`DEC-001`, `ADR-042`, `<TICKET>-<NNN>`). Match it. The id grammar is `^[A-Z0-9]+(-[A-Z0-9]+)*-\d+$`.

## Step 2 — interview

Batch all relevant questions in one message. Show defaults so a fluent user can accept them all.

- **id** — next id in the project's series. With no decisions yet, `DEC-001`; otherwise the highest-numbered id of the same prefix, plus one.
- **title** — derived from `$ARGUMENTS`. One line, ≤80 chars.
- **status** — almost always `accepted`. `superseded` only when restoring a historical entry; an older entry being replaced is handled through `supersedes` on this one.
- **text** — what was decided. Required. A paragraph or two stating the choice clearly, like the body of an ADR.
- **context** (optional) — the situation that demanded a decision.
- **consequences** (optional) — second-order effects, downstream costs, what is now constrained.
- **alternatives** (optional) — what was rejected and why.
- **verification** (optional) — how the decision will be tested or validated.
- **source[]** / **source_ref** (optional) — sources informing the decision, and a locator within them.
- **supersedes[]** (optional) — older decision ids this entry replaces (see "Supersession" below).
- **closes_questions[]** / **defaults_for[]** (optional) — open questions this decision closes or commits a default for. **Author one direction per cross-reference:** prefer the OQ-side pointer (`closed_by` / `default_via`, set by `/spec-question`) and leave these empty. The graph checks verify whichever side is authored; both sides double the bookkeeping for no extra check.
- **codegen_include[]** (optional) — default `[text, consequences]`; add `verification` or `alternatives` when those fields matter to whoever implements the module.

If the decision is about **where a list of domain values lives** (a closed list in the spec, or a table in the database), use `${CLAUDE_PLUGIN_ROOT}/shared/vocabulary-placement.md`: the `text` names the list and the placement, and the `context` records the answer to its test (does a non-engineer need to add a value without a deploy?).

**Supersession keeps both bodies.** When the new decision replaces DEC-M: this entry (`accepted`) lists `supersedes = ["DEC-M"]`, and DEC-M's status flips to `superseded` with no forward pointer — the chain is one-way. Remind the user to flip DEC-M's status by editing `specs/decisions/DEC-M.toml`; this skill only creates. `decision_supersedes_chain_consistency` checks that every superseded entry is named by exactly one accepted entry and that the chain has no cycle. `report decisions` follows only accepted decisions.

## Step 3 — write the file

Write `specs/decisions/<id>.toml`. Refuse and stop if that file already exists. The file holds one `[[decision]]` block; use TOML's multi-line strings for prose fields:

```toml
[[decision]]
id = "DEC-NNN"
title = "..."
status = "accepted"
text = """
Body of the decision — one or two paragraphs stating the choice.
"""
consequences = """
What follows from the choice.
"""
source = ["..."]
source_ref = "..."
```

Strip optional fields the user didn't author.

**Check the write against the disk.** Wrap the write in the snapshot/compare pair in `${CLAUDE_PLUGIN_ROOT}/shared/write-check.md`, and quote the entry in Step 5 from the file read back.

## Step 4 — validate

Run from the project root:

```sh
specdev-analyzer validate --no-color
```

Show the output. If errors:

1. Read each error. Schema errors point at a TOML line; graph-check failures point at a rule name.
2. Common failure modes:
   - `REDUCE_UNRESOLVED` naming a `Decision` endpoint → a `supersedes`, `closes_questions`, or `defaults_for` entry names an id that doesn't exist.
   - `decision_supersedes_chain_consistency` → a superseded decision is named by no accepted decision's `supersedes[]`, or by more than one.
   - `[SCHEMA]` errors → a typo in a field name or a wrong status value.
3. Fix the modeling problem. Do not delete fields to silence the checker.
4. Validate again until clean.

## Step 4.5 — show where the decision reaches

```sh
specdev-analyzer report decisions --decision <id>
```

The `decisions` row lists every module this decision reaches. A new decision normally reaches **no** module: nothing links it yet. Say so plainly in Step 5 — a decision that reaches no module guides no code — and name the three ways to link it (the module's `guided_by[]`, a satisfied requirement's `decided_by[]`, the workspace tier).

**When this decision supersedes or corrects another,** look for older **accepted** decisions on the same subject: read their titles under `specs/decisions/` and run `report decisions --decision <older-id>` for each candidate. A module that an older decision reaches and this one does not still has the old text applied to it. Whether two texts contradict is your judgment and the user's — the report only puts them side by side. List each such module in Step 5 and ask the user whether to link this decision there or to correct the older one.

## Step 5 — report

One paragraph:
- Decision id and title written.
- Cross-references created this run.
- Open questions whose status now needs a flip (route to `/spec-question` path C or B), and an old decision whose status needs `superseded`.
- Where it reaches (Step 4.5): the module list, or "no module yet", and any module an older decision on the same subject reaches and this one does not.
- Suggested next step: another `/spec-decide`, `/spec-question` to flip an OQ against this decision, or `/spec-edit` to wire the decision into a requirement's `decided_by[]` or a module's `guided_by[]`.

## What this skill does NOT do

- **Edit existing decisions.** If the file already exists, stop and tell the user.
- **Flip existing OQ statuses or a superseded decision's status.** The user (or `/spec-question`) does that.
- **Author requirements / open questions / modules.** Use `/spec-design` for modules and `/spec-question` for open questions. Requirements are hand-authored under `specs/requirements/`.
