# Check the write against the disk

A skill's report of what it wrote must come from the disk, not from what it
meant to write. In the field, one run reported a file it had never written, and
one run changed a file its report never named. This page is the check. The
skills that write spec files run it around their write step.

## Around the write

Before the write, snapshot the files you are about to write:

```sh
snap=$(mktemp -t specdev-write-check.XXXXXX) && bash "${CLAUDE_PLUGIN_ROOT}/scripts/write-check" snapshot <target>... > "$snap"
```

Write the files. Then compare, naming the same targets:

```sh
bash "${CLAUDE_PLUGIN_ROOT}/scripts/write-check" compare "$snap" <target>...
```

It prints one line per finding, paths relative to the repository root:

| Line | Meaning | Do |
|---|---|---|
| `wrote <path>` | the target's content changed | report it |
| `not-written <path>` | the target's content did **not** change | **stop**: the write did not happen. Say so, and do not report it as written. Then retry the write |
| `unexpected <path>` | another file changed or appeared during the write step | **warn**: name the file. Do not revert it. A dev server (`next dev` rewrites `AGENTS.md`) or a person can change a file in the same tree |

Exit 0 means every target was written and nothing else changed. Exit 1 means at
least one `not-written` or `unexpected` line. Exit 2 means the check could not
run (not a git work tree): say that the check did not run, and continue.

## The report

Build the report from the file as it is now: read it back and quote the entry
you wrote from the disk. A report written from memory is exactly the defect this
check exists to catch.

`bash` is named explicitly because the script also runs under macOS's bash 3.2.
