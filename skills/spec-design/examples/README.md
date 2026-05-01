# spec-design — using the reference examples

Reference example projects under `${CLAUDE_PLUGIN_ROOT}/examples/` may exist and can be useful for shape inspiration. Note: as of v1, examples are inherited from the prior architecture and may not validate against the current `profiles + blocks + bridges` schemas. Treat them as illustrative, not authoritative.

## Using examples while authoring

1. Identify the target project's profile from its `spec.toml` (`[project].profile`).
2. List `${CLAUDE_PLUGIN_ROOT}/examples/` and pick a project whose shape resembles what is about to be authored (firmware module vs library module, with/without state machine, etc.).
3. Open a representative module's TOML and use it for naming/style inspiration.
4. **Confirm every field against the project-managed `specs/specdev/schemas/module.schema.yaml`** — examples drift; the project's aggregated schema does not.
5. If the example uses a field the schema rejects, the field comes from a non-inlined block — either inline the block (`specdev-analyzer inline <block>`) or drop the field.
