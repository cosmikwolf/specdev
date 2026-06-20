---
name: spec-question
description: This skill should be used in three situations. (A) NEW open question — when the user asks to "log an open question", "record an OQ", "we don't know yet", "TODO: figure out X", "this is unresolved", "open question about Y", "OQ-NNN", "we need to decide later". Authors one new entry under `specs/open_questions.toml` then validates; refuses if an OQ with the proposed id already exists. (B) DOWNGRADE / commit a default — when the user says "commit a default for OQ-NNN", "I need to unblock OQ-NNN", "unblock codegen on OQ-NNN", "gen refused on OQ-NNN", "I need to commit a working assumption", or pastes a `Refusing to generate code for module ... open question ... is unresolved` refusal. The skill walks them through flipping `status = "open"` to `status = "default_committed"` — the only in-spec escape from the codegen gate. (C) CLOSE — when the user says "close OQ-NNN", "OQ-NNN is resolved", "mark OQ-NNN closed", "OQ-NNN is moot now". The skill flips `status` to `closed` against an existing accepted Decision.
allowed-tools: Read, Write, Edit, Bash, Glob, Grep
argument-hint: <one-line title for the question>
---

# /spec-question — Open-question authoring

Record one open question per invocation. User input: `$ARGUMENTS` (a one-line title — what's unresolved). If `$ARGUMENTS` is empty, ask first.

An open question is a *known unknown* — something the project hasn't yet pinned down, but where a working assumption lets implementation proceed. The codegen gate refuses to generate code for any module reachable through an OQ with `status = "open"`. The three exits, in order of preference:

1. **Resolve via a decision:** record a status=accepted Decision that answers the question (`/spec-decide`), list the OQ id in that decision's `closes_questions[]`, then flip the OQ's status to `closed` and populate `closed_by`.
2. **Commit a default:** flip the OQ's status from `open` to `default_committed`, set `default_via = "DEC-..."` referencing an accepted decision that committed the working assumption, and provide a `verify = "..."` placeholder explaining how the assumption will be falsified later.
3. **Mark the question moot:** record a status=accepted Decision that lists the OQ in `closes_questions`, then flip the OQ's status to `closed`.

The `default_committed` exit is the in-spec escape: it leaves an audit trail (which decision committed which default) and forces verify-text authoring (a falsification hook that travels with the project). A CLI `--allow-open-questions` flag would erase that trail, so it is deliberately not offered.

If `specdev-analyzer` is not on PATH, tell the user to install it and stop.

Before any analyzer invocation, run the env preamble from `${CLAUDE_PLUGIN_ROOT}/shared/plugin-root-preamble.md`:

```sh
export SPECDEV_PLUGIN_ROOT="${CLAUDE_PLUGIN_ROOT}"
```

Skip if `SPECDEV_PLUGIN_ROOT` is already set or every invocation passes `--plugin-root`.

---

## Step 0 — what does the user actually want?

Three invocation paths, route based on what the user said:

- **A. New OQ authoring.** "Record an open question about X." → Step 1.
- **B. Downgrade open → default_committed.** User hit `codegen_unblocked` at gen time, or said "I want to commit a default for OQ-NNN", or "let me unblock codegen on OQ-NNN with a default." → Step 5 (downgrade flow).
- **C. Close an OQ.** User said "I've decided OQ-NNN — it's resolved" or "OQ-NNN is moot now." → tell them to use `/spec-decide` first to record the resolving Decision (with `closes_questions = ["OQ-NNN"]`), then return here for the status flip in Step 6.

If unclear, ask which.

## Step 1 — prerequisites (path A)

If `specs/specdev/` is absent at the project root, stop and tell the user to run `/spec-init` first.

If `specs/open_questions.toml` doesn't exist yet, that's fine — `/spec-init`'s bootstrap emits it as a comment-only file. The first OQ lands by appending an `[[open_question]]` block.

Read `spec.toml` to learn the project's `[project].profile`. The OQ schema's `affects` field is required at member-tier profiles (audio-plugin / firmware-layered / library-api) but optional at workspace tier (rust-workspace). At workspace tier, an empty `affects` means the OQ implicitly affects every module in every member crate.

## Step 2 — read the schema

Read `specs/specdev/schemas/open_questions.schema.yaml` to learn the binding contract. Required base fields: `id`, `title`, `status`, `text`. The schema's `allOf` enforces status invariants:

| status | required fields | forbidden fields |
|---|---|---|
| `open` | (none beyond base) | `default_via`, `closed_by`, `verify` |
| `default_committed` | `default_via`, `verify` | `closed_by` |
| `closed` | `closed_by` | `verify`, `default_via` |

`affects` is required at member tier with `minItems: 1`; optional at workspace tier (empty/missing means "applies to every module in every member crate"). `category` is informational: `hardware_ab` / `impl_time` / `design`.

Skim the existing `specs/open_questions.toml` (if any entries) to see the project's id-prefix convention (`OQ-001`, `OQ-LS-DB-PER-NOTE`). Match it. The id grammar is `^[A-Z0-9]+(-[A-Z0-9]+)*-\d+$`.

## Step 3 — interview (path A: new OQ)

Batch all relevant questions in one message. Show defaults so a fluent user can accept them all.

- **id** — next id in the project's series.
- **title** — derived from `$ARGUMENTS`. One line, ≤80 chars.
- **status** — `open` is the default for a freshly-noticed unknown. `default_committed` if the user already has a working assumption AND a decision committing it. `closed` if the user already has a decision resolving it (rare — usually the question existed first).
- **text** — the question itself. Should be concrete enough that any of the three exits (answer, commit a default, close as moot) is a clear next step.
- **affects[]** — requirement ids this question affects. At member tier, ≥1 required. At workspace tier, optional (empty = applies to every member-crate module).
- **default_via** — required when status = `default_committed`. The accepted Decision that committed the default. The Decision must already exist in `specs/decisions.toml` — if not, stop and route the user to `/spec-decide` first.
- **verify** — required when status = `default_committed`. One sentence: "Run scenario X, observe Y, confirm Z." This is the falsification hook that travels with the project — without it the working assumption is just a guess.
- **closed_by** — required when status = `closed`. The accepted Decision that closed the question.
- **category** (optional) — `hardware_ab` (waiting on bench measurement), `impl_time` (settles naturally during implementation), `design` (an open architectural choice).

Per the authoring convention in `open_questions.toml.template`: pick ONE direction per cross-reference. If you populate `closed_by` here, do NOT also populate the closing decision's `closes_questions[]` (and vice versa). If you populate `default_via`, do NOT also populate the committing decision's `defaults_for[]`. Graph checks verify whichever direction you author; both directions doubles the bookkeeping cost without doubling the verification.

## Step 4 — write the TOML (path A)

Append an `[[open_question]]` block to `specs/open_questions.toml`. Refuse and stop if the proposed `id` already appears.

Use TOML's multi-line string syntax for prose fields (`text`, `verify`):

```toml
[[open_question]]
id = "OQ-NNN"
title = "..."
status = "open"
text = """
The question itself.
"""
affects = ["REQ-NNN"]
category = "design"  # optional
```

For status = `default_committed`:

```toml
[[open_question]]
id = "OQ-NNN"
title = "..."
status = "default_committed"
text = """
The question + the working assumption being committed.
"""
default_via = "DEC-NNN"
verify = "Run scenario X at conditions Y; observe Z; confirm assumption holds."
affects = ["REQ-NNN"]
```

Strip optional fields the user didn't author.

## Step 5 — downgrade path (path B: open → default_committed)

This is the load-bearing escape from a `codegen_unblocked` refusal. The user hit `gen` and got:

> Refusing to generate code for module `<mod>`: open question `OQ-NNN` (`<title>`) is unresolved.

To unblock without answering the question outright, commit a default:

1. Read the existing OQ entry in `specs/open_questions.toml` (find the `[[open_question]]` block whose `id = "OQ-NNN"`). Confirm `status = "open"`.
2. Help the user identify which Decision will commit the default. Two paths:
   - The user already has an accepted Decision in `specs/decisions.toml` that pins the working assumption — use its id.
   - No such Decision exists — stop and tell the user to run `/spec-decide` first to record one. Provide the `id` to use as `default_via` once that lands.
3. Help the user write `verify` text. This is the load-bearing field — a default without a falsification hook is just a guess. Push for specificity: "Hardware A/B at C3 with FIX=1 sweep DT1 0..6, FFT-compare against TX81Z capture" beats "verify against hardware later." If the user can't articulate a verify path, the question isn't ready for `default_committed` — they should either keep it `open` and accept the gate, or convert it into a Decision and close it.
4. Edit the OQ block in place:
   - Change `status = "open"` to `status = "default_committed"`.
   - Add `default_via = "DEC-NNN"`.
   - Add `verify = "..."`.
5. Run `specdev-analyzer validate --no-color` and confirm it passes.
6. Tell the user that `gen` will now succeed for the previously-blocked module, with the placeholder + verify text inlined into the codegen prompt.

## Step 6 — close path (path C: open → closed)

Prerequisite: an accepted Decision exists that resolves the question. If not, stop and route the user to `/spec-decide`.

1. Read the existing OQ entry. Confirm `status = "open"` or `default_committed`.
2. Edit the OQ block:
   - Change `status` to `closed`.
   - Add `closed_by = "DEC-NNN"`.
   - Remove `verify` and `default_via` if present (the schema's `allOf` forbids them on closed entries — leaving them tripped the schema check).
3. Run `specdev-analyzer validate --no-color` and confirm it passes.

## Step 7 — validate (all paths)

Run from the project root:

```sh
specdev-analyzer validate --no-color
```

Show the output. If errors:

1. **`[SCHEMA]` allOf failure** — status invariants violated. E.g. `status = "default_committed"` without `default_via` or `verify`; `status = "open"` with `default_via`. Fix the field set per the table in Step 2.
2. **`REDUCE_UNRESOLVED` naming a `Decision` endpoint** — `default_via` or `closed_by` references an id that doesn't exist. Author the Decision via `/spec-decide` first, OR fix the typo.
3. **`REDUCE_UNRESOLVED` naming a `Requirement` endpoint** — `affects[]` references a requirement id that doesn't exist. Fix the reference or remove it.
4. **`open_question_status_consistency`** — `closed_by` or `default_via` points at a Decision whose status isn't `accepted`. Either the target Decision is `superseded` (use a different DEC), or its status was wrongly authored.
5. **`codegen_unblocked` warning naming a module** — informational. The OQ being authored (or a sibling) is `open` and reachable through that module. `gen` will refuse for that module until the OQ is downgraded or closed. Expected if the user is still in path A and hasn't committed a default yet.

## Step 8 — report

One paragraph:
- OQ id and path action taken (created / downgraded / closed).
- For path B (downgrade): the Decision id used as `default_via` and the verify text.
- For path A: which requirement(s) the OQ affects and which modules will be gated until the OQ is downgraded or closed (look up modules whose `satisfies[]` overlaps the OQ's `affects[]`).
- Suggested next step. After path A: typically `/spec-design` or `/spec-edit` for the affected modules, knowing they'll be gated. After path B or C: re-run the previously-blocked `gen` invocation.

## What this skill does NOT do

- **Edit an existing OQ's body text or affects list.** If `id` already exists and the user wants to rephrase the question, edit `specs/open_questions.toml` directly. The skill only handles status transitions (paths B and C) on existing entries; everything else is a creation flow.
- **Flip a Decision's status.** Closing an OQ via path C does NOT mark the closing Decision as `accepted` if it isn't already. Author that decision via `/spec-decide` first.
- **Author the Decision that closes / defaults a question.** Use `/spec-decide` for the Decision; then loop back here to flip the OQ status.
- **Bypass the gate.** No CLI flag exists; the in-spec escape is path B. If the user wants codegen to proceed without committing a default and without resolving the question, the answer is "you can't" — the gate is load-bearing.
