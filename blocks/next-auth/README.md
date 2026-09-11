# next-auth block

Authentication for a TypeScript project through
[Auth.js](https://authjs.dev/) (`next-auth` v5). Adds a `[next_auth]`
table to `Module` marking the one module that holds the configuration.

**Pinned line: `next-auth` 5.0.0-beta.32, exactly.** The `latest` tag is
`4.24.15` and is not a fallback — v4 predates the single-config shape
this block is built on, and authjs.dev documents v5 only. Same call the
`itm` block makes for its `0.9.0-rc.1` pin: the newest published version
carrying the current vocabulary is a pre-release, and the "stable" is
not a usable alternative. The pin is exact because a caret on a
pre-release accepts the next beta silently.

**The GA is a manual trigger.** The freshness sweep drops the
pre-release identifier when comparing, so this pin reads `ahead` today
and will read `current` the day 5.0.0 ships — it will never say "move".

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

## Reference package

`reference/typescript/` compiles every worked example in the codegen
prose against the pinned `next-auth`. Four cases: the config, the Edge
split, the derived route file, and reading the session. Gate:
`pnpm install --frozen-lockfile && pnpm run check`.

## Pairing

`next-auth+prisma` adds database-backed sessions and owns the adapter
dependency and the session-strategy decision.
