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

Check `specs/requirements.toml`. Every non-`re_export` item the module owns must `satisfies` a requirement, so the subagent (Step 2) needs requirement ids to already exist. If the file is absent or has no requirement covering what this module does, tell the user up front — they author requirements in `specs/requirements.toml` (and `/spec-question` / `/spec-decide` capture the reasoning behind them). Do not let the draft invent requirement ids.

## Step 1 — interview

The deep read (schema, profile prose, inlined-block READMEs, sibling modules) is delegated to the `module-designer` subagent in Step 2 — do **not** read those here. The main thread owns the interview, which is interactive and benefits from full conversation context; the subagent is one-shot.

Ask the user for fields not reasonably inferable from `$ARGUMENTS`. The canonical interview shape is whatever the profile's `module.schema.yaml` declares as required + the inlined-block fields the project intends to use — that schema is the binding contract.

For shipped profiles, illustrative interview-question lists are in `references/interview-shapes.md` (firmware-layered, library-api, rust-workspace routing). For user-authored profiles, read the profile's `module.schema.yaml` and `README.md` directly — no illustrative shape is provided.

Two questions are universal — ask them on every module regardless of profile (F-013, F-015 from the zfm audio-plugin dogfood):

- **`items[]` — what symbols does this module own?** Walk the user through the structural items they want emitted: `fn` / `struct` / `enum` / `trait` / `re_export` / etc., each with a short `description` and (when known) a `satisfies = [...]` list naming the requirements it covers. For a type with public methods, get the **type name and its public member names** — that pair is what lets the subagent scaffold the per-method items (one `<Type>::<member>` `kind = "fn"` item each, sharing a coarse requirement) so the user writes descriptions, not boilerplate. You do not need full signatures; method names plus intent are enough (the item model is prose-first per `references/toml-authoring-rules.md`). Modules with empty `items[]` give `/spec-gen` nothing to anchor on and offer no traceability surface to `/spec-sync-items`. If the user is genuinely architecting and doesn't know the items yet, record that as a `# TODO: items[]` comment instead of leaving the array silently absent.
- **`module.context_files = [...]` — does this module need out-of-band prose at codegen time?** When the module is deep-domain (DSP math, hardware sequencing, protocol details, MIDI quirks), point the user at any project-prose files (`docs/dsp-notes.md`, `docs/yamaha-fm-arch.md`) the codegen agent should inline alongside the spec. The analyzer reads each path's contents into the assembled prompt at `prompt --module` time. Skip this if the module is mechanical (config types, glue code) — empty `context_files` is the right answer for shallow modules.

Triad awareness (when the project authors `specs/decisions.toml` / `specs/open_questions.toml`): check whether any accepted decision should guide this module's design — if so, set `guided_by = ["DEC-..."]` so the rationale is inlined at codegen time. Check `specs/open_questions.toml` for open questions whose `affects` will reach this module: an open question hard-blocks `/spec-gen` for it, so tell the user up front. If the interview itself surfaces a genuine unknown, do not improvise a quiet answer in the spec — point the user at `/spec-question` to record it (or `/spec-decide` if the conversation settles a choice worth keeping).

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

## Step 4 — optional: extract and show

Once validate is clean, run:

```sh
specdev-analyzer extract --output /tmp/specdev-extract-<pid>
```

Read the module's YAML and show the user the first ~30 lines. This previews what `/spec-gen` will consume and surfaces any data-shape surprises early.

## Step 5 — report

One paragraph:
- Module name and path written.
- Key decisions in one line each (layer / visibility, ownership kind, error strategy, bindings).
- Anything the user asked for that wasn't placed in the file, and where it was recorded instead.
- Suggested next step: `/spec-design <next module>`, or `/spec-gen <ModuleName>` once at least one module exists and a code-path exists for the inlined blocks/bridges.

## Additional resources

- `${CLAUDE_PLUGIN_ROOT}/agents/module-designer.md` — the subagent dispatched in Step 2. Single source of truth for its input/output contract (`module_name` / `project_root` / `plugin_root` / `profile` / `language` / `description` / `interview_answers` / optional `existing_toml` / `package` / `source_hint`; three-section `## Candidate TOML` / `## Evidence` / `## Notes` output). Also dispatched by `/spec-edit`.
- `references/interview-shapes.md` — illustrative interview shapes per shipped profile (firmware-layered, library-api) and the rust-workspace routing rule.
- `references/toml-authoring-rules.md` — priority-ordered authoring rules shared with `/spec-edit` and the `module-designer` subagent: schema-as-contract, required fields, the per-method item model, naming conventions, layer discipline, comment policy.

## What this skill does NOT do

- **Edit existing modules.** That's `/spec-edit`'s job. If the target file exists, stop and tell the user.
- **Inline blocks or bridges.** That's `specdev-analyzer inline <block>` / `inline-bridge <a>+<b>`. If the user wants `peripherals` / `tasks` / `bindings.<a>+<b>` fields and the corresponding block/bridge isn't inlined, the schema will reject them; tell the user to inline first.
- **Generate code.** That's `/spec-gen`'s job.
