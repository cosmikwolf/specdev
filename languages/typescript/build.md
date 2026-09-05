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
    "typescript": "^5.9.0",
    "vitest": "^3.2.0",
    "prettier": "^3.6.0"
  }
}
```

- `"type": "module"` makes `.ts` files ESM. Generated code uses `import` /
  `export` only; no `require`.
- `packageManager` pins the package-manager version for corepack. `engines.node`
  states the runtime floor; both are project facts the spec's architecture
  carries.
- `scripts` are the toolchain's entry points; a profile may override or add to
  them.

### Dependency shapes

Every `npm:` dependency a spec names lands in one of these forms:

```json
{
  "dependencies": {
    "hosted_caret": "^1.2.3",
    "hosted_exact": "1.2.3",
    "hosted_range": ">=1.2.0 <2.0.0",
    "hosted_tag": "next",
    "scoped": "^2.0.0",
    "from_git": "github:acme/from_git#v1.4.0",
    "from_git_url": "git+https://github.com/acme/from_git.git#semver:^1.4.0",
    "local_sibling": "file:../local_sibling",
    "workspace_member": "workspace:*",
    "aliased": "npm:real-package@^3.0.0"
  }
}
```

- `dependencies` — shipped at runtime. `devDependencies` — build, test, lint
  and codegen tooling. `peerDependencies` — for libraries that expect the host
  to provide a package (a component library and its UI framework).
- `optionalDependencies` and `overrides` are escape hatches; do not commit
  overrides that generated manifests would fight over.
- Scoped packages (`@scope/name`) sort with their scope; the emitter sorts keys
  lexically so the file is byte-stable.

---

## Lockfile

The package manager owns the lockfile — `pnpm-lock.yaml`, `package-lock.json`
or `yarn.lock` — and rewrites it whenever `package.json` changes. It is
**externally owned**: committed, never generated, never recorded as a generated
file. The compile gate that runs after generation is what brings the lockfile
back in step; a gate that runs with `--frozen-lockfile` belongs in CI, where
lockfile staleness is the thing under test, not in the per-module gate.

---

## pnpm Workspaces

Multi-package repos use `pnpm-workspace.yaml` at the root and one lockfile for
every member:

```yaml
packages:
  - "packages/*"
```

Members depend on each other with `"workspace:*"`. Run `pnpm install` once at
the root.

---

## Toolchain Commands

| Purpose | pnpm | npm |
|---|---|---|
| Fetch deps | `pnpm install` | `npm install` |
| Fetch deps, lockfile must already match (CI) | `pnpm install --frozen-lockfile` | `npm ci` |
| Type check (the compile gate) | `pnpm exec tsc --noEmit` | `npx tsc --noEmit` |
| Format | `pnpm exec prettier --write .` | `npx prettier --write .` |
| Run tests | `pnpm exec vitest run` | `npx vitest run` |
| Build a library | `pnpm exec tsc -p tsconfig.json` | `npx tsc -p tsconfig.json` |
| Run a script | `pnpm run <name>` | `npm run <name>` |

- `tsc --noEmit` exiting 0 is the TypeScript analog of `cargo check` — a
  type-clean tree is the minimum bar for generated code. It does not run the
  code and does not see framework-level errors; a framework profile adds its own
  slower gate.
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

- `verbatimModuleSyntax` forces `import type` for type-only imports, which the
  generated code always uses.
- A framework profile ships its own `tsconfig.json` template (JSX mode, path
  aliases, plugin entries). Generated code must be clean under the project's
  committed options — do not weaken the options to make generated code pass.
- `tsconfig.json` is template-once: written at bootstrap, then externally owned.

---

## Formatter and Linter

- `prettier` formats; run it on emitted files **before** they are recorded, never
  after — a format pass after recording changes the bytes and reads as a
  hand-edit.
- A linter (`eslint` with the TypeScript plugin, or `biome`) is optional per
  project and runs in fix mode only before the record step, for the same
  reason.

---

## Externally Owned Trees

Never generated, never recorded, never spec modules:

- `node_modules/` — installed packages.
- Build output: `dist/`, `build/`, a framework's `.next/` or equivalent.
- The lockfile.
- Tool-generated declaration shims (`next-env.d.ts` and its kin).
- Generated clients from schema-first tools (an ORM's generated client, GraphQL
  codegen output) — the analog of Dart's `*.g.dart`.
- Editor and CI configuration.
