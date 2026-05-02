---
name: spec-validate
description: This skill should be used when the user asks to "validate specs", "verify spec consistency", "check spec errors", "check spec.toml", "find schema errors", or "check graph-check violations". Runs `specdev-analyzer validate` and translates findings into actionable feedback grouped by file.
allowed-tools: Read, Bash, Glob, Grep
---

# /spec-validate — Validate specs

Run the analyzer and translate its output into actionable feedback. Act as a second pair of eyes, not a rewriter — do not edit specs unless the user asks.

If `specdev-analyzer` is not on PATH, tell the user to install it (`cargo install --git https://github.com/cosmikwolf/specdev-analyzer`) and stop.

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

Capture stdout and the exit code. Validate runs in this order: layout discovery → schema validation (spec.toml + architecture.toml + requirements.toml + module TOMLs against the aggregated module schema) → kind resolution → reduction → graph checks declared in `specs/specdev/domain.toml`.

Exit codes:
- `0` — clean. Go to Step 4.
- `1` — findings present. Go to Step 2.
- `2` — setup error (no `spec.toml`, missing `specs/specdev/`, schema missing). Show the error verbatim and tell the user which prerequisite is missing — `/spec-init` if `specs/specdev/` is absent.

## Step 2 — present findings

For each finding, the analyzer emits a line like:

```
ERROR [RULE_ID] path/to/file.toml#/pointer — message
```

Group findings in this order, each group with one sentence of context before the verbatim findings:

1. **Layout / setup** — `LAYOUT-001..006`, `PLUGIN-001`. Project tree is incomplete or the plugin root is unresolvable.
2. **Schema errors** — `SCHEMA`, `SCHEMA-ARCH`, `SCHEMA-REQ`. A TOML file violates its JSON Schema (unknown field, bad enum, missing required key, pattern mismatch).
3. **Metaschema / aggregator** — `METASCHEMA-001..005`, `AGGREGATOR-005`. Inlined block/bridge schemas are misshapen or fail to compile into the aggregator. Usually a plugin-author bug surfaced after `inline`.
4. **Kind resolution** — `DOMAIN-003`, `REDUCE_UNRESOLVED`. A reduction's `extends` chain or relation endpoint references a kind/name that doesn't exist in the materialized graph.
5. **Graph checks** — rule names declared in `specs/specdev/domain.toml` `[validation.rules]` (e.g. for firmware-layered: `unique_module_names`, `modules_acyclic`, `layer_dependency`, `connection_endpoints_resolve`; for library-api: `re_exports_resolve`, `feature_dependencies_resolve`, etc.). These come after the spec parses cleanly and depend on cross-file invariants.
6. **Warnings and info**.

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

## Additional resources

- `examples/validation-report-walkthrough.md` — annotated walkthrough turning raw analyzer output into grouped findings and hand-off instructions.
