# next-auth+prisma — implementation guidance

Rules an agent follows when it writes or edits the Auth.js configuration that a Prisma client backs. No check
verifies them; review does.

## The adapter package

- The Auth.js Prisma adapter belongs to this pairing, not to either block: it is useless without both. Its version does
  not follow Prisma's; pin it on its own.
- Pass it the singleton client from the module that the binding's `client` names. Its parameter type accepts almost
  anything, so a passing type check does not prove a real client was passed. One client package in the lockfile is
  what protects this pairing.

## Session strategy

- With an adapter present, Auth.js defaults to database sessions; the binding states `session_strategy` explicitly.
- `database`: a row per session, revocable, one query per session read. `jwt`: a signed cookie, not revocable before
  expiry; the adapter still links accounts and stores verification tokens.
- A database session is read wherever the session reader runs. If middleware reads it, the Prisma driver adapter must
  run on the Edge runtime — HTTP or Edge-native drivers work, Node-only drivers do not; with those, use `jwt` or keep
  the session reader out of middleware. Where the profile has no middleware role, any driver works with `database`.
- With `database`, set `edge_split = true` and keep the adapter out of the edge-safe half.

## The schema

- `schema.prisma` holds the four models the adapter needs — user, account, session, verification token — with the
  fields Auth.js expects. Account and session cascade on user delete; otherwise orphaned rows read as live sessions.
- Keep the verification token model even when no email provider uses it yet.
- Run `prisma generate` before type-checking: the generated client types the adapter call.
