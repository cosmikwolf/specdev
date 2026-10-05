# next-auth block

Authentication for a TypeScript project through
[Auth.js](https://authjs.dev/) (`next-auth` v5). Adds a `[next_auth]`
table to `Module` marking the one module that holds the configuration.

**Version.** The block is built on Auth.js's single-config major, published as a pre-release; the older stable line
has another API and is not a fallback. The guidance says to pin it exactly (a range on a pre-release accepts the next
beta silently) and to watch for the final release by hand.

## What the table says

```toml
[next_auth]
config_file = "auth.ts"          # where the one NextAuth() call lives
providers   = ["github"]         # provider ids, one import each
edge_split  = true               # split the config for the Edge runtime
base_path   = "/api/auth"        # optional; this is the default
```

## What the table deliberately does not say

**No session strategy.** Without a database adapter Auth.js has exactly
one strategy, so there is nothing to decide. The choice appears only
when an adapter does, and an adapter needs a second unit — so it belongs
to the `next-auth+prisma` bridge, not here.

**No callbacks, no provider options, no custom pages.** Those are code
and live in the file `config_file` names. Restating them in TOML would
give one fact two sources.

**`providers` is not an enum.** Auth.js ships roughly eighty providers
and adds more between releases; an enumerated list here would reject a
valid provider the day it shipped. A wrong id fails at the import, which
is the checker that does know the list.

## What the block contributes

One package: `next-auth`. `@auth/prisma-adapter` is deliberately absent —
it is a join product, useless unless a database unit is present too, so
contributing it here would hand a project a dependency it cannot
satisfy. The `next-auth+prisma` bridge owns it.

## Pairing

`next-auth+prisma` adds database-backed sessions and owns the adapter
dependency and the session-strategy decision.
