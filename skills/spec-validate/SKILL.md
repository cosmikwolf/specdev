---
name: spec-validate
description: This skill should be used when the user asks to "validate specs", "verify spec consistency", "check spec errors", "check spec.toml", "find schema errors", or "check LAYER001/FSM020/PORT030/REQ001 violations". Runs specdev-analyzer validate and maps rule codes to actionable fixes.
allowed-tools: Read, Bash, Glob, Grep
---

!`${CLAUDE_PLUGIN_ROOT}/scripts/resolve-context`

# /spec-validate — Validate specs

Run the analyzer and translate its output into actionable feedback. Act as a second pair of eyes, not a rewriter — do not edit specs unless the user asks.

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

After the findings are presented, offer specific fixes. Read the offending TOML file and point at the exact line to change. Use `specdev/shared/rule-ids.md` as the decoder — it lists each firmware rule id, its severity, its meaning, and the typical fix pattern. For non-firmware domains, read the loaded `<Domain dir>/domain.toml` `[validation.rules]` table for the canonical id list. Do not invent rule ids.

## Step 4 — hand off for fixes

If the user says "fix them," do not edit files from this skill — validation and authoring are separate concerns. For each affected module, tell the user to run `/spec-edit <ModuleName>`, which performs the downstream impact analysis that re-validation alone would miss. For errors in `architecture.toml` or `spec.toml` itself, point to `/spec-init` or a direct edit. After the user applies fixes, re-invoke `/spec-validate` to confirm.

## Step 5 — report

End with one line:
- Clean: "All checks passed. `<N>` modules validated."
- Dirty: "`<E>` errors, `<W>` warnings. Rerun after fixes."

## Additional resources

- `examples/validation-report-walkthrough.md` — annotated walkthrough turning raw analyzer output into grouped findings and hand-off instructions.
