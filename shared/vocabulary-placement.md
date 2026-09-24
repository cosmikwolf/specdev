# Where a list of domain values lives

A module often needs a fixed set of domain values: channel types, statuses,
classifications, roles. There are two places to put them, and each has a real
cost. Choose deliberately, and record the choice in a decision.

## The two placements

**A closed list in the spec** — the values are written in the module spec (an
enum, a union type, a list in the description), and generated code carries them
as literals.

- It cannot rot and cannot admit a typo. Every value is reviewed.
- Generated code can state exactly what it accepts, and the compiler checks it.
- **A new value is a spec change, a regeneration and a deploy.** The spec is
  the gate for what is really a data edit.

**A table in the database** — the spec states the table (for example key,
label, position, active) and the code reads the values from it at run time.

- A person adds, renames or retires a value without a deploy.
- Generated code cannot state what it accepts; validation happens against the
  table at run time.
- **Do not derive the options from values already stored in other rows.** If
  the list is "the distinct values in use", renaming the last row that uses a
  value removes it from every dropdown for good. Use a table of its own, with an
  `active` flag instead of deletion.

## The test

**Does someone who is not an engineer need to add a value without a deploy?**

- **Yes** → a table. The module spec states the table and where the code reads
  it; it does not list the values. Seed values belong in a migration or a seed
  script, not in generated code.
- **No** → a closed list in the spec.

If the project's own rules already say where volatile data lives (a
`CLAUDE.md` line such as "volatile behaviour lives in the database"), those
rules decide the default; a choice against them needs its reason written down.

## Record the choice

Author a decision with `/spec-decide` that names the list, the placement, and
the answer to the test. Link it with `guided_by` from every module that reads or
writes the values, so the choice reaches each of their codegen prompts.
Otherwise the next person to touch the list re-derives the choice — or makes the
other one without knowing it was ever made.
