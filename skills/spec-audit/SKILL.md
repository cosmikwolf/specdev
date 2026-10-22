---
name: spec-audit
description: This skill should be used when the user asks to "audit <ModuleName>", "run spec-audit", "does the code do what the spec says", "does this module meet its requirements", "check the requirement prose against the code", "is this really isr_safe", "is the blocking flag right", "check the metadata against the code", or "review a module against its spec" — i.e. they want an agent to read one module's code against the claims no analyzer check can read — requirement text, descriptions, `isr_safe`, `blocking` and `metadata.*`. Read-only — it reports in chat and edits nothing. One module per run. For what the analyzer can prove (source exists, imports, tests exist, rule-pack entities), use /spec-validate. For a module's `[[items]]` surface, use /spec-sync-items.
allowed-tools: Read, Bash, Glob, Grep
---

# /spec-audit — Read one module's code against its claims

`/spec-validate` proves what extraction can see: the source exists, imports follow `depends_on`, the named tests exist, rule-pack entities match the code. It cannot read meaning. A requirement that says "the flash task fills the bitmap at boot" passes `check` as long as some test is named and some code exists. This skill is the reader: it takes one module, lists every claim no check reads, reads the code, and gives each claim a verdict with evidence.

It is a report. It **edits nothing and writes no file** — not the spec, not the code, not a report file. It runs no build and no test either (both write to the work tree): it reads. When only a test's result would settle a claim, the claim is `not shown`, and the report names the test to run. The user decides what to do with each finding, and records it through the skills that own that state.

Run it at milestones: after a module lands, before a release, after a large change to a module's code.

