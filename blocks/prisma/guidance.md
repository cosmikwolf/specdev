# prisma — implementation guidance

Rules an agent follows when it writes or edits code for a module with a `[prisma]` table. No check verifies them;
review does.

## Versions and packages

- Check the pinned Prisma major before copying any example: the current major needs a driver adapter, imports the
  client from the generator's `output` path (never from the client package), and keeps the database URL in
  `prisma.config.ts`, not in the schema.
- The CLI, the client and the driver adapter come from one release and stay on the same version; name the version
  when adding them (the CLI's default tag can be a release candidate).
- `[prisma.datasource].adapter` picks exactly one driver adapter, and the adapter decides the schema's provider. A
  native driver compiles at install time, so CI needs a build toolchain.

## Files the project owns

- `prisma/schema.prisma` (models, relations, the generator with an explicit `output`) and `prisma.config.ts` are
  hand-owned. The models live in the schema file, never restated in the spec.
- `[prisma.datasource].generated_client` repeats the generator's `output`. The generated client is never edited and
  never committed: add its directory to `.gitignore`.
- Run `prisma generate` after each schema edit, before type-checking: module code imports generated types.

## The singleton

- The module with `[prisma.datasource]` owns the one client instance, built with the adapter. Its body runs on the
  server only.
- In development the instance is kept on a global: dev servers reload modules, and a client per reload exhausts the
  connection pool.
- Derive its import from `generated_client`, relative to the module's own file; do not copy a literal path.
- A module with `[prisma].client` imports the singleton from the module that `client` names.

## Transactions

- Use the interactive form when later writes depend on earlier reads, and run every query in it on the transaction
  client, never the outer one. A body over `timeout` is rolled back. The array form is for independent writes.
- A function in another module that must join a transaction takes the transaction client as a parameter (defaulting
  to the singleton). If a spec says a call happens "within the same transaction" and the callee's spec has no such
  parameter, the specs contradict each other: fix the callee's spec first; do not inline its writes.

## Errors

- Branch on the known request error's code, never its message: unique violation → a field error; not found → 404;
  write conflict → retry the whole transaction; pool timeout → back off. Re-throw anything else.
