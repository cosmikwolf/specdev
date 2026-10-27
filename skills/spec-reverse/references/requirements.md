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
2. **Doc comments** on the module and its public items, and the project's own docs (README, a defaults table): the stated purpose, limits and guarantees.
3. **Code** only for what the two above do not cover. A requirement with any part from code only is marked `# LOW:`.

One requirement states one observable behavior or limit of the module, in one or two sentences, without naming functions or types. Group the tests that check the same behavior under one requirement. Skip what only restates a type or a signature.

Use a test only when it checks this module's behavior; a data-only module may be verified by the tests of the module that applies its data. A test of a surface no module owns (generated bindings, another language's code, a CI workflow file) verifies nothing here. A test that runs only under a condition (an environment variable, a database, a skip in the default run) still resolves in `verified_by`; name it, and say in the report that it does not run by default.

**When the code breaks the behavior in one case** (one value mapped wrongly, an error path that is never taken), ask the user which the requirement states: the intent — then the case is a defect, reported as a finding, and the code does not meet the requirement until it is fixed — or the code as it is. Without an answer, state the intent, mark it `# LOW:`, and name the case in the finding.

## 3. Fields

One file per requirement, `specs/requirements/<id>.toml`, holding one `[[requirement]]` table:

```toml
[[requirement]]
id = "REQ-RATE-001"
text = "Each client address has its own token budget, refilled at a fixed rate and never above its capacity; a request with no token left is refused."
nature = "module"
verified_by = ["refills at its rate and never above capacity", "keeps each client apart"]
source = ["reverse-<YYYY-MM-DD>"]
```

- `id` — `REQ-<AREA>-<NNN>`, where `<AREA>` is a short upper-case name for the module (`RATE` for `RateLimit`); it must match the enums schema's `RequirementId` pattern. Check `specs/requirements/` for ids in use.
- `text` — from §2.
- `nature = "module"`.
- `verified_by` — the test identities from `report tests` that check it, as plain strings: `report tests` prints YAML, so take a quoted title without its quotes and write a doubled `''` as one `'`. When they check only part of the behavior (three color codes of fifty), name them and say in the report which part has no test.
- `test_pending = true` — when no test checks it. Never invent a test name; `check` rejects one that does not resolve.
- `source` — the run's id from §1.

## 4. Confirm, then write

Show the proposed requirements next to the module TOML, with `# LOW:` on the ones that came from code only — the marker goes at the end of the `text` line: `text = "…"  # LOW: inferred from <file>:<line>, please confirm`. For each one the user chooses: **confirm**, **edit**, or **drop** (behavior that is a defect, or not worth a requirement). Write only what is confirmed or edited; a confirmed requirement is written without its `# LOW:` marker. A dropped behavior that is a defect goes to the user as a finding, not into the spec.

When the run has no user to confirm (an unattended run), write the proposals with their `# LOW:` markers kept, and say in the report that every requirement of the run is unconfirmed.
