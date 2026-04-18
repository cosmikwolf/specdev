---
name: spec-validate
description: Run specdev-analyzer validate and present the results with fix suggestions.
allowed-tools: Read, Bash, Glob, Grep
---

!`${CLAUDE_PLUGIN_ROOT}/scripts/resolve-context`

# /spec-validate — Validate specs

Run the analyzer and translate its output into actionable feedback. You are a second pair of eyes, not a rewriter — do not edit specs unless the user asks.

---

## Step 1 — run the analyzer

Run `specdev-analyzer validate --project <project_root> --no-color`. Capture stdout and the exit code.

Exit codes:
- `0` — clean. Go to Step 3.
- `1` — findings present. Go to Step 2.
- `2` — setup error (no `spec.toml`, schema missing, etc.). Show the error and tell the user which prerequisite is missing.

## Step 2 — present findings

For each finding, the analyzer emits a line like:

```
ERROR [RULE_ID] path/to/file.toml#/pointer — message
```

Group findings by file, in this order:

1. Schema errors (rule ids starting with the schema's section, e.g. unknown field, bad enum value).
2. Graph checks: `LAYER001`, `FSM020`–`FSM022`, `PORT030`, `REQ001`, `REQ002`, `OWN050`, `ISR060`.
3. Warnings and info.

For each group, give one-sentence context explaining what the rule guarantees, then list the findings verbatim under it. Do not paraphrase analyzer messages — the raw text is the most precise.

## Step 3 — suggest fixes

After the findings are presented, offer specific fixes. Read the offending TOML file and point at the exact line to change. Examples:

- `META010` on `specs/modules/Foo.toml#/metadata` — "required metadata object `ConcurrencyModel` is missing; add a `[metadata.ConcurrencyModel]` table with `protection`, `isr_safe`, and `reentrant`."
- `FSM021` on `specs/modules/Foo.toml#/state_machines/0/transitions/3` — "transition references `to = "bar"` but state `bar` is not declared in `states[]`; rename the target or add the state."
- `LAYER001` — "module at layer `application` depends on `pac` module `FooPac`; insert a driver or middleware layer, or downgrade the layer on this module if it truly is low-level."

Do not invent rule ids. If a finding's rule id is unfamiliar, read `<Domain dir>/domain.toml` to find its definition.

## Step 4 — apply fixes only on request

If the user says "fix them," apply the fixes one file at a time, re-running `specdev-analyzer validate` between edits, until clean or until the user stops you. If the user only asked to see the report, stop at Step 3.

## Step 5 — report

End with one line:
- Clean: "All checks passed. `<N>` modules validated."
- Dirty: "`<E>` errors, `<W>` warnings. Rerun after fixes."
