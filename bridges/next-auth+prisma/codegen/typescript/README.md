# next-auth+prisma — codegen prose (TypeScript)

The join product: an Auth.js configuration backed by a Prisma client.
Measured against **next-auth 5.0.0-beta.32**, **@auth/prisma-adapter
2.11.3** and **@prisma/client 7.10.0**; the reference package compiles it.

## What the bridge contributes

One package, `@auth/prisma-adapter`, and it lives here rather than in
either block on purpose. It is useless without both: it takes a Prisma
client and returns an Auth.js adapter. A project that inlined only one
side would receive a dependency it could not satisfy.

The adapter's version does **not** track Prisma's. It declares a wide
peer range on the client rather than a matching version, so pin it on
its own line and do not assume it moves with the client.

## The wiring

```ts
import { PrismaAdapter } from "@auth/prisma-adapter";
import NextAuth from "next-auth";
import GitHub from "next-auth/providers/github";

import { prisma } from "./db";

export const { handlers, auth, signIn, signOut } = NextAuth({
  adapter: PrismaAdapter(prisma),
  session: { strategy: "jwt" },
  providers: [GitHub],
});
```

`prisma` is the singleton exported by the module the binding's `client`
field names. Derive the import from that module's own path.

## What the compiler will NOT catch here

`PrismaAdapter`'s parameter type is a union wide enough to accept almost
anything. Measured: `PrismaAdapter(42)`, `PrismaAdapter("a string")`,
`PrismaAdapter(null)` and `PrismaAdapter({ nope: true })` **all
type-check**. So a green `tsc` says the option names and the strategy
are right; it says nothing about whether you handed it a real client.

What does protect this pairing is the resolve: both units must end up
with **one** `@prisma/client` in the lockfile. Pass the singleton the
spec names and let the composition gate assert the rest.

## Session strategy — the decision this bridge exists for

Once an adapter is present Auth.js has two strategies, and `database`
becomes its default. The binding must state one.

| `session_strategy` | what it does | cost |
| --- | --- | --- |
| `database` | a row per session; sign-out deletes it, and a session can be revoked server-side | every session read is a query |
| `jwt` | session lives in a signed cookie; the adapter still does account linking and verification tokens | not revocable until the token expires |

The constraint neither block can see on its own: **a `database` session
is read wherever `auth()` is called, and middleware runs in the Edge
runtime.** So if middleware calls `auth`, the Prisma driver adapter must
work there. Against the prisma block's `adapter` field:

| prisma `adapter` | `database` + middleware |
| --- | --- |
| `neon`, `d1`, `planetscale`, `libsql` | HTTP or Edge-native drivers — workable |
| `better-sqlite3`, `pg`, `mariadb`, `mssql` | Node-only drivers — use `jwt`, or keep `auth` out of middleware |

That table is why the field is here. The next-auth block cannot see which
driver adapter was chosen, and the prisma block cannot see whether
middleware calls `auth`.

When you choose `database`, set `edge_split = true` on the `[next_auth]`
table and keep the adapter out of the edge-safe half.

## The schema is yours, and the adapter requires four models

`schema.prisma` is hand-owned, and the adapter will not work until it
holds `User`, `Account`, `Session` and `VerificationToken` with the
fields Auth.js expects. Nothing in this bridge generates or checks them —
the reference package carries a known-good copy; start from it.

`Account` and `Session` both relate to `User` with
`onDelete: Cascade`. Without it, deleting a user leaves orphaned rows
that the adapter will later read as live sessions.

After editing the schema, run `prisma generate` before `tsc` — the
generated client is what types the adapter call.

## Verification tokens without an email provider

`VerificationToken` is required by the adapter's type even when no
provider uses it. Keep the model. Dropping it because nothing writes to
it today breaks the adapter on the day an email provider is added.
