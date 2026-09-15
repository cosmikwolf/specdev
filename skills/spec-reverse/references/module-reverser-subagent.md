# module-reverser subagent contract

`/spec-reverse --all` may fan out the per-module extraction work to a read-only subagent.

## Status

The subagent file is currently staged at `${CLAUDE_PLUGIN_ROOT}/skills/spec-reverse/references/module-reverser-agent.deferred.md` (not active — staged outside `agents/` so the loader cannot register it). Until it ships under `agents/`, `--all` mode runs sequentially: the per-module loop in `mode-full-project.md` invokes `specdev-analyzer context --module <Name>` once per module and authors each spec from the main thread.

For now, prefer `/spec-reverse <ModuleName>` (single-module mode) for the human-driven workflow. Use `/spec-reverse --all` with the understanding that it walks modules sequentially without subagent fan-out.

## Output contract (when the subagent activates)

The subagent's three-section output shape is profile-agnostic:

1. **`## Candidate TOML`** — verbatim TOML body ready to write. Obeys `${CLAUDE_PLUGIN_ROOT}/shared/comment-policy.md`; the only permitted comments are inline `# LOW: inferred from <source>:<line>, please confirm` markers.
2. **`## Evidence`** — one line per populated field: `<field>: <path>:<line-range> — <short quote>`.
3. **`## Notes`** — source files read, LOW-confidence field count, unresolved questions for the invoker.

The subagent does not validate, does not write files, and does not run the analyzer. The invoker handles those after the user confirms the candidate.

## Input contract (when the subagent activates)

Input keys: `module_name`, `project_root`, `profile`, `language`, `inlined_blocks` (array), `inlined_bridges` (array), optional `source_path` scoping.

The staged file now documents exactly these inputs, so promotion is a move plus dropping its header — but `/spec-reverse`'s `allowed-tools` still cannot dispatch a subagent, so that must change too before anything can call it.
