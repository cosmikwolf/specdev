# Firmware domain — authoring guidance

This file is the reference the `/spec-design` and `/spec-edit` skills load
when authoring or editing firmware module specs. It tells the LLM how to
produce a module TOML that validates cleanly against
`schema/firmware/v1/module.schema.yaml` on the first try.

The schema is the contract. This doc explains the reasoning behind it.

---

## File layout

A firmware project has one of two shapes depending on whether it declares
`[[packages]]` in `spec.toml`.

**Flat (single-package):**

```
my-firmware/
├── spec.toml                          # no [[packages]] section
└── specs/
    ├── architecture.toml              # cross-module port connections (D17)
    ├── modules/
    │   └── <ModuleName>.toml          # one module per file, PascalCase filename
    ├── requirements/
    │   └── <topic>.toml               # requirements grouped by topic (D16)
    └── ui/
        └── <ScreenGroup>.toml         # optional; for UI projects
```

**Packaged (multi-crate / multi-subproject, D24):**

```
my-firmware/
├── spec.toml                          # declares [[packages]] for each crate
├── crates/
│   ├── core/        src/...           # package "core", source_path = "crates/core"
│   └── drivers/     src/...           # package "drivers", source_path = "crates/drivers"
└── specs/
    ├── architecture.toml              # cross-module ports; endpoints may be
    │                                   #   <pkg>::<Module>.<port>
    ├── modules/
    │   ├── core/
    │   │   └── Clock.toml
    │   └── drivers/
    │       └── Led.toml               # depends_on = ["core::Clock"]
    └── requirements/
        └── <topic>.toml               # requirements are NOT packaged
```

One module per file. Filename matches the `name` field (`StatusLed.toml` for
`name = "StatusLed"`). The `package` is **derived from the filesystem path** —
never write a `package` field inside the TOML.

---

## Spec modules and packages

A **spec module** is one coherent unit of behavior at one layer, touching at
most one peripheral concern. This is a *stricter* rule than "one Rust module
equals one spec module":

- A Cargo workspace *crate* may contain multiple spec modules (e.g. `drivers`
  crate contains `Led`, `Display`, `Button`).
- A crate that exposes types-only has no spec modules — its types become shared
  ports in `architecture.toml` instead.
- A spec module never spans layers: if a single `.rs` file is half-driver and
  half-middleware, split it into two spec modules (and ideally two files) when
  writing the spec.

When a project declares `[[packages]]`, references across packages must be
**qualified** as `<pkg>::<Module>`:

- `depends_on = ["core::Clock"]` — drivers crate depends on a core type.
- `[[connections]] from = "core::Clock.tick_out" to = "drivers::Led.tick_in"`

Within the same package, use unqualified names — they resolve locally first.
The analyzer will error with `PKG044` on ambiguous unqualified references when
the same name exists in multiple packages.

---

## Layer hierarchy

Every module declares a `layer`. Dependencies flow downward only, per the
`layers.allowed_deps` table in `domain.toml`:

| Layer       | Purpose                                                | May depend on               |
|-------------|--------------------------------------------------------|-----------------------------|
| PAC         | Register-level peripheral access.                      | (nothing)                   |
| HAL         | Typed wrappers over PAC; the thinnest possible layer.  | PAC                         |
| driver      | One peripheral per module. Owns the hardware.          | HAL, PAC                    |
| middleware  | Services that compose multiple drivers.                | driver, HAL, PAC            |
| application | User-facing behavior; top of the tree.                 | middleware, driver, HAL, PAC|

Infer `layer` from the module's purpose: a module that touches registers is
PAC, one that wraps a peripheral is driver, one that coordinates several
drivers is middleware, one that implements end-user behavior is application.

---

## Module fields

Required:
- `name` — PascalCase identifier matching the filename.
- `layer` — one of the layers above.

Optional (but usually present):
- `description` — one sentence. Read first by both humans and LLMs.
- `depends_on` — names of other modules this one calls. Names only, not
  paths. Must reference modules whose layer satisfies `allowed_deps`.
- `external_deps` — Cargo-style `[{ name, version, features? }]` entries.
- `satisfies` — requirement ids (`REQ-<NAMESPACE>-NNN`) this module
  implements. Every id here must exist in `specs/requirements/`.
