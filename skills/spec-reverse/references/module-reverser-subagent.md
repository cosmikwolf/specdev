# module-reverser subagent contract

`/spec-reverse --all` fans out the per-module extraction work to a read-only subagent defined at `${CLAUDE_PLUGIN_ROOT}/agents/module-reverser.md`.

## Phase 10 status (2026-04-30)

The current `module-reverser.md` agent file is shaped for the retired `domains + languages` model — it expects `schema_dir`, `domain_dir`, `language_dir` inputs that have no equivalent in the `profiles + blocks + bridges` model. Until Phase 10 ships the proper reverse primitives, the `--all` mode should be treated as **manual** — invoke the subagent only after the user has accepted that its output will require schema-level rewrite to validate against the project's aggregated `module.schema.yaml`.

When Phase 10 lands, this file gets rewritten with the real subagent contract (input keys: `module_name`, `project_root`, `profile`, `language`, `inlined_blocks` array, `inlined_bridges` array, optional `source_path` scoping).

## Output contract (will remain)

The subagent's three-section output shape is profile-agnostic and will survive Phase 10:

1. **`## Candidate TOML`** — verbatim TOML body ready to write. Obeys `${CLAUDE_PLUGIN_ROOT}/shared/comment-policy.md`; the only permitted comments are inline `# LOW: inferred from <source>:<line>, please confirm` markers.
2. **`## Evidence`** — one line per populated field: `<field>: <path>:<line-range> — <short quote>`.
3. **`## Notes`** — source files read, LOW-confidence field count, unresolved questions for the invoker.

The subagent does not validate, does not write files, and does not run the analyzer. The invoker handles those after the user confirms the candidate.

## Invocation pattern (current state)

For now, use `/spec-reverse <ModuleName>` (single-module mode) which the human-driven workflow handles cleanly. Reserve `/spec-reverse --all` for after Phase 10 lands.
