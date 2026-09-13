# This project uses specdev

`spec.toml` and `specs/specdev/` are present, so this repository is spec-managed.

## What the skills own — do not hand-write these

- `specs/modules/*.toml` — written by `/spec-design` (new module), `/spec-edit` (existing module)
  and `/spec-reverse` (reverse-engineered from source).
- Every file recorded in `specs/specdev/gen-lineage.toml` — emitted by `/spec-gen` and overwritten
  the next time its module regenerates. Check that file before you edit any generated source.
- The build manifest (`Cargo.toml`, `pubspec.yaml`, `package.json`) **once it has a row in
  `gen-lineage.toml`**. Until then it is yours: a project that adopted specdev over existing code
  has a hand-written manifest that `/spec-gen` never produced, and `manifest --write` refuses to
  replace it (`MANIFEST-003`) for exactly that reason.

## What you write by hand

- `specs/requirements.toml` — requirements are yours to author; the skills read them.
- `spec.toml` and `specs/architecture.toml` — project identity and cross-module wiring. Bootstrap
  seeds them once and then leaves them alone; edit them directly.
- The body of an existing entry in `specs/decisions.toml` or `specs/open_questions.toml`.
  New entries go through `/spec-decide` and `/spec-question`.
- `specs/specdev/profile.yaml` `gates:` — the compile gates are project data; edit them directly.

## The loop

1. Spec first: `/spec-design <one line>` or `/spec-edit <Module>`.
2. Check: `/spec-validate` after any edit under `specs/`.
3. Generate: `/spec-gen`.
4. If a skill refuses, the refusal names the fix. Fix the spec and rerun; do not route around it by hand.

In a `rust-workspace` root, modules live in the member crates: run the module skills from the member
directory, not the workspace root.

## Feedback for the specdev developer

Keep `specdev-feedback.md` at the project root. Append one dated entry whenever a skill or
`specdev-analyzer` surprised you: a confusing error, a refusal you believe was wrong, a step you
had to work around, a missing capability. Format: `### <date> — <command> — <severity>` with
what happened, what you expected, and what you did. Severity: blocker / high / moderate / minor.
Tell the user each time you add an entry. Never delete entries; the file is append-only.
