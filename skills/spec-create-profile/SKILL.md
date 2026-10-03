---
name: spec-create-profile
description: "This skill should be used when the user asks to \"create a profile\", \"author a new profile\", \"scaffold a profile\", \"write a profile for <project-shape>\", \"add a profile that produces <kind of project>\", or \"/spec-create-profile <name>\" — i.e. authoring a NEW profile unit (the one-shot project-shape scaffolding: profile.yaml manifest + schemas + reduction + templates + implementation guidance + CI workflow template) for a project shape not already covered. New profiles only — refuses if `profiles/<name>/` already exists. To bootstrap an EXISTING profile into a project, use /spec-init. This skill authors profiles only, not blocks or bridges."
allowed-tools: Read, Write, Edit, Bash, Glob, Grep
argument-hint: <profile-name> (e.g. firmware-layered, library-api)
---

# /spec-create-profile — Author a new profile

Scaffold a complete profile — the one-shot **project-shape** scaffolding a project picks at `/spec-init`. A profile defines what a module spec looks like for that shape (its `module.schema.yaml`), how the project's TOML reduces to the universal graph (`reductions/project.yaml`), which graph checks run (`domain.toml.template`), the rules whoever writes the code follows (`guidance.md`), the compile gates CI runs (`profile.yaml` `gates:`), and the analyzer's dispatch contract (`profile.yaml`).

Profile authoring is mostly **copy-and-adapt**: the stable patterns (the enum vocabularies, the ledger schemas, the `requirements_have_satisfiers` graph check) are shared across profiles; the load-bearing work is the module schema, the graph-check set, the gates, and the guidance. Its correctness gate is *bootstrap into a scratch project → check → run the gate script*.

specdev does not generate code, so a profile ships no code templates or manifest emitter: the project's build manifest is the truth for dependencies, and the code is written by hand against the guidance.

**This skill authors; it does not bootstrap.** After it lands, a project adopts it with `/spec-init --profile <name>`.

User input: `$ARGUMENTS` — the profile name. If empty, ask what project shape the profile produces (firmware? library? a plugin/cdylib? a workspace root?) before anything else.

