# spec-design — illustrative interview shapes per shipped profile

These lists are **examples**, not the authoritative source. The binding contract is the profile's `module.schema.yaml` (which `required` lists, plus inlined-block contributions). When a shipped profile is updated, the schema is the truth — re-read it rather than this file.

For user-authored profiles, no illustrative shape is provided here; read the profile's `module.schema.yaml` and `README.md` directly.

## firmware-layered

- **Name** — PascalCase, matches `Identifier` pattern. Default: derived from `$ARGUMENTS`.
- **Layer** — one of `application`, `middleware`, `driver`, `HAL`, `PAC` (from `enums.schema.yaml#/$defs/LayerKind`). Default from description (e.g., "driver for an I2C sensor" → `driver`).
- **depends_on** — list of existing module names. Default empty.
- **Hardware** — pins / peripherals / clocks the module owns or borrows. Cross-check against `specs/architecture.toml` and the inlined MCU block's schema (e.g., for `stm32h5`: `peripherals[].kind` must be one of the block's enum).
- **Ports** — name + `port_type` + `direction` for each port. These appear as endpoints in `architecture.toml [[connections]]`.
- **Actions** — public action surface. One line per action: what it does, plus `blocking` and `isr_safe`. No parameter or return types — the code holds those.
- **State machine** — if the module has one, ask for initial state, state names, triggering events. Defer transition details to the TOML.
- **Errors** — one line per failure the module reports: what can go wrong and what the caller sees.
- **Metadata** — `MemoryModel`, `ConcurrencyModel`, `ErrorHandling`, `Ownership`. Pick explicit, well-justified values; do not default through.
- **Bindings** — if both `embassy` and `stm32h5-rs` are inlined and a bridge is present, ask whether this module declares any `bindings."embassy+stm32h5-rs".peripheral_tasks[]` entries (each entry: `task`, `peripheral`, optional `handoff` ∈ `{owned, shared}`).
- **satisfies** — requirement ids from `specs/requirements/`.
- **Code state** — `planned = true` when no code exists yet; otherwise `source_file` when the file is not at `src/<layer>/<name>.rs`.

## library-api

- **Name** — PascalCase.
- **visibility** — `pub` / `pub(crate)` / `pub(super)`.
- **depends_on** — list of existing module names.
- **items** — optional `[[items]]` entries for the module's contractual surface, checked by `items diff`. Re-exports use `kind = "re_export"` with `{target, visibility, alias?}`; other public-API items (`fn`, `struct`, `enum`, `trait`, `impl`, ...) require `satisfies` (≥1 requirement id).
- **satisfies** — requirement ids from `specs/requirements/`.
- **Code state** — `planned = true` when no code exists yet; otherwise `source_file` when the file is not at the conventional path.

Features and dependencies live in `Cargo.toml`; the module spec does not copy them.

## rust-workspace

`/spec-design` operates on a **single member crate**, not the workspace. If the project root is a rust-workspace, `cd` into the target member crate's directory (which has its own `specs/specdev/`) and run the skill there — the member's profile (typically `library-api` or `firmware-layered`) shapes the interview. If the user names a workspace member by name, ask which member to design under and route to its `specs/`.
