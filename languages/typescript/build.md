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

A package.json dependency value is one string. Each spec reference therefore
carries exactly one shape — a registry range in `version`, or one source key
that owns the whole value — and the emitter renders it as follows:

| Spec reference | Emitted value |
|---|---|
| `version = "^1.2.3"` (no source key) | `"^1.2.3"` — the range verbatim |
| `git = { url = "git+https://…/x.git", ref = "v1.4.0" }` | `"git+https://…/x.git#v1.4.0"` |
| `git = { url = "github:acme/x" }` | `"github:acme/x"` |
| `path = "../x"` | `"file:../x"` — copied, and its own deps installed |
| `link = "../x"` | `"link:../x"` — symlinked, nothing installed for it |
| `workspace = "*"` | `"workspace:*"` — a member of the enclosing pnpm workspace |

```json
{
  "dependencies": {
    "@scope/widgets": "^2.0.0",
    "from_git": "git+https://github.com/acme/from_git.git#v1.4.0",
    "linked_tool": "link:../linked_tool",
    "local_sibling": "file:../local_sibling",
    "next": "^16.3.0",
    "shared": "workspace:*"
  }
}
```

- A `version` beside a source key is an error: there is no sibling-version slot.
  Put a git range in the ref (`ref = "semver:^1.4.0"`) and a workspace range in
  the value (`workspace = "^1.0.0"`). The git url is emitted verbatim — write the
  form the package manager accepts.
- `test_only = true` routes a reference to `devDependencies` (build, test, lint
  and codegen tooling). Everything else is `dependencies`, shipped at runtime.
- **Not spec input:** package aliases (`"npm:real-package@^3.0.0"` values),
  dist-tag protocols in the source slot, `peerDependencies`,
  `optionalDependencies` and `overrides`. An unknown key in a reference is an
  error, never a silent registry fallback. A library that needs a peer
  dependency states it in prose until a driver project needs the key.
- Scoped packages (`@scope/name`) sort with their scope; the emitter sorts keys
  lexically so the file is byte-stable.

### What the spec carries

- `[project].name` → `name` (verbatim; a name the registry would reject is a
  warning, never a rewrite), `[project].description` → `description`.
- `architecture.toml`: `version`; `[environment] node` and `package_manager`
  (override the profile's `engines.node` and `packageManager` defaults);
  `[scripts]` (overrides the profile's default scripts key by key);
  `[[external_deps]]`. Module files carry `external_deps[]`. Blocks and bridges
  contribute `npm:` entries from their `deps.yaml`.
- Merge order: user (architecture, then modules) > blocks > bridges. The first
  entry for a package wins; two versions or two sources for one package warn.
- Fixed key order: `name, version, description, private, packageManager, type,
  engines, scripts, dependencies, devDependencies`. Two-space indent, trailing
  newline, empty maps omitted. An `app`-shaped profile emits `"private": true`.

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
every member. The root `spec.toml` `[workspace].members[]` is the source; the
manifest step emits **both** the root `package.json` (`"private": true`, root
tooling deps, root scripts) and the workspace file, members in declaration
order:

```yaml
packages:
  - "app"
  - "packages/shared"
```

- `pnpm-workspace.yaml` is **gen-owned**: written beside `package.json` on
  every `manifest --write` and overwritten each time, recorded like the
  manifest. pnpm also reads `overrides` and `catalog` from this file; the
  emitter writes `packages:` only, so a hand-added key does not survive the
  next write. Put overrides in the spec when a key exists for them; until then
  they are not supported.
- Members depend on each other with `workspace = "*"` in the spec
  (`"workspace:*"` in the manifest). A member `package.json` carries no
  workspace marker. Run `pnpm install` once at the root; members have no
  lockfile.

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
