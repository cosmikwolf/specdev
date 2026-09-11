# prisma block

Database access through [Prisma ORM](https://www.prisma.io/) for a
TypeScript project. Adds a `[prisma]` table to `Module` that says which
module owns the client singleton, which driver adapter it builds, and
which modules use it.

**Pinned line: Prisma 7.x** (`@prisma/client ^7.10.0`). Prisma 8 is not
generally available — the `prisma` CLI publishes `8.0.0-rc.N` on the
`latest` dist-tag, so a bare `pnpm add prisma` takes a release candidate.
Name the version.

**One pin is swept, and it pulls the other three.** The freshness sweep
reads contributed deps, so it watches `@prisma/client` and nothing else
here: the CLI and the adapter are user actions and the reference's own
`package.json` is not a contribution. That is deliberate and it still
works, because Prisma publishes the client, the CLI and every adapter
from one release at one version. When the sweep reports `@prisma/client`
behind, bump all four — block pin, reference client, reference CLI,
reference adapter — in one unit, and re-run the reference gate, which is
what certifies the prose against the new line.

No `pin-policy.yaml` ships here. A hold would need a swept pin to hold,
and the only swept pin is current.

## What the table says

Two shapes. A module carries exactly one of them, and the schema rejects
a module that carries both or neither.

```toml
# the singleton — one module per project
[prisma.datasource]
adapter          = "better-sqlite3"
generated_client = "src/generated/prisma"
schema_file      = "prisma/schema.prisma"   # optional; this is the default
```

```toml
# a consumer — any module that queries
[prisma]
client = "Db"
```

`client` names the module that owns the singleton. A
`model_uses_datasource` relation resolves it, so a mistyped name is a
`REDUCE_UNRESOLVED` error at validate time rather than an import that
fails at build time.

## What the table deliberately does not say

**No models, no fields, no relations.** Those live in `schema.prisma`,
which is hand-owned. Restating them here would give one fact two sources
and two ways to disagree. The table holds config inputs only.

**No datasource provider.** The adapter determines it — each of the eight
adapters serves exactly one provider — and `schema.prisma` already states
it. Repeating it would create a third place to get it wrong.

## What the block contributes

One package: `@prisma/client`. The `prisma` CLI, the chosen driver
adapter and `dotenv` are **user actions**, surfaced through
`recommended_external_deps` and documented in `codegen/typescript/README.md`.

The CLI cannot be contributed because it is a devDependency and block
contributions are runtime deps. The adapter cannot be contributed because
eight ship and exactly one applies to any project. This follows the
flutter-rust-bridge block, which contributes its runtime package and
leaves its codegen CLI to a documented user action.

## Reference package

`reference/typescript/` compiles every worked example in the codegen
prose against the pinned client. It uses SQLite through
`@prisma/adapter-better-sqlite3` because it needs no server. The gate is
`pnpm install --frozen-lockfile && pnpm run check`; the `check` script
runs `prisma generate` before `tsc --noEmit`, because the generated
client does not exist until it does.

## Pairing

An authentication block that stores sessions in the database shares
`@prisma/client` with this block. That shared package is what a
composition gate asserts on.
