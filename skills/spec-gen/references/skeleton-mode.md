# spec-gen — skeleton mode

Entered from `/spec-gen` Step 1.5 when the resolved mode is `skeleton`. **Claude does not emit code in this branch** — the analyzer's `gen --skeleton` subcommand handles the entire emission atomically.

## Mode resolution

Phase 13 (doc 16 §4) introduces `[project].handoff_mode` and a strict/skeleton split. Mode is resolved before any prompt assembly:

1. If `$ARGUMENTS` contains `--strict`, mode is `strict`.
2. Else if `$ARGUMENTS` contains `--skeleton`, mode is `skeleton`.
3. Else read `spec.toml [project].handoff_mode`:
   - `"skeleton"` → mode is `skeleton`
   - `"implementation"` or absent → mode is `strict`

`--strict` and `--skeleton` are mutually exclusive. A flag is not a target name — if `$ARGUMENTS` contains both `--strict|--skeleton` and a module name, treat the module name as the target and the flag as the mode.

## Procedure (skeleton mode only)

Run from the project root:

```sh
specdev-analyzer gen --project . --skeleton
```

The analyzer:

- Refuses if the source dir (`paths.source` from `spec.toml`) is non-empty, library-api isn't the profile (v1 limitation), or `cargo check` fails on the emitted tree.
- Emits `<source>/lib.rs` + one `<source>/<module_snake>.rs` per module spec, with `unimplemented!("REQ-…: <one-line>")` bodies projected from each module's `[[items]]` entries.
- Renders Cargo.toml.
- Writes `specs/HANDOFF.md` with the coverage matrix (requirement → satisfying modules; orphans flagged), per-module skeleton inventory, plugin-gaps free-text section, and suggested next steps.
- Runs `cargo check`; on failure rolls back the entire skeleton tree and exits non-zero.

After a successful run, read `specs/HANDOFF.md` and report:

- One sentence per module (stub count, re-export count).
- The orphan-requirement list (rows in the matrix flagged ORPHAN).
- The "suggested next steps" tail (which modules have `context_files` and are ready for `--strict`, which still need `context_files`).

## Out of scope in skeleton mode

Do not invoke any other `/spec-gen` step. Do not run `prompt` / emit source / run `cargo-toml` separately — `gen --skeleton` is the atomic command. If the user wants per-module strict generation after the skeleton is in place, re-invoke `/spec-gen <ModuleName> --strict` once `context_files` are populated.
