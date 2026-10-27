# TOML comment policy

Shared rule for every skill that authors or edits a spec TOML. Skills reference this file; each may add its own carve-out on top.

## Core rule

The only comment permitted in an emitted or edited spec TOML is the `#:schema` directive on line 1 (for IDE / Taplo schema integration). Do not add any of the following:

- File-header prose or author-name blocks.
- Section dividers (`# --- actions ---`).
- Comments that restate or summarize a nearby field (the field itself is the source of truth).
- Comments that describe the project as a whole — that belongs in `README.md`.

If a fact deserves to live in the TOML, it belongs in a schema field, not in a comment. If it does not fit the schema, it belongs somewhere other than the TOML.

## LOW-confidence carve-out (used only by `/spec-reverse`)

`/spec-reverse` annotates inferred fields with a single inline comment of the form:

```
field = "..."  # LOW: inferred from <source>:<line>, please confirm
```

These markers are transient review metadata — they exist so the user can confirm or override the inference, and they are removed from the file once the field is confirmed. No other skill emits LOW markers. `/spec-edit` treats existing LOW markers as a carve-out (leave them alone until cleared); `/spec-design` never produces them.

## Scope

This policy applies to every `.toml` file that lives under a project's `specs/` tree: `spec.toml`, `specs/architecture.toml`, `specs/modules/**/*.toml`, and the ledger entries under `specs/requirements/`, `specs/sources/`, `specs/decisions/` and `specs/open_questions/`. It does not apply to non-spec TOML (e.g. `Cargo.toml`), and it does not remove the comments bootstrap writes from the profile's templates (`spec.toml`, `architecture.toml`, `domain.toml`): those explain the profile's choices and stay until the user removes them.
