---
name: spec-decide
description: This skill should be used when the user asks to "record a decision", "write a DEC", "add a design decision", "add an ADR for X", "log an ADR", "capture an ADR", "decide <X>", "we decided to use Y", "lock in <choice>", "pin the choice to use Y", "commit to <choice>", "log this as DEC-NNN", "supersede DEC-NNN with a new decision", "this is the decision we're making", or needs to author a single new entry under `specs/decisions.toml`. Authors one accepted decision per invocation, then validates. **New decisions only — refuses if a decision with the proposed id already exists.** To supersede an existing decision, this skill can author the successor with `supersedes = [...]`; to mark an OQ as resolved, populate `closes_questions = [...]` and the OQ's status flips to `closed` on the next /spec-validate. To edit the body of an existing decision, edit `specs/decisions.toml` directly.
allowed-tools: Read, Write, Edit, Bash, Glob, Grep
argument-hint: <one-line title for the decision>
---

# /spec-decide — Decision authoring

Record one decision per invocation. User input: `$ARGUMENTS` (a one-line title — what was decided). If `$ARGUMENTS` is empty, ask first.

A decision pins an implementation choice over the alternatives that were considered. It carries the *how* and *why* alongside the requirements (the *what*) and modules (where the *what* is satisfied). Decisions cross-reference the rest of the spec via `requirement.decided_by[]` (added later via `/spec-edit`), `module.guided_by[]` (similarly), and the decision's own `closes_questions[]` / `defaults_for[]` / `supersedes[]` arrays. `architecture.project_decisions[]` is a declaration only: `validate` checks that its ids resolve, but no prompt reads it, so a decision listed there alone reaches no module.

The codegen prompt walk inlines `accepted` decisions into a module's prompt through exactly three paths: the module's own `guided_by[]`, the `decided_by[]` of every requirement the module `satisfies`, and the workspace tier. Authoring a decision here and then wiring it through one of those paths is how you make that text show up at codegen time. For a cross-slice rule ("read the service, not the Db"), wire it through `decided_by[]` on the requirements the affected modules satisfy. **Do not add it to `architecture.project_decisions[]` for that purpose:** the whole of `architecture.toml` is pasted into every module's prompt, so any edit to that file changes every module's input hash and `gen-plan` reports every generated module as stale.

If `specdev-analyzer` is not on PATH, tell the user to install it and stop.

Before any analyzer invocation, run the env preamble from `${CLAUDE_PLUGIN_ROOT}/shared/plugin-root-preamble.md`:

```sh
export SPECDEV_PLUGIN_ROOT="${CLAUDE_PLUGIN_ROOT}"
```

Skip if `SPECDEV_PLUGIN_ROOT` is already set or every invocation passes `--plugin-root`.

---

## Step 0 — prerequisites

If `specs/specdev/` is absent at the project root, stop and tell the user to run `/spec-init` first. Do not author blind.

If `specs/decisions.toml` doesn't exist yet, that's fine — `/spec-init`'s bootstrap emits it as a comment-only file. The first decision lands by appending a `[[decision]]` block.

## Step 1 — read the schema

Read `specs/specdev/schemas/decisions.schema.yaml` to learn the binding contract. Required fields: `id`, `title`, `status`, `text`. Optional prose: `context`, `consequences`, `alternatives`, `verification`. Optional refs: `source[]`, `source_ref`, `supersedes[]`, `closes_questions[]`, `defaults_for[]`. Optional codegen control: `codegen_include[]`.

Skim the existing `specs/decisions.toml` (if any entries) to see the project's id-prefix convention (`DEC-001`, `ADR-042`, `<TICKET>-<NNN>`). Match it. The id grammar is `^[A-Z0-9]+(-[A-Z0-9]+)*-\d+$` — flexible enough for most prefixes.

## Step 2 — interview

Batch all relevant questions in one message. Show defaults so a fluent user can accept them all.

