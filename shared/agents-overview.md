# Agents shipped by the specdev plugin

`${CLAUDE_PLUGIN_ROOT}/agents/` holds the plugin's subagents. **Every `.md` file
anywhere under that directory is auto-discovered by Claude Code and offered to
users** — including files in subdirectories, and including files that are not
agents at all.

That is why this document lives in `shared/` rather than in `agents/`: as
`agents/README.md` it was itself registered, and users saw an agent named
"README" in the picker.

## Shipped

- `module-designer` — drafts a single module TOML from an interview. The
  read-isolated drafting step of `/spec-design` and `/spec-edit`.

## Staging an agent that is not ready

Do **not** use a dot-prefixed subdirectory. `agents/.deferred/` was used for this
and did not work: the loader registered the file as
`specdev:.deferred:module-reverser` and offered a half-built agent, with an input
contract three renames out of date, to anyone who opened the picker.

Stage the file **outside `agents/`** instead — next to the skill that will
eventually call it, under that skill's `references/`, with a `.deferred.md`
suffix and a header saying it is not active. `module-reverser` is staged that way
at `skills/spec-reverse/references/module-reverser-agent.deferred.md`.

Promote by moving the file to `agents/<name>.md` and dropping the header. Check
first that the calling skill's `allowed-tools` can actually dispatch a subagent —
registration alone does not make an agent reachable.
