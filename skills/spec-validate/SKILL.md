---
name: spec-validate
description: This skill should be used when the user asks "is my spec internally consistent", "validate specs", "check the spec against the code", "run check", "verify spec consistency", "check spec errors", "check spec.toml", "find schema errors", "check graph-check violations", "did I break anything", or "is my spec OK". Runs `specdev-analyzer check` (validate, then the checks that read the code) and translates findings into actionable feedback grouped by file. For source-vs-spec drift on a module's `[[items]]` array, use /spec-sync-items. For inlined-block drift vs the plugin tree, use /spec-drift.
allowed-tools: Read, Bash, Glob, Grep
---

# /spec-validate — Check the spec, and the spec against the code

Run the analyzer and translate its output into actionable feedback. Act as a second pair of eyes, not a rewriter — do not edit specs or code unless the user asks.

If `specdev-analyzer` is not on PATH, tell the user to install it (follow `${CLAUDE_PLUGIN_ROOT}/shared/analyzer-install.md` — it derives the org from the plugin's own remote; both repositories are private) and stop. If it IS on PATH but reports `MANIFEST-002`, or a `MANIFEST-001` naming a file under the plugin tree, then the analyzer is older than the plugin — not a problem with any project file. Tell the user to re-run the install command with `--force` (or `cargo install --path <checkout> --force` from a local checkout), then retry.

Before any analyzer invocation, run the env preamble from `${CLAUDE_PLUGIN_ROOT}/shared/plugin-root-preamble.md`:

```sh
export SPECDEV_PLUGIN_ROOT="${CLAUDE_PLUGIN_ROOT}"
```

Skip if `SPECDEV_PLUGIN_ROOT` is already set or every invocation passes `--plugin-root`.

---

## Step 1 — run the analyzer

Run from the project root:

```sh
specdev-analyzer check --no-color
```

`check` runs `validate` first — layout discovery, schema validation of every spec file, kind resolution, reduction, and the graph checks in `specs/specdev/domain.toml` — and then the checks that read the code:

- **source rule** (`source_resolves`) — a module that is not `planned` must have resolvable source;
- **stale flags** (`stale_planned`) — a `planned` module whose source now resolves, or a planned requirement whose modules are all built;
- **open-question gate** (`implementation_unblocked`) — a module that an `open` question reaches (local or workspace tier) and that has code;
- **imports** (`internal_edges_declared`) — every import between project modules follows a declared `depends_on`;
- **tests** (`requirements_have_tests`, `verified_by_resolves`) — a requirement names its tests in `verified_by`, and each named test exists in the test discovery (`specdev-analyzer report tests` shows the identities);
- **rule packs** (`pack_entity_in_code`, `pack_attribute_matches`, `pack_key_ambiguous`) — each inlined block's or bridge's code checks: a spec entity the code lacks, an attribute that differs, or a key the code has at more than one place (`${CLAUDE_PLUGIN_ROOT}/shared/rule-packs.md`).

Use `specdev-analyzer validate --no-color` alone only when the code is not there to read (a spec-only review).

Exit codes:
- `0` — no error finding. Go to Step 4.
- `1` — at least one error finding. Go to Step 2.
- `2` — the check could not run (no `spec.toml`, missing `specs/specdev/`, a broken schema, `FORMAT-002`). Show the error verbatim. `/spec-init` if `specs/specdev/` is absent; for `FORMAT-002`, see "Format migration" below.

## Step 2 — present findings

For each finding, the analyzer emits **one line**: lowercase severity, a colon, the bracketed rule, the location (an absolute path), a colon, then the message. Captured from a real run, not paraphrased:

```
info: [LAYOUT-006] : no requirement or source entries under /path/to/proj/specs/requirements — none authored yet
error: [source_resolves] /path/to/proj/specs/modules/Store.toml: module `Store` has no resolvable source: set `source_file`, or `planned = true` while it is not built
1 error(s), 0 warning(s), 19 info(s)
```

The exit code follows the **error** count alone — warnings and infos do not change it. Measure the exit code without a pipe — `check | tail` reports `tail`'s status, not the analyzer's.

Group findings in this order, each group with one sentence of context before the verbatim findings:

1. **Layout / setup** — `LAYOUT-001..006`, `PLUGIN-001`, `FORMAT-*`, `CONTEXT-001/002`, `GATE-SCRIPT-001`. Project tree is incomplete, the plugin root is unresolvable, the spec format is not the current one, a module's `context_files` path is missing (`CONTEXT-001`) or unreadable / not UTF-8 (`CONTEXT-002`), or `scripts/specdev-gates.sh` differs from the `gates:` in `specs/specdev/profile.yaml` (`GATE-SCRIPT-001`: find out why it drifted, then `specdev-analyzer gate --emit`).
2. **Schema errors** — `SCHEMA`, `SCHEMA-ARCH`, `SCHEMA-REQ`. A TOML file violates its JSON Schema (unknown field, bad enum, missing required key, pattern mismatch).
3. **Metaschema / aggregator** — `METASCHEMA-001..006`, `AGGREGATOR-005`. Inlined block/bridge schemas are misshapen. Usually a plugin-author bug surfaced after `inline`.
4. **Kind resolution** — `DOMAIN-003`, `REDUCE_UNRESOLVED`. A reduction's `extends` chain or relation endpoint names a kind or entity that does not exist.
   `REDUCE-001` is different: a spec file did not parse, so the graph was never built and **no graph check ran**. A report with no group-5 findings proves nothing while `REDUCE-001` is present. Fix the parse error first and run again.
5. **Graph checks** — rule names declared in `specs/specdev/domain.toml`, and the `<unit>.<rule>` names of the inlined rule packs in `specs/specdev/checks/` (a `PACK-001` means a pack did not load and none of its checks ran) (e.g. `unique_qualified_names`, `modules_acyclic`, `layer_dependency`, `connection_endpoints_resolve`, `requirements_have_satisfiers`). They depend on cross-file invariants.
6. **Spec against the code** — `source_resolves`, `stale_planned`, `implementation_unblocked`, `internal_edges_declared`, `requirements_have_tests`, `verified_by_resolves`, `pack_entity_in_code`, `pack_attribute_matches`, `pack_key_ambiguous`, `PACK-002`.
7. **Warnings and info** — including `PLUGIN-AHEAD-001` (the plugin holds newer copies of inlined or profile files: name the count and offer `/spec-drift`; do not merge or refresh from this skill), `GATE-SCRIPT-002` (no gate script yet: `specdev-analyzer gate --emit`), and `FORMAT-001` (an older project format: see "Format migration").

Do not paraphrase analyzer messages — the raw text is the most precise.

## Step 3 — suggest fixes

After findings are presented, offer specific fixes. Read the offending TOML (or source file) and point at the exact line to change.

For graph-check rule names, the **project's** `specs/specdev/domain.toml` is authoritative — read its `[validation.rules]` table for severity, and its `[[project.graph_checks]]` block for the rule's `kind`. Common fixes:

- `unique_*` — two entities share a name; rename one.
- `*_acyclic` — break the cycle by extracting shared types or inverting a dependency.
- `layer_dependency` — a module depends on a higher layer than allowed; invert, or move shared types down.
- `*_endpoints_resolve` / `*_resolves` — the reference points at something that doesn't exist; fix it or declare the target.
- `SCHEMA` on a module — an unknown field usually means a block isn't inlined yet; a bad enum means the value is outside the schema's set.
- `source_resolves` — the code exists somewhere else (set `source_file`), or it is not built yet (set `planned = true`).
- `stale_planned` — the code landed: remove `planned = true`.
- `pack_entity_in_code` / `pack_attribute_matches` — the spec and the code disagree about a unit entry. Either the spec is ahead (mark the entry `planned = true` until it is built) or the code drifted (fix the code). `pack_key_ambiguous` — two code items share the key; rename one, or give the spec the name the code uses. For a rule-pack graph check (`<unit>.<rule>`), the unit's inlined `guidance/` file names the rule and says what it means.
- `implementation_unblocked` — an open question reaches built code. Route to `/spec-question` (close it, or commit a default); never silence it in the spec.
- `internal_edges_declared` — the code imports a module the spec does not list. Either the dependency is intended (add it to `depends_on` via `/spec-edit`, then check the layer rules) or the import is a mistake (fix the code).
- `requirements_have_tests` / `verified_by_resolves` — add the test, or correct the name in `verified_by` to the identity `specdev-analyzer report tests` shows. Mark `test_pending = true` only when the test is truly not written yet.
- Triad findings — unresolved `decided_by` / `guided_by` / `closed_by` references mean the named decision or question doesn't exist (fix the id or author it via `/spec-decide` / `/spec-question`); supersession-chain violations mean a `supersedes` link targets a missing or non-superseded entry.

Do not invent rule names or codes. If a code is unfamiliar, point the user at `specs/specdev/domain.toml` for the rule definition, or at the analyzer's `--help`.

## Format migration

`FORMAT-001` (info) means the project is on an older spec format; `FORMAT-002` (exit 2) means a workspace member's format major differs from its root's. Both route to `specdev-analyzer migrate`, run at the workspace root when there is one.

**Before running `migrate`, make sure the work tree is clean.** `migrate` rewrites files in place, and for a tree with no recorded baseline it cannot prove which files carry local edits — `git diff` is the only way to see what it changed. Run `git status --porcelain`; if it lists anything, stop and ask the user to commit or stash first, and run `migrate` only when they say go. Then `specdev-analyzer migrate --dry-run`, show the report, and run without `--dry-run` once the user agrees. After it, `git diff` shows every change.

## Step 4 — hand off for fixes

If the user says "fix them," do not edit files from this skill — checking and authoring are separate concerns:

- Module-spec errors: `/spec-edit <ModuleName>` (it does the downstream impact analysis).
- `architecture.toml` or `spec.toml` errors: edit directly.
- Code findings (imports, tests, sources): change the code, after reading `specdev-analyzer context --module <Module>` and `specdev-analyzer report decisions --module <Module>`.
- Metaschema / aggregator errors after an `inline`: `specdev-analyzer inline <block> --merge` if the block was upgraded, or the plugin author fixes the block (`specdev-analyzer plugin-lint --plugin <plugin-root>`).

After the user applies fixes, invoke `/spec-validate` again.

## Step 5 — report

End with one line:
- Clean: "All checks passed. `<N>` modules checked."
- Dirty: "`<E>` errors, `<W>` warnings. Run again after fixes."

Echo the analyzer's summary counts. When open questions exist, say how many are open and that the gate applies to every module they reach once it has code.

## Additional resources

- `${CLAUDE_PLUGIN_ROOT}/shared/rule-ids.md` — where validation rule names live (the project's own `specs/specdev/domain.toml`), the analyzer's stable code families, and the standard graph-check `kind` primitives.
- `examples/validation-report-walkthrough.md` — annotated walkthrough turning raw analyzer output into grouped findings and hand-off instructions.
