# itm+probe-rs bridge

Pairwise wiring between the `itm` block (ITM/DWT packet protocol) and
the `probe-rs` block (debug probe transport and trace-sink setup).
Contributes data-watch ↔ trace-source bindings under
`bindings."itm+probe-rs".armed_watches[]`.

This is the bridge that turns a stream of trace bytes into named
signals on a real time axis.

## When to use this bridge

Inline `itm+probe-rs` when:

- Your module has both an `[itm]` table (a decoder and its ports or
  watches) and a `[probe_rs]` table (a session and a trace source).
- You want DWT data watchpoints armed from the same spec that says what
  they mean.
- You want a time axis whose seconds are real, which means the decoder's
  clock and the trace unit's clock have to come from one number.

Don't inline if:

- You decode a captured trace file rather than a live target. The `itm`
  block alone covers that — its decoder takes any reader.
- You move trace bytes somewhere else without decoding them (a recorder,
  a relay). The `probe-rs` block alone covers that.
- Your instrumentation is software-only stimulus ports with no DWT
  watches and no need for reconstructed timestamps. The blocks compose
  loosely there and the binding table would be empty.

## What this bridge contributes

Two decisions, both genuinely unownable by either endpoint:

**1. Which comparator.** The `itm` block declares that comparator 2
carries a signed 16-bit value. The `probe-rs` block can arm comparator 2
against an address. Neither can see the other's number — one has a
`comparator` field, the other a `data_watch_units` list — and a mismatch
produces a decoder waiting forever for packets nobody will send. No
error, no packets, no explanation. That is ITMPR-001.

**2. Which clock.** probe-rs needs the trace unit's input frequency to
compute the SWO prescaler; itm needs the same frequency to turn
timestamp ticks into seconds. Two independently-typed numbers that must
agree, where a mismatch yields a well-formed packet stream on a time
axis wrong by exactly their ratio — the worst kind of failure, because
everything looks right.

One relation, `watch_armed_on`, from `itm.DataWatch` to
`probe-rs.TraceSource`. `REDUCE_UNRESOLVED` flags a binding naming a
watch or source that was never declared.

## What this bridge does NOT contribute

- **A new crate.** There is no join product. probe-rs's `SwoReader<'_>`
  implements `std::io::Read` and itm's `Decoder<R>` accepts any
  `R: Read`, so they compose directly. The bridge's reference crate
  compiles that join, which is the proof no adapter is needed — and the
  reason this bridge ships no `deps.yaml`.
- **The watched addresses.** `add_swv_data_trace` takes an address at
  runtime. Where it comes from is not this pairing's concern; a symbol
  lookup is a different bridge's job.
- **Programming the target's ITM.** The prescaler field records what the
  firmware or debugger already configured. See ITMPR-002.
- **De-framing.** A TPIU-formatted stream must be de-framed before
  decoding, and neither endpoint does it. See ITMPR-003.

## Inline order

```
/spec-inline itm
/spec-inline probe-rs
/spec-inline-bridge itm+probe-rs
```

Both blocks must be inlined first: the bridge's reduction references
`itm.DataWatch` and `probe-rs.TraceSource`, and an unresolved endpoint
kind surfaces as a silent `REDUCE_UNRESOLVED` rather than a clear error.

## Co-residency

A binding's `data_watch` and `trace_source` must both be declared on the
same module that carries the bindings table — in practice the module
owning the debug session. Cross-module wiring is not supported in v1.

## Why the decoder is not an endpoint

`[itm.decoder]` is a nameless singleton and materializes no entity, so
there is nothing for a relation to point at. The clock reconciliation is
therefore carried as a relation **attribute** (`timestamp_clock`) on the
watch binding rather than as a second relation. If a project ever needs
several decoders on one module, the decoder becomes an entity kind in the
itm block first and this bridge grows a second relation — not the other
way round.

## Sharp-test rationale

**Concrete pairing.** Both endpoints name specific published crates:
the itm decoder on one side, probe-rs on the other.

**Codegen-complete.** Conforming TOML produces working code: a decoder
built directly on the probe's reader, one resolved clock feeding both
the sink and the timestamp reconstruction, comparators armed and torn
down in the right order, and packets routed back to the watch that
declared them. Both worked examples are compiled by `reference/rust/`.

**The contribution belongs to the bridge.** Comparator agreement cannot
live in `itm` without teaching it about probe arming, and cannot live in
`probe-rs` without teaching it about packet semantics. Either would
break the other block's leg 3.

## Deferred

- **Stimulus-port bindings.** Software instrumentation needs no arming,
  so ports compose without a binding today. A binding would become
  useful for declaring which port a given trace source carries when a
  project runs two sources at once.
- **TPIU de-framing.** ITMPR-003 currently rejects a formatted stream.
  De-framing is a small, well-specified transform and could become part
  of this bridge once a project needs two trace sources on one pin.
- **Overflow accounting.** `TracePacket::Overflow` means packets were
  lost. The pairing knows both the requested baud and the probe in use,
  so it is the natural place to turn overflows into a "reduce the rate"
  diagnostic. Deferred until there is real measurement behind the advice.
- **Exception-trace routing.** Waiting on the itm block's deferred
  exception-trace vocabulary.
