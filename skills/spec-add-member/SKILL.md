---
name: spec-add-member
description: This skill should be used when the user asks to "adopt an existing crate as a workspace member", "add a member to a workspace", "promote a single-crate project to a workspace", "migrate a single-crate spec into a workspace", "make this crate the first member of a workspace", "fold an existing spec into crates/", or "move a validated crate under a rust-workspace root". Adopts a *validated* single-crate spec (library-api / firmware-layered / audio-plugin) into a `crates/<member>/` slot under a `rust-workspace` root, preserving requirement / decision / open-question ids by moving the tree wholesale. Orchestrates `git mv` + `specdev-analyzer bootstrap` + workspace wiring — no new analyzer surface. For a brand-new greenfield workspace (no existing crate to adopt), use /spec-init with `--profile rust-workspace` instead.
allowed-tools: Read, Write, Edit, Bash, Glob, Grep
argument-hint: [<existing-crate-dir>] [--into crates/<member-name>]
---

# /spec-add-member — Adopt an existing crate as a workspace member

Promote a **validated** single-crate specdev project into a member of a `rust-workspace` root. This skill mechanizes the manual recipe documented in `${CLAUDE_PLUGIN_ROOT}/profiles/rust-workspace/README.md` (§ "Adopting an existing single-crate project as a member"), so that ids stay stable **by construction** rather than by hand-verification.

**Why it's safe to move the whole tree.** A specdev crate is partitioned so the move is clean: the id-bearing baseline (`spec.toml`, `specs/requirements.toml`, `specs/decisions.toml`, `specs/open_questions.toml`, `specs/architecture.toml`, `specs/modules/`) and the crate's own machinery (`specs/specdev/`) all relocate together, and every path resolves relative to the member's own `spec.toml`. So a moved member validates standalone (the bidirectional-minimality invariant). `git mv` is a byte-preserving rename — requirement / decision / open-question ids come along verbatim.

**The ordering trap this skill exists to prevent.** `specdev-analyzer bootstrap` refuses a target whose `specs/specdev/` already exists (`BOOTSTRAP-001`). So when the workspace root is the *same directory* as the crate being adopted, the crate's tree must move **out** of the root **before** the workspace shell is bootstrapped there. Doing it in the wrong order is the exact friction that motivated this skill.

If `specdev-analyzer` is not on PATH, tell the user to install it (`cargo install --git https://github.com/cosmikwolf/specdev-analyzer`) and stop. If it IS on PATH but reports `MANIFEST-002`, or a `MANIFEST-001` naming a file under the plugin tree, then the analyzer is older than the plugin — not a problem with any project file. Tell the user to re-run the install command with `--force` (or `cargo install --path <checkout> --force` from a local checkout), then retry. Do not attempt to move or wire files without the analyzer.

---

## Step 0 — session env preamble

Run the export from `${CLAUDE_PLUGIN_ROOT}/shared/plugin-root-preamble.md` before any analyzer invocation:

```sh
export SPECDEV_PLUGIN_ROOT="${CLAUDE_PLUGIN_ROOT}"
```

`CLAUDE_PLUGIN_ROOT` is set in this skill body but is **not** inherited by Bash subshells; exporting `SPECDEV_PLUGIN_ROOT` once makes plugin-root resolution work for the rest of the session.

## Step 0.5 — resolve scenario

There are two shapes. Detect which one applies before touching any file:

- **A — Promote in place (the common case).** The existing single-crate project's own root becomes the new workspace root. Its `spec.toml` has **no** `[workspace]` table. This is the reported/verified path: a validated single-crate project you now want to grow into a multi-crate workspace, with the original crate as the first member.
- **B — Add to an existing workspace.** A `rust-workspace` root already exists (a `spec.toml` with a `[workspace]` table), and you are folding a *separate* existing crate into it as a new member. The root is already bootstrapped, so the root-bootstrap step (Step 4) is skipped.

