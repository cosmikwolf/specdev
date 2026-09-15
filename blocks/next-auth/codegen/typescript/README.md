# next-auth — codegen prose (TypeScript)

How to write the module that carries a `[next_auth]` table. Everything
here is measured against **next-auth 5.0.0-beta.32**, and the reference
package under `reference/typescript/` compiles every example below.

## The pin is a pre-release, and that is the point

`5.0.0-beta.32` is the `beta` tag. The `latest` tag is `4.24.15`, and it
is not a fallback: v4 predates the single-config shape this block is
built on, and the project's own current documentation site covers v5
only — v4 lives on the previous domain. Choosing `latest` would mean
teaching an API upstream no longer documents.

The pin is **exact**. A caret on a pre-release silently accepts the next
beta, and betas may break between betas.

**When 5.0.0 ships, nothing will tell you.** The freshness sweep drops
the pre-release identifier when it compares, so this pin reads `ahead`
today and will read `current` the day the GA lands. Treat the GA as a
manual trigger.

## One module holds the configuration

Auth.js v5 has exactly one configuration site, and everything else
imports from it. That is why `[next_auth]` marks one module rather than
describing a feature.

```ts
import NextAuth from "next-auth";
import GitHub from "next-auth/providers/github";

export const { handlers, auth, signIn, signOut } = NextAuth({
  providers: [GitHub],
});
```

Four exports, and each has one job:

| export | used by |
| --- | --- |
| `handlers` | the derived route file, as `GET` and `POST` |
| `auth` | server components, route handlers and middleware, to read the session |
| `signIn` / `signOut` | server actions and client components |

The module's body must run on the server. When the project's profile
offers a server-guard field, set it and let the profile emit the guard.

`providers` in the spec is a list of ids; each id becomes an import from
`next-auth/providers/<id>` and an entry in this array. A wrong id fails
at the import, which is the checker that knows the real list — the spec
deliberately does not carry an enumerated copy of it.

## The derived route file

`base_path` decides where it lives; the default `/api/auth` puts it at
`app/api/auth/[...nextauth]/route.ts`. It is always these two lines.

```ts
import { handlers } from "./config";

export const { GET, POST } = handlers;
```

Never add a third export. `handlers` carries `GET` and `POST` only, and
asking for another method is a compile error — which the reference
proves.

## `edge_split = true`

Set it when middleware calls `auth` and any part of the configuration
reaches something the Edge runtime cannot load. Split the config in two:
an edge-safe half with providers and callbacks, and the full call that
adds the rest.

**There is usually no middleware module.** A spec-managed project has
only the roles its profile defines. Read the project's profile role list
before assuming middleware exists: where no role produces the
`middleware.ts` (Next 16: `proxy.ts`) file, nothing does — no
derived-file rule writes one either — and the project cannot gate routes
in middleware. Gate them per module instead: each protected `page` or
`route_handler` calls `auth()` in its body (or a small `service` module
that wraps `auth()` and checks the role) and redirects or returns 401
itself. That path works today, keeps every guard inside a module the
spec knows about, and makes `edge_split` a question of where the config
loads rather than a promise that middleware exists. With no middleware
calling `auth`, `edge_split = true` is still valid and still the
conservative choice.

```ts
import type { NextAuthConfig } from "next-auth";
import NextAuth from "next-auth";
import GitHub from "next-auth/providers/github";

export const authConfig = {
  providers: [GitHub],
  pages: { signIn: "/signin" },
} satisfies NextAuthConfig;

export const { handlers, auth, signIn, signOut } = NextAuth({
  ...authConfig,
});
```

`satisfies NextAuthConfig` rather than a type annotation: it checks the
object without widening it, so the provider list keeps its literal types.

The edge-safe half must import nothing Node-only. A database adapter is
the usual reason this field gets set, and the unit that adds one has more
to say about it.

## Reading the session

`auth()` is awaitable and returns `null` when nobody is signed in.

```ts
import { auth } from "./config";

export async function currentUserEmail(): Promise<string | null> {
  const session = await auth();
  return session?.user?.email ?? null;
}
```

Optional-chain every hop. `session`, `session.user` and `user.email` are
each independently absent in a legitimate state, so a non-null assertion
here is a crash waiting for its first anonymous request.

## What this block does not do

It configures authentication; it does not persist anything. Without a
database adapter Auth.js keeps the session in a signed cookie, which is
the only strategy available — so the block has no session-strategy field
to offer. Persisting users, accounts and sessions needs a database unit
as well, and the pairing is where that choice appears.
