---
name: spec-create-block
description: This skill should be used when the user asks to "create a block", "author a new block", "scaffold a block", "write a block for <framework/library/hardware>", "add a block that contributes <X>", or "/spec-create-block <name>" — i.e. authoring a NEW block unit (schema + reduction + codegen prose + deps + README, and a compiled reference crate for discrete-case blocks) for a concrete framework, library, or hardware family. New blocks only — refuses if `blocks/<name>/` already exists. To inline an EXISTING block into a project, use /spec-inline. This skill authors blocks only, not bridges (use /spec-create-bridge to wire two blocks).
allowed-tools: Read, Write, Edit, Bash, Glob, Grep
argument-hint: <block-name> (concrete; e.g. embassy, nih-plug, ws2812-rs)
---

# /spec-create-block — Author a new block

Scaffold a complete, correctness-gated **block** — the reusable unit that
contributes a namespaced `[<block>]` table to a module spec plus the reduction
and codegen prose that turn it into source. A block names **one concrete**
framework / library / hardware family and stands alone; cross-block wiring lives
in bridges, never here.

For a discrete-case block (a finite set of worked-example kinds — task/channel,
panel/region, widget kinds), the scaffold also includes a compiled
`reference/<lang>/` crate whose examples are drawn from the *real* crate API. The
reference + `plugin-lint` are what stop the untested-prose bug class: a wrong
type or generic arity fails at authoring time here, not at a downstream
`/spec-gen` + `cargo build`.

**This skill authors; it does not inline.** After it lands, a project inlines the
block with `/spec-inline <name>`.

User input: `$ARGUMENTS` — the block name. If empty, ask what concrete
framework/library/hardware the block is for before doing anything else.

If `specdev-analyzer` is not on PATH, tell the user to install it
(`cargo install --git https://github.com/cosmikwolf/specdev-analyzer`) and stop.

Before any analyzer invocation, run the env preamble from
`${CLAUDE_PLUGIN_ROOT}/shared/plugin-root-preamble.md`:

```sh
export SPECDEV_PLUGIN_ROOT="${CLAUDE_PLUGIN_ROOT}"
```

Skip if `SPECDEV_PLUGIN_ROOT` is already set or every invocation passes
`--plugin-root`.

The exact file shapes this skill scaffolds are in
`references/block-artifact-templates.md` — read it before Step 3. This body is the
orchestration; that file is the material.

---

## Step 0 — prerequisites, scope guard, target root, the SHARP TEST

1. **Choose the target plugin root.** Default to `${CLAUDE_PLUGIN_ROOT}`; test
   writability with `test -w "<root>/blocks" && echo writable`. If the plugin
   root is **read-only** (an installed plugin), a new block cannot live there —
   ask the user for a writable plugin-tree checkout to author into (they will
   later PR it upstream or point `--plugin-root` at it; v1 sharing is by
   upstream-PR or file-copy). Confirm `<root>/.claude-plugin/plugin.json` exists.

2. **Refuse if the block already exists.** If `<root>/blocks/<name>/` is present,
   stop: "That block already exists — edit it directly, or `/spec-inline <name>`
   to inline it into a project." New-authoring only.

3. **Name discipline.** The block name is lowercase-hyphenated and *concrete*
   (`nih-plug`, `stm32h5-rs`, `ws2812-rs`), naming a specific thing — never a role
   (`runtime`, `hal`, `gui`). ⚠️ **Hyphen/underscore:** a hyphenated block name
   (`nih-plug`) takes an **underscore** schema table key (`[nih_plug]`), while the
   directory, `$id`, and reduction entity **qualifier** keep the hyphen
   (`nih-plug.Parameter`). Pick the name aware of this split; it is load-bearing
   for any future bridge that references your entity kinds. (For a block with no
   intra-block relation the qualifier is *implicit* — established by file location,
   appearing in no authored field; it becomes explicit only in an intra-block
   relation's `from_kind`/`to_kind` or when a bridge references your kinds. Choose
   the name right anyway — renaming later breaks those references.)