- **id** — next id in the project's series. If `decisions.toml` is empty, `DEC-001`. Otherwise read the highest-numbered id of the same prefix and increment.
- **title** — derived from `$ARGUMENTS`. One line, ≤80 chars.
- **status** — almost always `accepted`. Only use `superseded` if the user is restoring a historical reasoning trail entry; in normal authoring, the status is `accepted` and any older entry being replaced gets its status flipped via the `supersedes` field on this new one.
- **text** — what was decided. Required. Should read like the body of an ADR — a paragraph or two stating the choice clearly.
- **context** (optional) — the situation that demanded a decision. Helpful when the *why* isn't obvious from the title.
- **consequences** (optional) — second-order effects, downstream costs, what's now constrained. Inlined into the codegen prompt by default.
- **alternatives** (optional) — what was rejected and why. Usually noise for codegen — opt in via `codegen_include` if the alternatives matter to the implementation agent.
- **verification** (optional) — how the decision will be tested or validated. Useful when codegen should emit a matching test stub; opt in via `codegen_include`.
- **source[]** (optional) — kebab-case identifiers (`tx81z-manual`, `ymfm`, `rfc-8259`) informing the decision.
- **source_ref** (optional) — free-form locator within the source(s): page, section, line, file path.
- **supersedes[]** (optional) — older decision ids this entry replaces. Each targeted decision must already exist in `decisions.toml` and will have its status flipped to `superseded` separately. Verify after authoring with `/spec-validate`; the `decision_supersedes_chain_consistency` graph check enforces orphan-freedom.
- **closes_questions[]** (optional) — open-question ids this decision closes. The OQ's status flips to `closed` separately (edit `open_questions.toml`).
- **defaults_for[]** (optional) — open-question ids this decision commits a working default for. The OQ's status flips to `default_committed` separately. **Per the authoring convention in `decisions.toml.template`, prefer the OQ-side pointer (`OQ.default_via = "DEC-..."`) over `defaults_for[]` here** — the OQ-side pointer is what the codegen-gate walk reads.
- **codegen_include[]** (optional) — opt-in expansion of which prose fields the codegen prompt walk inlines. Default is `[text, consequences]`. Add `verification` when the decision should drive test-stub emission; add `alternatives` only when the rejected paths inform the implementation.

If the decision is about **where a list of domain values lives** (a closed list in the spec, or a table in the database), use `${CLAUDE_PLUGIN_ROOT}/shared/vocabulary-placement.md`: the `text` names the list and the placement, and the `context` records the answer to its test (does a non-engineer need to add a value without a deploy?).

If the user describes a decision that supersedes an existing accepted decision, fill `supersedes = [...]` with that id AND remind the user that they need to flip the OLD decision's status from `accepted` to `superseded` via direct edit of `specs/decisions.toml`. Do NOT flip the old decision's status from this skill — `/spec-decide` only creates.

## Step 3 — write the TOML

Append a `[[decision]]` block to `specs/decisions.toml`. Refuse and stop only if an **uncommented** `id = "<id>"` line already exists — check with `grep -E '^id = "DEC-NNN"' specs/decisions.toml`. The bootstrapped template carries `# id = "DEC-001"` inside a comment; that is an example, not a decision, and it must not block the first `DEC-001`. Use TOML's multi-line string syntax for prose fields (`text`, `context`, `consequences`, `alternatives`, `verification`):

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

Strip optional fields the user didn't author. The schema accepts their absence; padding the file with empty placeholders is noise.

## Step 4 — validate

Run from the project root:

```sh
specdev-analyzer validate --no-color
```

Show the output. If errors:

1. Read each error. Schema errors point at a TOML line; graph-check failures point at a rule name.
2. Common failure modes:
   - `REDUCE_UNRESOLVED` naming a `Decision` endpoint → a `supersedes`, `closes_questions`, or `defaults_for` entry references an id that doesn't exist. Fix the reference or remove it.
   - `decision_supersedes_chain_consistency` → a Decision with `status = "superseded"` isn't named in any other decision's `supersedes[]` (orphan), OR is named in more than one (multi-successor). Fix the chain.
   - `[SCHEMA]` errors → a typo in a field name or wrong status enum value.
3. Fix the modeling problem. Do not delete fields to silence the checker.
4. Re-validate until clean.

## Step 5 — report

One paragraph:
- Decision id and title written.
- Cross-references created (`closes_questions`, `defaults_for`, `supersedes` populated this run).
- Whether any OQ statuses now need flipping by hand (e.g., the user said this DEC closes OQ-NNN; that OQ's status currently reads `open` and should be flipped to `closed` via `/spec-question` path C). Tell the user explicitly.
- Suggested next step: `/spec-decide <next>` for another decision, `/spec-question` (path C) to flip an OQ's status to `closed` against this decision, or direct edit of `specs/requirements.toml` / a module TOML to wire the new decision into a requirement's `decided_by[]` or a module's `guided_by[]`.

## What this skill does NOT do

- **Edit existing decisions.** If `id` already exists, stop and tell the user.
- **Flip existing OQ statuses.** When `closes_questions` / `defaults_for` references an OQ, the OQ's own status field still reads `open` — the user must update it (or `/spec-edit` post-extension).
- **Flip a superseded decision's status.** Authoring `supersedes = [DEC-OLD]` does not edit DEC-OLD's `status` field — the user must flip that to `superseded` separately.
- **Author requirements / open questions / modules.** Use `/spec-design` for modules and `/spec-question` for open questions. Requirements are still hand-authored in v1.
