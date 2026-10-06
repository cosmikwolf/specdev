# next-auth — implementation guidance

Rules an agent follows when it writes or edits code for the module with a `[next_auth]` table: the one Auth.js
configuration of a Next.js app. No check verifies them; review does.

## Version

- The block targets Auth.js's single-config major, published as a pre-release. Pin it exactly: a range on a
  pre-release accepts the next beta, and betas can break. The older stable line has another API; do not fall back to
  it. Nothing announces the final release; watch for it by hand.

## The configuration module

- One module holds the configuration and exports the route handlers, the session reader, and sign-in and sign-out.
  Everything else imports them from it. Its body runs on the server only (use the profile's server guard when it has
  one).
- Each id in `providers` is one provider import and one entry in the provider list; a wrong id fails at its import.
- The route file under `base_path` (default `/api/auth`) only re-exports the GET and POST handlers — nothing else.

## Edge split and route gating

- `edge_split = true` splits the configuration in two: an edge-safe half (providers, callbacks, pages; nothing
  Node-only) and the full configuration that adds the rest. Check the edge-safe half against the config type without
  widening it.
- Check the profile's roles before assuming middleware exists. Where no role produces the middleware (proxy) file,
  routes are gated per module: each protected page or route handler reads the session and redirects or returns 401.
  `edge_split = true` stays valid and conservative either way.

## Reading the session

- The session reader returns nothing when nobody is signed in. Optional-chain every hop (session, user, field); never
  assert non-null.

## Scope

- This block configures authentication and persists nothing. Without a database adapter the session lives in a signed
  cookie. Persistence comes from a database unit and the bridge that pairs them.
