# TypeScript Build & Toolchain Patterns

Pure toolchain reference for TypeScript projects on Node. These patterns apply to any domain. Framework build steps (`next build` and its kin) belong to the profile that uses this tier.

---

## package.json

The package manifest. Minimal library package:

```json
{
  "name": "my-package",
  "version": "0.1.0",
  "description": "One-line package description.",
  "type": "module",
  "packageManager": "pnpm@10.15.0",
  "engines": { "node": ">=22.18.0" },
  "main": "./dist/index.js",
  "types": "./dist/index.d.ts",
  "scripts": {
    "build": "tsc -p tsconfig.json",
    "check": "tsc --noEmit",
    "test": "vitest run",
    "format": "prettier --write ."
  },
  "dependencies": {},
  "devDependencies": {
    "@types/node": "^22.0.0",
    "typescript": "^7.0.0",
    "vitest": "^5.0.0",
    "prettier": "^3.6.0"
  }
}
```

- `"type": "module"` makes `.ts` files ESM. Code uses `import` / `export` only;
  no `require`.
- `packageManager` pins the package-manager version for corepack. `engines.node`
  states the runtime floor; both are project facts that live here.
- `scripts` are the toolchain's entry points; a profile may override or add to
  them.
- **The pins are facts with a date, not defaults.** Registry-checked 2026-09-09:
  `typescript 7.0.2`, `vitest 5.0.0`, `prettier 3.9.6`. `@types/node` is pinned
  to `^22` on purpose — its major tracks the Node major in `engines`, not the
  newest release (26.x at that date), and Vitest 5 peers `^22 || >=24`. Re-read
  the registry before you copy these into a new profile; a pin recalled from
  memory anchors every API claim written against it to a guess.

### Dependency shapes

A package.json dependency value is one string, in one of these forms:

| Form | Value |
|---|---|
| registry range | `"^1.2.3"` |
| git | `"git+https://…/x.git#v1.4.0"`, `"github:acme/x"` |
| path (copied, its deps installed) | `"file:../x"` |
| link (symlinked, nothing installed for it) | `"link:../x"` |
| workspace member | `"workspace:*"` |

- Build, test, lint and code-generation tooling are `devDependencies`; everything shipped at runtime is
  `dependencies`.
- `package.json` is the project's and the only source of truth for dependencies, scripts, `packageManager` and
  `engines`; the spec copies none of it.

---

## Lockfile

The package manager owns the lockfile — `pnpm-lock.yaml`, `package-lock.json`
or `yarn.lock` — and rewrites it whenever `package.json` changes. It is
**committed** and never hand-edited. The profile's quick gate brings it back in
step locally — its first step installs without `--frozen-lockfile`, so a new
dependency can move the lockfile. In CI the install is frozen (pnpm refuses to
change the lockfile when `CI` is set), so a stale lockfile fails CI, which is
the point.

---

## pnpm Workspaces

Multi-package repos use `pnpm-workspace.yaml` at the root and one lockfile for every member:

```yaml
packages:
  - "app"
  - "packages/shared"
```

- Members depend on each other with `"workspace:*"`. A member `package.json` carries no workspace marker. Run
  `pnpm install` once at the root; members have no lockfile.
- pnpm also reads `overrides`, `catalog` and `onlyBuiltDependencies` from this file.

---

## Toolchain Commands

| Purpose | pnpm | npm |
|---|---|---|
| Fetch deps | `pnpm install` | `npm install` |
| Fetch deps, lockfile must already match (CI) | `pnpm install --frozen-lockfile` | `npm ci` |
| Type check | `pnpm exec tsc --noEmit` | `npx tsc --noEmit` |
| Format | `pnpm exec prettier --write .` | `npx prettier --write .` |
| Run tests | `pnpm exec vitest run` | `npx vitest run` |
| Build a library | `pnpm exec tsc -p tsconfig.json` | `npx tsc -p tsconfig.json` |
| Run a script | `pnpm run <name>` | `npm run <name>` |

- `tsc --noEmit` exiting 0 is the TypeScript analog of `cargo check` — a
  type-clean tree is the minimum bar for generated code. It does not run the
  code and does not see framework-level errors; a framework profile adds its own
  slower gate.
- **None of these commands is "the compile gate".** The gate is the profile's
  `gates:` registry in `specs/specdev/profile.yaml` — a `label`, a `scope` and
  `steps` as argv lists — and `specdev-analyzer gate --label <l> [--module <m>]…`
  is what runs it and writes the rows. This table is the menu a profile author
  picks steps from, not a procedure anyone follows by hand. A typical TypeScript
  registry has a module-scope gate (`[pnpm, install]`, `[pnpm, exec, tsc,
  --noEmit]`, `[pnpm, exec, vitest, run, --passWithNoTests]`) and, for a
  framework, a batch-scope gate holding the slower whole-tree build.
- `node --version` and the package-manager version are machine facts. The
  project pins them through `engines` and `packageManager`, which are files.

---

## tsconfig.json

```json
{
  "compilerOptions": {
    "target": "ES2022",
    "module": "NodeNext",
    "moduleResolution": "NodeNext",
    "strict": true,
    "noUncheckedIndexedAccess": true,
    "exactOptionalPropertyTypes": true,
    "verbatimModuleSyntax": true,
    "isolatedModules": true,
    "skipLibCheck": true,
    "rootDir": "src",
    "outDir": "dist"
  },
  "include": ["src", "tests"]
}
```

- `verbatimModuleSyntax` forces `import type` for type-only imports.
- A framework profile ships its own `tsconfig.json` template (JSX mode, path
  aliases, plugin entries). Code must be clean under the project's committed
  options — do not weaken the options to make code pass.
- `tsconfig.json` is a **paired file**: the profile declares it under
  `paired_files:`, and `specdev-analyzer bootstrap` writes it **only when
  absent**. After that it is the project's — `next build` rewrites it on its
  first run.

---

## Formatter and Linter

- `prettier` formats.
- A linter (`eslint` with the TypeScript plugin, or `biome`) is optional per
  project.

---

## Externally Owned Trees

Never hand-edited, never spec modules:

- `node_modules/` — installed packages.
- Build output: `dist/`, `build/`, a framework's `.next/` or equivalent.
- The lockfile.
- Tool-generated declaration shims (`next-env.d.ts` and its kin).
- Incremental-build state (`*.tsbuildinfo`), written by `tsc` beside the
  tsconfig that produced it.
- Agent-instruction blocks a dev server maintains in `AGENTS.md` / `CLAUDE.md`.
  Measured: `next dev` re-adds its block on every run.
- Generated clients from schema-first tools (an ORM's generated client, GraphQL
  codegen output) — the analog of Dart's `*.g.dart`.
- Editor and CI configuration.