If `specdev-analyzer` is not on PATH, tell the user to install it
(follow `${CLAUDE_PLUGIN_ROOT}/shared/analyzer-install.md` — it derives the org from the plugin's own remote; both repositories are private) and stop. If it IS on PATH but reports `MANIFEST-002`, or a `MANIFEST-001` naming a file under the plugin tree, then the analyzer is older than the plugin — not a problem with any project file. Tell the user to re-run the install command with `--force` (or `cargo install --path <checkout> --force` from a local checkout), then retry.

Before any analyzer invocation, run the env preamble from
`${CLAUDE_PLUGIN_ROOT}/shared/plugin-root-preamble.md`:

```sh
export SPECDEV_PLUGIN_ROOT="${CLAUDE_PLUGIN_ROOT}"
```

Skip if `SPECDEV_PLUGIN_ROOT` is already set or every invocation passes
`--plugin-root`.

The exact file inventory + shapes are in
`references/profile-artifact-templates.md` — read it before Step 2. This body is
the orchestration; that file is the material.

---

## Step 0 — prerequisites, scope guard, target root, the SHAPE decisions

1. **Choose the target plugin root.** Default `${CLAUDE_PLUGIN_ROOT}`; test writability (`test -w "<root>/profiles" && echo writable`). If read-only, ask for a writable plugin-tree checkout (sharing is upstream-PR or file-copy). Confirm `<root>/.claude-plugin/plugin.json` exists.

2. **Refuse if the profile already exists.** If `<root>/profiles/<name>/` is present, stop: "That profile already exists — edit it directly, or `/spec-init --profile <name>` to bootstrap it into a project." New-authoring only.

3. **Name discipline.** Lowercase-hyphenated, `^[a-z][a-z0-9-]*$` (the manifest enforces this). Describes the *shape*, not a framework (`firmware-layered`, `library-api`, `audio-plugin`, `rust-workspace`).

4. **Screen for analyzer-work scope.** A profile is authorable as **pure content** only when it reuses existing analyzer machinery. A **novel graph-check primitive** is not in this skill's scope: reuse the existing primitives (`requirements_have_satisfiers`, `modules_acyclic`, `layer_dependency`, `entity_has_inbound_relation`, the `*_resolves` family, …). If the shape needs a brand-new check kind, stop and tell the user it needs an analyzer change first.

5. **Language.** One of `rust` / `dart` / `typescript` / `c` / `cpp` — each has the tree-sitter adapter that `check` uses for test discovery and imports. The manifest names the primary `language`, and your gate steps must agree with `languages/<lang>/build.md` § "Toolchain Commands".

6. **Read the closest precedent end to end.** Pick the shipped profile nearest the target shape and read its whole tree — you will copy-and-adapt it:
   - `<root>/profiles/library-api/` (rust) — standalone library, no layers, `items[]` as a contract surface.
   - `<root>/profiles/firmware-layered/` (rust) — `no_std` binary, 5-tier layer ordering, `metadata` schema, a gate step with a `${root.*}` token.
   - `<root>/profiles/rust-app/` (rust) — standalone executable, testable core plus a thin entry point.
   - `<root>/profiles/audio-plugin/` (rust) — cdylib plugin, 4-tier layers, parameter bindings.
   - `<root>/profiles/rust-workspace/` (rust) — the workspace-root profile.
   - `<root>/profiles/flutter-app/` (dart) — roles, a two-step gate.
   - `<root>/profiles/nextjs-app/` (typescript) — roles, `paired_files` config templates, and **two** gates. Read this one whenever the new profile needs paired files or more than one gate.
   Also read `references/profile-artifact-templates.md`.

## Step 1 — interview the project shape

Batch these; show the precedents' answers as defaults.

- **Module shape** — what does a module spec carry for this project shape? This is the load-bearing decision; it drives `module.schema.yaml`. Firmware modules carry `layer` + `actions` + `ports` + `state_machines` + `hardware` + `metadata`; library modules carry `visibility` + optional `items`. Every field must state intent that constrains the code and have a check or a guidance rule — never a type, a signature or a dependency the code or the manifest already holds. `name`, `description`, `source_file`, `planned`, `satisfies`, `depends_on`, `guided_by` and `context_files` are universal.
- **Layer or role ordering?** Does the profile impose an ordered tier system (firmware: 5 layers; audio: 4; apps: roles)? If yes, the enum + a `layer_dependency` (or role) graph check; if no (library, workspace), skip it.
- **Graph checks** (`domain.toml.template`): which checks run + their severity. Every member-tier profile ships `requirements_have_satisfiers` (error) and the open-question gate `implementation_unblocked` (error); a workspace root ships its member and ledger checks. Add shape-specific checks by reusing existing primitives.
- **Implementation guidance** (`guidance.md`): the rules for code in this shape that no check can verify — file layout, the entry point, layer meaning, error strategy, the mistakes that compile and then fail. **Name the role a block fills, not a specific block**, unless the profile is genuinely single-framework and says so.

  ⚠️ **Every API shape a rule names must be read, never recalled.** Open the installed version's documentation and confirm the current form before writing a rule down. SDK APIs are deprecated and replaced between releases. This has shipped wrong: a UI profile documented a selection widget's per-item callback form that current releases had replaced with an ancestor wrapper; it was found only when a person ran the app. Prefer rules that are stable across releases; the guidance carries no version numbers.
- **Compile gates** (`profile.yaml` `gates:`). Each entry is a `label` and `steps`:

  ```yaml
  gates:
    - label: fast
      steps:
        - [pnpm, install]
        - [pnpm, exec, tsc, --noEmit]
        - [pnpm, exec, vitest, run, --passWithNoTests]
    - label: next-build
      steps:
        - [pnpm, exec, next, build]
  ```

  - `steps` is a list of **argv lists**, never a shell string. One process per argv list, run in order in the project root with stdin closed; the gate stops at the first non-zero exit. `steps` needs at least one entry and each argv needs at least one element — the schema refuses both empty forms.
  - `label` is **opaque** and carries no whitespace; two gates with one label are refused when the profile loads.
  - A `${root.<table>.<field>}` token in an argument resolves against the project's `specs/architecture.toml` when the gate script is rendered (`firmware-layered` uses `${root.platform.target_triple}`). A missing or empty field renders a gate that prints the cause and exits 2.
  - **Match the argv to the check you mean.** Measured: `flutter analyze` defaults to `--fatal-infos --fatal-warnings` and exits 1 on one unused import, so the shipped row is `[flutter, analyze, --no-fatal-infos, --no-fatal-warnings]`.
  - Choose steps the language tier already documents (`languages/<lang>/build.md`; its "Toolchain Commands" section where it has one), or copy the closest precedent profile's gates.
  - The analyzer renders the gates into `scripts/specdev-gates.sh`, which runs `specdev-analyzer check` first. CI runs the script.
- **CI toolchain and workflow template.** Which tools must CI install for the gates (the language toolchain, the package manager, Rust to build the analyzer)? The README lists them and says installs are frozen with a committed lockfile. Ship a GitHub Actions workflow template as a paired file (`templates/specdev-ci.yml.template` → `.github/workflows/specdev.yml`) that installs exactly those tools, builds the analyzer, and runs `sh scripts/specdev-gates.sh`. Copy the closest shipped profile's template.
- **Auxiliary schemas?** Does the shape need an extra schema beyond the standard set (firmware-layered's `metadata.schema.yaml`)? Bootstrap enumerates `schemas/` dynamically, so extras are fine.

## Step 2 — scaffold the profile (copy-and-adapt)

Copy the precedent profile's tree into `<root>/profiles/<name>/` and adapt. Follow `references/profile-artifact-templates.md` for the full inventory.

**"Reuse" never means byte-verbatim.** Even a file marked *reuse* below must have its `$id` rewritten to `https://specdev.dev/profiles/<name>/…` **and** every occurrence of the source profile's name scrubbed from `title` / `description` prose. *Reuse* = copy the shape, rewrite the identity; *adapt* = that plus real content changes.

| File | Adapt or reuse |
|---|---|
| `profile.yaml` | **Adapt.** `contract_version: 2`, `name`, `language`, **`gates`** (Step 1), `bootstrap.steps`, `context` (context-subcommand extras), `surface` flags, `paired_files` (config files written once when absent — at least the CI workflow template). Optional: `items`, `reverse`. Validated against the embedded `profile.schema.yaml`. |
| `schemas/module.schema.yaml` | **Adapt** (load-bearing). Set `unevaluatedProperties: false` at the root — **never** `additionalProperties: false` (block contributions arrive via the aggregator's `allOf` slot). Reference `inlined/languages/{{language}}/items.schema.yaml` for the optional `items[]`. |
| `schemas/enums.schema.yaml` | **Reuse + extend.** Copy `Identifier` / `ModuleRef` / `RequirementId` / `QualifiedPort` verbatim; add profile-specific vocabularies (e.g. a `LayerKind` enum). |
| `schemas/architecture.schema.yaml` | **Adapt.** `additionalProperties: false` at the top. Only fields a check reads (connections, project decisions, a platform fact a gate uses). |
| `schemas/requirements.schema.yaml`, `schemas/decisions.schema.yaml`, `schemas/open_questions.schema.yaml` | **Reuse** the shape (the ledgers are stable) — but rewrite the `$id` + title as above. |
| `reductions/project.yaml` | **Adapt.** `metaschema_version: "v1"`; maps this profile's TOML to universal `entities` + `relations`, with glob `from:` paths over the per-entry ledger directories. |
| `domain.toml.template` | **Adapt.** `[domain] name = "{{project_name}}"`, `[validation.rules]` (one entry per graph check → severity), `[[project.graph_checks]]`. |
| `spec.toml.template` | **Adapt.** Placeholders `{{project_name}}` / `{{profile}}` / `{{language}}`; `spec_version = "2.0"`. |
| `architecture.toml.template` | **Reuse/adapt** — an empty-or-example skeleton (omit when the architecture schema has nothing to fill). |
| `templates/specdev-ci.yml.template` | **Adapt.** The CI workflow (Step 1). |
| `guidance.md` | **Author** (load-bearing). Rules only: no fenced code and at most 4,096 bytes (`plugin-lint` checks these), no version numbers (review checks this). Role-not-framework naming. Inlined to `specs/specdev/guidance/profile.md`. |
| `README.md` | **Author.** Overview, when-to-use, file inventory, graph-check list, **the CI toolchain list** with the frozen-install note. |

Invariants to hold:
- Schema `$id`s match `https://specdev.dev/profiles/<name>/<schema-base>.schema.yaml`.
- Only `module.schema.yaml` gets the `{{language}}` substitution; other schemas are byte-faithful (no stray `{{...}}` → `BOOTSTRAP-002`).
- The placeholder vocabulary is closed: `{{project_name}}`, `{{profile}}`, `{{language}}`.

## Step 3 — gate at authoring time (bootstrap → check → gates)

The gate is a real bootstrap round-trip in a throwaway scratch directory (use the session scratch dir, not the plugin tree):

1. **Bootstrap into a scratch project.** `<TARGET>` is a **required trailing positional** — omitting it exits 2:
   ```sh
   specdev-analyzer bootstrap --profile <name> --language <lang> --name smoke \
     --plugin-root <root> <scratch>/<name>-smoke
   cd <scratch>/<name>-smoke
   ```
   A missing required file or a malformed manifest fails here (`BOOTSTRAP-00x`). Confirm the scratch project has `scripts/specdev-gates.sh`, `specs/specdev/guidance/profile.md`, and every paired file.

2. **Author a minimal project.** Bootstrap writes no build manifest, so write the smallest one the gates need (`Cargo.toml`, `package.json` or `pubspec.yaml`), plus one requirement under `specs/requirements/`. Read the *bootstrapped* project's `specs/specdev/schemas/module.schema.yaml` `required:` list for the schema floor, author the smallest module the schema accepts (it `satisfies` the requirement), write the smallest source file the guidance's layout puts it in, and either a test the requirement names in `verified_by` (`specdev-analyzer report tests` shows the identity) or `test_pending = true` on the requirement. Then:
   ```sh
   specdev-analyzer check --no-color
   ```
   Zero **errors** confirms the schemas, reduction, graph checks, the source rule and the test rule compose.

3. **Run the gate script for real** — a registry that parses is not a registry that runs:
   ```sh
   sh scripts/specdev-gates.sh; echo "exit $?"
   ```
   It runs `check` and every gate in order. Exit `0` is a pass. Afterwards, set `planned = true` on the module and delete its source file: `check` must stay clean (a planned module with no code is silent). Another non-zero code means a step **ran and failed** — a real finding. Exit `2` means a step **could not run**: its executable could not start, or a `${root.*}` token resolved to nothing. A wrong token is a profile bug; a missing tool is an environment gap. The message says which — read it before you call the gate green. (Set `SPECDEV_ANALYZER` to the analyzer's path if it is not on `PATH`.)

4. **plugin-lint** for structural hygiene and the guidance rules:
   ```sh
   specdev-analyzer plugin-lint --plugin <root>
   ```

Iterate until bootstrap + check report **zero errors**, the gate script exits 0 and `plugin-lint` is clean; surface any finding verbatim. Delete the scratch project when done.

## Step 4 — report + next steps

Report the files written, the bootstrap + check results, each gate's label and exit code through the script, the `plugin-lint` result, and any analyzer-work items you had to defer (a novel graph-check primitive — Step 0.4). **Next:** `/spec-init --profile <name>` in a real project; `/spec-design` modules against it. To share, PR it upstream. **Live smoke of a just-authored profile is next-session** if the tree only just changed (bootstrap reads the plugin tree at invocation, but the skill loads at session start) — say so rather than claiming an end-to-end pass this session.

## What this skill does NOT do

- **Author a block or bridge.** Framework-specific content is a block; wiring two blocks is a bridge. A profile names *roles* and shapes; it does not embed a framework.
- **Bootstrap the profile into a project.** Use `/spec-init --profile <name>`.
- **Add a novel graph-check primitive.** That is an analyzer change (Step 0.4).
- **Ship code templates or a manifest emitter.** The project's build manifest is the truth; the code is written by hand against the guidance.
- **Publish / register / version.** No registry or semver in v1.
