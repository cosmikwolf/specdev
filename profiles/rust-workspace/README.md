# `rust-workspace` profile

A specdev profile for Cargo workspace shells. Owns the workspace root's
`Cargo.toml` (`[workspace]`, `[workspace.package]`,
`[workspace.dependencies]`); does NOT own per-member structure (modules,
requirements, features) — those live in each member's own `specs/` tree.

## When to use

Use `rust-workspace` when your project is a multi-crate Cargo workspace
where two or more members share `[package]` metadata or dependencies.
For a single-crate library, use `library-api` directly. For a
single-crate binary firmware project, use `firmware-layered`.

## Bootstrap flow

```bash
# Workspace shell (run at the workspace root):
specdev-analyzer bootstrap \
  --profile rust-workspace \
  --language rust \
  --name <project-name> \
  .

# Per member (run inside each member's directory):
mkdir -p crates/<member-name> && cd crates/<member-name>
specdev-analyzer bootstrap \
  --profile library-api \
  --language rust \
  --name <member-name> \
  .

# Then add the member to the workspace:
#   1. Edit the workspace's spec.toml: append [[workspace.members]]
#      with name = "<member-name>", path = "crates/<member-name>"
#   2. Edit the workspace's specs/architecture.toml: declare any
#      hoisted candidates under [[workspace_dependencies]] and any
#      shared package fields under [workspace_package]
```

## Composition with member profiles

The workspace profile carries no module / requirement / feature surface
— bidirectional minimality. Each member is a complete spec on its own;
extracting a member's `specs/specdev/` and bootstrapping it as a
standalone project must produce a valid library (or firmware) crate.

A member opts into workspace inheritance at two layers:

1. **Package fields:** the member omits `[project].version /
   .license / .repository` etc.; the cargo-toml emitter walks up to
   the workspace's `[workspace_package]` and emits
   `<field>.workspace = true`.
2. **Dependencies:** the member sets `source = "workspace"` on a
   `CrateReference`; the cargo-toml emitter renders `<dep> =
   { workspace = true, ... }`.

Both opt-ins are per-field / per-dep — a member can inherit `edition`
from the workspace while keeping its own `version`, or inherit a
registry crate from `[workspace.dependencies]` while keeping a
local-only dep inline.

## Triad model: requirements, decisions, open questions

Alongside `requirements.toml`, the profile carries two sibling
artifacts; together the three form the spec's triad:

1. **`requirements.toml` — the WHAT.** Intent and constraints the
   workspace must satisfy.
2. **`decisions.toml` — the HOW & WHY.** Each entry pins one
   implementation choice with `status = "accepted"` or
   `"superseded"`. Supersession keeps both bodies: the accepted
   successor lists `supersedes = ["DEC-M"]`, the superseded entry
   stays as reasoning trail. A decision may resolve questions via
   `closes_questions = [...]` (or the question-side `closed_by` /
   `default_via` — author one direction, not both).
3. **`open_questions.toml` — the KNOWN UNKNOWNS.** Each entry is
   `open`, `default_committed` (a decision commits a working
   default plus a one-line `verify` falsification plan), or
   `closed`.

Cross-references are typed: a requirement names the decisions that
pin its implementation via `decided_by = [...]`; decisions that
shape the workspace as a whole (member layout, MSRV policy, hoisted
dependency policy) go in `architecture.toml`'s
`project_decisions = [...]`. The workspace profile is the
coordinator tier — it has no module surface of its own, so there is
no `module.guided_by` here; that field lives in each member's spec.

### The codegen gate

An open question with `status = "open"` hard-blocks code generation:
`gen` aborts for any module reachable through the question's
`affects` list. `default_committed` passes the gate — the codegen
prompt inlines the committed placeholder plus its `verify` text.
`closed` passes — the prompt inlines the resolved answer through the
decision named in `closed_by`. No CLI flag bypasses the gate; the
in-spec escape is downgrading an open question to
`default_committed` with a working default. Use `/spec-question` for
that flow and `/spec-decide` to record decisions.

### Workspace scope is one-way

Workspace-level decisions implicitly apply to every member crate —
e.g. pinning the MSRV, or choosing `thiserror` for public error
types across all members. Member-crate specs reference only their
own crate-local decision ids, never workspace ids; that keeps each
member self-contained (extract a member's tree, bootstrap it
standalone, and its spec still validates).

Workspace open questions reach down the same way: at this tier
`affects` is optional, and a workspace question with no `affects`
implicitly affects every module in every member crate — an open
question about, say, where to split a crate boundary blocks `gen`
across the whole workspace until it is resolved or defaulted. Scope
a workspace question narrower by listing specific member requirement
ids in `affects`.

Workspace-tier decisions are auto-included in member-crate codegen
prompt walks (a decision's `text` and `consequences` are inlined),
so workspace decisions directly shape the code generated in members.

### Validation

`validate` checks the triad graph: supersession chains must be
acyclic and orphan-free (every superseded entry is pointed at by
exactly one accepted successor's `supersedes[]`), and every
`decided_by` / `project_decisions` / `closes_questions` /
`closed_by` / `default_via` reference must resolve. Gate-blocking
open questions surface as `validate` warnings before `gen` refuses
on them.

## Layout under `specdev/profiles/rust-workspace/`

```
spec.toml.template            # workspace marker + members list
architecture.toml.template    # [workspace_package] + [[workspace_dependencies]]
domain.toml.template          # member-name uniqueness + path uniqueness
requirements.toml.template    # rare workspace-level requirements
schemas/
  workspace.schema.yaml       # validates spec.toml's [workspace] table
  architecture.schema.yaml    # validates the workspace architecture.toml
reductions/
  project.yaml                # Member + WorkspaceDependency entities
codegen/
  rust/
    workspace.conventions.md  # cargo-toml emission rules
README.md                     # this file
```

No `module.schema.yaml` / `module.conventions.md` — workspaces have no
module surface; that's per-member.

## Why bootstrap drops ~240 files into your tree

Each `specdev-analyzer bootstrap` invocation inlines a complete copy of
the active profile's schemas, reductions, codegen prose, and language
tier under `specs/specdev/`. For a `rust-workspace` shell, that's
roughly 230–250 files: profile schemas + project reduction + the rust
language tier (`items.schema.yaml` + `items.scm`) + per-block schemas
that get inlined when you later run `specdev inline <block>`.

The workspace root and every member each get their own self-contained
inlined tree — that's not duplication, it's the "project owns its
content" invariant. Each member must be extractable and runnable as a
standalone project; sharing files across the workspace would violate
that. If you have N members, expect N+1 inlined trees of this size.
Subsequent `specdev inline` calls add files but won't multiply.

This is intentional and expected; the file count surprised the first
docs-first adopter (zfm dogfood F-008, 2026-05-04).
