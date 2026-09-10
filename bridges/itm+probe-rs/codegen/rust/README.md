# itm+probe-rs — Rust codegen conventions

These conventions describe how a `bindings."itm+probe-rs"` table on a
module spec maps to Rust source. They layer on top of the project's
profile's `module.conventions.md`, and on top of both endpoint blocks'
own codegen prose.

**Bridge prose wins.** Where this file describes how the trace byte
stream is consumed, it replaces the `probe-rs` block's instruction to
hand the reader to a caller and the `itm` block's instruction to take a
reader as a parameter: under this binding, they are joined here.

## Crate dependencies

This bridge contributes **no new crate**. probe-rs's `SwoReader<'_>`
implements `std::io::Read`, and itm's `Decoder<R>` accepts any
`R: Read`, so the two compose directly with no adapter. Both blocks'
deps.yaml supply everything the emitted code needs.

## The stream join

Emit the reader and the decoder together, in the trace thread:

```rust
use itm::{Decoder, DecoderOptions};

let reader = session.swo_reader()?;
let decoder = Decoder::new(reader, DecoderOptions { ignore_eof });

for item in decoder.singles() {
    match item {
        Ok(packet) => { /* route */ }
        Err(_) => continue,
    }
}
```

The decoder **borrows the session** for as long as it lives, so the
session cannot service memory reads while a decoder is open. Emit the
trace path on its own session and thread; do not try to interleave
trace decoding and memory sampling on one session.

A decode error is per-packet, not fatal — a trace link drops bytes
whenever the target outruns the probe.

**Do NOT** buffer the SWO bytes into a `Vec` and decode afterwards
unless the spec asks for a capture. The decoder pulls as fast as it is
driven, and an intermediate buffer only adds a place for an overflow to
hide.

## Clock reconciliation

This is half the reason the bridge exists. probe-rs needs the trace
unit's input frequency to compute the SWO prescaler; itm needs the same
frequency to turn timestamp ticks into seconds. They are two
independently-typed numbers that must agree, and **nothing in either
block can notice when they do not**: a mismatch yields a perfectly
well-formed packet stream on a time axis wrong by exactly their ratio.

Emit both from one value:

```rust
match source {
    TimestampClock::TraceSource => Ok(tpiu_clk_hz),
    TimestampClock::Decoder => {
        decoder_clock_hz.ok_or_else(|| "decoder clock_hz is required".into())
    }
}
```

`timestamp_clock = "trace_source"` (the default) takes the bound trace
source's `tpiu_clk_hz` and uses it for **both** `SwoConfig::new` and
`TimestampsConfiguration::clock_frequency`. `timestamp_clock = "decoder"`
takes `[itm.decoder].clock_hz` for the decoder while the sink keeps its
own — correct only on a part whose trace unit is fed by a different
clock than the prescaler is computed against.

```rust
let iter = decoder.timestamps(TimestampsConfiguration {
    clock_frequency: clock_hz,
    lts_prescaler,
    expect_malformed: true,
});
```

**Do NOT** emit a literal frequency in either place. Both must trace
back to the one resolved value, or the next person to change the clock
will change one of them.

## Comparator arming

The other half. Emit `add_swv_data_trace` per bound watch, taking the
comparator from the `itm` side and the address at runtime:

```rust
for (binding, address) in bindings {
    if binding.arm != Arm::OnStart {
        continue;
    }
    session.add_swv_data_trace(usize::from(binding.comparator), *address)?;
}
```

`arm = "manual"` emits the same call as a function the application
invokes once it knows the address — the usual case when the address
comes from a symbol lookup.

Routing decoded packets back needs the binding, because the packet
carries only the comparator number and does not name the address it
matched:

```rust
let comparator = match packet {
    TracePacket::DataTraceValue { comparator, .. } => *comparator,
    TracePacket::DataTraceAddress { comparator, .. } => *comparator,
    TracePacket::DataTracePC { comparator, .. } => *comparator,
    _ => return None,
};

bindings.iter().find(|binding| binding.comparator == comparator)
```

Teardown releases comparators first, then the unit —
`remove_swv_data_trace` per binding, then `disable_swv`. A disarm issued
after the unit is disabled has nothing to act on.

**Do NOT** arm a comparator the bound trace source does not list (see
ITMPR-001). **Do NOT** assume a packet belongs to the only watch you
declared; route by comparator even with one binding, because an
unrelated comparator armed by other tooling produces packets on the same
stream.

## Discipline rules

Cross-cutting constraints no single block can check, and that the
reduction cannot express as relations. Emitted code enforces them.

- **ITMPR-001 — comparator agreement.** The bound `itm` data watch's
  `comparator` must appear in the bound `probe_rs` trace source's
  `data_watch_units`. The itm block has a number; the probe-rs block has
  a list; neither can see the other. A disagreement produces a decoder
  waiting forever for packets from a comparator nobody armed — no error,
  no packets, no explanation. Reject at codegen time with a message
  naming both endpoints and both values.
- **ITMPR-002 — prescaler agreement.** `[itm.decoder].lts_prescaler`
  records what the **target** was programmed with. Nothing in this
  pairing programs it, so codegen must not imply it does; emit it into
  the generated documentation as a stated assumption.
- **ITMPR-003 — formatting.** A trace source with
  `continuous_formatting = true` produces a TPIU-framed stream that must
  be de-framed before packet decoding. The itm decoder does **not**
  de-frame. Reject the pairing at codegen time rather than emitting a
  decoder that will see nothing but malformed packets.
- **ITMPR-004 — sink kind.** `sink = "trace_memory"` is polled in blocks
  via `read_trace_data` and yields no `Read`, so the streaming join does
  not apply. Emit the block-polling path feeding a cursor over the
  returned bytes, or reject if the spec expects streaming.
