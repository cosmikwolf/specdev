# probe-rs — Rust codegen conventions

These conventions describe how a `probe_rs` block table on a module spec
maps to Rust source. They layer on top of the project's profile's
`module.conventions.md`; concerns specific to probe-rs (probe selection,
attach modes, memory access widths, trace-sink setup) live here.

The block targets [probe-rs](https://probe.rs) as its concrete library.
A module carrying `[probe_rs]` is **host-side**: it runs on a
workstation and talks to a separate device over SWD or JTAG. It is not
firmware, and none of the emitted code is `no_std`.

## Crate dependencies

This block contributes one crate:

```toml
probe-rs = "0.32"
```

The USB transport is pure Rust (`nusb`), so no system libusb is needed on
any host. Linux still needs a udev rule granting the user access to the
probe; that is a packaging concern, not a codegen one.

**Do NOT** add a decoder, symbol reader, plotting crate, or GUI crate
from this block. This block moves bytes between a host and a target and
nothing else. Anything that gives those bytes meaning is a separate
concern and arrives through a bridge or through the project's own
`architecture.toml` `[[external_deps]]`.

## Type emission

Each entity kind emits one type plus its constructor. Emit them in the
module named by the spec, following the project's profile's
`module.conventions.md` for visibility and file placement.

Session ids, sampler ids and trace-source ids are `Identifier`s and pass
through to Rust names unchanged. A field with a schema `default` is
emitted with that default already applied — do **not** emit an `Option`
for a field that has a default, because the spec has already answered it.

Fallible functions return the project's own error type. The examples
below use `Box<dyn Error>` so they stay about the probe-rs calls; real
emission maps `probe_rs::Error`, `DebugProbeError` and `ArmError` into
the project's enum per the project's profile's `module.conventions.md`.

**Do NOT** hold a `Session` in a `static` or share it between threads. A
`Session` owns the probe transport, is not `Sync`, and must live on one
thread. Emit a channel out of that thread when another thread needs the
data.

## Session emission

One `[[probe_rs.sessions]]` entry becomes one attach path. The three
steps are always the same and always in this order: choose a probe,
configure the wire, then attach.

```rust
use std::str::FromStr;

use probe_rs::probe::DebugProbeSelector;
use probe_rs::probe::list::Lister;
use probe_rs::{Permissions, Session};

let lister = Lister::new();

let mut probe = match probe_selector {
    Some(selector) => lister.open(DebugProbeSelector::from_str(selector)?)?,
    None => {
        let probes = lister.list_all();
        let info = probes.first().ok_or("no debug probe found")?;
        info.open()?
    }
};

if let Some(khz) = speed_khz {
    let _applied: u32 = probe.set_speed(khz)?;
}

let permissions: Permissions = if allow_erase_all {
    Permissions::new().allow_erase_all()
} else {
    Permissions::new()
};

let session: Session = probe.attach(chip, permissions)?;
```

`attach_mode = "attach_under_reset"` swaps the terminal call for
`probe.attach_under_reset(chip, permissions)?`. Everything before it is
identical.

`speed_khz` must be applied to the `Probe`, before attach. `set_speed`
returns the speed actually applied, which may be lower than requested.

When `probe_selector` is absent, emit the first-probe-found arm shown
above. When it is present, emit only the selector arm — a spec that names
a probe means to use that probe, and silently falling back to another one
is the wrong behaviour for a workstation with two targets plugged in.

**Do NOT** emit `Permissions::new().allow_erase_all()` unless
`allow_erase_all = true`. **Do NOT** call `set_speed` after `attach` —
`attach` consumes the `Probe` and there is no handle left to set it on.

## Sampler emission

One `[[probe_rs.samplers]]` entry becomes one periodic read loop. The
address list is a **runtime input**: this block does not model where
addresses come from.

`session.core(core_index)?` yields the `Core`; every read goes through
the `MemoryInterface` trait, which must be in scope. Dispatch on width:

```rust
use probe_rs::{MemoryInterface, Session};

let sample = match request.width {
    Width::U8 => Sample::Scalar(core.read_word_8(request.address)? as u64),
    Width::U16 => Sample::Scalar(core.read_word_16(request.address)? as u64),
    Width::U32 => Sample::Scalar(core.read_word_32(request.address)? as u64),
    Width::U64 => Sample::Scalar(core.read_word_64(request.address)?),
    Width::Bytes(len) => {
        let mut buffer: Vec<u8> = vec![0; len];
        core.read(request.address, &mut buffer)?;
        Sample::Bytes(buffer)
    }
};
```

Every scalar accessor takes a `u64` address and returns the value
directly. `read` fills a caller-owned byte slice instead, and is the only
correct choice for an aggregate: a struct read as four separate words can
tear across a write from the running target, while one `read` is a single
transaction.

`on_error` selects the pass body:

- `skip` — `out.push(read_one(&mut core, request).ok());` The pass
  continues and the session survives. One bad address must not stop the
  other signals.
- `abort` — `out.push(read_one(&mut core, request)?);` The first failure
  propagates and the loop stops.
- `reconnect` — drop the `Session`, re-run the attach path, resume.

`max_batch` chunks the address list (`requests.chunks(max_batch)`) so one
oversized request cannot stall the interval.

`interval_ms` becomes a deadline loop. The interval is a floor, not a
guarantee:

```rust
use std::time::{Duration, Instant};

let interval = Duration::from_millis(interval_ms);
let mut deadline = Instant::now();

deadline += interval;
if let Some(remaining) = deadline.checked_duration_since(Instant::now()) {
    std::thread::sleep(remaining);
} else {
    deadline = Instant::now();
}
```

Advancing the deadline from the previous deadline stops one slow pass
from shifting every later one; re-basing on overrun stops the loop from
running passes back-to-back trying to catch up.

**Do NOT** busy-wait between passes — the probe transport is the
bottleneck and spinning steals the CPU the consumer needs. **Do NOT**
halt the core to read memory: these accessors work against a running
target, and halting changes the behaviour being observed. **Do NOT** call
`session.core()` once and keep the `Core` across passes; it borrows the
session mutably and holding it blocks everything else.

## Trace-source emission

One `[[probe_rs.trace_sources]]` entry becomes one sink setup plus the
reader that drains it. Every `SwoConfig` setter is a **consuming builder
method**: reassign the result, never call one for its side effect.

```rust
use probe_rs::architecture::arm::component::TraceSink;
use probe_rs::architecture::arm::swo::SwoConfig;

let mut config: SwoConfig = SwoConfig::new(tpiu_clk_hz);

if let Some(rate) = baud {
    config = config.set_baud(rate);
}

config = if manchester {
    config.set_mode_manchester()
} else {
    config.set_mode_uart()
};

config = config.set_continuous_formatting(continuous_formatting);

session.setup_tracing(core_index, TraceSink::Swo(config))?;
```

`sink` selects the variant: `TraceSink::Swo(config)`,
`TraceSink::Tpiu(config)`, or the unit variant `TraceSink::TraceMemory`
(which carries no clock or baud, so emit no `SwoConfig` for it).

`SwoConfig::new` takes the frequency **in Hz** of the clock feeding the
trace unit. The prescaler producing the requested baud is derived from
it, so a wrong value yields a garbled stream rather than an error.

Draining differs by sink kind. A streaming sink yields a `std::io::Read`:

```rust
use std::io::Read;

let mut reader = session.swo_reader()?;
let read: usize = reader.read(buffer)?;
```

`trace_memory` is polled in blocks instead —
`let data: Vec<u8> = session.read_trace_data()?;`.

Emit the reader as a **return value**, handed to the caller. What the
bytes mean is not this block's concern; the emitted code must not inspect
them.

`data_watch_units` arms one DWT comparator per listed unit:

```rust
for &(unit, address) in watches {
    session.add_swv_data_trace(unit, address)?;
}
```

An armed comparator emits a trace packet whenever its watched address is
accessed, with no polling and no target-side instrumentation. Hardware
provides four at most and typically fewer.

Teardown runs in this order — comparators first, then the unit:

```rust
for &unit in units {
    session.remove_swv_data_trace(unit)?;
}
session.disable_swv(core_index)?;
```

A disarm issued after the unit is disabled has nothing to act on.

**Do NOT** parse, frame, or interpret the trace bytes here — not even to
find a sync sequence. **Do NOT** set `continuous_formatting = true`
unless several sources share the pin; a formatted stream must be
de-framed before decoding, and emitting that flag for a single-source
setup breaks every consumer downstream. **Do NOT** ignore the drain rate:
an SWO overflow is silent, so a reader called too rarely loses packets
with nothing reporting it.
