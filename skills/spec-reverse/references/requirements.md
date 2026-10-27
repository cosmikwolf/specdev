# spec-reverse — proposing requirements

A reversed module with no requirement carries no `satisfies` and no `items`: the spec then states the code's structure and nothing about its intent. So a reverse proposes the module's requirements, and the user decides which become spec.

A requirement read from code states what the code **does**, defects included. It is a proposal until the user confirms it, and its `source` says where it came from.

## 1. The source, once per run

Before the first requirement, write one source entry for the run, `specs/sources/reverse-<YYYY-MM-DD>.toml`:

```toml
[[source]]
id = "reverse-<YYYY-MM-DD>"
kind = "derived"
title = "Read from the code by /spec-reverse; states what the code does, confirm each requirement"
```

Every requirement this run writes names it: `source = ["reverse-<YYYY-MM-DD>"]`. `check` reports an unknown source id, and the user finds the run's requirements by that id. When the user later confirms a requirement against the intended behavior, they may replace the source with the real one (a protocol, a user story).

## 2. Where requirement text comes from

In this order, and quote each one:

1. **Test names.** A test named as behavior ("refills at its rate and never above capacity") is the best evidence: it states intent and proves it. `specdev-analyzer report tests` lists every test with its identity.
2. **Doc comments** on the module and its public items: the stated purpose, limits and guarantees.
3. **Code** only for what the two above do not cover, and then mark the requirement `# LOW:`.

One requirement states one observable behavior or limit of the module, in one or two sentences, without naming functions or types. Group the tests that check the same behavior under one requirement. Skip what only restates a type or a signature.

## 3. Fields

- `id` — the project's id pattern (the requirements schema's `RequirementId`), with a prefix for the module; check `specs/requirements/` for ids in use.
- `text` — from §2.
- `nature = "module"`.
- `verified_by` — the test identities from `report tests` that check it.
- `test_pending = true` — when no test checks it. Never invent a test name; `check` rejects one that does not resolve.
- `source` — the run's id from §1.

## 4. Confirm, then write

Show the proposed requirements next to the module TOML. For each one the user chooses: **confirm**, **edit**, or **drop** (behavior that is a defect, or not worth a requirement). Write only what is confirmed or edited, one file per requirement under `specs/requirements/`. A dropped behavior that is a defect goes to the user as a finding, not into the spec.
