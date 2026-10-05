---
name: spec-create-bridge
description: This skill should be used when the user asks to "create a bridge", "author a new bridge", "scaffold a bridge", "write a bridge between <X> and <Y>", "wire two blocks with a bridge that doesn't exist yet", or "/spec-create-bridge <a>+<b>" — i.e. authoring a NEW canonical bridge unit (binding schema, reduction, contract, implementation guidance, README) between two blocks that already exist in the plugin tree. New bridges only — refuses if `bridges/<a>+<b>/` already exists. To inline an EXISTING bridge into a project, use /spec-inline-bridge. This skill authors bridges only, not blocks.
allowed-tools: Read, Write, Edit, Bash, Glob, Grep
argument-hint: <a>+<b> (lex-ordered block pair; e.g. embassy+stm32h5-rs)
---

# /spec-create-bridge — Author a new canonical bridge

Scaffold a complete **canonical bridge** between two blocks that already exist in the plugin tree: the binding table a module declares, the relation the reduction materializes from it (so the graph checks run over the wiring), the contract that shapes its diagnostics, and the implementation guidance for the code at the seam. A bridge is the place for everything that needs **both** blocks to state — the join-product crate, the rules of the handoff, the cross-block constraints neither block can see alone.

specdev does not generate code, so a bridge ships no code templates and no dependency list. Its guidance names the join-product crate; the project's manifest declares it.

**This skill authors; it does not inline.** After it lands, a project inlines the bridge with `/spec-inline-bridge <a>+<b>`.

User input: `$ARGUMENTS` — the bridge name `<a>+<b>` (lex-ordered, `a < b`). If empty, ask for the two blocks to wire before doing anything else.

