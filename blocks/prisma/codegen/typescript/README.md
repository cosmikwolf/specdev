# prisma — codegen prose (TypeScript)

How to write a module that carries a `[prisma]` table. Everything here is
measured against **Prisma 7.10.0**; the block pins the 7.x line and the
reference package under `reference/typescript/` compiles every worked
example below.

Prisma 7 changed three things that older prose gets wrong. Read this
section before copying any snippet you have seen elsewhere.

1. **A driver adapter is mandatory.** `new PrismaClient()` with no adapter
   does not connect. The adapter package is chosen per database.
2. **The client is imported from the generator's `output` path**, not from
   `@prisma/client`. `output` is a required field in Prisma 7.
3. **The datasource URL lives in `prisma.config.ts`**, not in the
   `datasource` block of `schema.prisma`. The CLI removed `--schema` and
   `--url`.

## What the block contributes, and what you add

The block contributes exactly one package: **`@prisma/client`**. That is
the only package every Prisma project needs whatever its database.

Three things are **user actions**. They are listed in `deps.yaml` under
`recommended_external_deps`, so the design skill surfaces them, and none
of them is injected:

| what | why it is not contributed |
| --- | --- |
| `prisma` CLI | a devDependency; block contributions are runtime deps |
| a driver adapter | eight ship, exactly one applies; a static list cannot choose |
| `dotenv` | a devDependency, read by `prisma.config.ts` |

**Version lock.** The CLI, the client and the adapter are published from
one Prisma release and must sit on the **same version**. A mismatch fails
at generate time, not at run time. Add them together:

```sh
pnpm add -D prisma@7.10.0 dotenv
pnpm add @prisma/client@7.10.0 @prisma/adapter-better-sqlite3@7.10.0
```

Do **not** widen the client to `^8`. The `prisma` CLI publishes
`8.0.0-rc.N` on the `latest` dist-tag, so a bare `pnpm add prisma` takes a
release candidate. Name the version.

## Adapter table

`[prisma.datasource].adapter` holds the short name in column 1. Each row
is one measured package at 7.10.0. The provider column is what the
`datasource` block in your `schema.prisma` must say — the adapter
determines it, so the spec never repeats it.

| `adapter` | package | class | `schema.prisma` provider |
| --- | --- | --- | --- |
| `pg` | `@prisma/adapter-pg` | `PrismaPg` | `postgresql` |
| `neon` | `@prisma/adapter-neon` | `PrismaNeon` | `postgresql` |
| `better-sqlite3` | `@prisma/adapter-better-sqlite3` | `PrismaBetterSqlite3` | `sqlite` |
| `libsql` | `@prisma/adapter-libsql` | `PrismaLibSql` | `sqlite` |
| `d1` | `@prisma/adapter-d1` | `PrismaD1` | `sqlite` |
| `mariadb` | `@prisma/adapter-mariadb` | `PrismaMariaDb` | `mysql` |
| `planetscale` | `@prisma/adapter-planetscale` | `PrismaPlanetScale` | `mysql` |
| `mssql` | `@prisma/adapter-mssql` | `PrismaMssql` | `sqlserver` |

Each adapter pulls its own driver as a regular dependency, so
`@prisma/adapter-pg` brings `pg` and `@prisma/adapter-better-sqlite3`
brings `better-sqlite3`. Install the driver explicitly anyway when you
import it yourself. `better-sqlite3` is a native module and compiles at
install time, so a CI image needs a build toolchain.

## Project files you own

`prisma init` writes two files. **Both are hand-owned in v1.** Codegen
never emits them and `gen-record` never records them.

```
prisma/schema.prisma    models, relations, the generator block
prisma.config.ts        datasource url, schema path, migrations path
```

`schema.prisma` needs the `prisma-client` generator with an explicit
`output`. That path is what `[prisma.datasource].generated_client` must
repeat, because it is the import path every generated module uses:

```prisma
generator client {
  provider = "prisma-client"
  output   = "../src/generated/prisma"
}

datasource db {
  provider = "sqlite"
}
```

`prisma.config.ts` sits at the project root:

```ts
import "dotenv/config";
import { defineConfig } from "prisma/config";

export default defineConfig({
  schema: "prisma/schema.prisma",
  migrations: { path: "prisma/migrations" },
  datasource: { url: process.env["DATABASE_URL"] },
});
```

**Deferred, on purpose:** emitting either file from the spec. The models
belong to `schema.prisma` and restating them in TOML would give one fact
two sources. Revisit when a second project hand-edits the two apart.

## The singleton module

