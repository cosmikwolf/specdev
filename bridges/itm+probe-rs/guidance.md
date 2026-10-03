# itm+probe-rs — implementation guidance

Rules an agent follows when it writes or edits code that decodes a probe's SWO trace stream as ITM/DWT packets. No
check verifies them yet; review does. ITMPR-001..004 are rules a future unit check will verify.

## The stream join

- No crate is added: the probe's SWO reader is a byte reader, and the decoder takes any byte reader. Under this
  binding the two are joined here, in the trace thread.
- The decoder borrows the session while it lives: the trace path gets its own session and thread; memory sampling
  does not share it.
- A decode error is per packet: keep going. Do not buffer the stream before decoding unless the spec asks for a
  capture.

## One clock

- The trace unit's input frequency sets the SWO prescaler and converts timestamp ticks to seconds. Both come from one
  resolved value; never a literal in either place.
- `timestamp_clock = "trace_source"` (default) uses the trace source's `tpiu_clk_hz` for both. `"decoder"` uses
  `[itm.decoder].clock_hz` for the decoder only — correct only where another clock feeds the trace unit.

## Comparators

- Arm one comparator per bound watch: on start for `arm = "on_start"`, or from a function the application calls
  when it knows the address for `arm = "manual"`.
- Route decoded data packets by comparator number, even with one binding: other tooling can arm comparators on the
  same stream.
- Tear down comparators first, then the trace unit.

## Rules

- **ITMPR-001:** the watch's `comparator` appears in the trace source's `data_watch_units`; otherwise the decoder
  waits forever. Refuse a mismatch, naming both values.
- **ITMPR-002:** `[itm.decoder].lts_prescaler` states what the target was programmed with; nothing here programs it.
  Document it as an assumption.
- **ITMPR-003:** a trace source with `continuous_formatting = true` gives a framed stream that the decoder does not
  de-frame — refuse the pairing.
- **ITMPR-004:** a `trace_memory` sink is polled in blocks, not streamed: feed the decoder from the polled bytes, or
  refuse when the spec expects streaming.