If `specdev-analyzer` is not on PATH, tell the user to install it (follow `${CLAUDE_PLUGIN_ROOT}/shared/analyzer-install.md` — it derives the org from the plugin's own remote; both repositories are private) and stop. If it IS on PATH but reports `MANIFEST-002`, or a `MANIFEST-001` naming a file under the plugin tree, then the analyzer is older than the plugin — not a problem with any project file. Tell the user to re-run the install command with `--force` (or `cargo install --path <checkout> --force` from a local checkout), then retry.

Before any analyzer invocation, run the env preamble from `${CLAUDE_PLUGIN_ROOT}/shared/plugin-root-preamble.md`:

```sh
export SPECDEV_PLUGIN_ROOT="${CLAUDE_PLUGIN_ROOT}"
```

Skip if `SPECDEV_PLUGIN_ROOT` is already set or every invocation passes `--plugin-root`.

The exact file shapes this skill scaffolds are in `references/bridge-artifact-templates.md` — read it before Step 3. This body is the orchestration; that file is the material.

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
   stop and tell the user to author the missing block first, with
   `/spec-create-block <name>`. Do not
   scaffold a bridge to a nonexistent block — its `from_kind`/`to_kind` would
   reference entity kinds that don't exist.

6. **Language.** The two blocks share a framework language; the bridge's
   guidance is written for it. Read each block's `guidance.md` to confirm.

7. **Read the material.** Read `references/bridge-artifact-templates.md` in this
   skill — the exact file shapes plus the invariants each must satisfy. The
   shipped bridges are the worked precedents: `<root>/bridges/embassy+stm32h5-rs/`
   (intra-module, embedded), `<root>/bridges/egui+nih-plug/` (plugin shell, host)
   and `<root>/bridges/next-auth+prisma/` (named cross-module endpoint). Read the
   one whose shape matches this pairing end to end — all five files.

## Step 0.5 — does this bridge earn its keep?

Four tests, in order. Any one of them failing means **do not author a bridge** —
say so and stop. The cost of getting this wrong is asymmetric: an unnecessary
bridge is five files and a permanent maintenance surface, while a missing one is
a few lines someone writes by hand.

0. **Is the binding an edge at all? (cheapest — run it first.)** A bridge declares
   relations only (`entities: []`), and each binding is an edge between two entity
   kinds the endpoint blocks own. Three plausible shapes are not edges:
   - an **attribute** of one entity (a task's priority) bound to something that is
     not modelled as an entity (an interrupt vector);
   - a **cardinality** constraint ("at least as many X as Y") — no primitive
     expresses it, and the compiler usually does already;
   - one line of **glue** — test 2 says to hand-author it.

   "These two things must agree" is not automatically an edge. A threshold, a count
   or a single line gives the bridge nothing to declare; the rule belongs in the
   owning block's guidance (or, where an edge already exists, in a rule pack check
   over it, `related_attrs_allowed`).

1. **Endpoint resolvability (mechanical — a hard gate).**
   A bridge's bindings table lives on **one module**, and both endpoints must
   **resolve** against materialized entities. That is the requirement. It is
   *not* a requirement that both endpoints live on the binding's own module.

   Two shapes work, and they are gated differently:

   - **Intra-module** (every shipped bridge but `next-auth+prisma`):
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
   bridge cannot express the wiring. Stop: no bridge. Tests 2 and 3 help name which block owns the substantial half.

2. **Asymmetry.** Estimate each side's contribution. If one side is small enough
   to hand-author — on the order of a dozen lines of glue, with no rule worth
   stating — the pairing does not need a bridge. Push the substantial half **down into the block that owns it**
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

A bridge is defined by *what it wires* and *what rules the seam carries*. Batch these questions; show the precedent answers as defaults so a fluent author can accept them.

- **The join product.** Is there a crate that exists *only because these two blocks are paired* — one neither block may own without reaching across the pairing (e.g. `embassy-stm32` for `embassy+stm32h5-rs`, `nih_plug_egui` for `egui+nih-plug`)? Shared dependencies belong to the join, not to either block. The guidance names it and the rules for using it; the project declares it in its manifest. It must stay compatible with what both blocks' frameworks need — check the current releases (Step 2). There may be none: some bridges wire things both blocks already provide.

- **The binding table.** What does a module declare to invoke this wiring? Name the table (e.g. `peripheral_tasks`, `parameter_widgets`) and its per-entry fields. Two fields are almost always the endpoint references (e.g. `task`, `peripheral`); the rest are attributes that constrain the code at the seam (`handoff`, `widget_kind`, `range_hint`, …). Every field must constrain the code, as for a block (no types, no signatures).

- **The relation.** The reduction materializes one relation per binding entry:
  - `from_kind` / `to_kind` — **path-qualified** entity kinds, `<block-dir>.<Kind>`. ⚠️ **Hyphen-vs-underscore trap:** the qualifier is the block's **directory name**, which may be hyphenated (`nih-plug.Parameter`, `stm32h5-rs.Peripheral`) — **not** the schema table key, which is often the underscore form (`nih_plug`). The `for_each` path and the binding fields use the underscore table key (`bindings."<a>+<b>"` + `nih_plug.parameters[]`). Read each block's `reduction.yaml` for the exact `<Kind>`, and take the qualifier from the block's directory name.
  - direction — semantic; "X uses Y" reads as `X → Y`.
  - `extends` — a canonical relation, almost always `depends_on` (so the existing acyclic / layer / reachability graph checks run over the wiring with zero domain knowledge).
  - `attrs` — which binding fields become relation attributes.

- **binding_locality + diagnostic_template** (`contract.yaml`). `binding_locality` is a **closed 2-value enum** — pick by *where the bound entities live*, not by copying a precedent:
  - `endpoint_module` — both endpoints are ordinary per-module tables declared on the **same** module that carries the bindings (`embassy+stm32h5-rs`: a Task + a Peripheral on one module).
  - `plugin_shell_unique` — the binding targets the project's **single `plugin_shell`-layer module**; auto-resolves when exactly one such module exists (`egui+nih-plug`).
  The `diagnostic_template` is the `hint:` line appended to a REDUCE_UNRESOLVED finding; it may substitute `{bridge_name}`, `{parameter}`, `{decl_module}`, `{binding_module}`. Copy the matching precedent's template and adjust the wording.

- **Rules at the seam.** What must whoever writes the handoff code know? Ownership across the seam, which side builds what, the constraints between the two blocks' attributes that no single block can check (widget kind ↔ parameter type; comparator ↔ armed units). Each constraint a check can verify goes in the bridge's `checks.toml` as a rule named `<a>+<b>.<rule>` (`${CLAUDE_PLUGIN_ROOT}/shared/rule-packs.md`; `related_attrs_allowed` compares an attribute pair across the binding relation); the guidance names that rule id. These become `guidance.md`.

## Step 2 — verify the rules against the real API

Every guidance rule must be true for the current releases of both frameworks and the join product. Resolve each current release from the registry (crates.io `/api/v1/crates/<name>` with a `User-Agent` header, pub.dev `/api/packages/<name>`, npm `dist-tags.latest`) or the context7 docs tool, and check each rule that names an API behaviour against that release's documentation or source. Every handoff bug found while dogfooding the bridges was an API written from memory — a wrong generic arity, a constructor argument order, a widget that does not exist in the pinned crate.

If a registry or the docs are unreachable, say so and stop. Guidance carries **no version numbers**: name the behaviour, and tell the reader to check the version in the project's manifest where an API differs between releases.

## Step 3 — scaffold the bridge

Write the files under `<root>/bridges/<a>+<b>/`, following the exact shapes in `references/bridge-artifact-templates.md`:

| File | Notes |
|---|---|
| `bindings.schema.yaml` | `$id: https://specdev.dev/bridges/<a>+<b>.v1`. Contributes `properties.bindings."<a>+<b>"` (one namespace key), `additionalProperties: false` on the table body. Inline `Identifier` patterns — never `$ref` a profile's enums (bridges must compose against any profile carrying both blocks). |
| `reduction.yaml` | `metaschema_version: "v1"`. **`entities: []`** (the metaschema rejects bridge-qualified entities — endpoints live in the blocks). One relation per binding table, `extends: <canonical>`, path-qualified `from_kind`/`to_kind`, `for_each: 'bindings."<a>+<b>".<table>[]'` (quoted bridge segment), `endpoints` + `attrs`. |
| `contract.yaml` | `binding_locality` + `diagnostic_template`. The endpoint pair is encoded in the directory name — do not restate it. |
| `guidance.md` | The seam's rules, verified in Step 2: the join product, ownership, the cross-block constraints with their rule ids. Rules only: no fenced code and at most 4,096 bytes (`plugin-lint` checks these), no version numbers (review checks this). May name the two constituent *blocks* (that is the composition seam) but writes "the project's profile", never a profile name (`PROSE-PROFILE-001`). |
| `README.md` | Author-facing catalog doc (never inlined): When-to-use, What it contributes / does NOT contribute, the earn-its-keep rationale, the co-residency rule, deferred items. Naming a profile HERE is fine. |

## Step 4 — gate at authoring time

1. **Run plugin-lint:**
   ```sh
   specdev-analyzer plugin-lint --plugin <root>
   ```
   Must be clean for this bridge across:
   - `BRIDGE-LEX-001` — the directory name is lex-ordered.
   - `GUIDANCE-001/002/003` — `guidance.md` present, no fenced code, under the cap.
   - `PROSE-PROFILE-001` — no profile name in `guidance.md`.
   - `PACK-LINT-001..005` — `checks.toml` passes its schema, each query compiles and holds the captures its check names, each primitive is known, each rule carries the `<a>+<b>.` prefix.

   `plugin-lint` does not compile a bridge's schema or reduction; Step 4.2 proves both, so do not skip it.

   Surface any finding verbatim and fix the scaffold; run again until clean.

2. **Prove the relation materializes.** A silent `validate` does not prove an edge exists. In a scratch project with both blocks and the bridge inlined (`/spec-init`, `/spec-inline <a>`, `/spec-inline <b>`, `/spec-inline-bridge <a>+<b>`), author a module with one binding entry (and `planned = true`, since it has no code) and run `specdev-analyzer reduce`: the relation must appear with the expected `from` and `to`. Then break one endpoint name and confirm `check` reports `REDUCE_UNRESOLVED` with the contract's hint.

## Step 5 — report + next steps

Report:
- Files written under `bridges/<a>+<b>/`.
- `plugin-lint` result (codes cleared) and the `reduce` proof (the edge as dumped).
- **Next:** in a project that has both blocks inlined, run `/spec-inline-bridge <a>+<b>`, then add a `bindings."<a>+<b>".<table>` table to a module and `/spec-validate`. To share the bridge, PR it into the canonical specdev plugin.
- **Live smoke is next-session:** a freshly-authored skill or a bridge in the plugin tree is exercised end-to-end only after a plugin-enabled relaunch that loads it; note this rather than claiming an end-to-end pass this session.

## What this skill does NOT do

- **Author a block.** A bridge wires two *existing* blocks — author the missing one first with `/spec-create-block <name>`. This skill refuses if either endpoint block is missing.
- **Inline the bridge.** Use `/spec-inline-bridge <a>+<b>`.
- **Author a project-local ad-hoc bridge.** Those live under `specs/specdev/reductions/bridges/local/<a>+<b>.yaml` + a matching inline schema, have a **reduced** shape (no `contract.yaml`, no guidance, no lint), and are hand-authored per project. This skill authors *canonical* bridges.
- **Ship code, code templates or a dependency list.** The code is written by hand against the guidance; the project's manifest owns dependencies.
- **Publish / register / version.** No registry or semver in v1 (versioning is plugin-wide; sharing is by upstream-PR or file-copy).
- **Write rules from memory.** Verify each rule against the current releases (Step 2).