- `metadata` — closed object; see next section.
- `hardware` — pins, clocks, peripherals this module owns.
- `ports` — typed data-flow endpoints used by `architecture.toml`.
- `actions` — public interface functions.
- `state_machines` — FSMs embedded in the module.
- `errors` — error variants the module raises.

A well-typed module often needs only `name`, `layer`, `description`,
`metadata`, and `actions`.

---

## Metadata — what to declare, what to omit

The `metadata` object is **closed**: only `MemoryModel`, `ConcurrencyModel`,
`ErrorHandling`, `Ownership`, `Timing` are accepted. Any other top-level key
is rejected by the schema.

`domain.toml` lists three categories in `required_metadata.module`:
`MemoryModel`, `ConcurrencyModel`, `ErrorHandling`. Declare these on every
module even if the values match the defaults.

Inside a category, values matching the domain default trigger META011
(warning) rather than an error. Prefer to *omit* default-valued optional
fields to keep specs lean.

### Defaults (from `domain.toml`)

| Category         | Field         | Default         |
|------------------|---------------|-----------------|
| MemoryModel      | allocation    | `static_alloc`  |
| MemoryModel      | owner         | `owned`         |
| ConcurrencyModel | protection    | `none`          |
| ConcurrencyModel | isr_safe      | `false`         |
| ConcurrencyModel | reentrant     | `false`         |
| ErrorHandling    | strategy      | `result_type`   |
| Ownership        | owner         | `owned`         |

### Concurrency inference rules

- Shared state across ISRs → `protection = "critical_section"` or
  `"disable_interrupts"`.
- Shared state across threads, scalar only → `protection = "atomic"`.
- Shared state across threads, compound → `protection = "mutex"`.
- No shared state → `protection = "none"`, `isr_safe = false` unless the
  module is pure (no reads or writes to shared memory).

### Error-handling inference rules

- Rust → `strategy = "result_type"`, `error_type = "<ModuleName>Error"`.
- C → `strategy = "errno"` or `"enum_variant"` depending on project
  conventions; declare in `spec.toml`.
- Application-layer modules with fatal init → `strategy = "panic"` is
  acceptable; lower layers must be recoverable.

---

## Hardware section

`hardware.pins[]`, `hardware.clocks[]`, `hardware.peripherals[]` describe
what the module owns. Each entry has a stable set of known keys (`name`,
`port`, `pin`, `role`, …) but accepts unknown keys so domain evolution does
not break existing specs. Still — avoid inventing keys unless the domain
genuinely lacks one; open an issue instead.

Ownership is enforced across the project: the same `peripheral.name` in two
modules triggers OWN050. Pin conflicts (same `port` + `pin`) are flagged
the same way.

---

## Ports

A port is a typed endpoint wired up in `specs/architecture.toml`. Each port
has:

- `name` (local to the module)
- `port_type` (the logical type — `AudioDataPort`, `LedCommandPort`, etc.)
- `direction` (`in`, `out`, `bidirectional`)
- optional `flow_type` (the data that flows; `PcmFrame`, `StatusPattern`).
- optional `conjugated` (Rust/C++ port symmetry).

Never declare port-to-port wiring in a module file. That lives in
`architecture.toml`.

---

## Actions

One action per public interface function. Keep them close to the real API:

- `name` in snake_case.
- `params[]` — each `{ name, type, direction }`. Inline form is fine for ≤3
  params; use expanded `[[actions.params]]` for more.
- `returns` — a type literal. Use `Result<T, E>` for Rust, `int` (return
  code) for C.
- `blocking` — `true` if the function may wait on hardware, DMA, network.
- `isr_safe` — `true` if callable from ISR. Must be `false` whenever
  `blocking` is `true` (enforced by ISR060).

Constructor actions are conventionally named `new` or `init`. They take
owned peripheral handles plus an optional config struct.

---

## State machines

An FSM has `name`, `initial`, `states`, and `transitions[]`. Transitions
name their `from`, `to`, and `event` in snake_case. The analyzer checks:

- `initial` ∈ `states` (FSM020)
- every `from`/`to` ∈ `states` (FSM021)
- every state reachable from `initial` (FSM022; warning)