4. **Run the SHARP TEST before writing anything** (this is the block's authoring
   gate — the analog of the bridge's lex-order check). A block must pass all
   three legs; if any fails, stop and tell the user *which* leg and why, rather
   than scaffolding a block that will not lint or compose:
   - **Leg 1 — concreteness.** Does the block name a *specific* framework /
     library / hardware family? Counter-test: if "which X?" can substitute for the
     name (a "concurrency" block that "could be embassy or rtic"), leg 1 fails.
   - **Leg 2 — codegen-completeness.** Can codegen prose be authored that produces
     *working* code from conforming TOML? If the block is a pure abstraction with
     no concrete emission, leg 2 fails.
   - **Leg 3 — no cross-block reach.** Does the block stand alone? It must not
     reference another block's types or reach into its territory — that belongs in
     a bridge. (Naming a companion block in the *author-facing README* is fine;
     reaching in *codegen prose* is a leg-3 violation.)

5. **Language.** `rust` or `dart`; default to the language of the profile the
   block targets. One language per invocation (the common case); re-invoke for
   a second language's codegen + reference.

6. **Read the precedents.** Read `references/block-artifact-templates.md` and an
   existing shipped block whose shape matches: `<root>/blocks/embassy/`
   (topology-only, two entity kinds, no relations) or `<root>/blocks/egui/`
   (declares an intra-block relation). Read the whole unit — all files + its
   `reference/rust/`.

## Step 1 — interview the block's contribution

Batch these; show precedent answers as defaults.

- **Entity kinds the block contributes.** What sub-units does the `[<block>]`
  table declare (embassy: `tasks` + `channels`; egui: `panels` + `regions`;
  stm32h5-rs: `peripherals` + `pins`)? For each, the plural table name and its
  per-entry fields (`name` is universal; plus kind-specific fields with their
  types, enums, defaults).
- **Intra-block relation?** Does one entity kind reference another *within* the
  block (egui's `region_in_panel`: a Region names its Panel)? If so, capture the
  from/to kinds + the field. Most blocks have none (`relations: []`). This is
  intra-block only — cross-block references are a bridge.
- **Crate deps the block contributes** (`deps.yaml`): the crate(s) this block's
  generated code needs, with features. Plus any `recommended_external_deps`
  (surfaced to the user, not auto-injected).

  ⚠️ **Never take the version from the user's memory or your own — resolve it.**
  Ask the registry what the current release is (crates.io `/api/v1/crates/<name>`,
  pub.dev `/api/packages/<name>`, or the context7 docs tool), and pin *that*. This
  is the one fact Step 2's API research cannot check: everything there is verified
  *against* the pin, so a pin recalled from training data anchors all of it to a
  guess. Blocks have shipped wrong this exact way — a library went 1.0 and renamed
  its core vocabulary, and prose written from pre-1.0 memory referenced names that
  no longer existed. If the registry is unreachable, say so and stop; do not guess
  a version.

  For a **git** source, pin a `rev` (or `tag`) — never a bare repo URL. An
  unpinned git dep resolves to the default branch at emit time, so projects track
  a moving HEAD while the reference crate proves one fixed commit; the reference
  then stays green through an upstream break every project hits. Use the same rev
  the reference crate builds against. `plugin-lint` enforces this
  (`REF-PIN-001` / `REF-PIN-002`).
- **Discrete case set (reference vocabulary)** — the finite set of worked-example
  cases the codegen prose covers (usually the entity kinds or their variants).
  These become `reference/<lang>/src/<case>.rs` + `reference_cases.yaml`.
  **Opt-out:** a block whose prose is *uniform* (no finite case set) or targets an
  **unreleased** API (e.g. a HAL not yet published — it cannot compile a
  reference) ships **no** reference; record the reason in `deps.yaml`/`README.md`
  and rely on the relevant bridge's reference for the real path. Confirm an
  opt-out deliberately; don't default into it.
- **Reference target shape** (for a discrete-case block). The invariant is "a lib
  that **type-checks without link**" — so the only question is whether the crate
  builds on the *host*. Add a `target = <triple>` in `.cargo/config.toml` **only if
  a dep forces it** — a chip-specific HAL pinned to a non-host target (e.g.
  `embassy-stm32` → `thumbv8m…`). Otherwise use a plain host lib with no triple.
  `#![no_std]` is **independent** of the target: a `no_std` but *portable* crate
  (`heapless`, a protocol lib) type-checks on the host with no triple — mark it
  `#![no_std]` but add **no** target. Do not assume `no_std` ⇒ cross-target.
- **Companion-block affinity** (README only). Which blocks does this pair with,
  via which bridge? This is author-facing context — it goes in `README.md`, never
  in codegen prose (leg 3).

## Step 2 — resolve the REAL API (the load-bearing step)

**This is why the reference crate exists.** Author codegen prose and the reference
examples from the *real* pinned crate API, never from memory — every prose bug in
practice was a hand-transcription error (a wrong generic arity, a constructor
signature, a type/method that doesn't exist in the pinned crate).

For each type, constructor, and method the codegen prose documents, and every
worked case:

1. Resolve its **actual** signature from the **pinned** crate — in order of
   preference: the context7 MCP docs tool (`resolve-library-id` → `query-docs`)
   for the exact version/rev; the local checkout under `~/.cargo`
   (`~/.cargo/registry/src/**/<crate>-<ver>/` for crates-io,
   `~/.cargo/git/checkouts/**` for git deps) — `grep`/`Read` the real
   `pub fn` / `pub struct` / generic parameter list; `cargo doc -p <crate>` as a
   last resort.
2. Author the reference example and the README worked-example from *that*
   signature, annotating the type **explicitly** so the compiler checks arity.

If a crate is unreachable, say so and stop rather than transcribe from memory. (A
block opting out of the reference — unreleased API — still authors prose against
whatever docs exist, and says clearly it is unverified.)

## Step 3 — scaffold the block (+ reference crate)

Write the files under `<root>/blocks/<name>/`, following
`references/block-artifact-templates.md`. The block files:

| File | Notes |
|---|---|
| `schema.yaml` | `$id: https://specdev.dev/blocks/<name>.v1` (BLOCK-LINT-006 checks this). Contributes `properties.<block-key>` (underscore key for a hyphenated name). `additionalProperties: false` on the `[<block>]` table and each nested object — a block table is a **leaf**, so use `additionalProperties`, **not** `unevaluatedProperties` (that is only for aggregator targets like a profile's module schema). Inline `Identifier`/`RustTypeExpression` in `$defs` — never `$ref` a profile's `enums.schema.yaml`. |
| `reduction.yaml` | `metaschema_version: "v1"`. One entity per kind, `extends: Module`, `from: "specs/modules/*.toml"`, `for_each: "<block-key>.<plural>[]"`, `name: "${root.name}.${.name}"`, `attrs` 1:1 with the schema fields. `relations: []` unless the block has an intra-block relation (then `extends: depends_on`, path-qualified `from_kind`/`to_kind`). |
| `codegen/<lang>/README.md` | The load-bearing prose (~1500–1700 tokens): sectioned by concern (crate deps, type emission, per-kind emission, error handling), a worked example per kind from Step 2's resolved API, and at least one explicit **do-NOT** rule per section. **Leg 3:** no sibling-block names. **PROSE-PROFILE-001:** write "the project's profile's `module.conventions.md`", never a specific profile name. |
| `codegen/<lang>/deps.yaml` | `crates[]` the block contributes + `recommended_external_deps[]`. |
| `README.md` | Author-facing (never inlined): When to use, What it contributes, sharp-test rationale (all three legs), companion-block/bridge affinity, deferred items. Naming a profile or companion block HERE is fine. |

Then, for a **discrete-case** block, the **reference crate** under
`<root>/blocks/<name>/reference/<lang>/` (a **sibling** to `codegen/`, never
nested — inline never copies `reference/`, so it is CI-only):

- `Cargo.toml` — `version = "0.0.0"`, `publish = false`, `[lib]`; deps mirror what
  the block contributes at codegen, pinned (git deps by exact `rev`).
- `.cargo/config.toml` — `target = <triple>` for a cross-target reference; omit
  for host-target.
- `src/lib.rs` — crate root; `#![no_std]` for embedded; `pub mod <case>;` per case.
- `src/<case>.rs` — one per case, mirroring the shape `/spec-gen` emits, with the
  explicit type annotation.
- `reference_cases.yaml` — `reference_cases: [<case>, …]` (opaque names; drives
  `REF-COVERAGE-001`).

## Step 4 — gate at authoring time (compile + lint)

1. **Compile the reference** (discrete-case blocks only). Rust:
   ```sh
   cd <root>/blocks/<name>/reference/rust && cargo build
   ```
   (the `.cargo/config.toml` target applies for a cross-target reference; install
   it first with `rustup target add <triple>` if the build reports it missing —
   an environment gap, **not** a prose bug). Dart:
   ```sh
   cd <root>/blocks/<name>/reference/dart && dart pub get && dart analyze
   ```
   (use `flutter pub get && flutter analyze` when the reference's pubspec depends
   on the Flutter SDK; the analyzer is the type-check gate — no binary build is
   needed). In either language a wrong type / arity / constructor fails on a
   *type error* — fix the prose **and** the reference against the real API
   (Step 2); do not `#[allow]` / `// ignore:` it green. Distinguish a type-check
   failure (the gate working) from a toolchain-missing failure.

2. **Run plugin-lint:**
   ```sh
   specdev-analyzer plugin-lint --plugin <root>
   ```
   Clear every finding for this block:
   - `BLOCK-LINT-001..004` — required files present (`schema.yaml`,
     `reduction.yaml`, a `codegen/<lang>/README.md`, `README.md`).
   - `BLOCK-LINT-005` — `schema.yaml` is valid YAML and compiles as JSON Schema.
   - `BLOCK-LINT-006` — `$id` matches `https://specdev.dev/blocks/<name>.v1`.
   - `BLOCK-LINT-007` — `reduction.yaml` passes the embedded reduction metaschema.
   - `REF-COVERAGE-001` / `REF-ARITY-001` — reference coverage + generic-arity
     consistency (discrete-case blocks). `REF-ARITY-001` matches **by base type
     name**: for each base name that appears in the reference *source* (`Vec`,
     `I2c`, …), every occurrence of that name in the README must use the **same
     number of generic arguments**. Only names present in the reference are checked
     (README-only types like `Result`/`Option` never false-flag) — so keep the type
     text identical between the README example and the reference source. It is a
     **consistency** check only; the Step 4.1 compile is the correctness gate.
   - `PROSE-PROFILE-001` — no profile name in the inlined `codegen/<lang>/README.md`.

   Surface any finding verbatim and fix; re-run until clean.

3. **Optional round-trip smoke:** in a scratch project, `/spec-init` a matching
   profile, `/spec-inline <name>`, author a minimal module using the block's table,
   and `/spec-validate` — zero schema findings confirms the schema composes. (This
   needs a project; the reference compile + plugin-lint are the authoring-time
   gate that runs without one.)

## Step 5 — report + next steps

Report the files written, the reference build/analyze result + target (or the
opt-out reason), the `plugin-lint` codes cleared, and the sharp-test verdict.
**Next:** in a project, `/spec-inline <name>`, then author a module using the
`[<block-key>]` table and `/spec-validate`. To wire this block to another, author
a bridge with `/spec-create-bridge <a>+<b>`. To share, PR it upstream. **Live
smoke is next-session** if the skill/tree only just changed (they load at session
start) — say so rather than claiming an end-to-end pass this session.

## What this skill does NOT do

- **Author a bridge.** Cross-block wiring is a bridge — use `/spec-create-bridge`.
  Keep sibling-block reach out of this block's codegen (leg 3).
- **Inline the block.** Use `/spec-inline <name>`.
- **Multi-language in one invocation.** One language per run.
- **Publish / register / version.** No registry or semver in v1 (versioning is
  plugin-wide; sharing is by upstream-PR or file-copy).
- **Fabricate the reference or prose from memory.** Author against the real pinned
  API and compile the reference; the oracle-loop re-distillation from a real
  `/spec-gen` run is a separate follow-up pass.
