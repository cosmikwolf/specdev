---
name: spec-design
description: This skill should be used when the user asks to "design a module", "add a module spec", "author a driver spec", "design <ModuleName>", or needs to write a single module TOML under specs/modules/. Authors one module TOML per invocation, then validates it. **New modules only — refuses if `specs/modules/<ModuleName>.toml` already exists.** To modify an existing module, use /spec-edit.
allowed-tools: Read, Write, Edit, Bash, Glob, Grep, Task
argument-hint: <one-line module description>
---

# /spec-design — Module authoring

Design one module per invocation. User input: `$ARGUMENTS` (a one-line description of what the module does). If `$ARGUMENTS` is empty, ask first.

Produce a TOML file at `specs/modules/<ModuleName>.toml` that:

1. Validates against the project's aggregated `specs/specdev/schemas/module.schema.yaml` (which includes profile-shipped fields plus any inlined-block contributions).
2. Fits the existing project: consistent naming, uses peripherals declared in `specs/architecture.toml`, depends only on modules that exist (or modules explicitly noted as follow-up work).

If `specdev-analyzer` is not on PATH, tell the user to install it and stop.

Before any analyzer invocation, run the env preamble from `${CLAUDE_PLUGIN_ROOT}/shared/plugin-root-preamble.md`:

```sh
export SPECDEV_PLUGIN_ROOT="${CLAUDE_PLUGIN_ROOT}"
```

Skip if `SPECDEV_PLUGIN_ROOT` is already set or every invocation passes `--plugin-root`.

---

## Step 0 — prerequisites

If `specs/specdev/` is absent at the project root, stop and tell the user to run `/spec-init` first. Do not author blind.

Read `spec.toml` to learn the project's `[project].profile` and `[project].language`. The profile shapes which fields the module schema accepts.

Check `specs/requirements/` (its top level only: `specs/requirements/retired/` holds retired requirements, which no module may satisfy). The module `satisfies` requirements, so the subagent (Step 2) needs requirement ids to already exist. If there is no requirement covering what this module does, tell the user up front — they author requirements under `specs/requirements/` (and `/spec-question` / `/spec-decide` capture the reasoning behind them). Do not let the draft invent requirement ids.

## Step 1 — interview

The deep read (schema, profile prose, inlined-block READMEs, sibling modules) is delegated to the `module-designer` subagent in Step 2 — do **not** read those here. The main thread owns the interview, which is interactive and benefits from full conversation context; the subagent is one-shot.

Ask the user for fields not reasonably inferable from `$ARGUMENTS`. The canonical interview shape is whatever the profile's `module.schema.yaml` declares as required + the inlined-block fields the project intends to use — that schema is the binding contract.

For shipped profiles, illustrative interview-question lists are in `references/interview-shapes.md` (firmware-layered, library-api, rust-workspace routing). For user-authored profiles, read the profile's `module.schema.yaml` and `README.md` directly — no illustrative shape is provided.

Three questions are universal — ask them on every module regardless of profile:

- **Does the code exist yet?** A new module almost always has no code: write `planned = true`. `check` then stays silent about the missing source, and `check --since` reports the flag so it cannot be forgotten. When the code lands, the flag goes (`check` warns about a `planned` module whose source resolves). When code already exists, leave `planned` out and set `source_file` if the file is not where the profile's convention puts it — `check` reports a module with no resolvable source as an error.
- **`module.context_files = [...]` — what must someone read before editing this module?** Reference reading only: a datasheet section, a protocol note, a hardware reference (`docs/HARDWARE_REFERENCE.md`). `specdev-analyzer context --module` lists these paths for whoever edits the code, and `validate` checks that each exists. A choice or a known problem is not reference reading: it belongs in a decision or an open question. Leave the list empty for a mechanical module.
- **`items[]` — only if the project keeps a contract surface.** `items[]` is optional: use it for a public API whose symbols are the contract (a `library-api` crate), where `items diff` checks it against the code. For a type with public methods, get the **type name and its public member names** so the subagent can scaffold one `<Type>::<member>` `kind = "fn"` item each, sharing a coarse requirement — prose-first, never signatures (`references/toml-authoring-rules.md`). Otherwise leave `items[]` out: the module's `satisfies`, `depends_on` and description carry the intent.

Triad awareness: check whether any accepted decision under `specs/decisions/` should guide this module — if so, set `guided_by = ["DEC-..."]` so `report decisions --module` lists it for whoever writes the code. Check `specs/open_questions/` for open questions whose `affects` will reach this module: once the module has code, `check` reports each `open` one as an error (`implementation_unblocked`), so tell the user up front. If the interview itself surfaces a genuine unknown, do not improvise a quiet answer in the spec — point the user at `/spec-question` to record it (or `/spec-decide` if the conversation settles a choice worth keeping).

**A fixed set of domain values** (statuses, types, classifications): before you write the values into the spec, ask the test in `${CLAUDE_PLUGIN_ROOT}/shared/vocabulary-placement.md` — does a non-engineer need to add a value without a deploy? If yes, the module states a table, not the values. Either way, the choice needs a decision (`/spec-decide`) linked from this module's `guided_by`.

Batch all relevant questions in one message. Show defaults so a fluent user can accept them all.

## Step 2 — dispatch the `module-designer` subagent, then write

Dispatch the `module-designer` subagent to draft the Candidate TOML. It reads the context cone, the schema, profile prose, inlined-block READMEs, and sibling modules in an *isolated* context — keeping that 5–15-file read out of this conversation.

