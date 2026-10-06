---
name: spec-audit
description: This skill should be used when the user asks to "audit <ModuleName>", "run spec-audit", "does the code do what the spec says", "does this module meet its requirements", "check the requirement prose against the code", "is this really isr_safe", "is the blocking flag right", "check the metadata against the code", or "review a module against its spec" — i.e. they want an agent to read one module's code against the claims no analyzer check can read: requirement text, descriptions, `isr_safe`, `blocking` and `metadata.*`. Read-only: it reports in chat and edits nothing. One module per run. For what the analyzer can prove (source exists, imports, tests exist, rule-pack entities), use /spec-validate. For a module's `[[items]]` surface, use /spec-sync-items.
allowed-tools: Read, Bash, Glob, Grep
---

# /spec-audit — Read one module's code against its claims

`/spec-validate` proves what extraction can see: the source exists, imports follow `depends_on`, the named tests exist, rule-pack entities match the code. It cannot read meaning. A requirement that says "the flash task fills the bitmap at boot" passes `check` as long as some test is named and some code exists. This skill is the reader: it takes one module, lists every claim no check reads, reads the code, and gives each claim a verdict with evidence.

It is a report. It **edits nothing and writes no file** — not the spec, not the code, not a report file. The user decides what to do with each finding, and records it through the skills that own that state.

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

In a workspace, run from the **member** directory that holds the module, not the workspace root.

Read `specs/modules/<Module>.toml`. Then:

- `planned = true` — the module is not built. Stop: there is no code to read. Say so in one line.
- Otherwise find the source:

  ```sh
  specdev-analyzer extract-items --module <Module>
  ```

  Its `source_file` is the path the analyzer resolves. `source_file: null` means the source does not resolve: stop and route to `/spec-validate` (the `source_resolves` finding names the fix).

Then run the analyzer's own checks once:

```sh
specdev-analyzer check --no-color
```

If any error finding names this module's spec file or source file, show those lines verbatim and say that `check` proves presence and the audit judges meaning — the presence errors come first. Ask whether to continue. Continue without asking when the module has no error finding.

## Step 2 — list the claims

Number the claims `C1`, `C2`, … in this order. Each claim records where it lives (file and key) and its exact text or value.

1. **Requirements.** Every id in the module's `satisfies`, and in each `[[items]]` entry's `satisfies`. Read the requirement's `text` from `specs/requirements/<id>.toml`. Run `specdev-analyzer report requirements` once: for each id, `satisfied_by` lists the other modules that share it, and `verified_by` lists its tests (`test_pending: true` means none are written yet). A requirement shared by several modules is judged only for **this module's part** — name the other modules, and do not read their code to finish the judgement.
2. **Descriptions.** The module's `description`, and every other `description` in the module file (actions, ports, state machines, errors). The source rule proves the code exists; nothing checks what it does.
3. **Fields no check reads.** On a `firmware-layered` module: `isr_safe = true` and `blocking = false` on each action, and every field under `[metadata.<Category>]`. On any profile: the names in each action's `params` (the code's parameters must carry them), and any other module field whose meaning is a property of the code and that no check reads. Skip `isr_safe = false` and `blocking = true`: they promise nothing, so they cannot be wrong — say once how many you skipped. Get each field's meaning in this order:
   1. the `description` of the field, or of its enum value, in the project's inlined schemas (`specdev-analyzer context --module <Module>` lists them under `files.schema`);
   2. `references/claim-meanings.md`, for the fields the plugin ships;
   3. the plain meaning of the name — and say in the report that no definition exists.

Do **not** list what a check already covers: `name`, `layer`, `depends_on`, `source_file`, `planned`, `guided_by`, `ports` endpoints, and the rule-pack tables (`[[rtic.*]]`, `[[mk20dx.*]]`, bindings and the other inlined unit tables). `/spec-validate` owns those.

Show the claim list to the user before Step 3, with the count. For a module with more than about 40 claims, ask whether to audit all of them or a subset (requirements first is a good default).

## Step 3 — read the code

Read, in this order, only as far as each claim needs:

1. The source file, whole. For a `mod.rs` module, the directory it heads.
2. The guidance files `context` lists (`files.guidance`): they define the project's terms (what "the ISR" or "the audio thread" is in this project).
3. The code this module calls in other project modules — only to settle a claim. A call into a module that this one does not list in `depends_on` is not this skill's finding; `check` reports it.
4. The tests in each requirement's `verified_by`. `specdev-analyzer report tests` maps each test identity to its file; read the test body.

Decisions in `guided_by` can explain why the code looks the way it does. `specdev-analyzer report decisions --module <Module>` lists them; read one when a claim's verdict depends on it.

## Step 4 — judge each claim

Each claim gets exactly one verdict:

| Verdict | Meaning |
|---|---|
| `holds` | The code does what the claim says, and you can point at the lines that show it. |
| `contradicted` | The code does something the claim excludes. Point at the lines. |
| `not shown` | The code alone cannot settle it: a timing budget, behaviour that depends on another module's part, a property of hardware. Say what would settle it (a measurement, a test, another module's audit). |

Rules:

- **Every verdict cites `path:line`** and quotes the line or lines, short. No evidence, no verdict.
- **Absence of a contradiction is not `holds`.** If you cannot find the code that does the thing, the verdict is `not shown` — or `contradicted`, when the claim says the code does it and the place it would be does something else.
- **A flag is a claim about every path.** `isr_safe = true` or `blocking = false` holds only when no path from the action does what the flag excludes. One path that does is `contradicted`; cite it.
- **Requirements get a second line: the tests.** For each `verified_by` test: `exercises` (the test drives the behaviour the text names and asserts its result) or `does not exercise` (it runs, but would pass if the behaviour were wrong — cite the assertion). For `test_pending = true`: `no test`.
- **Judge the text as written.** A requirement that says more than the code does is `contradicted` or `not shown` for that part, even when the extra part looks like a wording slip; say which part.

## Step 5 — report in chat

One heading line: module, source file, claim count, and the verdict counts.

Then one table, in claim order:

| Claim | Where | Verdict | Evidence |
|---|---|---|---|
| C1 `REQ-012` | `specs/requirements/REQ-012.toml` text | holds · tests: no test | `src/store.rs:88` `self.scan_headers()?;` |
| C7 `read_block.blocking` | `specs/modules/Store.toml` action `read_block` | contradicted | `src/store.rs:131` `while !spi.done() {}` |

Then, for each `contradicted` and `not shown` claim, a short paragraph: what the claim says, what the code does, the quoted lines, and the hand-off below. `holds` claims need no paragraph.

Hand-offs — name one per finding, and do not do it:

- **The spec is wrong, the code is right** → `/spec-edit <Module>` (a field or description), or edit the requirement file.
- **The code is wrong** → a code change; the user decides. `/spec-validate` after it.
- **The requirement is moot** → `/spec-decide` (it can retire the requirement).
- **Nobody knows which is right** → `/spec-question` to record an open question.
- **A test does not exercise its requirement** → write a test that does; until then, `test_pending = true` on the requirement is the truthful state.

End with one line: the verdict counts, and "Nothing was changed."

## Additional resources

- `references/claim-meanings.md` — what `isr_safe`, `blocking` and each `metadata.*` category claim about the code, and what evidence settles each.
