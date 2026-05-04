# spec-design — where to look for shape inspiration

When authoring a new module, the canonical reference shapes live alongside the inlined blocks and profiles, not in a separate `examples/` tree. Read these files (which always exist after bootstrap + inline) for shape guidance:

- **`${CLAUDE_PLUGIN_ROOT}/profiles/<profile>/README.md`** — what the profile shapes (which top-level fields, which graph checks, which file layout). Authoring guidance for the project shape itself.
- **`${CLAUDE_PLUGIN_ROOT}/blocks/<name>/README.md`** — design-time conventions for each inlined block (when to use it, which fields to author, leg-3 "do NOT mention sibling block" rules).
- **`${CLAUDE_PLUGIN_ROOT}/bridges/<a>+<b>/README.md`** — wiring patterns the bridge prescribes for `bindings.<a>+<b>` entries.
- **`specs/specdev/schemas/module.schema.yaml`** — the project-managed aggregator. The contract for what fields are valid on a module. Any pattern in a README that the schema rejects has drifted; trust the schema.
- **Recently authored modules under `specs/modules/`** — for naming/style conventions consistent with the project's own choices.

## Why no separate examples tree

Pre-doc-15 versions of the plugin shipped per-domain example projects under `examples/`. Under the current profiles + blocks + bridges model, the README per profile/block/bridge is the authoring contract — examples drift, READMEs are versioned with the schema they describe.
