# This project uses specdev

`spec.toml` and `specs/specdev/` are present, so this repository is spec-managed. The specs state intent —
what each module is for, what it may depend on, which requirements it satisfies — and `specdev-analyzer check`
checks that intent against the code. specdev does not generate code: you and the agent write it.

## Before you edit code

Run `specdev-analyzer context --module <Module>` for the module that owns the file, and read what it lists:

- the module spec, its schema, and the specs of the modules next to it;
- the inlined implementation guidance (`specs/specdev/guidance/`) of the profile and of every inlined block and
  bridge — the rules the code must follow that no check can verify;
- the module's `context_files` — reference reading its authors pinned (hardware notes, protocol references).

Then run `specdev-analyzer report decisions --module <Module>`: the accepted decisions and the open questions that
reach the module. The code follows those decisions; an `open` question that reaches it blocks building on it.

If no module owns the file yet, design one first (`/spec-design`), or mark the work as planned.

## What the skills own — do not hand-write these

- `specs/modules/*.toml` — written by `/spec-design` (new module), `/spec-edit` (existing module)
  and `/spec-reverse` (reverse-engineered from source).
- New ledger entries: `/spec-decide` writes `specs/decisions/<id>.toml`, `/spec-question` writes
  `specs/open_questions/<id>.toml`.
- `specs/specdev/` — the inlined machinery. Change it through `/spec-inline`, `/spec-inline-bridge` and
  `specdev-analyzer migrate`; a local edit there is allowed, and `drift` reports it.

## What you write by hand

- The code, the tests and the build manifest (`Cargo.toml`, `pubspec.yaml`, `package.json`) — the manifest is the
  only source of truth for dependencies, features and build profiles.
- Requirements under `specs/requirements/` — yours to author; the skills read them. A moot one is retired, never
  deleted: move it to `specs/requirements/retired/` with `retired_by = "DEC-…"` (`/spec-decide`).
- `spec.toml` and `specs/architecture.toml` — project identity and cross-module wiring.
- The body of an existing decision or open question.
- `specs/specdev/profile.yaml` `gates:` — the compile gates are project data. After an edit, run
  `specdev-analyzer gate --emit` to rewrite `scripts/specdev-gates.sh`.

## The loop

1. Spec first: `/spec-design <one line>` or `/spec-edit <Module>`. A module with no code yet carries
   `planned = true`; remove it when the code lands. When the module's code exists but some of its rule-pack
   entries (one entry of an inlined block's table) do not, put `planned = true` on those entries, not on the module.
2. Read the context (above), then write the code and its tests. A requirement's `verified_by` names the test
   by the identity `specdev-analyzer report tests` shows.
3. Check: `/spec-validate` (`specdev-analyzer check`). CI runs `sh scripts/specdev-gates.sh`, which runs `check`
   and then every compile gate.
4. If a check fails, the finding names the fix. Fix the spec or the code; do not route around it.

In a `rust-workspace` root, modules live in the member crates: run the module skills from the member
directory, not the workspace root.

## Feedback for the specdev developer

Keep `specdev-feedback.md` at the project root. Append one dated entry whenever a skill or
`specdev-analyzer` surprised you: a confusing error, a refusal you believe was wrong, a step you
had to work around, a missing capability. Format: `### <date> — <command> — <severity>` with
what happened, what you expected, and what you did. Severity: blocker / high / moderate / minor.
Tell the user each time you add an entry. Never delete entries; the file is append-only.
