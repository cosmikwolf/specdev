---
name: spec-question
description: "This skill should be used in three situations. (A) NEW open question — when the user asks to \"log an open question\", \"record an OQ\", \"we don't know yet\", \"TODO: figure out X\", \"this is unresolved\", \"open question about Y\", \"OQ-NNN\", \"we need to decide later\". Authors one new file `specs/open_questions/<id>.toml` then validates; refuses if an OQ with the proposed id already exists. (B) DOWNGRADE / commit a default — when the user says \"commit a default for OQ-NNN\", \"I need to unblock OQ-NNN\", \"check fails on OQ-NNN\", \"I need to commit a working assumption\", or pastes an `implementation_unblocked` finding (`Module ... is blocked: open question ... is unresolved, and code for it exists`). The skill walks them through flipping `status = \"open\"` to `status = \"default_committed\"` — the only in-spec escape from the open-question gate. (C) CLOSE — when the user says \"close OQ-NNN\", \"OQ-NNN is resolved\", \"mark OQ-NNN closed\", \"OQ-NNN is moot now\". The skill flips `status` to `closed` against an existing accepted Decision."
allowed-tools: Read, Write, Edit, Bash, Glob, Grep
argument-hint: <one-line title for the question>
---

# /spec-question — Open-question authoring

Record one open question per invocation. User input: `$ARGUMENTS` (a one-line title — what's unresolved). If `$ARGUMENTS` is empty, ask first.

An open question is a *known unknown* — something the project hasn't yet pinned down. The open-question gate (`implementation_unblocked`, run by `specdev-analyzer check`) reports an error for every module that an `open` question reaches **once that module has code**: building on an unanswered question is the failure it catches. Designing and planning around an open question is fine. The three exits, in order of preference:

1. **Resolve via a decision:** record a status=accepted Decision that answers the question (`/spec-decide`), then flip the OQ's status to `closed` and set `closed_by`.
2. **Commit a default:** flip the OQ's status from `open` to `default_committed`, set `default_via = "DEC-..."` referencing an accepted decision that committed the working assumption, and give a `verify = "..."` line saying how the assumption will be falsified later.
3. **Mark the question moot:** record a status=accepted Decision that closes it, then flip the OQ's status to `closed`.

The `default_committed` exit is the in-spec escape: it leaves an audit trail (which decision committed which default) and forces a falsification hook that travels with the project. There is deliberately no CLI flag that bypasses the gate.

**Who may commit a default.** `hard_gate = true` (the default) means a person resolves the question or commits its default — an agent proposes, the user decides. `hard_gate = false`, set by a person, lets an agent commit a recorded default; `check --since <ref>` reports each one so nothing slips by unseen. Never flip `hard_gate` yourself.

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
- **B. Downgrade open → default_committed.** User hit `implementation_unblocked` in `check`, or said "I want to commit a default for OQ-NNN". → Step 5.
- **C. Close an OQ.** User said "I've decided OQ-NNN — it's resolved" or "OQ-NNN is moot now." → tell them to use `/spec-decide` first to record the resolving Decision, then return here for the status flip in Step 6.

If unclear, ask which.

## Step 1 — prerequisites (path A)

If `specs/specdev/` is absent at the project root, stop and tell the user to run `/spec-init` first.

Read `spec.toml`. If `spec_version` is below `2.0`, the project still uses the combined `specs/open_questions.toml`: tell the user to run `specdev-analyzer migrate` first (see `/spec-validate`, "Format migration") and stop.

Read `[project].profile`. The OQ schema's `affects` field is required at member-tier profiles but optional at workspace tier (`rust-workspace`). At workspace tier, an empty `affects` means the OQ affects every module in every member crate, and the gate reads it from each member.

## Step 2 — read the schema

Read `specs/specdev/schemas/open_questions.schema.yaml` to learn the binding contract. Required base fields: `id`, `title`, `status`, `text`. The schema's `allOf` enforces status invariants:

| status | required fields | forbidden fields |
|---|---|---|
| `open` | (none beyond base) | `default_via`, `closed_by`, `verify` |
| `default_committed` | `default_via`, `verify` | `closed_by` |
| `closed` | `closed_by` | `verify`, `default_via` |

`affects` is required at member tier with `minItems: 1` — an open question that affects nothing should be rephrased into a decision or deleted. `category` is informational: `hardware_ab` / `impl_time` / `design`. `hard_gate` defaults to `true`.

List `specs/open_questions/` to see the project's id-prefix convention (`OQ-001`, `OQ-LS-DB-001`). Match it. The id grammar is `^[A-Z0-9]+(-[A-Z0-9]+)*-\d+$`.

## Step 3 — interview (path A: new OQ)

Batch all relevant questions in one message. Show defaults so a fluent user can accept them all.

- **id** — next id in the project's series.
- **title** — derived from `$ARGUMENTS`. One line, ≤80 chars.
- **status** — `open` is the default for a freshly-noticed unknown. `default_committed` if the user already has a working assumption AND a decision committing it. `closed` if a decision already resolves it (rare — usually the question existed first).
- **text** — the question itself. Concrete enough that any of the three exits is a clear next step.
- **affects[]** — requirement ids this question affects. At member tier, ≥1 required.
- **default_via** — required when status = `default_committed`. The accepted Decision that committed the default; it must already exist under `specs/decisions/` — if not, stop and route the user to `/spec-decide` first.
- **verify** — required when status = `default_committed`. One sentence: "Run scenario X, observe Y, confirm Z."
- **closed_by** — required when status = `closed`. The accepted Decision that closed the question.
- **category** (optional) — `hardware_ab` (waiting on bench measurement), `impl_time` (settles during implementation), `design` (an open architectural choice).

**One direction per cross-reference.** Author `closed_by` here *or* the decision's `closes_questions[]`, not both; author `default_via` here *or* the decision's `defaults_for[]`, not both. Prefer the OQ-side pointer (`closed_by` / `default_via`): it travels with the entry the gate reads, and the decision stays about the decision. The graph checks verify whichever direction is authored; both directions double the bookkeeping for no extra check.

## Step 4 — write the file (path A)

Write `specs/open_questions/<id>.toml`, wrapped in the snapshot/compare pair in `${CLAUDE_PLUGIN_ROOT}/shared/write-check.md`. Refuse and stop if that file already exists.

The file holds one `[[open_question]]` block. Use TOML's multi-line strings for prose fields (`text`, `verify`):

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

This is the escape from an `implementation_unblocked` error. `check` reported:

> Module `<mod>` is blocked: open question `OQ-NNN` (`<title>`) is unresolved, and code for it exists.

1. Read `specs/open_questions/OQ-NNN.toml`. Confirm `status = "open"`. If `hard_gate` is `true` or absent, the user makes this call — present the default and wait for their yes.
2. Identify the Decision that commits the default:
   - An accepted Decision under `specs/decisions/` already pins the working assumption — use its id.
   - None exists — stop and tell the user to run `/spec-decide` first. Give the `id` to use as `default_via` once it lands.
3. Write `verify` text with the user. A default without a falsification hook is just a guess. Push for specificity: "Hardware A/B at C3 with FIX=1, sweep DT1 0..6, FFT-compare against the reference capture" beats "verify against hardware later." If the user can't say how the assumption would fail, keep the question `open`, or turn it into a Decision and close it.
4. Edit the file in place: `status = "default_committed"`, add `default_via = "DEC-NNN"`, add `verify = "..."`.
5. Run `specdev-analyzer check --no-color` and confirm the gate finding is gone.

## Step 6 — close path (path C: open → closed)

Prerequisite: an accepted Decision exists that resolves the question. If not, stop and route the user to `/spec-decide`.

1. Read `specs/open_questions/OQ-NNN.toml`. Confirm `status = "open"` or `default_committed`.
2. Edit it: `status = "closed"`, add `closed_by = "DEC-NNN"`, remove `verify` and `default_via` if present (the schema forbids them on closed entries).
3. Run `specdev-analyzer check --no-color`.

**Every path writes one file under `specs/open_questions/`.** Wrap that write in the snapshot/compare pair in `${CLAUDE_PLUGIN_ROOT}/shared/write-check.md`, and report the entry as read back from the file.

## Step 7 — check (all paths)

Run from the project root:

```sh
specdev-analyzer check --no-color
```

Show the output. If errors:

1. **`[SCHEMA]` allOf failure** — status invariants violated. Fix the field set per the table in Step 2.
2. **`REDUCE_UNRESOLVED` naming a `Decision` endpoint** — `default_via` or `closed_by` names an id that doesn't exist. Author the Decision via `/spec-decide` first, or fix the typo.
3. **`REDUCE_UNRESOLVED` naming a `Requirement` endpoint** — `affects[]` names a requirement that doesn't exist. Fix or remove the reference.
4. **`open_question_status_consistency`** — `closed_by` or `default_via` points at a Decision whose status isn't `accepted`.
5. **`implementation_unblocked` naming a module** — the question (or a sibling) is `open`, reaches that module, and the module has code. Expected right after path A when the affected code already exists: the gate stays red until the question is downgraded or closed. Also expected once for every `category = "impl_time"` question: it settles during implementation, so it is still `open` when the first code lands. That is not a failure — close it (path C) with the decision the code made, or commit a default (path B). A module that is still `planned` with no code is not reported.

## Step 8 — report

One paragraph:
- OQ id and the path taken (created / downgraded / closed).
- For path B: the Decision id used as `default_via` and the verify text.
- For path A: which requirements the OQ affects, and which modules the gate reaches (modules whose `satisfies[]` overlaps `affects[]`) — say which of them already have code and so fail `check` now.
- Suggested next step. After path A: design or plan the affected modules knowing the gate applies once code exists. After path B or C: rerun `check`.

## What this skill does NOT do

- **Edit an existing OQ's text or affects list.** Edit `specs/open_questions/<id>.toml` directly; this skill handles creation and the two status transitions.
- **Flip a Decision's status, or author the deciding Decision.** Use `/spec-decide`, then come back for the OQ flip.
- **Flip `hard_gate`.** That is a person's call, made by editing the file.
- **Bypass the gate.** No CLI flag exists; the in-spec escape is path B.
