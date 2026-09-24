# Changelog

All notable changes to the specdev plugin are documented in this file. The
format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

This file starts at the first tagged release. Earlier versions were never
tagged, so there is no fixed commit to anchor a historical entry to — a
backfilled section would be a guess sitting beside commit-verified ones and
would read as equally solid. `git log` has the rest.

The plugin and `specdev-analyzer` version **independently**. They are not
meant to match. `contract_version` is the compatibility number between them,
and it is `1`.

## [Unreleased]

### Added

- **`/spec-decide` shows where a decision reaches** (Step 4.5, `report
  decisions --decision <id>`): a new decision is in no prompt until something
  links it, and on a supersede the skill compares the reach of older accepted
  decisions on the same subject, so stale text still landing in a module's
  prompt is found before codegen, not after.
- **`shared/min-analyzer-version`** names the `specdev-analyzer` version
  this plugin is released with — the one its install docs pin, which
  `scripts/check-analyzer-pin` asserts. The analyzer reads it and warns when
  it is older. The shared preamble tells a skill what to do with the warning:
  tell the user once, continue.

### Changed

- **`/spec-gen` reads the analyzer's new exit codes.** `derive` exit 3 (a
  target held back as `edited` / `conflict`) is carried to the report as an
  action item and the run continues to lineage recording; `gen --tail` exit 2
  (could not run) is reported as a tree or tool defect, not as a spec
  refusal. Needs the `specdev-analyzer` release that introduces those codes.

### Fixed

- **`pnpm-workspace.yaml` has one owner per project shape, and the docs now
  agree.** In a single `nextjs-app` app it is a paired file: written once,
  then the project's, never gen-recorded. Only at a TypeScript workspace root
  (`[workspace]` in `spec.toml`) does `manifest` write it and `/spec-gen`
  record it. `/spec-gen` and the profile's conventions doc used to disagree.
- **`/spec-validate` explains `REDUCE-001`:** a spec file did not parse, so
  no graph check ran, and a report without graph-check findings proves
  nothing until the next run.

### Added

- The README points at `specdev-analyzer report graph`, the module map
  grouped by the profile's declared layers with wrong-way dependencies in red.

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