---

## Errors

Each `errors[]` entry has a PascalCase `variant` and a `description`. The
list drives the generated error enum. Variants should be specific — prefer
`BufferUnderrun` to `Io`, prefer `PeerDisconnected` to `NetworkError`.

---

## Requirements and satisfies

Write requirements in `specs/requirements/<topic>.toml` with ids
`REQ-<NAMESPACE>-NNN`. Modules reference them via `satisfies = [...]`. Any
id in `satisfies` that does not exist in the requirements files triggers
REQ002.

Every requirement with `verified_by = []` triggers REQ001 (warning) at
project validate time — this flags orphaned requirements without failing
the build.

### Sources (traceability to external documents)

A requirement often paraphrases a clause from an external document
(protocol spec, standard, user brief). Declare each document once as a
top-level `[[source]]` and reference it by id:

```toml
[[source]]
id = "midi2-ump"
kind = "protocol"           # protocol | user | regulatory | derived
title = "MIDI 2.0 Universal MIDI Packet Specification"
version = "1.1.2"
url = "https://midi.org/..."

[[requirement]]
id = "REQ-UMP-0001"
text = "UMP messages shall be 32, 64, 96, or 128 bits."
source = ["midi2-ump"]       # one or more source ids
source_ref = "§2.1.4"        # free-form locator
```

Rules:

- `source` is an array — a requirement may derive from multiple documents
  when clauses overlap (e.g. UMP + MIDI-CI).
- `source_ref` is free-form (section, page, table, clause) and applies
  to every id listed in `source`. Keep it short and greppable.
- `kind = "user"` covers stakeholder requirements with no external spec.
  Title the source something like "Product brief" or "User stories".
- `kind = "derived"` covers internal decisions with no document. Prefer
  writing a short design note and referencing it rather than using
  `derived` liberally.
- An id in `source[]` that does not resolve to a declared `[[source]]`
  triggers REQ003 (error).

---

## Architecture file

`specs/architecture.toml` is a flat list of `[[connections]]` between
`Module.port` endpoints. Each endpoint must resolve to a declared port in
the named module; otherwise PORT030 fires.

Use architecture for wiring only. No state lives here — state belongs to
the modules themselves.

---

## Naming conventions

| Element            | Style                       | Example             |
|--------------------|-----------------------------|---------------------|
| Module name        | PascalCase                  | `StatusLed`         |
| Filename           | PascalCase.toml             | `StatusLed.toml`    |
| Action name        | snake_case                  | `set_pattern`       |
| Port name          | snake_case                  | `audio_out`         |
| Port type          | PascalCase                  | `AudioDataPort`     |
| State name         | snake_case                  | `discovering`       |
| Event name         | snake_case, `_evt` suffix   | `audio_start_evt`   |
| Error variant      | PascalCase                  | `BufferUnderrun`    |
| Requirement id     | `REQ-<NAMESPACE>-NNN`       | `REQ-AUDIO-001`     |
| Metadata key       | CamelCase                   | `MemoryModel`       |

---

## Common anti-patterns

1. **Inline cross-module connections.** Wiring goes in `architecture.toml`,
   not inside a module. If you find yourself writing `connect X.a to Y.b`
   inside a module, stop.
2. **Declaring shared resources in two modules.** One module owns each pin,
   peripheral, and clock. Borrowers do not declare; they receive the
   resource through the owner's action parameters.
3. **Mixing `isr_safe` and `blocking`.** An ISR-safe action must not block.
   Split into a blocking setup and a non-blocking query.
4. **Fat modules.** One peripheral per module. If a module reads a sensor
   AND formats output, split it at the layer boundary.
5. **Unused ports.** A port with no entry in `architecture.toml` is dead
   weight. Remove it or wire it.
6. **Implicit dependencies.** If an action takes a value produced by
   another module, that other module belongs in `depends_on`.

---

## When in doubt

- Read the existing modules in the project before authoring a new one.
- Prefer the narrowest description that answers "what does this do?".
- When a field's default is fine, omit it.
- Ask the user rather than guess when the answer cannot be inferred from
  `spec.toml` + existing specs + the current description.
