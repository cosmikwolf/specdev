# TypeScript Syntax Patterns

Pure language syntax reference for TypeScript code generation. These patterns apply to any domain. Framework-specific layout (routes, components, server actions) belongs to the profile that uses this tier, not here.

---

## File Layout

An ESM TypeScript package:

```
src/<module>.ts               # Implementation modules (one per spec module)
src/<module>.tsx              # Modules that render JSX
src/<module>/index.ts         # Directory module (the ESM analog of `mod.rs`)
src/index.ts                  # Public entry point (re-exports), when the package is a library
tests/<module>.test.ts        # Unit tests
package.json                  # Package manifest
tsconfig.json                 # Compiler options
```

- One language spans **two extensions and two grammars**. `.ts` is the default;
  `.tsx` is required for any file containing JSX. `<T>x` type assertions do
  not parse under the TSX grammar — use `x as T` in `.tsx` files.
- Visibility is `export`. A declaration not exported is module-private and is
  never part of the module's contract surface.
- Externally owned trees are never spec modules: `node_modules/`, build output
  (`dist/`, `.next/`), generated declaration files (`*.d.ts` emitted by a tool),
  and any framework-generated type shims.

---

## Source-to-spec mapping

- **`src/<snake_case>.ts` or `.tsx`** → one spec module. File stem is
  snake_case; the spec `name` is PascalCase (`event_card.tsx` → `EventCard.toml`).
  The spec's `paths.source` points at `src`.
- **`src/<snake_case>/index.ts`** → the same module in directory form. Resolution
  tries `<snake>.ts`, `<snake>.tsx`, `<snake>/index.ts`, `<snake>/index.tsx`, in
  that order; the first that exists wins.
- A module whose file name the framework fixes (route files, entry points) sets
  `source_file` in its spec explicitly instead of relying on the convention.
- **`src/index.ts`** (the export barrel) → not a spec module; it exposes the
  public surface of the modules.
- **Config files** (`tsconfig.json`, bundler and test-runner config) → never
  spec modules; owned by the toolchain.

---

## Naming Conventions

- Types (classes, interfaces, type aliases, enums, namespaces): `UpperCamelCase`
- Functions, methods, variables, parameters, properties: `lowerCamelCase`
- Constants: `lowerCamelCase`, or `SCREAMING_SNAKE_CASE` only for true
  compile-time constants shared across modules
- Files: `snake_case` stems, one module per file
- Private class members: the `#name` syntax (hard privacy) for state; the
  `private` modifier where a subclass or test needs structural access
- No `I` prefix on interfaces; no Hungarian notation

---

## Doc Comments

Use `/** … */` JSDoc blocks on every exported declaration; the first sentence is
the summary.

```ts
/**
 * A token-bucket rate limiter.
 *
 * Refills `capacity` tokens per `interval`. Not safe across workers.
 */
export class RateLimiter {
  /** Attempts to consume `tokens`; returns `false` when exhausted. */
  tryConsume(tokens: number): boolean { … }
}
```

Type information lives in the signature, not in `@param {type}` tags. `@param`
and `@returns` carry prose only.

---

## Strictness

All generated code compiles under `"strict": true` with these additions, which
the profile's `tsconfig.json` template sets:

- `noUncheckedIndexedAccess` — index reads are `T | undefined`.
- `exactOptionalPropertyTypes` — `label?: string` does not admit `undefined`
  assigned explicitly.
- No `any` in generated code. Use `unknown` and narrow.
- No non-null assertions (`x!`). Restructure so the type carries the proof, or
  handle the `undefined` case explicitly.
- `as` casts only at trust boundaries (parsed JSON, DOM), and each one names why
  in a comment.

---

## Error Handling

- Programming errors (violated preconditions): throw `TypeError` / `RangeError`
  — thrown, not caught.
- Recoverable domain failures: one error class per module, extending `Error`,
  with `name` set so it survives serialization:

```ts
/** Thrown when the device rejects a command. */
export class DeviceRejectedError extends Error {
  override readonly name = "DeviceRejectedError";
  constructor(readonly reason: string) {
    super(`device rejected: ${reason}`);
  }
}
```

- Alternatively, model expected failure as data with a discriminated union:

```ts
export type ParseResult =
  | { ok: true; value: Frame }
  | { ok: false; message: string };
```

Pick one style per module and state it in the module spec's prose.

---

## Async

- `Promise<T>` for one-shot async operations; `AsyncIterable<T>` or a typed
  event emitter for sequences.
- `async`/`await` bodies; never raw `.then()` chains in generated code.
- Cancellation: `AbortSignal` threaded as the last parameter (`signal?: AbortSignal`);
  long-lived objects expose `dispose(): void` or implement `Symbol.dispose`.

---

## Class and Module Patterns

### Immutable value type

```ts
export interface Reading {
  readonly celsius: number;
  readonly timestamp: Date;
}
```

Prefer an `interface` plus a factory function over a class when there is no
behavior.

### Service with lifecycle

```ts
export class SensorService {
  #sub: (() => void) | undefined;
  constructor(private readonly transport: Transport) {}
  async start(): Promise<void> { … }
  dispose(): void {
    this.#sub?.();
  }
}
```

### Interface + implementation

```ts
export interface Clock {
  now(): Date;
}
export class SystemClock implements Clock {
  now(): Date {
    return new Date();
  }
}
```

### Plain-function module

A module need not export a class. A set of exported functions over an exported
type is the idiomatic unit for stateless logic:

```ts
export interface Cart { readonly lines: readonly Line[] }
export function total(cart: Cart): number { … }
export function addLine(cart: Cart, line: Line): Cart { … }
```

---

## Test File Pattern

`tests/<module>.test.ts` using the project's test runner (the profile names it):

```ts
import { describe, expect, it } from "vitest";
import { RateLimiter } from "../src/rate_limiter";

describe("RateLimiter", () => {
  it("consumes up to capacity", () => {
    const limiter = new RateLimiter({ capacity: 2 });
    expect(limiter.tryConsume(2)).toBe(true);
    expect(limiter.tryConsume(1)).toBe(false);
  });
});
```

- One `describe` per type or function group under test; `it` names are
  behavior sentences.
- Component tests render with the framework's testing library and live in the
  same layout as `.test.tsx`.

---

## Items Naming

The `[[items]]` vocabulary for TypeScript lives in `items.schema.yaml` (the
`kind` enum) with extraction driven by `items.scm`. Naming rules the
extractor and authors share:

- Members qualify with `.`: `Store.put` (`kind = "method"`), `Circle.diameter`
  (`kind = "getter"`). Type parameters never appear in a name (`Box.put`, not
  `Box<T>.put`).
- A getter/setter pair legitimately shares one name — identity is `(name, kind)`.
- Public means exported at top level, and for members: no `private` /
  `protected` modifier and no `#` prefix. A `#`-prefixed member, or any member
  of a non-exported host, is never an item.
- Overloads: one item per implementation. Overload signatures are not items.
- Not items: constructors (spec the class), fields (state belongs to the
  enclosing declaration), `import` statements, destructured top-level bindings,
  config-file exports.
