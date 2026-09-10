---
name: spec-create-profile
description: This skill should be used when the user asks to "create a profile", "author a new profile", "scaffold a profile", "write a profile for <project-shape>", "add a profile that produces <kind of project>", or "/spec-create-profile <name>" — i.e. authoring a NEW profile unit (the one-shot project-shape scaffolding: profile.yaml manifest + schemas + reduction + templates + codegen conventions) for a project shape not already covered. New profiles only — refuses if `profiles/<name>/` already exists. To bootstrap an EXISTING profile into a project, use /spec-init. This skill authors profiles only, not blocks or bridges.
allowed-tools: Read, Write, Edit, Bash, Glob, Grep
argument-hint: <profile-name> (e.g. firmware-layered, library-api)
---

# /spec-create-profile — Author a new profile

Scaffold a complete profile — the one-shot **project-shape** scaffolding a
project picks at `/spec-init`. A profile defines what a module spec looks like
for that shape (its `module.schema.yaml`), how the project's TOML reduces to the
universal graph (`reductions/project.yaml`), which graph-checks run
(`domain.toml.template`), how source is emitted (`codegen/<lang>/*.conventions.md`),
and the analyzer's dispatch contract (`profile.yaml`).

Profile authoring is mostly **copy-and-adapt**: the stable patterns
(`CrateReference`, the enum vocabularies, the `requirements_have_satisfiers`
graph-check) are shared across profiles; the load-bearing work is the module
schema, the codegen conventions prose, the graph-check set, and the manifest.
**Unlike blocks and bridges, a profile ships NO reference crate** — its codegen
prose is uniform and generics-free, so its correctness gate is *bootstrap into a
scratch project → validate → (optionally) gen-and-compile a minimal module*, the
same smoke pattern the shipped `hello-firmware` / `hello-plugin` fixtures use.

**This skill authors; it does not bootstrap.** After it lands, a project adopts it
with `/spec-init --profile <name>`.

User input: `$ARGUMENTS` — the profile name. If empty, ask what project shape the
profile produces (firmware? library? a plugin/cdylib? a workspace root?) before
anything else.

If `specdev-analyzer` is not on PATH, tell the user to install it
(`cargo install --git https://github.com/cosmikwolf/specdev-analyzer`) and stop. If it IS on PATH but reports `MANIFEST-002`, or a `MANIFEST-001` naming a file under the plugin tree, then the analyzer is older than the plugin — not a problem with any project file. Tell the user to re-run the install command with `--force` (or `cargo install --path <checkout> --force` from a local checkout), then retry.

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

1. **Choose the target plugin root.** Default `${CLAUDE_PLUGIN_ROOT}`; test
   writability (`test -w "<root>/profiles" && echo writable`). If read-only, ask
   for a writable plugin-tree checkout (sharing is upstream-PR or file-copy).
   Confirm `<root>/.claude-plugin/plugin.json` exists.

2. **Refuse if the profile already exists.** If `<root>/profiles/<name>/` is
   present, stop: "That profile already exists — edit it directly, or
   `/spec-init --profile <name>` to bootstrap it into a project." New-authoring only.

3. **Name discipline.** Lowercase-hyphenated, `^[a-z][a-z0-9-]*$` (the manifest
   enforces this). Describes the *shape*, not a framework (`firmware-layered`,
   `library-api`, `audio-plugin`, `rust-workspace`).

4. **Screen for analyzer-work scope (do this before promising a content-only
   scaffold).** A profile is authorable as **pure content** (manifests + schemas
   + templates + prose) only when it reuses existing analyzer machinery. Two
   things a new profile might need that are **NOT** in this skill's scope — if the
   answer needs them, stop and tell the user it needs an analyzer change first:
   - **A novel build-manifest shape.** `profile.yaml` carries one manifest key
     per language, each with a closed `shape` enum: `cargo_toml.shape`
     (`library` / `binary` / `plugin` / `workspace`), `pubspec_yaml.shape`
     (`app` / `package` / `workspace`) and `package_json.shape` (`app` /
     `library` / `workspace`). If the project shape maps to one of these, you
     are content-only. A genuinely new shape needs analyzer work.
   - **A novel graph-check primitive.** Reuse the existing primitives
     (`requirements_have_satisfiers`, `modules_acyclic`, `entity_has_inbound_relation`,
     the `*_resolves` family, …). A brand-new check kind needs analyzer work.

