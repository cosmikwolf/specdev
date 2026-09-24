---
name: spec-validate
description: This skill should be used when the user asks "is my spec internally consistent", "validate specs", "verify spec consistency", "check spec errors", "check spec.toml", "find schema errors", "check graph-check violations", "did I break anything", or "is my spec OK". Runs `specdev-analyzer validate` and translates findings into actionable feedback grouped by file. For source-vs-spec drift on a module's `[[items]]` array, use /spec-sync-items. For inlined-block drift vs the plugin tree, use /spec-drift.
allowed-tools: Read, Bash, Glob, Grep
---

# /spec-validate — Validate specs

Run the analyzer and translate its output into actionable feedback. Act as a second pair of eyes, not a rewriter — do not edit specs unless the user asks.

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
specdev-analyzer validate --no-color
```

Capture stdout and the exit code. Validate runs in this order: layout discovery → schema validation (spec.toml + architecture.toml + requirements.toml + decisions.toml + open_questions.toml + module TOMLs against the aggregated module schema) → kind resolution → reduction → graph checks declared in `specs/specdev/domain.toml` (including the triad checks: supersession-chain consistency, decision/question reference resolution, and the `codegen_unblocked` warning for modules gated by an open question).

Exit codes:
- `0` — clean. Go to Step 4.
- `1` — findings present. Go to Step 2.
- `2` — setup error (no `spec.toml`, missing `specs/specdev/`, schema missing). Show the error verbatim and tell the user which prerequisite is missing — `/spec-init` if `specs/specdev/` is absent.

## Step 2 — present findings

For each finding, the analyzer emits **one line**: lowercase severity, a colon, the bracketed rule,
the location (an absolute path, with no `#` pointer), a colon, then the message. Captured from a real
run, not paraphrased:

```
error: [SCHEMA] /path/to/proj/specs/modules/Broken.toml/visibility: "public" is not one of "pub", "crate" or "private"
warning: [CONTEXT-100] /path/to/proj/specs/modules/Good.toml: module `Good` has no `context_files`. Strict codegen has no out-of-band prose to anchor on; consider adding `context_files = ["docs/<...>.md"]`.
info: [ITEMS-001] /path/to/proj/specs/modules/Good.toml: module `Good` has 0 items[] entries — `gen` has nothing to anchor codegen to.
```

The run ends with a count line, and the exit code follows the **error** count alone — warnings and
infos do not change it:

```
loaded 2 modules / 0 requirements / 0 sources / 12 graph checks
1 error(s), 2 warning(s), 21 info(s)
```

`1 error(s)` → exit 1; `0 error(s)` → exit 0, whatever the warning and info counts are. Measure the
exit code without a pipe — `validate | tail` reports `tail`'s status, not the analyzer's.

Group findings in this order, each group with one sentence of context before the verbatim findings:

1. **Layout / setup** — `LAYOUT-001..006`, `PLUGIN-001`. Project tree is incomplete or the plugin root is unresolvable.
2. **Schema errors** — `SCHEMA`, `SCHEMA-ARCH`, `SCHEMA-REQ`. A TOML file violates its JSON Schema (unknown field, bad enum, missing required key, pattern mismatch).
3. **Metaschema / aggregator** — `METASCHEMA-001..006`, `AGGREGATOR-005`. Inlined block/bridge schemas are misshapen or fail to compile into the aggregator. Usually a plugin-author bug surfaced after `inline`.
4. **Kind resolution** — `DOMAIN-003`, `REDUCE_UNRESOLVED`. A reduction's `extends` chain or relation endpoint references a kind/name that doesn't exist in the materialized graph.
   `REDUCE-001` is different: a spec file did not parse (the message names it; for a module, `LAYOUT-005` names it too), so the graph was never built and **no graph check ran**. A report with no group-5 findings proves nothing while `REDUCE-001` is present. Fix the parse error first and re-run; graph-check findings can appear only on the next run.
