# spec-gen — workspace mode (W1–W6)

Triggered when the project root's `spec.toml [project].profile == "rust-workspace"`. Walks every member crate in topological dep-order and emits a single workspace-level summary at the end. Per **G-001** (codegen-pass dogfood 2026-05-01).

If the user invoked `/spec-gen` from inside a member crate of a workspace (member's `spec.toml [project].profile` is e.g. `library-api`, but a parent dir has a `spec.toml` declaring `[workspace]`), treat the invocation as single-package on that member. The workspace-level walk is only triggered when invoked at the workspace root itself.

## W1 — read the workspace manifest

Read `spec.toml [workspace]` and `[[workspace.members]]`. Each entry has a `path` (relative to workspace root) and a `name` (the crate name). If `[workspace.members]` is missing or empty, stop with an error: a rust-workspace project with no members is malformed.

## W2 — gate on workspace-root validation

```sh
specdev-analyzer validate --no-color
```

Run from the workspace root. The rust-workspace profile validates the workspace manifest itself (members exist, paths point at directories with `spec.toml`, no duplicate names). If it errors, stop and tell the user to run `/spec-validate` at the workspace root first.

## W3 — build the dep graph

For each member, parse the member's `spec.toml`, `architecture.toml` (if present), and every `specs/modules/*.toml`. Collect `external_deps[]` entries. An edge `member_a → member_b` exists when:

- `external_deps[].source.path` resolves (relative to member_a's dir) to member_b's directory, OR
- `external_deps[].source == "workspace"` AND the dep's `crate` field matches member_b's `name` AND the workspace's `[workspace.dependencies]` entry for that crate uses a path-dep into member_b.

Topologically sort. If a cycle is found, stop with an error pointing at the cycle (Cargo would refuse to build it anyway, but surfacing it here saves a round-trip).

## W4 — per-member walk

For each member in topological order:

1. `cd <workspace>/<member.path>`.
2. Run Steps 2–8 of single-package mode against that member directory. Each member has its own `specs/specdev/`, its own profile (typically `library-api` or `firmware-layered` or `audio-plugin`), and its own `Cargo.toml` output. Use `--project <member-dir>` if invoking the analyzer from elsewhere. The Step 4 preamble is **per member** — assemble it once for that member's module set (`gen --preamble --project <member-dir>`), then loop `gen --module <Name> --tail` over the member's modules. There is no workspace-level shared preamble; each member's `specs/specdev/` prose is independent.
3. Capture per-member status: validate result, prompt assembly status, source emit count, cargo-toml result, compile-gate outcome per label (Step 8 runs `specdev-analyzer gate --label <l>` per gate in the member's own `gates:` registry, from its inlined `profile.yaml`; the runner records under that label, in the member's own lineage).

If any member fails validate, do NOT skip downstream members — record the failure and continue. The point of the workspace walk is to surface every member's status in one report. (Exception: stop the walk if member_a fails to emit and member_b structurally depends on member_a's emitted source — `cargo check` on member_b will fail to find symbols. Record the cascade clearly.)

## W5 — emit workspace-root Cargo.toml

After every member has been processed, emit the workspace-root Cargo.toml:

```sh
specdev-analyzer manifest --write
```

Run from the workspace root. The analyzer detects `[workspace]` and renders workspace-root mode (`[workspace]` + `[workspace.package]` + `[workspace.dependencies]`).

Then record it (the per-member walk already records member files via single-package Step 6.5; the workspace-root Cargo.toml needs its own record against the workspace root's `specs/specdev/`):

```sh
specdev-analyzer gen-record Cargo.toml
```

## W6 — workspace-level summary

Emit ONE report at the end with one line per member:

```
ym2414b-core      ✓ validate (12m, 8r, 0 graph) | prompt (4 modules) | emit (12 files) | cargo-toml ✓ | cargo check ✓
ym2414b-sysex     ✓ validate (3m, 1r, 0 graph) | prompt (3 modules) | emit (5 files)  | cargo-toml ✓ | cargo check ✓
workspace         ✓ Cargo.toml (workspace-root)
```

Then a follow-up paragraph: any per-member warnings, any cross-member coupling that surfaced, suggested next steps.