5. **Language.** One of `rust` / `dart` / `typescript` (the manifest enum also
   admits `c` / `cpp`, which have an audit tier but no codegen adapter — do not
   pick them). Default `rust`. A profile can support multiple languages (one
   `codegen/<lang>/` tree each), but author one language's conventions per
   invocation; the manifest names the primary `language`. The language decides
   which manifest key you fill (Step 0.4) and which `languages/<lang>/build.md`
   your gate steps must agree with.

6. **Read the closest precedent end to end.** Pick the shipped profile nearest the
   target shape and read its whole tree — you will copy-and-adapt it:
   - `<root>/profiles/library-api/` (rust) — standalone `library` crate, no
     layers, features load-bearing.
   - `<root>/profiles/firmware-layered/` (rust) — `no_std` `binary`, 5-tier
     layer ordering, `metadata` schema, a gate step with a `${root.*}` token.
   - `<root>/profiles/rust-app/` (rust) — standalone executable, testable core
     plus a thin entry point.
   - `<root>/profiles/audio-plugin/` (rust) — `plugin`/cdylib, 4-tier, features.
   - `<root>/profiles/rust-workspace/` (rust) — the `workspace`-root profile.
   - `<root>/profiles/flutter-app/` (dart) — `pubspec_yaml`, a two-step gate.
   - `<root>/profiles/nextjs-app/` (typescript) — `package_json` `shape: app`,
     `paired_files` config templates, `derived_files` route rules, and **two**
     gates (a module-scope `fast` and a batch-scope `next-build`). Read this one
     whenever the new profile needs derived files or more than one gate.
   Also read `references/profile-artifact-templates.md`.

## Step 1 — interview the project shape

Batch these; show the precedents' answers as defaults.

- **Build-manifest shape** (Step 0.4): for rust, `cargo_toml.shape` —
  `library` / `binary` / `plugin` / `workspace`, plus edition and whether
  `[[features]]` synthesis is on (library-api/audio-plugin: yes;
  firmware-layered: no). For dart, `pubspec_yaml.shape` — `app` / `package` /
  `workspace` — plus `environment`. For typescript, `package_json.shape` —
  `app` / `library` / `workspace` — plus `package_manager`, `engines` and the
  default `scripts` map.
- **Module shape** — what does a module spec carry for this project shape? This is
  the load-bearing decision; it drives `module.schema.yaml`. Firmware modules carry
  `layer` + `actions` + `ports` + `state_machines` + `hardware` + `metadata`;
  library modules carry `visibility` + `items` + `feature_gates`. Enumerate the
  fields *this* shape needs. `items[]` is universal (via the `{{language}}` $ref).
- **Layer ordering?** Does the profile impose an ordered tier system (firmware: 5
  layers; audio: 4)? If yes, the layer enum + a `layer_dependency` graph-check; if
  no (library, workspace), skip it.
- **Graph-checks** (`domain.toml.template`): which checks run + their severity.
  Every profile ships `requirements_have_satisfiers` for orphan-requirement parity. Add
  shape-specific checks by reusing existing primitives (acyclic, layer-dependency,
  orphan/inbound-relation). A novel check kind is out of scope (Step 0.4).
- **Codegen conventions** (`codegen/<lang>/module.conventions.md` — any
  `*.conventions.md`, named for what it describes, not the profile): the uniform
  rules mapping a module spec to source for this shape. **Name the role a block
  fills, not a specific block** — a profile is pluralistic; it lists frameworks
  only as `e.g.` examples, unless the profile is genuinely single-framework and
  says so. (This is the symmetric side of the block/bridge `PROSE-PROFILE-001`
  discipline.)

  ⚠️ **Every API shape in this prose must be read, never recalled.** Blocks and
  bridges have a reference crate that compiles their claims; a profile has no
  such artifact, so its conventions prose is the *least* checked substrate in
  the tree. Whatever framework, SDK or standard-library shape you name — a
  widget constructor, a lifecycle hook, an idiom — open the installed version's
  documentation and confirm the current form before writing it down. SDK APIs
  are deprecated and replaced between releases, and prose written from training
  data compiles against a version that no longer exists.

  This has already shipped wrong: a UI profile documented a selection widget's
  per-item callback form that current releases had replaced with an ancestor
  wrapper. It type-checked at authoring time against nothing, reached a real
  project, and was found only when a person ran the app.

  Prefer shapes that are stable across releases. When you must name a
  version-sensitive one, say which version you verified it against, so the next
  author knows how old the claim is.