The module carrying `[prisma.datasource]` owns the one client instance.
Its body **must run on the server only**. When the project's profile
offers a server-guard field, set it and let the profile emit the guard;
otherwise open the body with `import "server-only"` yourself.

Hold the instance on a global in development. Next-generation dev servers
reload modules on every edit, and a fresh client per reload exhausts the
database connection pool.

```ts
import { PrismaClient } from "./generated/prisma/client";
import { PrismaBetterSqlite3 } from "@prisma/adapter-better-sqlite3";

const adapter = new PrismaBetterSqlite3({
  url: process.env.DATABASE_URL ?? "file:./dev.db",
});

const globalForPrisma = globalThis as unknown as { prisma?: PrismaClient };

export const prisma = globalForPrisma.prisma ?? new PrismaClient({ adapter });

if (process.env.NODE_ENV !== "production") globalForPrisma.prisma = prisma;
```

The import path is `[prisma.datasource].generated_client` made relative to
the module's own file. The snippet above is the common case: a module at
`src/db.ts` with `generated_client = "src/generated/prisma"` reaches it at
`./generated/prisma/client`. Re-derive the path when either side moves;
do not copy the literal. And never write `from "@prisma/client"` — in
Prisma 7 that package does not export your generated client.

## A consumer module

A module carrying `[prisma].client` imports the singleton the named module
exports. The name resolves through the `model_uses_datasource` relation,
so a wrong name is a validate error rather than a broken import.

```ts
import { prisma } from "./db";

export async function listEvents() {
  return prisma.event.findMany({ orderBy: { startsAt: "asc" } });
}
```

## Transactions

Use the interactive form when later writes depend on earlier reads. The
callback gets a transaction client; use `tx`, never the outer `prisma`,
or that query runs outside the transaction.

```ts
import { Prisma } from "./generated/prisma/client";

await prisma.$transaction(
  async (tx) => {
    const event = await tx.event.findUniqueOrThrow({ where: { id } });
    await tx.booking.create({ data: { eventId: event.id, seat } });
  },
  {
    maxWait: 5000,
    timeout: 10000,
    isolationLevel: Prisma.TransactionIsolationLevel.Serializable,
  },
);
```

`maxWait` is how long a call waits to acquire the transaction (default
2000 ms). `timeout` is how long the body may run (default 5000 ms). A body
that outruns `timeout` is rolled back.

Pass an array instead of a callback for independent writes that must land
together. The array form is one round trip and cannot read between steps.

## Error mapping

Catch `Prisma.PrismaClientKnownRequestError` and branch on `.code`. The
code is the stable contract; the message is not.

```ts
import { Prisma } from "./generated/prisma/client";

try {
  await prisma.user.create({ data: { email } });
} catch (e) {
  if (e instanceof Prisma.PrismaClientKnownRequestError) {
    if (e.code === "P2002") return { error: "that email is taken" };
    if (e.code === "P2025") return { error: "not found" };
  }
  throw e;
}
```

| code | meaning | what to do |
| --- | --- | --- |
| `P2002` | unique constraint violation | map to a field error |
| `P2025` | record not found | map to a 404 |
| `P2034` | write conflict or deadlock | **retry the transaction** |
| `P2024` | connection pool timeout | back off; check the pool size |

`P2034` is the only one of these the docs name as retryable. Retry the
whole `$transaction` call, not the inner query.

Re-throw anything you do not recognise. Swallowing an unknown code turns a
schema mistake into a silent wrong answer.

## Generate sequencing

`prisma generate` is a **user action**, and codegen never runs it and
never records its output.

```
1. edit prisma/schema.prisma
2. pnpm exec prisma generate      <- writes the generated client
3. /spec-gen                      <- writes the module bodies
4. pnpm exec tsc --noEmit         <- the module bodies see the client
```

Step 2 must precede step 4, because a module body imports a type that does
not exist until `prisma generate` writes it. The generated client
directory is **externally owned**: add it to the never-record list, the
same way a generated binding directory is treated. A generated tree that
`gen-record` hashes will report drift on every regeneration.

**Never recording it is only half the rule — never commit it either.** No
profile template can do this half for you: the path is whatever this
project set as `[prisma.datasource].generated_client`, so no static line
knows it. Add it to `.gitignore` yourself. For the worked layout above:

```gitignore
# prisma generate output — regenerated by step 2, never committed
/src/generated/
```

`.gitignore` is written once when absent and is yours afterwards, so
appending to it is expected, not a hand-edit of generated output. The
database file needs nothing: the profile template already ignores `*.db`
and its journal and WAL sidecars.
