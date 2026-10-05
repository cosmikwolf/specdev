# prisma block

Database access through [Prisma ORM](https://www.prisma.io/) for a
TypeScript project. Adds a `[prisma]` table to `Module` that says which
module owns the client singleton, which driver adapter it builds, and
which modules use it.

**Version.** The guidance is written for the current Prisma major (driver adapter mandatory, client imported from
the generator's `output`, the URL in `prisma.config.ts`). The CLI, the client and the adapter come from one release
at one version; name the version when adding them, because the CLI's default tag can be a release candidate.

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

## What the project adds

`@prisma/client`, the `prisma` CLI (a devDependency), the one driver adapter that fits the database, and `dotenv`
— all in the project's `package.json`. The guidance lists the adapter table and the version lock.

## Pairing

An authentication block that stores sessions in the database shares
`@prisma/client` with this block. That shared package is what a
composition gate asserts on.
