# `specs/requirements/README.md` stub content

`/spec-init` Step 6 writes this file verbatim as part of scaffolding the `specs/requirements/` directory. Copy the body between the horizontal rules below into the created file — no modifications.

---
Each `.toml` file in this directory holds one entry: a `[[requirement]]` (file `<id>.toml`) or an external `[[source]]` document it derives from (a protocol spec, a standard, a user brief). One file per entry keeps parallel branches from colliding.

- **Ids.** `REQ-<AREA>-NNN` (e.g. `REQ-API-001`, `REQ-NOSTD-007`), unique across the project.
- **Sources.** A requirement may name source ids in `source = [...]`, with a free-form `source_ref` locator (page, section).
- **Satisfied by modules.** Modules list the requirements they cover in `satisfies = [...]`; `requirements_have_satisfiers` reports a requirement no module satisfies, unless the requirement is `planned = true` (not built yet).
- **Verified by tests.** `verified_by = [...]` names the tests that verify it, by the identity `specdev-analyzer report tests` shows; `check` reports a named test that does not exist. `test_pending = true` says the test is not written yet — truthful, and visible in `check --since`.
- **Nature.** `nature` says how the requirement is met: `module` (default), `architecture`, `build`, or `test`.
- **Decided by.** `decided_by = [...]` names the decisions that pin how it is met; they reach every module that satisfies it.

Run `specdev-analyzer check` to verify all of the above against the specs and the code.
---

This template is profile-agnostic — requirements are declared the same way regardless of which profile (firmware-layered, library-api, audio-plugin, rust-workspace, or a user-authored profile) the project uses.
