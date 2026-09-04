# probe-rs block

Host-side debug-probe block targeting [probe-rs](https://probe.rs) 0.32.
Contributes a namespaced `[probe_rs]` table on `Module` describing a
**debugger connection**: which target to attach to, which memory to
sample while it runs, and how to configure the on-chip trace sink.

The block models the *debugger*, never the firmware. A module carrying
`[probe_rs]` runs on a workstation and talks to a separate device.

## When to use this block

Inline `probe-rs` when your project:

- Reads or writes target memory from a host program — a live variable
  viewer, a telemetry bridge, a test harness that asserts against device
  state, a flashing tool.
- Wants those reads to happen **against a running target**. probe-rs
  reads memory over SWD without halting the core, which is what makes
  non-intrusive observation possible.
- Configures SWO, TPIU, or on-chip trace memory, and wants the raw byte
  stream out.

Don't use this block if:

- You are writing the firmware. probe-rs is a host library; the code that
  runs on the microcontroller belongs to whatever HAL or framework block
  the target needs.
- Your host talks to the device over a normal serial port, USB CDC, or a
  network socket. There is no debug probe in that picture and this block's
  entire surface would be inert.
- You only need to flash a binary once, at build time. `probe-rs run` and
  `cargo-flash` already do that from the command line, with no code to
  generate.

## What this block contributes

| Kind | Table | Models |
|---|---|---|
| `Session` | `[[probe_rs.sessions]]` | Probe selection, target chip, attach mode, wire speed, permissions |
| `Sampler` | `[[probe_rs.samplers]]` | A periodic memory-read group: interval, access width, batch size, error policy |
| `TraceSource` | `[[probe_rs.trace_sources]]` | SWO / TPIU / trace-memory sink setup and DWT comparator arming |

Two intra-block relations, `sampler_on_session` and `trace_on_session`,
make a sampler or trace source that names an undeclared session fail at
validate time rather than at attach time.

## What this block does NOT contribute

**Meaning.** The block hands out raw addresses in and raw bytes out,
which is exactly what probe-rs itself does.

- **Where addresses come from.** A sampler's address list is a runtime
  input. Recovering an address for a named symbol is a debug-info concern
  and belongs in a bridge against whichever unit owns symbol resolution —
  see `elf-dwarf+probe-rs`.
- **What the trace bytes mean.** The sink hands out a `std::io::Read` and
  stops there. Packet decoding belongs in a bridge against whichever unit
  owns the packet protocol — see `itm+probe-rs`.
- **How anything is displayed.** No plotting, no UI, no storage format.

Keeping all three out is what lets the block stay small and lets a
project swap the symbol source or the decoder without touching it.

## Sharp-test rationale

**Leg 1 — concreteness.** The block names probe-rs, one specific library
with one specific API. It is not a "debugger" or "probe" abstraction:
"which X?" has a single answer. A GDB-server-based tool or an
OpenOCD-driven one would be a different block, because the vocabulary
(`SwoConfig`, `TraceSink`, `MemoryInterface`) is probe-rs's own.

**Leg 2 — codegen-completeness.** Conforming TOML produces working code:
an attach path, a read loop with real width dispatch and a real deadline,
and a trace setup that streams bytes. Every worked example in the codegen
prose is compiled by `reference/rust/`, so a signature that moves
upstream breaks CI here rather than in a downstream project.

**Leg 3 — no cross-block reach.** The codegen prose names no other
block's types. Both places where a cross-block temptation exists are
answered the same way — hand the caller a value and stop: an address is
a `u64` the caller supplies, and the trace stream is an `impl Read` the
caller consumes. Both couplings live in bridges.

## Companion units

| Companion | Bridge | What the pairing adds |
|---|---|---|
| `elf-dwarf` | `elf-dwarf+probe-rs` | Turns a named global variable into an address, width and decoded type, so a sampler can read symbols instead of raw addresses |
| `itm` | `itm+probe-rs` | Feeds the SWO byte stream into an ITM/DWT packet decoder, and reconciles the trace clock so timestamps mean something |

The block also pairs naturally with a UI block for display, but that
coupling needs no bridge: sampled values reach the UI as ordinary data
over a channel.

## Deferred

- **RTT.** probe-rs ships `probe_rs::rtt`, a second transport that is
  neither memory sampling nor trace. It deserves its own entity kind (a
  channel has a direction, a buffer and a mode) and is deferred until a
  project needs it.
- **Flashing.** `probe_rs::flashing` is a different verb from observing.
  Deferred until a project wants flash-then-observe in one tool.
- **Register access.** Core registers are readable through the same
  session, but only meaningfully while halted, which contradicts the
  running-target premise the sampler is built on.
- **Chip-name validation.** The target registry is a runtime artifact of
  the pinned probe-rs version, so a wrong name surfaces as a clear
  attach-time error rather than a schema error.
