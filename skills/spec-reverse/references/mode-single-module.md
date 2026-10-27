# spec-reverse — single-module mode

Entered from the main SKILL.md when `$ARGUMENTS` is a bare module name (not `--all`). Produces one module TOML under `specs/modules/`, and the requirements the user confirms for it.

## Single-module reverse

Entered after the main SKILL.md's Steps 0–2 have run (env preamble, `analyzer reverse` pre-flight, authoritative guides read).

1. Read `specs/architecture.toml` when it exists. If it is missing, stop only when the profile's module schema requires a field whose value names an architecture entry (`layer` names one); then say *"Run `/spec-reverse` (no args) first to produce architecture."* Otherwise continue without it — `validate` reports a missing `architecture.toml` as info (`LAYOUT-005`) only.
2. Read every existing `specs/modules/**/*.toml` for cross-module context.
3. Check if `specs/modules/**/<ModuleName>.toml` exists; if so, ask **Compare / Replace / Cancel**.
4. Locate the source. `specdev-analyzer context --module <ModuleName>` prints `files.source` — the file the analyzer resolves for the name, before the module exists, from `[paths].source` and the language's file names (`<snake>.rs` or `<snake>/mod.rs`, `<snake>.ts`, `<snake>.dart`, `<snake>.py`). When the module lives elsewhere, the spec sets `source_file`. If no file fits, ask the user. Never reverse a generated file (see `mode-full-project.md`, Step A).
5. **Author a stub `specs/modules/<ModuleName>.toml`** with the fields the module schema requires and `source_file` when the default does not resolve. `extract-items` takes a module name, not a path, so the stub must exist before step 6. Until step 7's `depends_on` is written, `check` reports the stub's imports as `internal_edges_declared` errors; that is expected, and step 7 clears them.
6. Run `specdev-analyzer extract-items --module <ModuleName>` from the project root. The YAML output lists every public item in the file (kind, name, span). It is authoritative about what is in the source.
7. Run `specdev-analyzer report imports --root <the module's file>`: the in-project imports, each resolved to a file. They give `depends_on` in every language.

### Fields

The module schema (`files.schema` from `context`, with the files it `$ref`s) is the checklist. For each property:

- **Required fields** — always. Choose an enum value by its `description` in the enums schema, and quote the code that decided it.
- **Optional fields** — only with evidence. Omit a field the code gives no evidence for.
- **`description`** — one sentence stating the module's *purpose* at a level higher than the rest of the spec. Do not restate anything other fields carry. If the sentence could be deleted without losing information a reader cannot get from the rest of the file, rewrite it. Target ≤ 20 words. Good: *"Drives the onboard status LED with solid, blinking, and breathing patterns."* Bad: *"A driver module that exposes set_pattern and tick actions and owns GPIO pin PA5."*
- **`depends_on`** — every in-project import from step 7 whose file belongs to another module. `check` reports a missing one (`internal_edges_declared`), so a wrong list does not survive validation.
- **`satisfies`** and **`items`** — after the requirements step below: an item needs `satisfies`. See "Items".
- **Profile rules** — when the profile README (`files.profile_readme`) has a `## Reverse` section, follow it for the fields it names.

For each field, quote the source lines used as evidence.

### Requirements

Follow `requirements.md` in this folder: propose the module's requirements, the user confirms or edits each one, and only then are they written. Then fill `satisfies` on the module and one `items` entry for each public item that carries a confirmed requirement, described in prose (never by signature).

### Items

`extract-items` lists every public name — in Rust also a `pub` name in a private module and a trait impl on a private type. Use the name exactly as it prints (`IntoText for T`, generics and all). Each one ends up in one of three places — ask the user when it is not clear:

1. **An item** — it carries a confirmed requirement (one requirement may cover several items, as twelve plot functions share "plots show …"; and a module may satisfy a requirement another module also satisfies): one `items` entry, its name and kind in the form the language's items schema gives (`${CLAUDE_PLUGIN_ROOT}/skills/spec-design/references/toml-authoring-rules.md`, "The item surface"), described in prose.
2. **Private** — it is a helper that is public by accident (a Python name without `_`, an `export` nothing uses): propose making it private in the code. That is a code change; the user decides. An export that a test imports is not an accident: it is item or left out.
3. **Left out** — it stays public and carries no requirement (a type only tests use, a type that only names a function's argument, a Python name missing from the package's `__all__`). `items diff --module <Name>` then lists it as `in_source_only`, and `/spec-sync-items` will raise it again. Say this in the report, with the count.

### Writing

Present the candidate module TOML and the proposed requirements, with `# LOW:` markers next to inferred fields only (per `${CLAUDE_PLUGIN_ROOT}/shared/comment-policy.md`). Wait for confirmation, then write `specs/modules/<ModuleName>.toml` and the confirmed requirement files.

Run `specdev-analyzer check --no-color`; if errors, show them and ask whether to iterate. For unfamiliar rule names, read the project's `specs/specdev/domain.toml` `[[project.graph_checks]]` blocks for the rule's `kind` and meaning.

Report: source files analyzed, field confidence summary, item count, requirements proposed / confirmed / tested / `test_pending`, check result.