If `specdev-analyzer` is not on PATH, tell the user to install it (follow `${CLAUDE_PLUGIN_ROOT}/shared/analyzer-install.md` — it derives the org from the plugin's own remote; both repositories are private) and stop. If it IS on PATH but reports `MANIFEST-002`, or a `MANIFEST-001` naming a file under the plugin tree, then the analyzer is older than the plugin — not a problem with any project file. Tell the user to re-run the install command with `--force` (or `cargo install --path <checkout> --force` from a local checkout), then retry.

Before any analyzer invocation, run the env preamble from `${CLAUDE_PLUGIN_ROOT}/shared/plugin-root-preamble.md`:

```sh
export SPECDEV_PLUGIN_ROOT="${CLAUDE_PLUGIN_ROOT}"
```

Skip if `SPECDEV_PLUGIN_ROOT` is already set or every invocation passes `--plugin-root`.

---

## Step 1 — pick the module

The argument is one module name, matching `specs/modules/<Module>.toml`. If none is given, ask for one; list the files under `specs/modules/` to help. Do not audit more than one module in a run — one module's claims, code and tests fit in one context; a whole tree does not.

**The project** below means the whole repository the module lives in, from the workspace root down: the sibling members, path dependencies, and the captures and reference material under the root (the sources in `specs/sources/` say where a reference lives) count as project content. Code outside it is what a package registry or vendored SDK supplies.

In a workspace, run every command from the **member** directory that holds the module, not the workspace root.

Read `specs/modules/<Module>.toml`. If it has `planned = true`, the module is not built: stop, and say so in one line. Otherwise run, and keep the output of, these three:

```sh
specdev-analyzer context --module <Module>        # profile, schema files, guidance files
specdev-analyzer extract-items --module <Module> | grep source_file
specdev-analyzer check --no-color
```

- `context`: `profile:` names the module's profile (a workspace root's profile is not the member's); `files.schema` and `files.guidance` are read in Steps 2 and 3.
- `extract-items`: `source_file` is the path the analyzer resolves (write it in the report as the spec does, without a leading `./`). `source_file: null` means the source does not resolve: stop and route to `/spec-validate` (the `source_resolves` finding names the fix).
- `check`: of the info lines, only `PLUGIN-AHEAD-001` bears on the audit (Step 5, Notes); ignore the rest. If an error finding names this module's spec file or source file, show those lines verbatim and say that `check` proves presence and the audit judges meaning — the presence errors come first. Ask whether to continue. With no such error, continue without asking.

## Step 2 — list the claims

Number the claims `C1`, `C2`, … in this order. Each claim records where it lives (file and key) and its exact text or value.

1. **Requirements.** Every id in the module's `satisfies`, and in each `[[items]]` entry's `satisfies` — one claim per id, however many places name it. Read each requirement's `text` from `specs/requirements/<id>.toml` (one file per id; a `specs/requirements.toml` that says `MOVED` is a stub from an older format — ignore it). Run `specdev-analyzer report requirements` once (it covers every requirement; read it as YAML — an entry's `children:` block can print before its `id:` — and pick out this module's ids): `satisfied_by` lists every module that satisfies the id, this one included, and `verified_by` lists its tests (`test_pending: true` means none are written yet). A requirement that other modules satisfy too is judged for **this module's part**: the behaviour this module's code does, including the calls it makes into other modules for it, and the code that applies the commands or values it returns (Step 3) — that code may live in a module that is not in `satisfied_by`. Write the other modules in the claim's Where cell (`shared with Storage, LedTask`); the behaviour they do on their own paths is theirs.
2. **Descriptions.** The module's `description`, and every other `description` in the module file (actions, ports, state machines, errors, and each `[[items]]` entry). One description is one claim. A clause that gives only history or a motive ("matched 2026-09-14", "after the stack overflowed") is not a statement about the code. An effect the code is said to have is a statement, even in a "so …" clause ("so no temporary lands on the stack"). In a clause that mixes both, judge the statement part. The source rule proves the code exists; nothing checks what it does.
3. **Action parameters.** On a profile with actions, one claim per action: the names in its `params`, and each `direction`, against the code's parameters. (`[[items]]` names and kinds are not claims here: `items diff` and `/spec-sync-items` own the items surface.)
4. **Fields no check reads.** On a `firmware-layered` module: `isr_safe = true` and `blocking = false` on each action, then every field under `[metadata.<Category>]`, one claim per field, in file order. On any profile: every other module field that says something about the code (for example `visibility`; not `context_files`, which lists reading for agents). `check` reads the code for these only: source resolution, imports against `depends_on`, `verified_by` tests, the open-question gate, and the rule packs' `[[code_checks]]`; everything else that describes the code is a claim. Skip `isr_safe = false` and `blocking = true`: they promise nothing, so they cannot be wrong; count them for the heading line.
5. **Unit tables no code check reads.** A rule pack's code checks compare some unit tables with the code. For each unit whose table the module uses (`[[<unit>.<table>]]`), find the table's entity kind: `specs/specdev/reductions/{blocks,bridges}/<unit>.yaml` has an entry with `for_each: "<unit>.<table>[]"` and its `kind`; the full kind is `<unit>.<kind>`. Then open `specs/specdev/checks/{blocks,bridges}/<unit>.toml` (a folder or file that does not exist means the unit has no checks). Each `[[code_checks]]` entry names the `entity_kind` it covers. (`check` prints each pack's counts as `PACK-000 … (<g> graph check(s), <c> code check(s))`; `0 code check(s)` means none.) Every entry of a table whose kind **no** `[[code_checks]]` entry covers is a claim — one claim per entry (for example a DMA channel the code must take). A pack with only `[[graph_checks]]` checks the spec against itself, not against the code.

Get each field's meaning in this order:

1. the `description` of the field — or of the `$defs` type it `$ref`s — in the project's inlined schemas under `specs/specdev/schemas/`, when it defines what the field or its values mean (a one-line label such as "Direction of an action parameter." does not; go on to 2). Follow each `$ref`: `context`'s `files.schema` list omits the files reached by `$ref` (for example `metadata.schema.yaml`, `enums.schema.yaml` and the language items schema). JSON Schema gives an enum value no description of its own; a unit table's meaning is in the unit's inlined schema and guidance;
2. when the schema has none: `references/claim-meanings.md`, for the fields and values the plugin ships;
3. when neither has it: the plain meaning of the name.

Say in the report's Notes which source each group of meanings came from.

Do **not** list what a check already covers: `[[items]]` names and kinds (`items diff`), `name`, `layer`, `source_file`, `planned`, `guided_by`, `ports` endpoints, `depends_on` (but only partly: `check` reports a `use` import or a qualified path such as `crate::led::X` with no `depends_on` entry; it does not see a path inside a macro's arguments, and it does not report a `depends_on` entry with no use — both go under "Other findings"), and the unit-table entries a `[[code_checks]]` entry covers. `/spec-validate` owns those.

Audit every claim without asking. Only when a module has more than about 40 claims, or its source passes about 2,000 lines before its test code (the first `#[cfg(test)]` or equivalent), show the counts and ask whether to audit all of them or a subset (requirements first is a good default).

## Step 3 — read the code

Read, in this order, only as far as each claim needs:

1. The source file, whole, except its test code (`#[cfg(test)]` modules and the like), which you read only as tests. For a `mod.rs` module, the directory it heads.
2. The guidance files `context` listed (`files.guidance`), when a claim uses a project term: they define the project's terms (what "the ISR" or "the audio thread" is in this project). Guidance says what the code *should* do; it is never evidence of what the code *does*.
3. The code this module calls in other project modules, when a claim depends on what the call does. Follow the call as deep as the claim needs, and stop at code outside the project: there, trust its documented behaviour and say so. A claim that the code matches a reference that is not in the repository ("as the C driver does") is `not shown`, unless a test or a capture in the repository compares the two (then judge the cases it covers); name what would settle it. A reference cited as a source, when the text also states the value or behaviour ("C DOUBLECLICKMS = 250"), is provenance: judge the stated value. A caller or collaborator the text names that is not in the repository is `not shown`. A neighbour's **spec** is a hint, not evidence: when its flags disagree with its code, judge by the code and report the neighbour's spec under "Other findings".
4. The tests in each requirement's `verified_by`, when any requirement has one. `specdev-analyzer report tests` lists every test as a YAML entry with `identity:` and `file:`; match the identity and, when two entries share it, the `file:` too. Read the test body. A test that drives the whole program (a device or bench test) exercises this module's part when its assertion observes that behaviour. A test anywhere in the repository that exercises this module but that no `verified_by` names goes under "Other findings" when it would cover one of the module's requirements.

`specdev-analyzer report decisions --module <Module>` lists the decisions that reach the module (the `modules:` block; the `decisions:` list after it is every decision in the project). Read one when a claim's verdict depends on it, and read every decision a claim's text cites by id, whether or not the command lists it. A decision whose text this module's code contradicts goes under "Other findings", whether or not it is in `guided_by`. Workspace-tier decisions (`via: workspace`) matter only when their text names this module or what it does.

## Step 4 — judge each claim

Each claim gets exactly one verdict:

| Verdict | Meaning |
|---|---|
| `holds` | The code does what the claim says, and you can point at the lines that show it. |
| `contradicted` | The code does something the claim excludes, or lacks something the claim says it has. Point at the lines. |
| `not shown` | The code alone cannot settle it: a timing budget, a property of hardware, behaviour in code outside the project. Say what would settle it (a measurement, a test). |

Rules:

- **Every verdict cites evidence.** Code: `path:line` and the line or lines, quoted short. Absence: the search and its result (`grep -rn PATTERN_BUF src/` — 0 hits). A computation over cited code ("565→888→565 is exact for all 65536 values") counts when you show the code and how you computed it. No evidence, no verdict.
- **Absence of a contradiction is not `holds`.** If you cannot find the code that does the thing, the verdict is `not shown` — or `contradicted`, when the claim says the code does it and the place it would be does something else.
- **A claim with several statements takes the verdict of its worst statement:** `contradicted` when one is false, else `not shown` when one is unsettled, else `holds`. Count them in the verdict — `contradicted (k false)`, `contradicted (k false, m not shown)`, `not shown (m)` — and in the paragraph quote each false and each unsettled statement and say the rest hold. In a list field (`shared_state`, `params`), each stale or missing entry is one false statement.
- **A requirement that names a mechanism states it.** "Yields around the predicted deadline" is false when the code reaches the effect another way (fixed-size chunks); the effect itself ("output is not delayed") is a separate statement, usually `not shown`.
- **A general statement covers every case the text does not limit.** "A copy marks the destination saved" covers a copy from an empty source; if that case clears the destination instead, the statement is false for it (and the hand-off is usually to edit the text).
- **What the code does beyond the claim is not a contradiction.** A description that names 8 of 13 commands is not false about the other 5. Note the omission under "Other findings" when it matters to a reader of the spec.
- **A flag is a claim about the action's own code, on every path.** `isr_safe = true` or `blocking = false` holds only when no path in the action does what the flag excludes — including a path today's callers do not reach, and a wait whose condition is false today (both can change). One such path is `contradicted`; cite it, and say why it does not fire today. Code outside the repository is trusted as documented even when its source is on disk.
- **An `.await` is a wait for `blocking` only when the awaited code does not yield.** Awaiting a future that yields does not block the caller, whatever the callee's spec says. Follow an awaited project `async fn` far enough to see that it yields; a busy-wait inside it blocks. A synchronous call blocks when the callee's code waits.
- **A field with one value is judged against the whole module.** When the code uses a mix no single value expresses (a critical section for some state, atomics for other state), the verdict is `contradicted`, and the hand-off is `/spec-question`: the schema, not the value, may be what needs to change. When one other value fits the code, the hand-off is `/spec-edit` with that value. When a decision explains why the code differs from the spec, the spec is the side to change.
- **Requirements carry their tests.** For each `verified_by` test: `exercises` (the test drives the behaviour the text names and asserts its result), `exercises part` (name the statements it leaves out), or `does not exercise` (it runs, but would pass if the behaviour were wrong — cite the assertion). Write them in the Verdict cell, after the verdict. For `test_pending: true`: `no test`.

## Step 5 — report in chat

The heading line:

```
/spec-audit <Module> — <source file> — <N> claims: <h> holds, <c> contradicted (<f> false statements), <s> not shown (<k> flags skipped)
```

Drop the last parenthesis when no flag was skipped. The false-statement count keeps one long description with many defects from reading as one defect.

Then **Notes**, one line each: where the field meanings came from (Step 2); `PLUGIN-AHEAD-001`, when `check` printed it (the inlined schemas are older than the plugin's; the audit uses the project's copies as they are, and `/spec-drift` shows the difference); a subset audited, when it was.

Then one table, in claim order:

| Claim | Where | Verdict | Evidence |
|---|---|---|---|
| C1 `REQ-012` | `specs/requirements/REQ-012.toml` text; shared with `Index` | holds · tests: no test | `src/store.rs:88` `self.scan_headers()?;` |
| C7 `read_block.blocking` | `specs/modules/Store.toml` action `read_block` | contradicted | `src/store.rs:131` `while !spi.done() {}` |

Keep each Evidence cell to the one or two citations that decide the verdict; the rest goes in the paragraph.

Then, for each `contradicted` and `not shown` claim, a short paragraph: what the claim says, what the code does, the quoted lines, and the hand-off below. `holds` claims need no paragraph, except a short test list when a requirement names more than three tests. When one cause makes several claims false (a removed feature named in a requirement, a description and a param list), say "same cause as C<n>" in the later paragraphs instead of repeating the evidence, so the reader sees one defect, not several.

Then **Other findings**: what you found while reading that is not one of this module's claims — a decision whose text disagrees with the code, a neighbour's spec that disagrees with its code, a `depends_on` entry the code never uses, behaviour the spec omits that a reader needs. One line each, with evidence and a hand-off. These get no verdict and do not enter the counts.

Hand-offs — name one per finding, and do not do it:

- **The spec is wrong, the code is right** → `/spec-edit <Module>` (any field, description, unit-table entry, or a `depends_on` entry the code never uses), or edit the requirement file.
- **The code is wrong** → a code change; the user decides. `/spec-validate` after it.
- **The requirement is moot** → `/spec-decide` (it can retire the requirement).
- **A decision's text is wrong** → `/spec-decide` (a successor that supersedes it).
- **Nobody knows which is right**, or the schema cannot express the code → `/spec-question` to record an open question.
- **A test does not exercise its requirement** → write a test that does; until then, `test_pending = true` on the requirement is the truthful state.
- **A test exercises part** → add tests for the rest, or link existing ones in `verified_by` (edit the requirement file).

End with one line: the verdict counts, and "Nothing was changed."

## Additional resources

- `references/claim-meanings.md` — what `isr_safe`, `blocking`, action `params` and each `metadata.*` field claim about the code, and what evidence settles each.