Identify:
1. **Source crate dir** — the validated single-crate project to adopt (from `$ARGUMENTS`, or ask). Must contain `spec.toml` + `specs/specdev/`.
2. **Member slot** — `crates/<member-name>` relative to the workspace root (from `--into`, or derive `<member-name>` from the source crate's `[project].name`; confirm with the user).
3. **Workspace root** — Scenario A: the source crate's own directory. Scenario B: the existing workspace root dir (ask if ambiguous).

State which scenario you detected and the three resolved paths back to the user before proceeding.

## Step 1 — preflight

Refuse to proceed unless all hold; surface the specific failure and stop otherwise:

1. **Source is a real specdev crate** — `<source>/spec.toml` and `<source>/specs/specdev/` both exist.
2. **Source validates clean** — run `specdev-analyzer validate --project <source> --no-color` and require **0 errors**. Adopting a broken spec just relocates the breakage; make the user fix validate first. (Warnings are fine.)
3. **Source is single-crate** (Scenario A) — its `spec.toml` has no `[workspace]` table. If it *does*, it is already a workspace root; this skill adopts *into* a workspace, not workspace-into-workspace — stop and explain.
4. **Member slot is free** — `<workspace-root>/crates/<member-name>/` does not already exist. If it does, stop; never overwrite an existing member.
5. **Git awareness** — check whether the workspace root is inside a git work tree (`git -C <workspace-root> rev-parse --is-inside-work-tree`). If yes, use `git mv` in Step 3 (preserves rename history). If no, use plain `mv` — ids are still preserved byte-for-byte; only git rename-tracking is lost. Tell the user which you'll use.

## Step 2 — confirm the move plan

Show the user the exact operations before running them — this is a tree restructure, so make it reviewable:

- Scenario A: "`<source>` becomes the workspace root; its crate tree moves to `crates/<member-name>/`, then a `rust-workspace` shell is bootstrapped at the root."
- Scenario B: "`<source>`'s crate tree moves to `<workspace-root>/crates/<member-name>/` and is registered as a member of the existing workspace."

Get an explicit go-ahead before Step 3. Nothing is written until then.

## Step 3 — move the crate tree into the member slot

This is the ordering-critical step: it relocates the baseline **and** frees the root of `specs/specdev/` (Scenario A), which Step 4 requires.

From the workspace root (`mv` variant shown; substitute `git mv` when Step 1.5 said git):

```sh
mkdir -p crates/<member-name>
git mv spec.toml crates/<member-name>/spec.toml
git mv specs     crates/<member-name>/specs
```

For **Scenario B**, the source crate is a *separate* directory, so move from there into the workspace:

```sh
mkdir -p <workspace-root>/crates/<member-name>
git -C <source> mv spec.toml <workspace-root>/crates/<member-name>/spec.toml   # or mv across trees
git -C <source> mv specs     <workspace-root>/crates/<member-name>/specs
```

Do **not** bootstrap the member fresh and copy files over it — that is the longer, error-prone path the wholesale move replaces. Move the tree as-is; it already carries the correct machinery for its profile.

## Step 4 — bootstrap the workspace shell (Scenario A only)

The root is now clear of `specs/specdev/`. Bootstrap the `rust-workspace` shell in place:

```sh
specdev-analyzer bootstrap \
  --profile rust-workspace \
  --language rust \
  --name <workspace-project-name> \
  .
```

If this returns `BOOTSTRAP-001` ("target tree already exists"), Step 3 did not fully clear the root — verify `./specs/specdev/` is gone and `./spec.toml` moved into the member slot, then retry. Do not hand-write workspace files to work around it.

**Scenario B:** skip this step — the workspace root is already bootstrapped.

## Step 5 — populate missing manifests if pre-manifest-era

If the adopted crate was bootstrapped before the manifest formats shipped, it may be missing `profile.yaml` / per-block `deps.yaml` / per-bridge `contract.yaml`. Populate them **without disturbing ids**:

```sh
specdev-analyzer migrate --project crates/<member-name>
```

`migrate` byte-copies only the *missing* manifests from plugin HEAD; it is a true no-op when they are already present, and it freezes `origin` / `inlined_at` — the baseline (requirements/decisions/open_questions and their ids) is never touched. Run `migrate --project crates/<member-name> --dry-run` first if you want to see the plan. Skip entirely if the crate was bootstrapped recently against a manifest-era plugin. Also fix `crates/<member-name>/spec.toml` `[project].name` here if the member name changed during adoption.

## Step 6 — register the member in the workspace

Edit the **workspace root's** `spec.toml`: replace the template `placeholder-member` entry (or append, for Scenario B) with the real member:

```toml
[[workspace.members]]
name = "<member-name>"   # MUST match crates/<member-name>/spec.toml [project].name
path = "crates/<member-name>"
```

Then, in the workspace root's `specs/architecture.toml`, wire any cross-crate concerns the member needs:
- **Shared package fields** — set `edition` / `version` / `license` / `repository` / `rust-version` under `[workspace_package]`; the member inherits each by *omitting* that field from its own `[project]` (the cargo-toml emitter renders `<field>.workspace = true`).
- **Hoisted dependencies** — declare shared deps (including a path dep on a sibling member) under `[[workspace_dependencies]]`; the member opts in per-dep by setting `source = "workspace"` on its `CrateReference`.

Inheritance is opt-in and per-field/per-dep — a freshly-adopted member validates as-is with no inheritance edits. Only wire what the user actually wants shared.

## Step 7 — verify

Validate the member tree **and** the workspace root:

```sh
specdev-analyzer validate --project crates/<member-name> --no-color
specdev-analyzer validate --project . --no-color
```

Both must report **0 errors**. Common failures:
- Member `SCHEMA` errors after `migrate` — a field the refresh tightened; fix the offending field, don't delete it.
- Root error that a member `name` doesn't match — the `[[workspace.members]].name` must equal the member's `[project].name`; reconcile them.

Re-run until both are clean.

## Step 8 — report

Summarize:
- Scenario (A promote-in-place / B add-to-existing) and the three resolved paths.
- The move performed (`git mv` vs `mv`), and that ids were preserved.
- Whether `migrate` ran.
- The member registration + any `[workspace_package]` / `[[workspace_dependencies]]` wiring applied.
- Both validate results (member + root).
- **Next steps**: add further members with `/spec-init --non-interactive --profile <p> ... crates/<next>` then re-run this skill's registration step, or `/spec-inline` blocks into the member as usual.

## What this skill does NOT do

- **Adopt a broken spec.** Step 1 requires the source to validate clean first; fix validate before adopting.
- **Merge two workspaces.** The source must be a single crate (Scenario A) or a standalone crate (Scenario B), not itself a workspace root.
- **Author or edit module specs.** Modules move verbatim with the tree; changing them is `/spec-edit`'s job.
- **Decide what to hoist.** It surfaces `[workspace_package]` / `[[workspace_dependencies]]` as options and applies only what the user chooses; it does not auto-hoist shared deps.
- **Rewrite ids.** The whole point is id stability — the move is byte-preserving. If the user wants renamed ids, that is a separate manual edit after adoption.

## Additional resources

- `${CLAUDE_PLUGIN_ROOT}/profiles/rust-workspace/README.md` — the canonical manual recipe this skill mechanizes (§ "Adopting an existing single-crate project as a member") plus the composition model (`[workspace_package]` inheritance, `source = "workspace"` deps).
- `${CLAUDE_PLUGIN_ROOT}/shared/plugin-root-preamble.md` — the Step 0 env preamble rationale.
