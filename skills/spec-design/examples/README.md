# spec-design — using the reference examples

The canonical catalog of example module TOMLs lives at `specdev/examples/README.md`. Do not re-list modules here — consult that file for the authoritative per-project tables.

## Using examples while authoring

1. Identify the target project's domain from its `spec.toml` (`[project] domain`).
2. Open `specdev/examples/README.md` and find the project for that domain.
3. From the table there, pick a module whose shape resembles what is about to be authored (driver vs middleware vs view, with/without FSM, etc.).
4. Open the picked module's TOML and model the new file on its field order and style.
5. Confirm every field against `<Schema dir>/module.schema.yaml` (the authoritative contract) before writing — examples drift; the schema does not.
