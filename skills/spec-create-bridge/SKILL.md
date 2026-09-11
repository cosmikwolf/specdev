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
(follow `${CLAUDE_PLUGIN_ROOT}/shared/analyzer-install.md` — it derives the org from the plugin's own remote; both repositories are private) and stop. If it IS on PATH but reports `MANIFEST-002`, or a `MANIFEST-001` naming a file under the plugin tree, then the analyzer is older than the plugin — not a problem with any project file. Tell the user to re-run the install command with `--force` (or `cargo install --path <checkout> --force` from a local checkout), then retry.

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

## Step 0.5 — does this bridge earn its keep?

Three tests, in order. Any one of them failing means **do not author a bridge** —
say so and stop. The cost of getting this wrong is asymmetric: an unnecessary
bridge is six files, a reference crate, and a permanent maintenance surface,
while a missing one is a few lines someone writes by hand.

1. **Endpoint resolvability (mechanical — check this first, it is a hard gate).**
   A bridge's bindings table lives on **one module**, and both endpoints must
   **resolve** against materialized entities. That is the requirement. It is
   *not* a requirement that both endpoints live on the binding's own module.

   Two shapes work, and they are gated differently:

   - **Intra-module** (all five bridges shipped before `next-auth+prisma`):
     the reduction qualifies both endpoints with `${root.name}`, so both
     declarations must sit on the module carrying the table. Ask the user where
     the two things live; if they are on different modules, this shape cannot
     express it.
   - **Named cross-module**: one endpoint is `${root.name}.<...>` and the other
     is a **name the binding itself carries**, resolved against another module's
     entity. `bridges/next-auth+prisma` is the worked precedent — an Auth.js
     config module binding to whichever module owns the Prisma datasource.
     Verified by dumping `specdev-analyzer reduce`, not by a silent `validate`:
     the edge materializes with `from: Auth.auth` (module `Auth`) and
     `to: Db.datasource` (module `Db`), and a wrong name gives an ordinary
     `REDUCE_UNRESOLVED` with the contract's hint rendered.

   **The real trap is not locality, it is which field `for_each` selects.** If
   `for_each` selects the binding OBJECT and an endpoint template reads a CHILD
   field of it, then omitting that field aborts the reducer —
   `template path ... resolved to nothing`, a plugin-author message, exit 2 —
   **even when the field is `required`**, because reduction runs before schema
   enforcement. Isolated on one file during `next-auth+prisma`'s build: omitting
   a required field read only as an `attrs` selector gave a clean SCHEMA error at
   exit 1; omitting a required field read by the endpoint template crashed. So
   **select the naming field itself** (`for_each: 'bindings."<a>+<b>".client'`,
   endpoint `${.}`), which makes the crash unreachable — an absent field yields
   no pick, no iteration, and the author gets the ordinary "required property"
   error. This is the same construction that makes the `prisma` block's own
   relation safe.

   `binding_locality` changes none of this. That field selects a diagnostic
   template and is otherwise inert — it has zero consumers in the analyzer's
   `src/`, measured. Neither of its two values describes a named cross-module
   endpoint; pick the closer one and say so in `contract.yaml`.

   If neither shape fits — the two things are in **different crates**, say — a
   bridge cannot express the wiring. Stop and use test 2 or 3 instead.

2. **Asymmetry.** Estimate each side's contribution. If one side is small enough
   to hand-author — on the order of a dozen lines of glue — the pairing does not
   need a bridge. Push the substantial half **down into the block that owns it**
   and leave the thin half to be hand-authored. `blocks/iroh/README.md`'s
   "Why the channel surface lives in this block" section is the worked precedent:
   the endpoint runtime and its channel surface are substantial and belong to the
   transport block; whatever drains the event channel is a handful of
   consumer-specific lines and belongs to nobody.

3. **Cross-cutting, or single-block in disguise?** Read the heavy half's
   description back. If it names **no concept from the other block**, it is not
   cross-cutting — it only looked that way because a consumer was in view while
   describing it. It belongs in its own block. A real bridge's contribution
   cannot be stated without naming both sides.

When a pairing fails these tests, the honest report is: "the substantial half is
`<X>`, which is a `<block>` concern; author or extend that block, and hand-author
the `<N>`-line consumer side. No bridge." That is a successful outcome for this
skill, not a refusal.

## Step 1 — interview the bridge's semantics

A bridge is defined by *what it wires* and *what join-product it owns*. Batch
these questions; show the precedent answers as defaults so a fluent author can
accept them.

- **The join-product crate(s).** Which crate exists *only because these two
  blocks are paired* — one neither block may own without reaching across the
  pairing (e.g. `embassy-stm32` for `embassy+stm32h5-rs`)? Capture its name and
  features. There may be none (some bridges only wire types both blocks already
  provide) — then `deps.yaml` is omitted.

  ⚠️ **Resolve the version from the registry; never from memory.** Query
  crates.io (`/api/v1/crates/<name>`) or the context7 docs tool for the current
  release and pin that. Step 2's API research verifies everything *against* the
  pin, so a recalled pin anchors all of it to a guess. For a git source, pin a
  `rev` or `tag` — a bare repo URL means projects track a moving HEAD while the
  reference proves one commit. If the registry is unreachable, stop rather than
  guess.

  A join product is the **most** exposed crate in the system: by construction no
  block owns it, so nothing else in the tree is watching it. It must also stay
  compatible with what *both* endpoint blocks pin — check their `deps.yaml`
  versions before choosing, and make the reference crate build the whole set
  together. `plugin-lint` compares your pins against the reference
  (`REF-PIN-001` / `REF-PIN-002`).

- **The binding table.** What does a module declare to invoke this wiring? Name
  the table (e.g. `peripheral_tasks`, `parameter_widgets`) and its per-entry
  fields. Two fields are almost always the endpoint references (e.g. `task`,
  `peripheral`); the rest are attributes carried into codegen (`handoff`,
  `widget_kind`, `range_hint`, …). v1 bindings are **co-resident** (intra-module)
  by default — both endpoints declared on the same module that carries the
  table.

- **The relation.** The reduction materializes one relation per binding entry:
  - `from_kind` / `to_kind` — **path-qualified** entity kinds, `<block-dir>.<Kind>`.
    ⚠️ **Hyphen-vs-underscore trap:** the qualifier is the block's **directory
    name**, which may be hyphenated (`nih-plug.Parameter`, `stm32h5-rs.Peripheral`) —
    **not** the schema table key, which is often the underscore form (`nih_plug`).
    The entity qualifier uses the hyphenated dir name; the `for_each` path and the
    binding fields (below) use the underscore table key
    (`bindings."<a>+<b>"` + `nih_plug.parameters[]`). Read each block's
    `reduction.yaml` for the exact `<Kind>` (e.g. `Parameter`, `Task`) — the
    reduction gives you the kind but *not* the qualifier, so take the qualifier
    from the block's directory name.
  - direction — semantic; "X uses Y" reads as `X → Y`.
  - `extends` — a canonical relation, almost always `depends_on` (so the
    existing acyclic / layer / reachability graph-checks run over the wiring
    with zero domain knowledge).
  - `attrs` — which binding fields become relation attributes.

- **binding_locality + diagnostic_template** (`contract.yaml`). `binding_locality`
  is a **closed 2-value enum** — pick by *where the bound entities live*, not by
  copying a precedent (the two shipped bridges legitimately differ):
  - `endpoint_module` — both endpoints are ordinary per-module tables declared on
    the **same** module that carries the bindings (intra-module co-residency). The
    common case (`embassy+stm32h5-rs`: a Task + a Peripheral on one module).
  - `plugin_shell_unique` — the binding targets the project's **single
    `plugin_shell`-layer module**; auto-resolves when exactly one such module
    exists (`egui+nih-plug`: a Region → a Parameter, both on the plugin shell).
  The `diagnostic_template` is the `hint:` line appended to a REDUCE_UNRESOLVED
  finding; it may substitute `{bridge_name}`, `{parameter}`, `{decl_module}`,
  `{binding_module}`. Choose the value that matches your wiring, then copy **that**
  precedent's template and adjust the wording (an `endpoint_module` template and a
  `plugin_shell_unique` template read very differently).

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
  Add a `target = <triple>` **only if a dep forces it** (a chip-specific HAL that
  won't build for the host); a portable `no_std` pairing type-checks on the host
  with no triple — `#![no_std]` is independent of the target, so do not assume
  `no_std` ⇒ cross-target. A coherent bridge's two endpoint blocks **share** a
  nature (both embedded, or both host). If they *disagree* (one embedded, one host), the reference target is
  ambiguous **because the pairing itself is almost certainly incoherent** — treat
  that as a sharp-test signal to reconsider whether this bridge should exist,
  rather than picking a nature by fiat.

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
   - `REF-ARITY-001` — matched **by base type name**: for each base name in the
     reference source (`I2c`, `Vec`, …), every occurrence in the README must use
     the same number of generic args (only names present in the reference are
     checked, so README-only types never false-flag). Keep the type text identical
     between README and reference source.
   - `PROSE-PROFILE-001` — no profile name in the inlined `codegen/<lang>/README.md`.
   - bridge structural checks (schema standalone-compiles, `entities: []`, etc.).

   `REF-ARITY-001` is a README↔reference **consistency** check only: it confirms
   the two *agree* on generic arity, not that the types are real. The **Step 4.1
   compile is the correctness gate** — an internally-consistent but wrong
   reference passes arity yet fails the build. Never read a clean `REF-ARITY-001`
   as "the API is correct"; that is what the compile is for.

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
