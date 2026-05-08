---
name: spec-design
description: This skill should be used when the user asks to "design a module", "add a module spec", "author a driver spec", "design <ModuleName>", or needs to write a single module TOML under specs/modules/. Authors one module TOML per invocation, then validates it. **New modules only — refuses if `specs/modules/<ModuleName>.toml` already exists.** To modify an existing module, use /spec-edit.
allowed-tools: Read, Write, Edit, Bash, Glob, Grep
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

## Step 1 — read the authoritative guides

Run the analyzer's context cone for the proposed module name. The cone tells the skill which files to read; you read only those, instead of the whole `specs/modules/` tree:

```sh
specdev-analyzer context --module <ProposedName> --output /tmp/specdev-ctx-$$.yaml
```

For a module that does not yet exist, the cone returns `status: module-not-found`; `files.schema`, `files.block_readmes`, `files.bridge_readmes`, `files.profile_readme`, `files.module_conventions`, and `files.architecture` are still populated (they don't depend on M). `related_modules` is empty in this path; `excluded_module_count` carries the total existing-module count for orientation.

Then read every file path listed under `files:` and (when the module exists) under `related_files:`. The schemas establish the contract; the READMEs are the authoring view (distinct from the codegen prose `/spec-gen` consumes). The architecture file carries project-wide picks. `${CLAUDE_PLUGIN_ROOT}/profiles/<profile>/codegen/<language>/module.conventions.md` is *codegen* prose — read but do not over-apply; it informs which spec choices have semantic weight downstream.

When the module already exists or a sibling-shape inference is needed, the cone's `related_files` gives the bounded sibling set to read (lex-sorted, project-relative). The cone is one hop deep — if a sibling references something farther out, re-invoke `context --module <Sibling>` for that next-hop view.

## Step 2 — interview

Ask the user for fields not reasonably inferable from `$ARGUMENTS`. The canonical interview shape is whatever the profile's `module.schema.yaml` declares as required + the inlined-block fields the project intends to use — that schema is the binding contract.

For shipped profiles, illustrative interview-question lists are in `references/interview-shapes.md` (firmware-layered, library-api, rust-workspace routing). For user-authored profiles, read the profile's `module.schema.yaml` and `README.md` directly — no illustrative shape is provided.

Two questions are universal — ask them on every module regardless of profile (F-013, F-015 from the zfm audio-plugin dogfood):

- **`items[]` — what symbols does this module own?** Walk the user through the structural items they want emitted: `fn` / `struct` / `enum` / `trait` / `re_export` / etc., each with a short `description` and (when known) a `satisfies = [...]` list naming the requirements it covers. Modules with empty `items[]` give `/spec-gen` nothing to anchor on and offer no traceability surface to `/spec-sync-items`. If the user is genuinely architecting and doesn't know the items yet, record that as a `# TODO: items[]` comment instead of leaving the array silently absent.
- **`module.context_files = [...]` — does this module need out-of-band prose at codegen time?** When the module is deep-domain (DSP math, hardware sequencing, protocol details, MIDI quirks), point the user at any project-prose files (`docs/dsp-notes.md`, `docs/yamaha-fm-arch.md`) the codegen agent should inline alongside the spec. The analyzer reads each path's contents into the assembled prompt at `prompt --module` time. Skip this if the module is mechanical (config types, glue code) — empty `context_files` is the right answer for shallow modules.

Batch all relevant questions in one message. Show defaults so a fluent user can accept them all.

## Step 3 — write the TOML

Write `specs/modules/<ModuleName>.toml` following the priority-ordered authoring rules in `references/toml-authoring-rules.md`: schema-as-contract, required fields by profile, required metadata for firmware-layered, inline-table preference, naming conventions, layer discipline, one-module-per-file, and the comment policy carve-outs.

## Step 4 — validate

Run from the project root:

```sh
specdev-analyzer validate --no-color
```

Show the output. If errors:

1. Read each error. Schema errors point at a TOML line; graph-check failures point at a rule name. Map rule names by reading `specs/specdev/domain.toml` `[validation.rules]` + `[[project.graph_checks]]`.
2. Fix the modeling problem. Do not delete fields to silence the checker.
3. Re-validate until clean.

If the user raises a structural issue ("actually this should be two modules"), stop, write a short plan, and wait for confirmation before rewriting.

## Step 5 — optional: extract and show

Once validate is clean, run:

```sh
specdev-analyzer extract --output /tmp/specdev-extract-<pid>
```

Read the module's YAML and show the user the first ~30 lines. This previews what `/spec-gen` will consume and surfaces any data-shape surprises early.

## Step 6 — report

One paragraph:
- Module name and path written.
- Key decisions in one line each (layer / visibility, ownership kind, error strategy, bindings).
- Anything the user asked for that wasn't placed in the file, and where it was recorded instead.
- Suggested next step: `/spec-design <next module>`, or `/spec-gen <ModuleName>` once at least one module exists and a code-path exists for the inlined blocks/bridges.

## Additional resources

- `references/interview-shapes.md` — illustrative interview shapes per shipped profile (firmware-layered, library-api) and the rust-workspace routing rule.
- `references/toml-authoring-rules.md` — priority-ordered authoring rules shared with `/spec-edit`: schema-as-contract, required fields, naming conventions, layer discipline, comment policy.

## What this skill does NOT do

- **Edit existing modules.** That's `/spec-edit`'s job. If the target file exists, stop and tell the user.
- **Inline blocks or bridges.** That's `specdev-analyzer inline <block>` / `inline-bridge <a>+<b>`. If the user wants `peripherals` / `tasks` / `bindings.<a>+<b>` fields and the corresponding block/bridge isn't inlined, the schema will reject them; tell the user to inline first.
- **Generate code.** That's `/spec-gen`'s job.
