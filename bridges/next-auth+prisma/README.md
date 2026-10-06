# next-auth+prisma bridge

Backs an Auth.js configuration with a Prisma client: database-persisted
users, accounts, sessions and verification tokens.

Requires the `next-auth` and `prisma` blocks. Lex order puts `next-auth`
first.

## Why the adapter lives here

`@auth/prisma-adapter` is a join product. It takes a Prisma client and
returns an Auth.js adapter, so it is useless unless both blocks are
present — the `next-auth` block's guidance would otherwise tell a project
without Prisma to add a package it cannot use. That is the rule that
moved `nih_plug_egui` out of the `egui` block and into `egui+nih-plug`.

## What the binding says

```toml
[bindings."next-auth+prisma"]
client           = "Db"      # module owning [prisma.datasource]
session_strategy = "jwt"     # or "database"
```

`client` resolves to that module's Datasource, so a wrong name is a
`REDUCE_UNRESOLVED` error at validate time rather than a broken import.

`session_strategy` is the decision this bridge exists for. Without an
adapter Auth.js has one strategy and nothing to choose; with one, the
choice depends on **both** which Prisma driver adapter is in use and
whether middleware calls `auth` — two facts that live in two different
blocks. The guidance carries the table.

## The module holding the binding

It carries `[next_auth]` and this binding. It does **not** need
`[prisma].client`: it never queries a model, it hands the client to the
adapter, and this binding is its relationship to the database.

## What the compiler does not catch

`PrismaAdapter` accepts almost any argument without complaint, so a type check proves the option names and the
strategy, not that the argument is a real client.
