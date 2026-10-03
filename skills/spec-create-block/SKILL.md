---
name: spec-create-block
description: This skill should be used when the user asks to "create a block", "author a new block", "scaffold a block", "write a block for <framework/library/hardware>", "add a block that contributes <X>", or "/spec-create-block <name>" — i.e. authoring a NEW block unit (schema + reduction + implementation guidance + README) for a concrete framework, library, or hardware family. New blocks only — refuses if `blocks/<name>/` already exists. To inline an EXISTING block into a project, use /spec-inline. This skill authors blocks only, not bridges (use /spec-create-bridge to wire two blocks).
allowed-tools: Read, Write, Edit, Bash, Glob, Grep
argument-hint: <block-name> (concrete; e.g. embassy, nih-plug, ws2812-rs)
---

# /spec-create-block — Author a new block

Scaffold a complete **block** — the reusable unit that contributes a namespaced `[<block>]` table to a module spec, the reduction that puts its entities in the graph, and the implementation guidance that whoever writes the code follows. A block is a **rule pack**: its fields state intent that constrains the code, and each field has a check or a guidance rule. A block names **one concrete** framework / library / hardware family and stands alone; cross-block wiring lives in bridges, never here.

specdev does not generate code, so a block ships no code templates and no dependency list: the project's build manifest owns dependencies, and the code is written by hand against the block's guidance.

**This skill authors; it does not inline.** After it lands, a project inlines the block with `/spec-inline <name>`.

User input: `$ARGUMENTS` — the block name. If empty, ask what concrete framework/library/hardware the block is for before doing anything else.

If `specdev-analyzer` is not on PATH, tell the user to install it (follow `${CLAUDE_PLUGIN_ROOT}/shared/analyzer-install.md` — it derives the org from the plugin's own remote; both repositories are private) and stop. If it IS on PATH but reports `MANIFEST-002`, or a `MANIFEST-001` naming a file under the plugin tree, then the analyzer is older than the plugin — not a problem with any project file. Tell the user to re-run the install command with `--force` (or `cargo install --path <checkout> --force` from a local checkout), then retry.

Before any analyzer invocation, run the env preamble from `${CLAUDE_PLUGIN_ROOT}/shared/plugin-root-preamble.md`:

```sh
export SPECDEV_PLUGIN_ROOT="${CLAUDE_PLUGIN_ROOT}"
```

Skip if `SPECDEV_PLUGIN_ROOT` is already set or every invocation passes `--plugin-root`.

The exact file shapes this skill scaffolds are in `references/block-artifact-templates.md` — read it before Step 3. This body is the orchestration; that file is the material.

---

## Step 0 — prerequisites, scope guard, target root, the SHARP TEST

1. **Choose the target plugin root.** Default to `${CLAUDE_PLUGIN_ROOT}`; test writability with `test -w "<root>/blocks" && echo writable`. If the plugin root is **read-only** (an installed plugin), ask the user for a writable plugin-tree checkout to author into (they will later PR it upstream or point `--plugin-root` at it). Confirm `<root>/.claude-plugin/plugin.json` exists.

2. **Refuse if the block already exists.** If `<root>/blocks/<name>/` is present, stop: "That block already exists — edit it directly, or `/spec-inline <name>` to inline it into a project." New-authoring only.

3. **Name discipline.** The block name is lowercase-hyphenated and *concrete* (`nih-plug`, `stm32h5-rs`, `ws2812-rs`), naming a specific thing — never a role (`runtime`, `hal`, `gui`). ⚠️ **Hyphen/underscore:** a hyphenated block name (`nih-plug`) takes an **underscore** schema table key (`[nih_plug]`), while the directory, `$id`, and reduction entity **qualifier** keep the hyphen (`nih-plug.Parameter`). Pick the name aware of this split; it is load-bearing for any future bridge that references your entity kinds.

4. **Run the SHARP TEST before writing anything.** A block must pass all three legs; if any fails, stop and tell the user *which* leg and why:
   - **Leg 1 — concreteness.** Does the block name a *specific* framework / library / hardware family? Counter-test: if "which X?" can substitute for the name (a "concurrency" block that "could be embassy or rtic"), leg 1 fails.
   - **Leg 2 — rule-pack completeness.** Does every field state intent that constrains the code — and have a check (a graph check, a reference that must resolve, a future unit check) or a guidance rule? A field that only restates the code (a type, a signature, a dependency version) fails: the code and the manifest already hold it — a spec states intent, not syntax. A pure abstraction with nothing to constrain fails too.
   - **Leg 3 — no cross-block reach.** Does the block stand alone? It must not reference another block's types or reach into its territory — that belongs in a bridge. Naming a companion block in the *author-facing README* is fine; reaching into another block's territory in the **guidance** is a leg-3 violation. Grep the drafted `guidance.md` for every other block's name (not this block's own name) and judge each hit: naming a crate the framework itself depends on is not a reach; telling the code what another block's table means, or what code another block owns, is.

