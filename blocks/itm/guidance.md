# itm — implementation guidance

Rules an agent follows when it writes or edits code for a module with an `[itm]` table: decoding the ARM CoreSight
ITM/DWT packet protocol. It models the protocol, never the transport. No check verifies these rules; review does.

## Scope and dependencies

- The decoder reads any byte reader passed in. No probe, transport or USB crate here.
- The decoder crate needs the version that carries DWT packets; check `Cargo.toml` before writing packet matches.
  Its serial-port feature is not needed when the caller supplies the reader.

## Decoding

- A packet that does not match a declared port or comparator is "not mine", not an error: decode functions return
  an absent value. Errors are for a malformed stream.
- A decode error is per packet: keep iterating.
- `mode` picks the path: `singles` yields packets in arrival order; `timestamped` groups packets under a
  reconstructed time, with `lts_prescaler` mapped one variant per spec value.
- A timestamp is exact only when synchronised; the other kinds bound the true time. Never flatten a bounded time to
  one number without recording that it is bounded — a run of them means the link saturates.
- `ignore_eof` stays true for a live stream: an idle target gives a temporary end of input.

## Stimulus ports

- Each `[[itm.stimulus_ports]]` entry decodes instrumentation packets of its port at the declared `payload` width,
  little-endian.
- A payload shorter than the declared width gives no value; never zero-pad it.
- A text port keeps a per-port buffer: a string splits across packets, and a packet can split a code point, so
  decode lossily and hold back the partial line.
- An overflow packet means data was lost; surface it as an event.
- A declared port that never appears is normal.

## Data watches

- Each `[[itm.data_watches]]` entry decodes the DWT packets of one comparator; the comparator number is the only
  link to the spec entry.
- `emits` picks the packet kind; a value is read at `value_type`, and a length mismatch gives no value.
- Keep the access direction: a comparator armed on both directions reports each value twice.
- A comparator number that never appears means the debugger's arming disagrees with the spec.
