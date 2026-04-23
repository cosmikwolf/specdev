# `specs/requirements/README.md` stub content

`/spec-init` Step 6 writes this file verbatim as part of scaffolding the `specs/requirements/` directory. Copy the body between the horizontal rules below into the created file — no modifications.

---

Each `.toml` file in this directory declares one or more requirements and, optionally, the external `[[source]]` documents (protocol specs, standards, user briefs) those requirements are derived from. Each requirement has an id (`REQ-<NAMESPACE>-NNN`) and may reference one or more source ids via `source = [...]` with a free-form `source_ref` locator. Modules reference requirements via their `satisfies = [...]` field. Run `specdev-analyzer validate` to check that every satisfied id is declared, every `source` id resolves, and every requirement is satisfied by at least one module.

---

This template is domain-agnostic — requirements are declared the same way in firmware, egui, and any future domain.