5. **Language.** Guidance is language-specific where the framework is. One block targets the language of its framework (`rust`, `dart`, `typescript`).

6. **Read the precedents.** Read `references/block-artifact-templates.md` and an existing shipped block whose shape matches: `<root>/blocks/embassy/` (topology-only, two entity kinds, no relations) or `<root>/blocks/egui/` (declares an intra-block relation). Read the whole unit — `schema.yaml`, `reduction.yaml`, `guidance.md`, `README.md`.

## Step 1 — interview the block's contribution

Batch these; show precedent answers as defaults.

- **Entity kinds the block contributes.** What sub-units does the `[<block>]` table declare (embassy: `tasks` + `channels`; egui: `panels` + `regions`; stm32h5-rs: `peripherals` + `pins`)? For each, the plural table name and its per-entry fields (`name` is universal; plus kind-specific fields with their types, enums, defaults). For each field, ask: what does it constrain in the code, and how would anyone know the code broke it? A field with no answer does not belong.
- **Intra-block relation?** Does one entity kind reference another *within* the block (egui's `region_in_panel`: a Region names its Panel)? If so, capture the from/to kinds + the field. Most blocks have none (`relations: []`). Cross-block references are a bridge.
- **Rules no check can verify.** What must whoever writes the code know that the schema cannot say? Thread or interrupt discipline, ownership, required idioms, the mistakes that compile and then fail. These become `guidance.md`.
- **Companion-block affinity** (README only). Which blocks does this pair with, via which bridge? Author-facing context — it goes in `README.md`, never in the guidance (leg 3).

## Step 2 — verify the rules against the real API

Every guidance rule must be true for the framework's current release. Resolve the current release from the registry — crates.io `/api/v1/crates/<name>` (it returns 403 without a `User-Agent` header; send one), pub.dev `/api/packages/<name>`, npm `https://registry.npmjs.org/<name>` (`dist-tags.latest`) — or the context7 docs tool. Then check each rule that names an API behaviour against that release's documentation or source (`~/.cargo/registry/src/**/<crate>-<ver>/`, the context7 `query-docs` tool).

Never write a rule from memory: blocks have shipped wrong exactly that way — a library went 1.0 and renamed its core vocabulary, and prose written from pre-1.0 memory named things that no longer existed. If the registry or docs are unreachable, say so and stop.

Guidance carries **no version numbers**: name the behaviour ("the timer driver feature that picks any timer"), and tell the reader to check the version in the project's manifest where an API differs between releases.

## Step 3 — scaffold the block

Write the files under `<root>/blocks/<name>/`, following `references/block-artifact-templates.md`:

| File | Notes |
|---|---|
| `schema.yaml` | `$id: https://specdev.dev/blocks/<name>.v1` (BLOCK-LINT-006 checks this). Contributes `properties.<block-key>` (underscore key for a hyphenated name). `additionalProperties: false` on the `[<block>]` table and each nested object — a block table is a **leaf**, so use `additionalProperties`, **not** `unevaluatedProperties`. Inline `Identifier` in `$defs` — never `$ref` a profile's `enums.schema.yaml`. |
| `reduction.yaml` | `metaschema_version: "v1"`. One entity per kind, `extends: Module`, `from: "specs/modules/*.toml"`, `for_each: "<block-key>.<plural>[]"`, `name: "${root.name}.${.name}"`, `attrs` 1:1 with the schema fields. `relations: []` unless the block has an intra-block relation (then `extends: depends_on`, path-qualified `from_kind`/`to_kind`). |
| `guidance.md` | The rules from Step 1, verified in Step 2. Rules only: no fenced code and at most 4,096 bytes (`plugin-lint` checks these), no version numbers (review checks this). Sectioned by concern; each rule one bullet. **Leg 3:** no sibling-block names. **PROSE-PROFILE-001:** write "the project's profile", never a profile name. It is inlined into every project that uses the block, and `specdev-analyzer context --module` lists it for whoever edits a module. |
| `README.md` | Author-facing (never inlined): when to use, what it contributes, the sharp-test rationale (all three legs), companion-block/bridge affinity, deferred items. Naming a profile or companion block HERE is fine. |

## Step 4 — gate at authoring time

1. **Run plugin-lint:**
   ```sh
   specdev-analyzer plugin-lint --plugin <root>
   ```
   Clear every finding for this block:
   - `BLOCK-LINT-001/002/004` — `schema.yaml`, `reduction.yaml`, `README.md` present.
   - `BLOCK-LINT-005` — `schema.yaml` is valid YAML and compiles as JSON Schema.
   - `BLOCK-LINT-006` — `$id` matches `https://specdev.dev/blocks/<name>.v1`.
   - `BLOCK-LINT-007` — `reduction.yaml` passes the embedded reduction metaschema.
   - `GUIDANCE-001/002/003` — `guidance.md` present, no fenced code, under the cap.
   - `PROSE-PROFILE-001` — no profile name in `guidance.md`.

   Surface any finding verbatim and fix; run again until clean.

2. **Leg 3 by grep:** `grep -n -e <other-block> … <root>/blocks/<name>/guidance.md` over every other block name. Judge each hit: a crate the framework itself depends on is fine; a rule about another block's table or code is a reach — move it to a bridge, or rephrase it without the sibling.

3. **Round-trip smoke:** in a scratch project, `/spec-init` a matching profile, `/spec-inline <name>`, author a minimal module using the block's table, with `planned = true` (it has no code), and run `specdev-analyzer check`. Zero schema findings confirms the schema composes; `specdev-analyzer reduce` shows the block's entities in the graph; `specs/specdev/guidance/blocks/<name>.md` is the inlined guidance.

## Step 5 — report + next steps

Report the files written, the `plugin-lint` codes cleared, the leg-3 grep result, and the sharp-test verdict. **Next:** in a project, `/spec-inline <name>`, then author a module using the `[<block-key>]` table and `/spec-validate`. To wire this block to another, author a bridge with `/spec-create-bridge <a>+<b>`. To share, PR it upstream. **Live smoke is next-session** if the skill/tree only just changed (they load at session start) — say so rather than claiming an end-to-end pass this session.

## What this skill does NOT do

- **Author a bridge.** Cross-block wiring is a bridge — use `/spec-create-bridge`. Keep sibling-block reach out of this block's guidance (leg 3).
- **Inline the block.** Use `/spec-inline <name>`.
- **Ship code, code templates or a dependency list.** The code is written by hand against the guidance; the project's manifest owns dependencies.
- **Publish / register / version.** No registry or semver in v1 (versioning is plugin-wide; sharing is by upstream-PR or file-copy).
- **Write rules from memory.** Verify each rule against the current release (Step 2).
