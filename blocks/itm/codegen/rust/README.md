# itm — Rust codegen conventions

These conventions describe how an `itm` block table on a module spec
maps to Rust source. They layer on top of the project's profile's
`module.conventions.md`; concerns specific to the ITM/DWT packet
protocol (decoder construction, timestamp reconstruction, payload
widths) live here.

The block targets the [itm](https://github.com/rtic-scope/itm) crate as
its concrete decoder for the ARM CoreSight ITM/DWT packet protocol. It
models the **protocol**, never the transport: the decoder takes any
`std::io::Read` and does not care where the bytes came from.

## Crate dependencies

This block contributes one crate:

```toml
itm = { version = "0.9.0-rc.1", default-features = false }
```

The pin is a release candidate deliberately, and Cargo needs the
pre-release named exactly. It is the newest published version and the
only one carrying the current packet vocabulary; the older "stable"
release predates DWT packet support entirely. `default-features = false`
drops the `serial` feature and its `nix` dependency, which reads a trace
stream off a tty — not needed when the reader is supplied by the caller.

**Do NOT** add a probe, transport, or USB crate from this block. The
decoder's input is an `impl Read` parameter; whatever fills it is
someone else's concern and arrives through a bridge or through the
project's own `architecture.toml` `[[external_deps]]`.

## Type emission

Emit one enum per spec enum (`Payload`, `ValueType`) and one decoded-value
type per concern, in the module named by the spec, following the
project's profile's `module.conventions.md`.

Decode functions return `Option<T>`, not `Result<T, E>`. A packet that
does not match a declared port or comparator is **not an error** — the
stream carries every source interleaved, so most packets are "not mine"
to any given decoder. Reserve errors for a malformed stream.

**Do NOT** emit a decoder that owns its reader as a struct field unless
the spec calls for it. `Decoder::new` consumes the reader and
`.singles()` / `.timestamps()` consume the decoder, so the natural
emission is a function returning the iterator.

## Decoder emission

The `[itm.decoder]` singleton becomes one decoder construction. `mode`
selects which iterator — this is the one place the two paths diverge,
and they diverge completely.

`mode = "singles"` yields one packet at a time in arrival order:

```rust
use itm::{Decoder, DecoderOptions, Singles};

pub fn singles<R: Read>(reader: R, ignore_eof: bool) -> Singles<R> {
    Decoder::new(reader, DecoderOptions { ignore_eof }).singles()
}
```

`mode = "timestamped"` groups packets under a reconstructed offset from
the trace clock's start:

```rust
use itm::{LocalTimestampOptions, Timestamps, TimestampsConfiguration};

pub fn timestamped<R: Read>(
    reader: R,
    ignore_eof: bool,
    clock_hz: u32,
    lts_prescaler: LocalTimestampOptions,
    expect_malformed: bool,
) -> Timestamps<R> {
    Decoder::new(reader, DecoderOptions { ignore_eof }).timestamps(TimestampsConfiguration {
        clock_frequency: clock_hz,
        lts_prescaler,
        expect_malformed,
    })
}
```

`lts_prescaler` maps one variant per spec value —
`disabled` → `LocalTimestampOptions::Disabled`, `enabled` →
`Enabled` (a prescaler of 1), `div4` / `div16` / `div64` →
`EnabledDiv4` / `EnabledDiv16` / `EnabledDiv64`.

Both iterators yield a `Result`. A decode error is **per-packet, not
fatal**: keep iterating.

```rust
for item in singles(reader, false).take(limit) {
    match item {
        Ok(packet) => collected.push(packet),
        Err(_) => continue,
    }
}
```

`Timestamps` yields a `TimestampedTracePackets` carrying `packets`,
`malformed_packets`, `consumed_packets`, and a `timestamp`. That
timestamp is an **enum, not a duration**: `Timestamp::Sync` is exact,
while `UnknownDelay`, `AssocEventDelay` and `UnknownAssocEventDelay` each
mean the true time is only bounded. A run of the degraded variants is the
decoder reporting that the link is saturating.

```rust
matches!(batch.timestamp, itm::Timestamp::Sync(_))
```

**Do NOT** flatten a degraded timestamp to a single number without
recording that it was degraded — a plot drawn from bounded times that
claims to be exact is a lie the user cannot see. **Do NOT** set
`ignore_eof = false` for a live stream: an idle target produces a
temporary EOF that is not the end of the trace. **Do NOT** stop
iteration on the first `Err`.

## Stimulus-port emission

One `[[itm.stimulus_ports]]` entry becomes one payload decoder for
`TracePacket::Instrumentation`. Attribute first, then decode:

```rust
match packet {
    TracePacket::Instrumentation { port, .. } => Some(*port),
    _ => None,
}
```

`payload` selects the width. Every scalar is little-endian, which is the
byte order every Cortex-M writes:

```rust
Payload::U32 => PortValue::Unsigned(u32::from_le_bytes(bytes.try_into().ok()?) as u64),
Payload::F32 => PortValue::Float(f32::from_le_bytes(bytes.try_into().ok()?)),
Payload::Bytes => PortValue::Bytes(bytes.to_vec()),
```

`payload = "text"` needs a **per-port accumulator**, because a written
string can split across packets. Emit the partial-line hold-back:

```rust
buffer.partial.push_str(&String::from_utf8_lossy(bytes));
let mut lines: Vec<String> = buffer.partial.lines().map(|line| line.to_string()).collect();

if !buffer.partial.ends_with('\n') {
    let tail = lines.pop().unwrap_or_default();
    buffer.partial = tail;
} else {
    buffer.partial.clear();
}
```

`TracePacket::Overflow` is the one protocol packet a consumer must not
ignore: it means the target produced data faster than the link carried
it, so packets were lost. Emit it as a surfaced event.

**Do NOT** zero-pad a payload shorter than the declared width — return
`None`. A silently extended value is a plausible wrong number, which is
worse than a missing one. **Do NOT** treat a port that never appears as
an error; firmware that never writes a declared port is normal. **Do
NOT** emit `String::from_utf8` for a text port — a payload can split
mid-codepoint, and the lossy form is correct here.

## Data-watch emission

One `[[itm.data_watches]]` entry becomes one decoder for the DWT
data-trace packets of one comparator. These packets are produced by
hardware the debugger armed, with **no firmware involvement**, which is
what makes them non-intrusive.

All three data-trace variants carry the comparator number, and that is
the only link back to the spec entry — the packet does not name the
address it matched:

```rust
match packet {
    TracePacket::DataTraceValue { comparator, .. } => Some(*comparator),
    TracePacket::DataTraceAddress { comparator, .. } => Some(*comparator),
    TracePacket::DataTracePC { comparator, .. } => Some(*comparator),
    _ => None,
}
```

`emits` selects the variant to match, and for `value` the payload is read
at `value_type`:

```rust
let (access_type, bytes) = match packet {
    TracePacket::DataTraceValue { access_type, value, .. } => (access_type, value),
    _ => return None,
};

let access = match access_type {
    MemoryAccessType::Read => Access::Read,
    MemoryAccessType::Write => Access::Write,
};

ValueType::U32 => WatchEvent::Unsigned {
    access,
    value: u32::from_le_bytes(bytes.as_slice().try_into().ok()?) as u64,
},
```

Note `bytes.as_slice()`: a `DataTraceValue` payload is a `Vec<u8>`, so
the conversion goes through a slice, unlike the `Instrumentation`
payload which is already borrowed.

`emits = "address"` hands `data` out undecoded; `emits = "pc"` yields
`pc` as a `u32` directly.

**Do NOT** ignore `access_type`. A comparator armed on both directions
emits each value twice, once per direction, and code that drops the
distinction double-counts every write. **Do NOT** infer the payload width
from the payload length — read it at the declared `value_type` and
return `None` on a mismatch, because the length can be short for reasons
other than a narrower type. **Do NOT** assume a comparator number is
stable across runs; it identifies the unit the debugger armed, so a
number that never appears means the arming disagreed with the spec.