5. **Graph checks** — rule names declared in `specs/specdev/domain.toml` `[validation.rules]` (e.g. for firmware-layered: `unique_qualified_names`, `modules_acyclic`, `layer_dependency`, `connection_endpoints_resolve`; for library-api: `feature_unique_names`, `feature_dependencies_resolve`, `module_feature_gates_resolve`, etc.). These come after the spec parses cleanly and depend on cross-file invariants.
6. **Warnings and info** — including `PLUGIN-AHEAD-001`: the plugin holds newer copies of inlined or profile files than this project. Name the count and offer `/spec-drift`, which shows each file and the command that brings it forward; do not merge or refresh from this skill. Also `codegen_unblocked`: a module is reachable from an open question (`status = "open"` in `specs/open_questions.toml`) and `/spec-gen` will refuse it. This is a warning at validate time and a hard refusal at gen time; the fix lives in `/spec-question` (close the question, or commit a provisional default), not in this skill.

Do not paraphrase analyzer messages — the raw text is the most precise.

## Step 3 — suggest fixes

After findings are presented, offer specific fixes. Read the offending TOML and point at the exact line to change.

For graph-check rule names, the **project's** `specs/specdev/domain.toml` is authoritative — read its `[validation.rules]` table for severity, and read the `[[project.graph_checks]]` block for the rule's `kind` (`unique`, `acyclic`, `layer_dependency`, `endpoints_resolve`, etc.) which tells you what was checked. Common fixes:

- `unique_*` violations — two entities share a name; rename one.
- `*_acyclic` violations — break the cycle by extracting shared types or inverting a dependency.
- `layer_dependency` violations — a module depends on a higher layer than its `[layers.allowed_deps]` permits; invert or move shared types into a lower layer.
- `*_endpoints_resolve` / `*_resolves` violations — the reference (port, requirement id, feature name, re-export target) points at something that doesn't exist; fix the reference or declare the target.
- `SCHEMA-ARCH` referencing `[platform]` / `[runtime]` / `[logger]` — the firmware-layered architecture template requires these tables; `/spec-init` may have been skipped or fields were deleted.
- `SCHEMA` on a module — an unknown field usually means a block hasn't been inlined yet (e.g., `peripherals = [...]` requires `specdev-analyzer inline stm32h5-rs`); a bad enum means the value is outside the schema's allowed set.
- Triad findings — unresolved `decided_by` / `guided_by` / `closed_by` references mean the named decision or question doesn't exist (fix the ID or author the entry via `/spec-decide` / `/spec-question`); supersession-chain violations mean a `supersedes` link targets a missing or non-superseded entry; `codegen_unblocked` warnings route to `/spec-question`, not to spec edits.

Do not invent rule names or codes. If a code is unfamiliar, point the user at `specs/specdev/domain.toml` for the rule definition, or at the analyzer's `--help` for the code prefix's family.

## Step 4 — hand off for fixes

If the user says "fix them," do not edit files from this skill — validation and authoring are separate concerns:

- For module-spec errors: `/spec-edit <ModuleName>` (it performs downstream impact analysis that re-validation alone misses).
- For `architecture.toml` or `spec.toml` errors: edit directly, or rerun `/spec-init` against a fresh directory if the project shape itself is wrong.
- For metaschema / aggregator errors after an `inline`: the user re-runs `specdev-analyzer inline <block> --merge` if the block was upgraded, or the plugin author needs to fix the block (run `specdev-analyzer plugin-lint --plugin <plugin-root>`).

After the user applies fixes, re-invoke `/spec-validate` to confirm.

## Step 5 — report

End with one line:
- Clean: "All checks passed. `<N>` modules validated."
- Dirty: "`<E>` errors, `<W>` warnings. Rerun after fixes."

The analyzer's summary line reports what it loaded (modules / requirements / sources / graph checks, plus decisions / open questions counts when the project authors them — with how many questions are still open). Echo it: the open-question count is the user's heads-up that `/spec-gen` will refuse affected modules.

## Additional resources

- `${CLAUDE_PLUGIN_ROOT}/shared/rule-ids.md` — where validation rule names live (the project's own `specs/specdev/domain.toml`), the analyzer's stable code families (`LAYOUT-*`, `SCHEMA*`, `METASCHEMA-*`, `DOMAIN-*`, `BLOCK-LINT-*`, `BRIDGE-LEX-*`, `BOOTSTRAP-*`, `INLINE-*`, `MERGE-*`, `PLUGIN-*`, `LINEAGE-*`, `DRIFT-INVARIANT-*`), and the standard graph-check `kind` primitives (`unique`, `acyclic`, `layer_dependency`, `endpoints_resolve`, `reachability`).
- `examples/validation-report-walkthrough.md` — annotated walkthrough turning raw analyzer output into grouped findings and hand-off instructions.
