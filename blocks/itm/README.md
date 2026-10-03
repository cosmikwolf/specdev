# itm block

Packet-protocol block targeting the [itm](https://github.com/rtic-scope/itm)
crate 0.9.0-rc.1, a decoder for the ARM CoreSight ITM/DWT packet
protocol (ARM Appendix D4). Contributes a namespaced `[itm]` table on # PLANNING-OK: ARM architecture manual appendix, not a D-entry
`Module` describing **how to read a trace stream**: which stimulus ports
carry what, which DWT comparators mean what, and how the stream is
timestamped.

The block models the protocol, not the transport. The decoder takes any
`std::io::Read`, so the same spec serves a live probe, a capture file, or
a socket.

## When to use this block

Inline `itm` when your project:

- Consumes an ITM or DWT trace stream from a Cortex-M3/M4/M7/M33 target
  and needs the packets turned into typed values.
- Reads firmware instrumentation — an `ITM_SendChar`-style write to a
  stimulus port, whether that carries text or a binary sample.
- Reads DWT data watchpoints, where hardware emits a packet on every
  access to a watched address with no firmware involvement at all.
- Needs a real time axis over those events, reconstructed from the
  stream's own timestamp packets.

Don't use this block if:

- Your target has no trace unit. Cortex-M0/M0+ have no ITM, and the
  entire surface would be inert.
- Your firmware sends telemetry over a normal serial port, USB, or RTT.
  Those are byte streams with their own framing; nothing here applies.
- You only need to move trace bytes around without interpreting them.
  Decoding is the whole point of this block.

## What this block contributes

| Kind | Table | Models |
|---|---|---|
| `StimulusPort` | `[[itm.stimulus_ports]]` | A software instrumentation channel: port number and payload type |
| `DataWatch` | `[[itm.data_watches]]` | A DWT comparator: unit number, packet kind, payload width |
| — | `[itm.decoder]` | Iterator mode, timestamp clock and prescaler, EOF and malformed-packet policy |

`[itm.decoder]` is a **nameless singleton**: it carries settings rather
than identity, so the code reads it directly and it is not materialized
as a graph entity. This follows the nih-plug block's treatment of its
process-config table.

The block declares **no relations**. A stimulus port and a data watch are
independent sources that happen to share a stream; neither references the
other.

## What this block does NOT contribute

- **The bytes.** Every decoder takes an `impl Read` parameter.
  Nothing here opens a probe, a port, or a file.
- **Arming the comparators.** A `DataWatch` entry says what comparator 2
  *means*; something else has to actually arm comparator 2 on the target.
  That binding is a bridge — see `itm+probe-rs`.
- **Configuring the target's ITM.** The prescaler field records what the
  target was programmed with so timestamps can be reconstructed. It does
  not program anything.
- **Display or storage.** No plotting, no UI, no file format.

## Sharp-test rationale

**Leg 1 — concreteness.** The block names one specific published crate
decoding one specific documented protocol. "Which X?" has a single
answer: the vocabulary (`TracePacket`, `Instrumentation`,
`DataTraceValue`, `LocalTimestampOptions`) is the itm crate's own, and a
different decoder would model the same wire protocol with different
types and therefore be a different block.

**Leg 2 — rule-pack completeness.** Every field constrains the code: the
decoder mode and timestamp settings, each port's payload width, each
comparator's packet kind and value width. The guidance carries the rules
no check can verify — partial-line buffering on a text port, the access
direction kept, no zero-padding of a short payload.

**Leg 3 — no cross-block reach.** The guidance names no other
block's types. The input is an `impl Read` type parameter, which is the
narrowest possible seam: the block cannot reach toward a transport
because it never names one.

## Why a separate block rather than part of a probe block

The obvious alternative is to fold decoding into whichever unit supplies
the bytes. Three things argue against it, and they are the reason this
block exists:

1. **The protocol outlives the transport.** The same packets arrive from
   a debug probe today and from a capture file in a regression test
   tomorrow. A decoder welded to a probe cannot be used on a file.
2. **The two have different release clocks.** The packet protocol is
   fixed by an ARM specification; a probe library changes with its
   hardware support. Pinning them together forces one to move when only
   the other did.
3. **Neither can express the pairing alone.** A probe library knows about
   trace pins and baud rates but nothing about stimulus ports; a decoder
   knows about ports but nothing about how to arm a comparator. The
   binding between them is exactly a bridge's job.

## Companion units

| Companion | Bridge | What the pairing adds |
|---|---|---|
| `probe-rs` | `itm+probe-rs` | Feeds the SWO byte stream into the decoder, arms the declared comparators, and reconciles the trace clock so timestamps mean something |

## Deferred

- **Exception tracing.** `TracePacket::ExceptionTrace` carries the
  interrupt entry/exit events an RTOS-aware viewer wants. Modelling it
  needs a vocabulary for exception numbers and an opinion about how to
  pair entry with exit; deferred until a project needs the timeline.
- **PC sampling.** `TracePacket::PCSample` supports a statistical
  profiler, which is a different tool from a variable viewer.
- **Event counters.** `TracePacket::EventCounterWrap` reports CPI, LSU
  and fold counters — profiling data, deferred with PC sampling.
- **Port paging.** Stimulus ports above 31 are reachable only through the
  `Extension` packet's page mechanism, which not every implementation
  emits. The schema allows the numbers; the guidance does not cover pages.
- **A stable upstream release.** The crate's newest release is a 2022
  release candidate. It is what probe-rs itself uses in its own trace
  example, so it is the path the ecosystem takes; revisit if upstream ever
  ships a final release.