- **Compile gates** (`profile.yaml` `gates:`) — **the profile must declare at
  least one gate with `scope: module`.** This is not optional in practice: with
  no module-scope gate, `report gen-plan` reads `gates_declared: false`, every
  module reads `verified: false` forever, and `specdev-analyzer gen --module <M>
  --tail` refuses to assemble a prompt at all. A profile that ships no gate ships
  a project that cannot generate code.

  The grammar. Each entry is a `label`, a `scope` and `steps`:

  ```yaml
  gates:
    - label: fast
      scope: module
      steps:
        - [pnpm, install]
        - [pnpm, exec, tsc, --noEmit]
        - [pnpm, exec, vitest, run, --passWithNoTests]
    - label: next-build
      scope: batch
      steps:
        - [pnpm, exec, next, build]
  ```

  - `steps` is a list of **argv lists**, never a shell string. One process per
    argv list, run in order in the project root with the inherited environment
    and stdin closed; the gate stops at the first non-zero exit. A shell string
    would need `sh -c`, would collapse N exit codes into one, and would add a
    second quoting grammar. `steps` needs at least one entry and each argv needs
    at least one element — the schema refuses both empty forms.
  - `label` is **opaque** and carries no whitespace. It is the row key in
    `gen-lineage.toml`, paired with the module. Two gates with one label are
    refused when the profile loads, because their rows would overwrite each
    other. Renaming a label makes every old row unreadable; the runner prunes
    them and prints one `pruned …` line each.
  - `scope: module` = one execution per run, one recorded row per module in the
    run — **not** one execution per module. `scope: batch` = one row, no module;
    use it for a whole-tree build too slow to key per module.
  - A `${root.<table>.<field>}` token in an argument resolves against the
    project's `specs/architecture.toml` at run time (`firmware-layered` uses
    `${root.platform.target_triple}`). A missing or empty field is an error, not
    an empty string.
  - **Match the argv to the prose you would otherwise have written.** Measured:
    `flutter analyze` defaults to `--fatal-infos --fatal-warnings` and exits 1 on
    one unused import, while the prose it replaced said "errors only" — so the
    shipped row is `[flutter, analyze, --no-fatal-infos, --no-fatal-warnings]`.
    A mechanical translation can silently make the gate stricter than the check
    it replaces.
  - Choose steps the language tier already documents: `languages/<lang>/build.md`
    § "Toolchain Commands" is the list of commands that language actually has.
