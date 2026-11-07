# Changelog

All notable changes to the specdev plugin are documented in this file. The
format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

This file starts at the first tagged release. Earlier versions were never
tagged, so there is no fixed commit to anchor a historical entry to — a
backfilled section would be a guess sitting beside commit-verified ones and
would read as equally solid. `git log` has the rest.

The plugin and `specdev-analyzer` version **independently**. They are not
meant to match. `contract_version` is the compatibility number between them,
and it is `2`.

## [Unreleased]

### Changed

- **License: Apache-2.0** (was MIT). `LICENSE` holds the Apache-2.0 text and `NOTICE` the copyright line;
  `LICENSE.md` is gone.
- **Install text names the public `cosmikwolf` repositories.** `README.md`, `scripts/resolve-context` and
  `shared/analyzer-install.md` no longer describe private repositories or an `<org>` placeholder; the install guide
  still reads a fork's org off the plugin's remote.

## [0.6.0] — 2026-10-27

### Added

- **Public spec export** (#283): `[export]` in `spec.toml` and `specdev-analyzer export` publish the decisions,
  requirements and open questions as Markdown beside the code; the decision field `record` (process notes, in every
  profile) is never exported, and `/spec-decide` writes approvals there. `/spec-validate` and the README describe
  `export_current` and the `cited_ids_*` rules. The install doc gives the prebuilt binaries (#284).

- **`/spec-reverse` for TypeScript, Dart and Python** (#273). The skill takes its field checklist from the
  project's module schema instead of a Rust list, reads `depends_on` from `report imports`, and fixes `[paths]
  source` and `toolchain` from the tree before the first module. It skips generated files. `python-library`,
  `python-ffi`, `ts-library`, `dart-package` and `flutter-app` declare `reverse.vocabulary: profile`, so their
  pre-flight passes with no block inlined. The `firmware-layered` inference rules moved to its README (`## Reverse`).
- **`/spec-reverse` proposes requirements** (#273): from test names and doc comments, with `verified_by` from
  `report tests`, `test_pending` when untested, and a `derived` source that marks them as read from the code. The
  user confirms each one; then items carry `satisfies`.

### Fixed

- **Bootstrap writes the Python import name in paths** (#274): new placeholder `{{project_identifier}}` (the
  project name with each `-` as `_`); `python-library` and `python-ffi` use it for `[paths] source` and the native
  module, so `pattern-analysis` gives `src/pattern_analysis`.
- **Schema text names the per-file layout** (#277): requirement schemas, `RequirementId` and decisions' `source`
  say `specs/requirements/` and `specs/sources/`, not the 1.x `specs/requirements.toml`. `library-api` README has a
  `## Reverse` section (`visibility`, the crate root).
- **A requirement's `source` id must name a `[[source]]` entry** (#272): every profile with requirements has the
  relation `requirement_derived_from_source`, so an unknown id is `REDUCE_UNRESOLVED`.

## [0.5.0] — 2026-10-23

### Added

- **Code apart from the specs** (`[paths] code`, needs specdev-analyzer with the code root): the project rules say
  which paths are relative to the code root, the commit order (code repository first, then the spec change with the
  submodule bump), and which CI runs which gate script. `/spec-audit` opens the source through `context`'s
  `files.source` and reads printed code paths through `files.code_root`; `/spec-validate` explains `LAYOUT-007`,
  `code_root_commit`, `code_root_clean` and `code_root_recorded`; `module-designer` and `/spec-reverse` write
  `source_file` relative to the code root.

## [0.4.1] — 2026-10-22

### Added

- **`ts-library` profile** (#269): a TypeScript package published to npm — ESM, `src/` built to `dist/` with
  declarations. Gates per package manager under `toolchains:` (pnpm, npm): frozen install, `tsc --noEmit`,
  `vitest run`, `tsc -p tsconfig.build.json`. Paired `tsconfig.json` (type-check), `tsconfig.build.json` (emit),
  `.gitignore`, CI.
- **`dart-package` profile** (#269): a pure Dart package for pub.dev. Gates `dart pub get`, `dart analyze
  --fatal-infos`, `dart format --set-exit-if-changed`, `dart test` — the analysis and formatting bar pub.dev scores.
  Paired `analysis_options.yaml` (recommended lints, strict modes), `.gitignore` (`pubspec.lock` not committed).

### Fixed

- **`/spec-create-profile` reads gate exit code 2 correctly**: a tool can exit 2 on its own findings
  (`dart analyze`), not only when a step could not start.

## [0.4.0] — 2026-10-22

### Added

- **`python-ffi` profile**: a Python package built around a Rust extension (PyO3, maturin). The Rust crate is a
  second specdev project (`library-api` + the `pyo3` block). `specs/architecture.toml` `[native] module` names the
  extension; a `stubs` gate (`python -m mypy.stubtest`) holds the `.pyi` stub to the built module, before pytest,
  pyright and ruff. Gates for uv, poetry and pip. `/spec-init` offers it.

- **`pyo3` block**: a `[pyo3]` table naming the extension modules, classes, functions and exceptions a Rust crate
  exposes to Python (Rust names, which are also the Python names); a rule pack with one code check per entry kind
  (`pack_entity_in_code`), measured on code that compiles against PyO3; guidance for the declarative module form,
  names, free-threading, errors and the build.

- **`python-library` profile**: an importable Python package (`src/` layout); modules carry optional `items[]` and
  no visibility field (the `_` prefix is the visibility). Gates per tool under `toolchains:` — uv, poetry and pip —
  each running pytest, pyright and `ruff check` through `python -m`; a CI template for uv. `/spec-init` asks for the
  tool; `/spec-create-profile` lists Python and the `toolchains:` precedent, and its templates say contract 3.

- **Python language tier** (`languages/python/`): items schema and query (`fn`, `class`, `const`, `type`; methods and
  class constants as `Class.name`; privacy by the `_` prefix; `.pyi` stubs too), pytest test discovery, import
  queries (relative, absolute, `from . import <submodule>`, `if TYPE_CHECKING:` not a dependency), and `build.md`
  with uv, poetry and pip commands side by side. No Python profile yet.
- **`toolchains:` in `profile.yaml`** for a language with more than one common tool: each tool's lockfiles and
  gates. A project declares its tool in `spec.toml` (`[project] toolchain`). `/spec-create-profile` documents it.

### Changed

- **Profile contract 3.** Every `profile.yaml` declares `contract_version: 3` (the `python` language and the
  `toolchains` key). Needs specdev-analyzer with contract 3; an older analyzer reports `MANIFEST-002`.

### Removed

- **Items carry no signature** (#247). `signature_hint` is gone from the Rust, TypeScript and Dart items schemas;
  a value left in a project fails `validate` with "`signature_hint` is not allowed: it was removed …", once
  `/spec-drift` brings the schema forward. A signature that really is the contract (FFI, a macro's calling form)
  goes in the item's `description`, in prose. No analyzer ever read a source signature, so every hint reported
  drift. `/spec-sync-items` loses the `update-spec` action, which existed only for that drift: four actions remain
  (`skip`, `add-to-spec`, `remove-from-spec`, `rename`).

### Added

- **`decision_reference_superseded` (warning)** (#251). A module's `guided_by` or a requirement's `decided_by` that
  names a superseded decision is stale: `report decisions` drops it, so the module loses that guidance. Six profile
  templates get the check; an existing project adds the two blocks to its own `specs/specdev/domain.toml`.
- **`write-check` takes a directory target** (#255): it covers every new or changed file under it.
- **Authoring rules: trait members and macro-generated items are not items** (#254). A trait declaration is one
  item; `items diff` expands no macro; a `macro_rules!` is an item only with `#[macro_export]`.
- **A decision that an accepted decision supersedes must be `superseded`** (#246). Every profile's
  `domain.toml.template` gets a second `decision_supersedes_chain_consistency` check: two accepted, contradicting
  decisions no longer pass `validate`. An existing project gets it by adding the block to its own
  `specs/specdev/domain.toml`.

### Fixed

- **`/spec-decide` says `guided_by` and `decided_by` stay inside one tier** (#250), and that a workspace decision's
  reach shows in a member's report, not the root's.
- **`/spec-audit` no longer says `check` misses a qualified path** such as `crate::led::X`: #238 fixed that in
  analyzer v0.3.4; only a path inside a macro's arguments is unseen.
- **Temporary files have no literal `XXXXXX` on macOS** (#239). `/spec-design`, `/spec-edit` and the write check
  use `mktemp "${TMPDIR:-/tmp}/<name>.XXXXXX"`; `mktemp -t` kept the template in the path and one agent read it as
  unfilled.
- **The module-designer brief says an impl item needs `target_type`** (#240), with one example.
- **`implementation_unblocked` on an `impl_time` question is expected once** (#241): `/spec-question` and
  `/spec-validate` say so, so it does not read as a failure.
- **The requirements README template puts sources in `specs/sources/`** (#242). It said `specs/requirements/`,
  which the analyzer rejects (ENTRY-FILE-001).
- **The `parameter_bindings` schema describes a claim about the code** (#248), not tokens a compiler checks.
- **`/spec-reverse` single-module mode runs without `architecture.toml`** (#249) unless the profile's module schema
  needs it (`firmware-layered`: `layer`).
- **`context_files` accepts a binary file** (#243): the module schemas say an image or a PDF is allowed.

## [0.3.5] — 2026-10-15

### Fixed

- **The Rust imports query captures qualified paths** (#238): a `crate::`, `super::` or `self::` path in an
  expression, a pattern or a type, not only `use` lines, so `internal_edges_declared` reports a module that uses
  another without naming it in `depends_on`. The analyzer embeds this query, so the fix takes effect with the
  analyzer release that carries it.

## [0.3.4] — 2026-10-07

### Added

- **`/spec-audit <Module>`**: an agent reads one module's code against the claims no check reads — each
  requirement's text (and whether its tests exercise it), the descriptions, `isr_safe`, `blocking` and `metadata.*` —
  and reports `holds`, `contradicted` or `not shown` for each, with `path:line` evidence. It reports in chat and
  edits nothing. No analyzer change.

## [0.3.3] — 2026-10-06

### Added

- **`/spec-validate` names the requirement-tree rule, `report requirements`, and the workspace id rules** (#234,
  #235, #236): a parent requirement is covered by its children; a member id may not repeat a workspace id; a
  workspace open question must name a requirement id that exactly one member defines. Needs specdev-analyzer 0.3.3.

## [0.3.2] — 2026-10-06

### Added

- **Retire a requirement instead of deleting it** (#229): `/spec-decide` Step 4.6 moves it to
  `specs/requirements/retired/` with `retired_by`; the six requirements schemas document the field; `/spec-design`
  reads the top level of `specs/requirements/` only; the project rules say a moot requirement is retired. Needs
  specdev-analyzer 0.3.2.
- **The SessionStart hook relays the analyzer-version warning** (#226): at a project root it runs
  `specdev-analyzer --version` and passes on its `ANALYZER-VERSION-` line, or says the analyzer is not on `PATH`, so
  an old analyzer is reported even when `check` runs outside a skill.
- **`/spec-validate` names `check --members`, `requirement_size` and the retired-requirement rule** (#230, #232,
  #229).

### Changed

- **The project rules name the entry-level `planned` flag** for a module whose code exists but some of whose
  rule-pack entries do not (#231); `/spec-validate` says to put the `migrate` report in the migration's commit
  message (#227).

## [0.3.1] — 2026-10-06

### Fixed

- **Rust test discovery sees tests made by a local `macro_rules!` macro** (#223): `languages/rust/tests.scm`.
- **Rust imports inside `#[cfg(test)]` code, `#[cfg(all(…, test, …))]` code or a `#[test]` fn are test-only**
  (#224): `languages/rust/imports.scm`. Needs specdev-analyzer 0.3.1.

## [0.3.0] — 2026-10-05

**Checked intent: specdev no longer generates code.** Specs state intent — what each module is for, what it may
depend on, which requirements it satisfies, which decisions guide it — and `specdev-analyzer check` checks that intent
against the code you and the agent write. Needs specdev-analyzer 0.3.0. Existing projects move with
`specdev-analyzer migrate` (format `spec_version` 2.0); commit or stash first, then run it with `--dry-run`.

### Removed

- `/spec-gen`, every `codegen/` directory (codegen prose, `*.conventions.md`, `deps.yaml`), the stub and shim
  templates, the reference crates and their `reference_cases.yaml`, and `languages/*/syntax.md`.
- The audio-plugin `[plugin]` / `[audio]` architecture fields and `runtime.panic_handler`, and the `spec.toml`
  `crate_shape` / `crate_type` keys: the code and `Cargo.toml` hold them (`migrate` lists each removed value).

### Added

- **Implementation guidance:** every profile, block and bridge ships a short `guidance.md` (rules only), inlined to
  `specs/specdev/guidance/` and listed by `specdev-analyzer context --module` for whoever edits the code.
- **Rule packs:** a block or bridge can ship its checks as data (`checks.toml` plus tree-sitter queries). Shipped:
  `rtic` (tasks and resources against the code), `mk20dx` (no pin claimed twice), `egui+nih-plug` (a widget fits its
  parameter and sits in a controls region).
- **Format 2.0:** one file per decision, requirement, source and open question; module `planned` and `source_file`;
  requirement `verified_by` and `test_pending`; open-question `hard_gate`.
- **The gate script:** each profile's `gates:` render into `scripts/specdev-gates.sh` (`check`, then every gate), and
  a CI workflow template is written once at bootstrap.
- Test discovery (`tests.scm`) and import edges (`imports.scm`) for Rust, TypeScript, Dart, C and C++.
- `/spec-create-bridge` Step 0.5 test 0: is the binding an edge between two entities at all.

### Changed

- The skills are rewritten for checked intent: `/spec-design` writes `planned = true` for a module with no code;
  the skills read `context --module` and `report decisions --module` before code edits and want a clean work tree
  before `migrate`; `/spec-validate` runs `check`.
- `firmware-layered`, `library-api` and `rust-app` guidance: a module filed away from `src/<name>.rs` or
  `src/<name>/mod.rs` sets `source_file`.
- `rust-workspace`: the root declares no gate; its script runs each member's script.

### Fixed

- **Cursor: the plugin-root preamble handles an empty `CLAUDE_PLUGIN_ROOT`**
  (#220). Cursor sets that variable for plugin hooks only, so the agent's
  shell copied an empty value into `SPECDEV_PLUGIN_ROOT`. The preamble now
  says to take the directory above `skills/` from the `SKILL.md` path.

## [0.2.2] — 2026-09-24

### Added

- **nextjs-app: the `app` module's context cone lists every layout, page and
  boundary** (#208). A `renders` relation, built from the roles and extending
  `contains`, hangs them off the composition root; `context --module <app>`
  shows them under `contains`. Needs `specdev-analyzer` with `match:`
  relations. Existing projects get it through `migrate --refresh`.
- **nextjs-app: `single_app_module`** (warning, #208) — exactly one module
  has `role = "app"`. With none the cone above is empty; with two every page
  hangs off both.
- **`/spec-gen` Step 6.1 checks imports against declared dependencies**
  (`report imports`, #207). Each undeclared package is an action item in the
  report: an `external_deps[]` entry on that module. The run does not stop.
- **`imports.scm`** for the TypeScript and Dart language tiers (#207).
- **Skills check their write against the disk** (`scripts/write-check`,
  `shared/write-check.md`). `/spec-design`, `/spec-edit`, `/spec-decide` and
  `/spec-question` snapshot content hashes before the write and compare
  after: a target that did not change is `not-written` (the skill stops), any
  other file that changed is `unexpected` (the skill warns), and the report
  is built from the file read back. Runs under macOS's bash 3.2.
- **Guidance on where a list of domain values lives**
  (`shared/vocabulary-placement.md`, #211): a closed list in the spec or a
  table of its own, what each costs, and the test between them — does a
  non-engineer need to add a value without a deploy? `/spec-design` and
  `/spec-decide` point at it.
- **Prisma: a call into another module inside a transaction** (#205). The
  callee takes `Prisma.TransactionClient` and the caller passes `tx`; a spec
  that asks for such a call needs the callee's item to name that parameter, or
  no correct code exists. Compiled by the block's reference package.
- **`/spec-decide` shows where a decision reaches** (Step 4.5, `report
  decisions --decision <id>`): a new decision is in no prompt until something
  links it, and on a supersede the skill compares the reach of older accepted
  decisions on the same subject, so stale text still landing in a module's
  prompt is found before codegen, not after.
- **`shared/min-analyzer-version`** (#179) names the `specdev-analyzer`
  version this plugin is released with — the one its install docs pin, which
  `scripts/check-analyzer-pin` asserts. The analyzer reads it and warns when
  it is older. The shared preamble tells a skill what to do with the warning:
  tell the user once, continue.
- **Every profile checks that decision ids are unique** (`decision_ids_unique`,
  #197). Two `[[decision]]` entries with one id both reached the graph and
  `validate` exited 0; found in the field as two agents in separate worktrees
  both allocating `DEC-057`. The decisions schemas also stop claiming that
  `unique_qualified_names` catches a decision and a requirement sharing an id —
  it is Module-scoped.
- The README points at `specdev-analyzer report graph` (#203), the module map
  grouped by the profile's declared layers with wrong-way dependencies in red.

### Changed

- **`/spec-gen` reads the analyzer's new exit codes** (#160, #165). `derive`
  exit 3 (a target held back as `edited` / `conflict`) is carried to the
  report as an action item and the run continues to lineage recording;
  `gen --tail` exit 2 (could not run) is reported as a tree or tool defect,
  not as a spec refusal. Needs the `specdev-analyzer` release that introduces
  those codes.
- **`module-designer` holds no shell** (#199). Its read-only contract was
  prose: it held `Bash`, and in the field it wrote a spec file its Notes never
  mentioned. It now has Read, Grep and Glob; `/spec-design` and `/spec-edit`
  write the context cone with `context --output` before dispatch and pass the
  file.
- `/spec-edit` and `module-designer` read the new `contains` /
  `contained_by` context buckets; the impact matrix has a row for editing a
  module that contains others (#208).
- `/spec-validate` explains `PLUGIN-AHEAD-001` (the plugin holds newer copies
  of inlined or profile files; run `/spec-drift`, #210) and `METASCHEMA-006`.
- CI: `reference-crates` builds every reference crate into one target dir
  (#194, about −25% on that step), and `reference-crates` and `frontmatter`
  run once per PR push and once per merge to `main`, not twice.

### Fixed

- **A single-module `/spec-gen` checks its own plan row** (#206). It skipped
  Step 3.5, so an `edited` or `unrecorded` file was regenerated without being
  named. It now stops on both, and `edited` offers "regenerate and discard"
  for an edit that only repaired broken output.
- **`unrecorded` is defined by the module's source path** (#216) —
  `source_file`, else the convention path — matching the analyzer's
  `gen-plan`, so a committed file at the convention path is never read as
  `new` and overwritten.
- **`pnpm-workspace.yaml` has one owner per project shape, and the docs now
  agree** (#214). In a single `nextjs-app` app it is a paired file: written
  once, then the project's, never gen-recorded. Only at a TypeScript workspace
  root (`[workspace]` in `spec.toml`) does `manifest` write it and `/spec-gen`
  record it. `/spec-gen` and the profile's conventions doc used to disagree.
- **`/spec-validate` explains `REDUCE-001`** (#196): a spec file did not
  parse, so no graph check ran, and a report without graph-check findings
  proves nothing until the next run.

## [0.2.1] — 2026-09-15

First tagged release. Requires `specdev-analyzer` **v0.2.0** or later.

### Fixed

- **Seven of seventeen skills were silently dropped from a session.**
  `spec-add-member`, `spec-create-profile`, `spec-init`, `spec-inline-bridge`,
  `spec-question`, `spec-reverse` and `spec-sync-items` had YAML frontmatter
  that failed to parse — an unquoted scalar containing `: ` or a leading `[`.
  Claude Code drops a skill whose frontmatter does not load, with no warning
  shown, so the commands were simply absent. All seven are fixed and a CI
  check now blocks the class.
- The session-start hook no longer injects the project rules twice per
  session, and no longer applies an enclosing repository's rules to a session
  started inside a nested project.
- `rust-workspace` no longer halts on the second `/spec-gen`; the workspace
  path was missing `--accept-edits`.
- `INLINE-001` and `INLINE-002` were documented **backwards** in three skills,
  and `INLINE-003` described the wrong condition.
- `/spec-validate`'s two output examples showed a format the analyzer never
  prints.
- The README's command table listed 12 commands; 17 ship. `/spec-config`,
  `/spec-add-member`, `/spec-create-block`, `/spec-create-bridge` and
  `/spec-create-profile` were missing.
- `/spec-gen` stated a false invariant about gate exit 2, and did not mention
  that analyzer progress lines go to stderr — check the exit code, not the
  streams.
- Two `/spec-reverse` reference files pointed at a path that exists in no
  project, and two others disagreed about subagent fan-out.
- `agents/README.md` was being loaded as an agent named "README".

### Added

- **`/spec-config`** — writes `.claude/rules/specdev.md` and
  `specdev-feedback.md`, each only when absent. Never overwrites, never
  merges.
- A **`SessionStart` hook** that tells a session it is in a spec-managed
  project, so agents run the `/spec-*` skills instead of hand-writing specs.
- `/spec-inline` now reports the manual user actions a block's README calls
  out — a CLI to install, a generator to run — so you do not miss a step
  before the next gate.
- CI checks that every skill and agent frontmatter parses and that a skill's
  `name` matches its directory.

### Changed

- **The install instructions now pin a version.** All three sites name
  `--tag v0.2.0` for the analyzer instead of tracking a moving branch, and
  `scripts/check-analyzer-pin` keeps them honest against the analyzer beside
  them. A stale pin is a silent failure: the install succeeds and you get the
  wrong tool.
- The `mk20dx` block takes `mk20dx-hal` 0.2.0 and `fugit` 0.6 (was 0.1.1 and
  0.3); the HAL's API moved with its dependency.
- The `next-auth` block and the `next-auth+prisma` bridge no longer name a
  specific profile when discussing middleware. They now tell you to read your
  own profile's role list, so both units stay correct under a profile that
  grows a middleware role.
- The README no longer claims the plugin and the analyzer report the same
  version. They do not, by design.

### Known issues — pre-release dependencies

Three shipped blocks pin a version that is not a stable release. None is an
accident; each is recorded here because a version number is a promise to a
reader.

- **`blocks/next-auth`** pins npm `next-auth` at exactly `5.0.0-beta.32`. The
  `latest` tag (`4.24.15`) is a major behind the API this block is built on.
- **`blocks/prisma`** pins `@prisma/client` at `^7.10.0`, which is stable — but
  the companion `prisma` CLI, which **you** install, resolves to a **release
  candidate** (`8.0.0-rc.13`, measured 2026-09-10) from its `latest` tag. Pin
  the CLI to a matching `7.x` by hand.
- **`blocks/itm`** pins the crate `itm` at `0.9.0-rc.1`. This is the newest
  version ever published (2022) and what the wider `probe-rs` ecosystem uses;
  there is no stable successor to move to.

### Also

- The `nextjs-app` profile's two gates (`fast`, `next-build`) must run
  together, in declared order, in one job — `fast` runs the `pnpm install`
  that `next-build` needs. Nothing enforces that if you wire `next-build`
  alone into your own CI.
