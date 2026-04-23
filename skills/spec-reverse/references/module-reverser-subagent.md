# module-reverser subagent contract

`/spec-reverse --all` fans out the per-module extraction work to a read-only subagent defined at `specdev/agents/module-reverser.md`. That file is the authoritative source — this document summarizes the contract skills need to know when invoking it.

## Authoritative definition

`specdev/agents/module-reverser.md` declares:

- `name: module-reverser`
- `model: sonnet`
- `tools: Read, Grep, Glob` (read-only by design — no `Write`, `Edit`, or `Bash`)

Read that file directly when changing the contract, adding extraction rules, or debugging a fan-out failure.

## Input contract (prompt the invoker must supply)

The invoker constructs one prompt per module with these keys:

| Key | Required | Purpose |
|---|---|---|
| `module_name` | yes | PascalCase module name (`StatusLed`). |
| `project_root` | yes | Absolute path to the project root (contains `spec.toml` and `specs/`). |
| `schema_dir` | yes | Resolved schema directory — the subagent reads `module.schema.yaml` etc. from here. |
| `domain_dir` | yes | Resolved domain directory — for `author.md` and domain conventions. |
| `language_dir` | yes | Resolved language directory — for `syntax.md`. |
| `source_hint` | no | Explicit path to the module's source file(s). If absent, the subagent infers from `module_name`. |
| `package` | only for multi-package projects | snake_case package name. When present, the invoker writes the candidate TOML to `specs/modules/<package>/<ModuleName>.toml`; the subagent does **not** add a `package` field to the TOML itself (the analyzer derives it from the filesystem path). |
| `source_path` | only for multi-package projects | Absolute or project-relative path scoping the subagent's Read/Grep/Glob to this subtree. Prevents cross-package evidence leakage during fan-out. |

If any required key is missing, the subagent returns an error message and stops — it must not guess paths.

## Output contract

The subagent returns exactly one assistant message with three sections:

1. **`## Candidate TOML`** — verbatim TOML body ready to write. Obeys `specdev/shared/comment-policy.md`; the only permitted comments are the `#:schema` directive and inline `# LOW: inferred from <source>:<line>, please confirm` markers.
2. **`## Evidence`** — one line per populated field: `<field>: <path>:<line-range> — <short quote>`.
3. **`## Notes`** — source files read, LOW-confidence field count, unresolved questions for the invoker.

The subagent does not validate, does not write files, and does not run the analyzer. The invoker handles those after the user confirms the candidate.

## Invocation pattern from the skill

One Agent tool use per `(package, module)` tuple, all in a **single assistant message** so they run concurrently. The skill then waits for every subagent to return before presenting batch confirmation (see `mode-full-project.md` Step B3).

Errors returned by a subagent (source not found, ambiguous split, missing input key) are surfaced alongside the successful candidates so the user can resolve them before the skill writes any files.
