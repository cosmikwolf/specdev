# probe-rs — implementation guidance

Rules an agent follows when it writes or edits code for a module with a `[probe_rs]` table. The module is host-side:
it runs on a workstation and reaches a device over SWD or JTAG. No check verifies these rules; review does.

## Scope

- This block moves bytes between host and target, nothing more. No decoder, symbol reader, plot or GUI crate here;
  meaning arrives through a bridge or the project's own dependencies.
- A field with a schema default is a plain value in the code, not an option: the spec has answered it.
- Errors map into the project's own error type.
- A session owns the probe transport: one thread, never a `static`, never shared. Another thread gets the data
  through a channel.

## Sessions

- Each `[[probe_rs.sessions]]` entry attaches in a fixed order: choose a probe, set the wire speed, attach (or
  attach under reset for `attach_mode = "attach_under_reset"`).
- With `probe_selector`, use that probe only; never fall back to another one. Without it, take the first probe found.
- Set `speed_khz` on the probe before attaching; the probe reports the speed it applied, which can be lower.
- Grant erase-all permission only when `allow_erase_all = true`.

## Samplers

- Each `[[probe_rs.samplers]]` entry is one periodic read loop. The address list is a runtime input.
- Read at the declared width. An aggregate is one block read, not several word reads that can tear against a
  running target's writes.
- `on_error`: `skip` records the failure and continues the pass; `abort` stops at the first failure; `reconnect`
  drops the session, attaches again and resumes.
- `max_batch` chunks the address list so one large request cannot stall the interval.
- `interval_ms` is a floor: advance a deadline from the previous deadline, and re-base it after an overrun. Never
  busy-wait.
- Never halt the core to read; take the core handle per pass, not across passes.

## Trace sources

- Each `[[probe_rs.trace_sources]]` entry is one sink setup plus the reader that drains it. The sink configuration
  builder consumes and returns itself: keep the result.
- The trace clock frequency is in Hz; a wrong value gives a garbled stream, not an error. The memory sink takes no
  clock or baud.
- Return the reader to the caller; this block never parses, frames or inspects trace bytes.
- Set continuous formatting only when several sources share the pin.
- Drain fast enough: an overflow is silent.
- Arm one data-watch comparator per listed unit (hardware has four at most). Tear down comparators first, then the
  trace unit.
