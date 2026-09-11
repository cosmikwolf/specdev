---
name: spec-config
description: This skill should be used when the user asks to "configure specdev for this project", "write the specdev rules file", "add the project rules", "set up specdev-feedback.md", "add the feedback file", "run spec-config", or when /spec-init finishes scaffolding a project. Writes two files that the project owns from then on, each only when it is absent - `.claude/rules/specdev.md` (the project-rules text, so the rules survive with hooks disabled) and `specdev-feedback.md` (an append-only log of surprises worth reporting to the specdev developer). It never overwrites either file and never merges; a second run writes nothing and says so.
allowed-tools: Read, Write, Bash, Glob
---

# /spec-config — Write the project rules file and the feedback file

Write two files at the project root, **each only when it is absent**:

| File | Content | Owner after the first write |
|---|---|---|
| `.claude/rules/specdev.md` | a verbatim copy of `${CLAUDE_PLUGIN_ROOT}/shared/project-rules.md` | the project |
| `specdev-feedback.md` | a one-line header | the project |

Both files belong to the user after this skill creates them. Rewrite them, extend them, or delete
them freely - nothing in specdev reads either file, and neither is recorded in
`specs/specdev/gen-lineage.toml`, so an edit is not drift.

The rules file is a **fallback**, not the primary path. The installed plugin prints its own live
copy of the same text at the start of every session. The project copy matters when hooks are off,
and it is the copy the user may tailor to the project.

---

## Step 1 — find the project root

From the current directory, walk up to the first directory that holds **both** `spec.toml` and
`specs/specdev/`. That directory is `<root>`:

```sh
dir="$(pwd)"
while :; do
    if [ -f "${dir}/spec.toml" ] && [ -d "${dir}/specs/specdev" ]; then echo "${dir}"; break; fi
    parent="$(dirname "${dir}")"
    [ "${parent}" = "${dir}" ] && { echo "no specdev project root found" >&2; break; }
    dir="${parent}"
done
```

If no root is found, stop. Tell the user this is not a specdev project yet and that `/spec-init`
scaffolds one. Do not create `spec.toml` and do not create either file outside a project root.

In a workspace, each member crate that holds its own `spec.toml` and `specs/specdev/` is its own
root. Run this skill from the member directory to configure that member.

## Step 2 — `<root>/.claude/rules/specdev.md`

Copy the shared file verbatim. `cp` guarantees the copy is byte-for-byte; do not retype the text
and do not reformat it:

```sh
mkdir -p "<root>/.claude/rules"
if [ -f "<root>/.claude/rules/specdev.md" ]; then
    echo "already present: .claude/rules/specdev.md"
else
    cp "${CLAUDE_PLUGIN_ROOT}/shared/project-rules.md" "<root>/.claude/rules/specdev.md"
    echo "created: .claude/rules/specdev.md"
fi
```

If the file is already present, leave it exactly as it is, whatever it holds. A project copy that
differs from the plugin's copy is an intended local edit, not an error, and not something to
report as stale.

## Step 3 — `<root>/specdev-feedback.md`

```sh
if [ -f "<root>/specdev-feedback.md" ]; then
    echo "already present: specdev-feedback.md"
else
    printf '%s\n' '# specdev feedback - append one dated entry per surprise; never delete an entry.' \
        > "<root>/specdev-feedback.md"
    echo "created: specdev-feedback.md"
fi
```

The entry format lives in the rules text: `### <date> - <command> - <severity>`, with what
happened, what you expected, and what you did. Severity is blocker / high / moderate / minor.

## Step 4 — report

State the outcome for each file on its own line: created, or already present. Name the full path of
each file that was created. When both files were already present, say plainly that the run wrote
nothing - that is the expected result of a second run, not a failure.

Then tell the user two things:

- The rules file is theirs to edit. Nothing re-syncs it.
- New entries go at the end of `specdev-feedback.md`. Entries are never deleted.

## What this skill does NOT do

- **Refresh or merge either file.** There is no re-sync path. The first write is the only write.
- **Record either file in `specs/specdev/gen-lineage.toml`.** Neither is a generated output. A
  lineage row would report every intended user edit as drift.
- **Read either file back.** No skill and no analyzer command reads them. They are Claude Code
  configuration and a human-facing log, not spec state.
- **Edit a file that exists.** It has no `Edit` tool for this reason.
- **Run the analyzer.** This skill needs no analyzer and works with none installed.
