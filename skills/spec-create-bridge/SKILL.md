---
name: spec-create-bridge
description: This skill should be used when the user asks to "create a bridge", "author a new bridge", "scaffold a bridge", "write a bridge between <X> and <Y>", "wire two blocks with a bridge that doesn't exist yet", or "/spec-create-bridge <a>+<b>" — i.e. authoring a NEW canonical bridge unit (the 6 bridge files + a compiled reference crate) between two blocks that already exist in the plugin tree. New bridges only — refuses if `bridges/<a>+<b>/` already exists. To inline an EXISTING bridge into a project, use /spec-inline-bridge. This skill authors bridges only, not blocks.
allowed-tools: Read, Write, Edit, Bash, Glob, Grep
argument-hint: <a>+<b> (lex-ordered block pair; e.g. embassy+stm32h5-rs)
---

# /spec-create-bridge — Author a new canonical bridge

Scaffold a complete, correctness-gated **canonical bridge** between two blocks
that already exist in the plugin tree: the six bridge files **plus** a compiled
`reference/<lang>/` crate whose worked examples are drawn from the *real* crate
API. The reference crate + `plugin-lint` are what stop the untested-prose bug
class — a wrong type, generic arity, or constructor signature fails
at authoring time here, not at a downstream `/spec-gen` + `cargo build`.

**This skill authors; it does not inline.** After it lands, a project inlines
the bridge with `/spec-inline-bridge <a>+<b>`.

User input: `$ARGUMENTS` — the bridge name `<a>+<b>` (lex-ordered, `a < b`). If
empty, ask for the two blocks to wire before doing anything else.

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
`references/bridge-artifact-templates.md` — read it before Step 3. This body is
the orchestration; that file is the material.

---

## Step 0 — prerequisites, scope guard, target root

1. **Parse `<a>+<b>`.** Split on `+`. Reject if not exactly two non-empty
   segments.

2. **Lex-order.** Require `a < b` (byte-wise). If the user typed `b+a`, reject
   with the corrected `<a>+<b>` and stop — do **not** silently rewrite (they may
   have meant a different pair). This mirrors `BRIDGE-LEX-001`; a mis-ordered
   directory fails `plugin-lint`.

3. **Choose the target plugin root.** Default to `${CLAUDE_PLUGIN_ROOT}`. When
   the specdev plugin is loaded from a writable working tree (the contributor
   case), that is the right target and the bridge becomes first-party. Test
   writability explicitly — `test -w "<root>/bridges" && echo writable`. If the
   plugin root is **read-only** (an installed plugin), a new canonical bridge
   cannot live there — ask the user for a writable plugin-tree checkout to
   author into (they will later PR it upstream or point `--plugin-root` at it;
   v1 sharing is by upstream-PR or file-copy — there is no bundle-install
   mechanism). Confirm `<root>/.claude-plugin/plugin.json` exists.

4. **Refuse if the bridge already exists.** If `<root>/bridges/<a>+<b>/` is
   present, stop: "That bridge already exists — edit it directly, or
   `/spec-inline-bridge <a>+<b>` to inline it into a project." This skill is
   new-authoring only.

5. **Both endpoint blocks must exist.** A bridge wires two *existing* blocks.
   Check `<root>/blocks/<a>/` and `<root>/blocks/<b>/`. If either is missing,
   stop and tell the user to author the missing block first (hand-author it,
   using an existing block under `<root>/blocks/` as the worked precedent; a
   `/spec-create-block` sibling is planned but may not be available yet). Do not
   scaffold a bridge to a nonexistent block — its `from_kind`/`to_kind` would
   reference entity kinds that don't exist.

6. **Language.** Default `rust`. A bridge's codegen + reference are per-language;
   author one language per invocation (the common case). Read each block's
   `blocks/<x>/codegen/<lang>/README.md` to confirm both support the language.

7. **Read the material.** Read `references/bridge-artifact-templates.md` in this
   skill — the exact file shapes plus the invariants each must satisfy. The two
   shipped bridges are the worked precedents: `<root>/bridges/embassy+stm32h5-rs/`
   (no_std / embedded, cross-target reference) and `<root>/bridges/egui+nih-plug/`
   (std / host-target reference). Read the one whose shape matches this pairing
   end to end — all six files plus its `reference/rust/` crate.

## Step 1 — interview the bridge's semantics

A bridge is defined by *what it wires* and *what join-product it owns*. Batch
these questions; show the precedent answers as defaults so a fluent author can
accept them.

- **The join-product crate(s).** Which crate exists *only because these two
  blocks are paired* — one neither block may own without reaching across the
  pairing (e.g. `embassy-stm32` for `embassy+stm32h5-rs`)? Capture its name,
  version (or git `rev`), and features. There may be none (some bridges only
  wire types both blocks already provide) — then `deps.yaml` is omitted.