The subagent has no shell, so it cannot run the analyzer — that is what makes its read-only contract enforced rather than stated. Write the cone to a file for it first, and **do not read that file here**; reading or pasting it defeats the reason the subagent exists:

```sh
ctx=$(mktemp -t specdev-designer-ctx.XXXXXX) && specdev-analyzer context --module <ModuleName> --output "$ctx" && echo "$ctx"
```

If `context` exits non-zero (malformed project schema, reduction failure), relay its error and stop — do not dispatch.

Pass the input contract from `agents/module-designer.md`:

- `module_name` — the PascalCase name.
- `project_root` — absolute project root.
- `context_yaml` — the absolute path the command above printed.
- `plugin_root` — the absolute plugin-tree path, i.e. `${CLAUDE_PLUGIN_ROOT}` expanded. **Pass it explicitly.** A subagent is a separate process: it inherits neither `CLAUDE_PLUGIN_ROOT` nor the `SPECDEV_PLUGIN_ROOT` the preamble exports, because that export lands in a Bash subshell on this thread. Omit it and the subagent has to hunt for the tree — measured, two agents in one batch found two different paths.
- `profile`, `language` — from `spec.toml` (Step 0).
- `description` — the one-line `$ARGUMENTS`.
- `interview_answers` — the answers gathered in Step 1, inline or as YAML. Include the type + public-member names so the subagent can scaffold per-method items.
- `package` — only for multi-package projects.

The subagent returns three sections: `## Candidate TOML`, `## Evidence`, `## Notes`. The cone is best-effort — on a brand-new module it returns `module-not-found` with an empty sibling set, which is expected, not a failure. If the subagent returns a **blocking error** instead (missing requirement ids, malformed project schema, unresolvable cone), relay it to the user and stop — those are real problems to fix in the spec, not draft around.

Review the Candidate against the Evidence and Notes. Surface anything the Notes flag (per-method descriptions to refine, requirements to author, downstream follow-ups). On the user's confirmation, write the Candidate TOML verbatim to `specs/modules/<ModuleName>.toml` (under `specs/modules/<package>/` for multi-package). The authoring rules in `references/toml-authoring-rules.md` are the subagent's contract; the main thread's job here is the comment-policy carve-out (the Candidate carries only the `#:schema` directive) and the file write.

**Check the write against the disk.** Wrap the write of `specs/modules/<ModuleName>.toml` in the snapshot/compare pair in `${CLAUDE_PLUGIN_ROOT}/shared/write-check.md`, and build the Step 5 report from the file read back, not from the Candidate.

If the user raises a structural issue ("actually this should be two modules"), stop, write a short plan, and wait for confirmation before re-dispatching.

## Step 3 — validate

Run from the project root:

```sh
specdev-analyzer validate --no-color
```

Show the output. If errors:

1. Read each error. Schema errors point at a TOML line; graph-check failures point at a rule name. Map rule names by reading `specs/specdev/domain.toml` `[validation.rules]` + `[[project.graph_checks]]`.
2. Fix the modeling problem. Do not delete fields to silence the checker.
3. Re-validate until clean.

If validation surfaces a structural issue ("actually this should be two modules"), stop, write a short plan, and wait for confirmation before re-dispatching the subagent.

## Step 4 — check against the code

Once validate is clean, run:

```sh
specdev-analyzer check --no-color
```

A `planned` module with no code passes. A module without `planned` and with no resolvable source is an error (`source_resolves`): either the code exists somewhere else (set `source_file`) or the module is planned (set `planned = true`). An `implementation_unblocked` error means an open question reaches a module that already has code — route to `/spec-question`.

## Step 5 — report

One paragraph:
- Module name and path written.
- Key decisions in one line each (layer / visibility, ownership kind, error strategy, bindings).
- Anything the user asked for that wasn't placed in the file, and where it was recorded instead.
- Suggested next step: `/spec-design <next module>`, or write the code. Before writing it, run `specdev-analyzer context --module <ModuleName>` and `specdev-analyzer report decisions --module <ModuleName>`, and read what they list — the inlined guidance, the `context_files`, the decisions. Remove `planned = true` when the code lands, and run `/spec-validate`.

## Additional resources

- `${CLAUDE_PLUGIN_ROOT}/agents/module-designer.md` — the subagent dispatched in Step 2. Single source of truth for its input/output contract (`module_name` / `project_root` / `plugin_root` / `profile` / `language` / `description` / `interview_answers` / optional `existing_toml` / `package` / `source_hint`; three-section `## Candidate TOML` / `## Evidence` / `## Notes` output). Also dispatched by `/spec-edit`.
- `references/interview-shapes.md` — illustrative interview shapes per shipped profile (firmware-layered, library-api) and the rust-workspace routing rule.
- `references/toml-authoring-rules.md` — priority-ordered authoring rules shared with `/spec-edit` and the `module-designer` subagent: schema-as-contract, required fields, the per-method item model, naming conventions, layer discipline, comment policy.

## What this skill does NOT do

- **Edit existing modules.** That's `/spec-edit`'s job. If the target file exists, stop and tell the user.
- **Inline blocks or bridges.** That's `specdev-analyzer inline <block>` / `inline-bridge <a>+<b>`. If the user wants `peripherals` / `tasks` / `bindings.<a>+<b>` fields and the corresponding block/bridge isn't inlined, the schema will reject them; tell the user to inline first.
- **Write code.** The user and the agent write it after the design, following the project rules (`.claude/rules/specdev.md`).
