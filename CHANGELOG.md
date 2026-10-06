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
