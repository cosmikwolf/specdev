# Module TOML authoring rules

Priority-ordered rules for any skill that writes or edits a module TOML (`/spec-design`, `/spec-edit`, and `/spec-reverse` after the user has confirmed inferred fields). When two rules conflict, the earlier rule wins.

1. **Schema is the contract.** Every field name must appear somewhere in the aggregator's reachable schemas. No invented keys. The schema uses `unevaluatedProperties: false`, which is `allOf`-aware — block-contributed properties pass through.
2. **Required fields by profile.** firmware-layered requires `name` + `layer`; library-api requires `name` + `visibility`. Whatever else the profile's `module.schema.yaml` lists as required must be present.
3. **Required metadata (firmware-layered).** `metadata.MemoryModel`, `metadata.ConcurrencyModel`, `metadata.ErrorHandling`, `metadata.Ownership` must all be present. Pick explicit values.
4. **Prefer inline tables for ≤3 params.** `params = [{name = "...", type = "u32", direction = "in"}]` is easier to read than an array-of-tables for short lists. Flip to `[[actions.params]]` when an action has >3 params.
5. **Naming conventions.**
   - Module names: PascalCase (`StatusLed`, `BtA2dpSink`).
   - Action names: `snake_case`.
   - Event names: `snake_case`, suffixed `_evt` (`set_solid_evt`, not `setSolid`).
   - State names: `snake_case`.
   - Error variants: `PascalCase` (they become Rust enum variants).
6. **Layer discipline (firmware-layered).** A module at layer `application` must not directly depend on a `PAC` module — walk through `middleware` / `driver` / `HAL`. The `layer_dependency` graph check enforces this.
7. **One module per file.** Each TOML holds exactly one module.
8. **Comment policy** — follow `${CLAUDE_PLUGIN_ROOT}/shared/comment-policy.md`. `/spec-design` and `/spec-edit` work from explicit user intent, not inference, so neither emits `# LOW:` markers — those are unique to `/spec-reverse` and are stripped once the user confirms each inferred field.

## When editing existing TOML (/spec-edit specifics)

- Preserve existing indentation and key order where not semantically changed.
- Do not drop required metadata sub-objects (firmware-layered).
- Keep inline-table vs array-of-tables form consistent with the rest of the file.
- If the existing file carries disallowed comments (file-header blurbs, section dividers, prose that restates a nearby field), strip them as part of your edit. Carve-out: leave `# LOW:` markers from `/spec-reverse` alone until the user has confirmed or cleared each one.