- **Auxiliary schemas?** Does the shape need an extra schema beyond the standard
  set (firmware-layered's `metadata.schema.yaml`)? Bootstrap enumerates `schemas/`
  dynamically, so extras are fine.

## Step 2 — scaffold the profile (copy-and-adapt)

Copy the precedent profile's tree into `<root>/profiles/<name>/` and adapt.
Follow `references/profile-artifact-templates.md` for the full inventory.

**"Reuse" never means byte-verbatim.** Even a file marked *reuse* below must have
its `$id` rewritten to `https://specdev.dev/profiles/<name>/…` (load-bearing per
the invariants) **and** every occurrence of the source profile's name scrubbed
from `title` / `description` prose (cosmetic, but a verbatim copy otherwise ships
the clone wearing `library-api`'s identity). *Reuse* = copy the shape, rewrite the
identity; *adapt* = that plus real content changes.

The required files (bootstrap fails if any is missing):

| File | Adapt or reuse |
|---|---|
| `profile.yaml` | **Adapt.** The analyzer dispatch manifest: `name`, `language`, one manifest key (`cargo_toml` — shape + edition + features + `architecture_deps` slots; or `pubspec_yaml`; or `package_json`), **`gates`** (the compile-gate registry — at least one `scope: module` entry, Step 1), `bootstrap.steps`, `context` (module fields fed to the codegen prompt + context-subcommand extras), `surface` flags. Optional: `paired_files` (config files written once when absent, then externally owned), `derived_files` (files the analyzer derives per matching module — `nextjs-app`'s route re-exports), `items`, `reverse`. Validated against the embedded `profile.schema.yaml`. |
| `schemas/module.schema.yaml` | **Adapt** (load-bearing). Set `unevaluatedProperties: false` at the root — **never** `additionalProperties: false` (block contributions arrive via the aggregator's `allOf` slot, which `additionalProperties` is not aware of). Reference `inlined/languages/{{language}}/items.schema.yaml` for `items[]`. |
| `schemas/enums.schema.yaml` | **Reuse + extend.** Copy `Identifier` / `ModuleRef` / `RequirementId` / `FeatureName` / `QualifiedPort` verbatim; add profile-specific vocabularies (e.g. a `LayerKind` enum). |
| `schemas/architecture.schema.yaml` | **Adapt.** `additionalProperties: false` at the top (fixed-shape file). Use the `CrateReference` `$ref` for dep entries. |
| `schemas/requirements.schema.yaml` | **Reuse.** Requirement + Source defs; include the `nature` field (default `"module"`, enum `[module, architecture, build, test]`). |
| `schemas/decisions.schema.yaml`, `schemas/open_questions.schema.yaml` | **Reuse** the shape (the triad is stable) — but rewrite the `$id` + title as above. |
| `reductions/project.yaml` | **Adapt.** `metaschema_version: "v1"`; maps this profile's TOML to universal `entities` + `relations`. |
| `domain.toml.template` | **Adapt.** `[domain] name = "{{project_name}}"`, `[validation]` + `[validation.rules]` (one entry per graph-check → severity), `[[project.graph_checks]]`. Ship `requirements_have_satisfiers`. |
| `spec.toml.template` | **Adapt.** Placeholders `{{project_name}}` / `{{profile}}` / `{{language}}`. |
| `architecture.toml.template`, `requirements.toml.template`, `decisions.toml.template`, `open_questions.toml.template` | **Reuse/adapt** — empty-or-example skeletons. |
| `codegen/<lang>/module.conventions.md` | **Author** (load-bearing prose). The uniform rules for this shape. **Filename:** the gate accepts any `*.conventions.md`; name it for *what it describes* (`module.conventions.md` for a module-shaped profile — what the shipped precedents use; `workspace.conventions.md` for rust-workspace), **not** the profile name. At least one `*.conventions.md` per supported language, or bootstrap fails. Role-not-framework naming (Step 1). |
| `README.md` | **Author.** Overview, when-to-use, file inventory, graph-check list. |

Invariants to hold:
- Schema `$id`s match `https://specdev.dev/profiles/<name>/<schema-base>.schema.yaml`.
- Only `module.schema.yaml` gets the `{{language}}` substitution; other schemas
  are byte-faithful (no stray `{{...}}` → `BOOTSTRAP-002`).
- The placeholder vocabulary is closed: `{{project_name}}`, `{{profile}}`,
  `{{language}}`.

## Step 3 — gate at authoring time (bootstrap → validate → gate registry → optional gen+compile)

A profile has no reference crate; its gate is a real bootstrap round-trip in a
throwaway scratch directory (use the session scratch dir, not the plugin tree):

1. **Bootstrap into a scratch project.** `<TARGET>` (the directory to bootstrap
   into) is a **required trailing positional** — omitting it exits 2:
   ```sh
   specdev-analyzer bootstrap --profile <name> --language <lang> --name smoke \
     --plugin-root <root> <scratch>/<name>-smoke
   cd <scratch>/<name>-smoke
   ```
   A missing required file or a malformed manifest fails here (`BOOTSTRAP-00x`).

2. **Author a minimal module.** Read the *bootstrapped* project's
   `specs/specdev/schemas/module.schema.yaml` `required:` list for the schema floor
   (e.g. `{name, visibility}` for a library shape). Note the `items[]` sub-schema is
   injected **at bootstrap** (the `{{language}}` `$ref` resolves to
   `inlined/languages/<lang>/items.schema.yaml` only after Step 3.1), so author the
   module *after* bootstrapping, not from the profile source. Author the smallest
   module the schema accepts (one requirement it `satisfies`), then:
   ```sh
   specdev-analyzer validate --no-color
   ```
   Zero **errors** confirms the schemas + reduction + graph-checks compose
   (warnings — e.g. `CONTEXT-100` "no context_files" — are acceptable, not gate
   failures).

3. **Check the gate registry landed and is not vacuous.** Bootstrap copies
   `profile.yaml` byte-faithfully, so the scratch project carries your `gates:`
   block. Prove it declares a module-scope gate:
   ```sh
   specdev-analyzer report gen-plan | head -1
   ```
   It must read `gates_declared: true`. `false` means the profile declares no
   `scope: module` gate, and every project bootstrapped from it will be refused
   at `gen --module <M> --tail`. Fix the profile, not the project.

4. **Run each declared gate for real** — a registry that parses is not a
   registry that runs. Against the minimal module of step 2:
   ```sh
   specdev-analyzer gate --label <label> --module <MinimalModule>   # scope: module
   specdev-analyzer gate --label <label>                            # scope: batch
   ```
   Exit `0` is a pass with rows written. The tool's own non-zero code means a
   step **ran and failed** — a real finding. Exit `2` means the gate **did not
   run** and **no row was written**: the registry declares no such label, a
   step's executable could not start, a `${root.*}` token resolved to nothing or
   to an empty value, or a module-scope gate was called with no `--module`. A
   wrong token or a wrong label is a profile bug; a missing tool is an
   environment gap. The message says which — read it before you call the gate
   green.

5. **Optional but recommended — gen + compile the minimal module** (the profile's
   smoke-fixture bar, mirroring `hello-firmware` / `hello-plugin`): run `/spec-gen`
   on the minimal module, which runs step 4's gates itself as its Step 8. A
   profile whose conventions emit non-compiling code is broken even though
   validate passed. For a cross-target Rust shape, `rustup target add <triple>`
   first (an env gap, not a prose bug).

6. **plugin-lint** for structural hygiene:
   ```sh
   specdev-analyzer plugin-lint --plugin <root>
   ```

Iterate until bootstrap + validate report **zero errors** (a standing warning like
`CONTEXT-100` is fine — do not loop trying to zero it), `gen-plan` reads
`gates_declared: true`, and every declared gate exits 0; surface any finding
verbatim. Delete the scratch project when done.

## Step 4 — report + next steps

Report the files written, the bootstrap + validate results, the `gates_declared`
verdict and each gate's label / scope / exit code, the gen+compile result (if
run), and any analyzer-work items you had to defer (novel manifest shape / novel
graph-check — Step 0.4). **Next:** `/spec-init --profile <name>` in a real project;
`/spec-design` modules against it; `/spec-gen`. To share, PR it upstream. **Live
smoke of a just-authored profile is next-session** if the tree only just changed
(bootstrap reads the plugin tree at invocation, but the skill loads at session
start) — say so rather than claiming an end-to-end pass this session.

## What this skill does NOT do

- **Author a block or bridge.** Framework-specific content is a block; wiring two
  blocks is a bridge. A profile names *roles* and shapes; it does not embed a
  framework.
- **Bootstrap the profile into a project.** Use `/spec-init --profile <name>`.
- **Add a novel build-manifest shape or graph-check primitive.** Those are
  analyzer changes (Step 0.4) — this skill authors profiles that reuse existing
  shapes + primitives. If the shape needs new analyzer machinery, say so and stop.
- **Ship a reference crate.** Profiles are uniform-prose; their authoring gate is
  bootstrap + validate + the declared `gates:` running for real + the
  gen-and-compile smoke, not a reference crate.
- **Multi-language conventions in one invocation.** One language's
  `*.conventions.md` per run.
- **Publish / register / version.** No registry or semver in v1.
