---
name: spec-inline-bridge
description: This skill should be used when the user names any `<a>+<b>` bridge (lex-ordered: `a < b`; e.g. `embassy+stm32h5-rs`, `egui+nih-plug`, or any user-authored bridge), or asks to "inline a bridge", "wire two blocks together", "wire <X> to <Y>", "connect two blocks", "add `<a>+<b>` to the spec", or "run specdev inline-bridge". Thin wrapper over `specdev-analyzer inline-bridge`. Resolves bridge names dynamically against `${CLAUDE_PLUGIN_ROOT}/bridges/` so user-authored bridges work the same as plugin-shipped ones.
allowed-tools: Read, Bash, Glob, Grep
argument-hint: <bridge-name> [--merge]
---

# /spec-inline-bridge — Thin wrapper over `specdev-analyzer inline-bridge`

Inline a canonical bridge between two already-inlined blocks. Bridges live at `${CLAUDE_PLUGIN_ROOT}/bridges/<a>+<b>/` (lex-ordered: `a < b`). They contribute schema for the `bindings.<a>+<b>.*` table on each module plus reduction rules and codegen prose for the wiring pattern. A bridge's inlined content is **split across** four locations in the project (there is no single `inlined/bridges/<a>+<b>/` directory): `schemas/inlined/bridges/<a>+<b>.schema.yaml`, `reductions/bridges/<a>+<b>.yaml` (plus its `.contract.yaml`), `codegen/<lang>/bridges/<a>+<b>/`, and an `upstream/bridges/<a>+<b>/` snapshot.

If `specdev-analyzer` is not on PATH, tell the user to install it (follow `${CLAUDE_PLUGIN_ROOT}/shared/analyzer-install.md` — it derives the org from the plugin's own remote; both repositories are private) and stop. If it IS on PATH but reports `MANIFEST-002`, or a `MANIFEST-001` naming a file under the plugin tree, then the analyzer is older than the plugin — not a problem with any project file. Tell the user to re-run the install command with `--force` (or `cargo install --path <checkout> --force` from a local checkout), then retry.

Before any analyzer invocation, run the env preamble from `${CLAUDE_PLUGIN_ROOT}/shared/plugin-root-preamble.md`:

```sh
export SPECDEV_PLUGIN_ROOT="${CLAUDE_PLUGIN_ROOT}"
```

Skip if `SPECDEV_PLUGIN_ROOT` is already set or every invocation passes `--plugin-root`.

---

## Step 1 — parse arguments

User input: `$ARGUMENTS`. Expected: `<bridge-name> [--merge]`.

- `<bridge-name>` (required) — `<a>+<b>` form, lex-ordered (`a < b`). Never hardcode the available set; list dynamically with `ls "${CLAUDE_PLUGIN_ROOT}/bridges/"` — every directory there is a valid choice, including user-authored bridges. Read `${CLAUDE_PLUGIN_ROOT}/bridges/<a>+<b>/README.md` for the wiring contract.
- `--merge` (optional) — required to re-inline an already-inlined bridge; performs a 3-way merge from the recorded upstream snapshot.

Pre-flight: both blocks named in the bridge **must already be inlined** in the project. The analyzer surfaces a clear error if not, but a friendly heads-up here saves the user a round-trip:

- Split `<bridge-name>` on `+` to get `<a>` and `<b>`. Check that both `specs/specdev/upstream/blocks/<a>/` and `specs/specdev/upstream/blocks/<b>/` exist — that per-block snapshot directory is written on every inline and is the reliable "is this block inlined" marker. (There is **no** single `specs/specdev/inlined/blocks/<x>/` directory: a block's inlined content is split across `schemas/inlined/blocks/<x>.schema.yaml`, `reductions/blocks/<x>.yaml`, `codegen/<lang>/blocks/<x>/`, and `upstream/blocks/<x>/`.) If either is missing, instruct: "Inline the missing block first: `/spec-inline <block>`."

If the bridge name violates lex-ordering (e.g. user types `b+a` when `a < b`), reject with the corrected `<a>+<b>` form and stop. Do not silently rewrite — the user might have typed the wrong block.

## Step 2 — run inline-bridge

From the project root:

```sh
specdev-analyzer inline-bridge <bridge-name> [--merge]
```

Capture stdout, stderr, and exit code.

Exit codes:
- `0` — clean. Go to Step 3.
- non-zero — surface stderr verbatim. Common error families:
  - `INLINE-003` — **the one you will actually see.** An endpoint block is not inlined; the message
    names it. Run `/spec-inline <block>` first, then retry. Note this is also what a *misspelled
    bridge name* produces: the endpoint check runs before the bridge-exists check, so a typo becomes
    an un-inlined endpoint name. `inline-bridge egui+nih-plugXX` reports
    ``requires `nih-plugXX` to be inlined first``, not "no such bridge". If the named endpoint looks
    like a typo rather than a real block, treat it as a bad bridge name and check
    `ls "${CLAUDE_PLUGIN_ROOT}/bridges/"`.
  - `INLINE-002` — bridge already inlined; rerun with `--merge`. Also covers a file collision in the
    destination (a file already sitting where a bridge-emitted file goes, including an existing
    `upstream/` snapshot).
  - `INLINE-001` — the bridge directory is missing from the plugin tree, or is missing its
    `bindings.schema.yaml` / `reduction.yaml`. A plugin-author bug rather than a user typo — a typo
    surfaces as `INLINE-003` above.
  - `BRIDGE-LEX-001` — bridge directory in plugin tree violates lex-ordering. Plugin-author bug; surface for the user (and consider `specdev-analyzer plugin-lint`).
  - `MERGE-001..003` — `--merge` conflict; see the conflict-resolution guidance under `/spec-inline`.
  - `PLUGIN-001` — plugin root not resolvable.

## Step 3 — validate and report

After a successful inline, run:

```sh
specdev-analyzer validate --no-color
```

Bridges introduce a new `bindings.<a>+<b>.*` schema slot on each module; modules that don't use the bridge are unaffected. New schema findings here usually mean a module declared `bindings.<a>+<b>.<x>[]` referencing a `task` / `peripheral` / etc. that the bridge's reduction can't resolve — `/spec-validate` walks the user through the fix.

Report:

- Bridge inlined (and whether `--merge` was used).
- Files written, naming all four locations: `schemas/inlined/bridges/<a>+<b>.schema.yaml`, `reductions/bridges/<a>+<b>.yaml` (plus its `.contract.yaml`), `codegen/<lang>/bridges/<a>+<b>/`, and an `upstream/bridges/<a>+<b>/` snapshot.
- New `lineage.toml` entry.
- Validate result.
- Suggested next: edit a module spec to add a `bindings."<a>+<b>".*` table, then `/spec-validate`. Or `/spec-design <new-module>` for a module that uses the bridge from the start.

## What this skill does NOT do

- **Inline blocks.** Use `/spec-inline`.
- **Author project-local bridges.** Project-local bridges live under `specs/specdev/schemas/inlined/bridges/<a>+<b>.schema.yaml` + `specs/specdev/reductions/bridges/local/<a>+<b>.yaml` and are hand-authored per project. This skill only handles canonical (plugin-shipped) bridges.
- **Edit module-spec `bindings` after the bridge inline.** Use `/spec-edit <module>` or hand-edit the module TOML.