- **The binding table.** What does a module declare to invoke this wiring? Name
  the table (e.g. `peripheral_tasks`, `parameter_widgets`) and its per-entry
  fields. Two fields are almost always the endpoint references (e.g. `task`,
  `peripheral`); the rest are attributes carried into codegen (`handoff`,
  `widget_kind`, `range_hint`, …). v1 bindings are **co-resident** (intra-module)
  by default — both endpoints declared on the same module that carries the
  table.

- **The relation.** The reduction materializes one relation per binding entry:
  - `from_kind` / `to_kind` — **path-qualified** entity kinds in the source
    blocks (e.g. `stm32h5-rs.Peripheral`, `embassy.Task`). Read each block's
    `reduction.yaml` to get the exact entity-kind names — do not guess them.
  - direction — semantic; "X uses Y" reads as `X → Y`.
  - `extends` — a canonical relation, almost always `depends_on` (so the
    existing acyclic / layer / reachability graph-checks run over the wiring
    with zero domain knowledge).
  - `attrs` — which binding fields become relation attributes.

- **binding_locality + diagnostic_template** (`contract.yaml`). v1 default is
  same-module co-residency (`endpoint_module`); the `diagnostic_template` is the
  `{bridge_name}`-substituted message emitted when a binding's endpoints are not
  co-resident. Copy a precedent and adjust the wording.

- **The discrete case set (reference vocabulary).** What is the finite set of
  worked-example cases this bridge's codegen prose covers (peripheral kinds,
  widget kinds, …)? These become `reference/<lang>/src/<case>.rs` + the
  `reference_cases.yaml` list. Pick opaque, keyword-free case names (`i2c`,
  `usart`, `gpio`; fold `enum`-like names into a combined file). **Opt-out:** a
  bridge whose prose is uniform (no finite case set) or targets an
  **unreleased** API ships no reference — record the reason in `README.md` and
  skip Steps 2's per-case work + the reference crate. This is rare for bridges;
  confirm it deliberately, don't default into it.

- **Reference target shape.** `no_std` cross-target (embedded blocks → a
  `target = <triple>` in `.cargo/config.toml`) vs `std` host-target (GUI/host
  blocks like egui/nih-plug → a plain host lib). Pick from the blocks' nature;
  the invariant is "a lib that **type-checks without link**," not embedded-ness.

## Step 2 — resolve the REAL API (the load-bearing step)

**This is the whole reason the skill reads the real API.** Every codegen-prose
bug found while dogfooding + building the reference crates was a
hand-transcription error against an API written *from memory* — a wrong generic arity (`I2c<'static, Async>`, 2 vs the real 3), a
constructor arg order, types/methods that don't exist in the pinned crate
(`ParamButton`, `Param::unit_string()`). The antidote is to author both the
prose and the reference **from the resolved API, never from memory.**

For each type, constructor, and method in the binding vocabulary and every
worked case:

1. Resolve its **actual** signature from the **pinned** crate — in order of
   preference:
   - the context7 MCP docs tool (`resolve-library-id` then `query-docs`) for the
     exact version/rev, if available;
   - the local checkout under `~/.cargo` (`~/.cargo/registry/src/**/<crate>-<ver>/`
     for crates-io, `~/.cargo/git/checkouts/**` for git deps) — `grep`/`Read` the
     real `pub fn` / `pub struct` / generic parameter list;
   - `cargo doc -p <crate>` in a throwaway crate as a last resort.
2. Author the reference example and the README worked-example from *that*
   signature. Annotate the endpoint type **explicitly** on the example's task/fn
   signature (e.g. `fn poll(i2c: I2c<'static, Async, Master>)`) — an explicit
   annotation is what forces the compiler to check arity; a bare
   `let x = T::new(...)` lets inference paper over a wrong type.

Do not proceed to scaffolding a case whose real signature you could not resolve.
If a crate is unreachable, say so and stop rather than transcribe from memory.

## Step 3 — scaffold the bridge + reference crate

Write the files under `<root>/bridges/<a>+<b>/`, following the exact shapes in
`references/bridge-artifact-templates.md`. The six bridge files:

| File | Notes |
|---|---|
| `bindings.schema.yaml` | `$id: https://specdev.dev/bridges/<a>+<b>.v1`. Contributes `properties.bindings."<a>+<b>"` (one namespace key), `additionalProperties: false` on the table body. Inline `Identifier` patterns — never `$ref` a profile's enums (bridges must compose against any profile carrying both blocks). |
| `reduction.yaml` | `metaschema_version: "v1"`. **`entities: []`** (the metaschema rejects bridge-qualified entities — endpoints live in the blocks). One relation per binding table, `extends: <canonical>`, path-qualified `from_kind`/`to_kind`, `for_each: 'bindings."<a>+<b>".<table>[]'` (quoted bridge segment), `endpoints` + `attrs`. |
| `contract.yaml` | `binding_locality` + `diagnostic_template`. Endpoint pair is encoded in the directory name — do not restate it. |
| `codegen/<lang>/README.md` | Codegen prose. **Profile-agnostic (`PROSE-PROFILE-001`):** may name the two constituent *blocks* (that's the composition seam) but must write "the project's profile", never a profile name. Worked examples drawn from Step 2's resolved API. |
| `codegen/<lang>/deps.yaml` | Optional. Only the join-product crate(s). Omit if the bridge introduces no new crate. |
| `README.md` | Author-facing catalog doc (never inlined): When-to-use, What it contributes / does NOT contribute, sharp-test rationale, co-residency rule, deferred items. Naming a profile HERE is fine (it is not inlined). |

Then the **reference crate** under `<root>/bridges/<a>+<b>/reference/<lang>/`
(a **sibling** to `codegen/`, never nested — inline never copies `reference/`,
so it is CI-only by structure):

- `Cargo.toml` — `version = "0.0.0"`, `publish = false`, `[lib]`; deps mirror
  what the bridge + both blocks contribute at codegen, pinned (git deps by exact
  `rev` for reproducible CI).
- `.cargo/config.toml` — `target = <triple>` for a cross-target reference; omit
  for a host-target reference.
- `src/lib.rs` — crate root; `#![no_std]` for embedded; `pub mod <case>;` per
  case; any shared `bind_interrupts!`/setup that would duplicate symbols if
  per-case.
- `src/<case>.rs` — one per case, mirroring the *shape* `/spec-gen` emits, with
  the explicit endpoint-type annotation (Step 2).
- `reference_cases.yaml` — `reference_cases: [<case>, …]`, the set you actually
  ship a compiled example for (opaque names). This drives `REF-COVERAGE-001`.

## Step 4 — gate at authoring time (compile + lint)

This is the payoff — catch the bug now.

1. **Compile the reference:**
   ```sh
   cd <root>/bridges/<a>+<b>/reference/<lang> && cargo build
   ```
   (the `.cargo/config.toml` target applies automatically for a cross-target
   reference; install it first with `rustup target add <triple>` if the build
   reports the target is not installed — that is an environment gap, **not** a
   prose bug). A wrong type / arity / constructor **fails here**. If it fails on
   a type error, fix the prose *and* the reference against the real API
   (Step 2) — do **not** `#[allow]` or hand-wave it green. A type-check failure
   is the gate working; distinguish it from a toolchain-missing failure.

2. **Run plugin-lint** from the project/plugin context:
   ```sh
   specdev-analyzer plugin-lint --plugin <root>
   ```
   Must be clean for this bridge across:
   - `BRIDGE-LEX-001` — directory / `$id` / references lex-ordered.
   - `REF-COVERAGE-001` — each `reference_cases.yaml` name has a
     `reference/<lang>/src/<name>.rs`.
   - `REF-ARITY-001` — every generic type the README names is used at the same
     generic arity as in the compiled reference source.
   - `PROSE-PROFILE-001` — no profile name in the inlined `codegen/<lang>/README.md`.
   - bridge structural checks (schema standalone-compiles, `entities: []`, etc.).

   Surface any finding verbatim and fix the scaffold; re-run until clean.

3. **Schema sanity:** the `bindings.schema.yaml` must standalone-compile as
   JSON Schema 2020-12 (plugin-lint covers the structural checks; a malformed
   schema surfaces there).

## Step 5 — report + next steps

Report:
- Files written under `bridges/<a>+<b>/` (+ the reference crate).
- Reference `cargo build` result and the target.
- `plugin-lint` result (codes cleared).
- **Next:** in a project that has both blocks inlined, run
  `/spec-inline-bridge <a>+<b>`, then add a `bindings."<a>+<b>".<table>` table to
  a module and `/spec-validate`. To share the bridge, PR it into the canonical
  specdev plugin — it is then covered by the reference-crate CI gate.
- **Live smoke is next-session:** a freshly-authored skill or a bridge in the
  plugin tree is exercised end-to-end only after a plugin-enabled relaunch that
  loads it; note this rather than claiming an end-to-end pass this session.

## What this skill does NOT do

- **Author a block.** A bridge wires two *existing* blocks — hand-author the
  block first (a `/spec-create-block` sibling is planned). This skill refuses if
  either endpoint block is missing.
- **Inline the bridge.** Use `/spec-inline-bridge <a>+<b>`.
- **Author a project-local ad-hoc bridge.** Those live under
  `specs/specdev/reductions/bridges/local/<a>+<b>.yaml` + a matching inline
  schema, have a **reduced** shape (no `contract.yaml`, no lint, no reference
  crate), and are hand-authored per project. This skill authors *canonical*
  bridges (full 6-file + reference shape).
- **Multi-language in one invocation.** One language per run; re-invoke for a
  second language's codegen + reference.
- **Publish / register / version.** No registry or semver in v1 (versioning is
  plugin-wide; sharing is by upstream-PR or file-copy).
- **Run `/spec-gen` or fabricate the reference from memory.** The reference is
  authored against the real pinned API and compiled; the oracle-loop
  re-distillation from a real `/spec-gen` run is a separate follow-up pass.
