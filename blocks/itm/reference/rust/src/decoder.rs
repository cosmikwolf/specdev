//! `decoder` — the `[itm.decoder]` singleton becomes one decoder
//! construction plus the iterator it is driven through.
//!
//! Mirrors the README worked example. The reader is generic: this block
//! never names where the bytes come from.

use std::io::Read;

use itm::{
    Decoder, DecoderOptions, LocalTimestampOptions, Singles, TimestampedTracePackets, Timestamps,
    TimestampsConfiguration, TracePacket,
};

/// `mode = "singles"`.
///
/// Yields one packet at a time in arrival order, with no timestamp
/// reconstruction. Correct when the firmware carries its own notion of
/// time, or when only the ordering of events matters.
pub fn singles<R: Read>(reader: R, ignore_eof: bool) -> Singles<R> {
    Decoder::new(reader, DecoderOptions { ignore_eof }).singles()
}

/// `mode = "timestamped"`.
///
/// Groups packets under a reconstructed offset from the trace clock's
/// start, built from the local and global timestamp packets in the
/// stream. `clock_frequency` and `lts_prescaler` must both match the
/// target's actual configuration: they are how ticks become seconds, so
/// a wrong value yields correct packets at wrong times.
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

/// The `lts_prescaler` enum, one variant per schema value. `enabled`
/// means a prescaler of 1.
pub fn prescaler_from_spec(value: &str) -> LocalTimestampOptions {
    match value {
        "disabled" => LocalTimestampOptions::Disabled,
        "div4" => LocalTimestampOptions::EnabledDiv4,
        "div16" => LocalTimestampOptions::EnabledDiv16,
        "div64" => LocalTimestampOptions::EnabledDiv64,
        _ => LocalTimestampOptions::Enabled,
    }
}

/// Driving the `singles` iterator.
///
/// The item type is `Result<TracePacket, DecoderError>`. A decode error
/// is per-packet, not fatal: keep iterating. A trace link drops bytes
/// whenever the target outruns the probe, and one corrupt packet should
/// cost one packet rather than the whole session.
pub fn collect_singles<R: Read>(reader: R, limit: usize) -> Vec<TracePacket> {
    let mut collected: Vec<TracePacket> = Vec::new();

    for item in singles(reader, false).take(limit) {
        match item {
            Ok(packet) => collected.push(packet),
            Err(_) => continue,
        }
    }

    collected
}

/// Driving the `timestamps` iterator.
///
/// Each item carries a batch of packets sharing one timestamp, plus the
/// malformed packets seen in the same window when `expect_malformed` is
/// set. `Timestamp` is an enum, not a plain duration: it reports how
/// trustworthy the time is, and a run of degraded variants is the
/// warning that the link is saturating.
pub fn collect_timestamped<R: Read>(
    reader: R,
    clock_hz: u32,
    limit: usize,
) -> Vec<TimestampedTracePackets> {
    let mut collected: Vec<TimestampedTracePackets> = Vec::new();

    let iter = timestamped(
        reader,
        false,
        clock_hz,
        LocalTimestampOptions::Enabled,
        true,
    );

    for item in iter.take(limit) {
        match item {
            Ok(batch) => collected.push(batch),
            Err(_) => continue,
        }
    }

    collected
}

/// Reading the quality of a reconstructed timestamp.
///
/// `Sync` is exact. The other three variants each mean the packet's true
/// time is only bounded, which is the decoder telling you the trace link
/// could not keep up. Emitted code must not flatten these to one number
/// without recording that it did.
pub fn timestamp_is_exact(batch: &TimestampedTracePackets) -> bool {
    matches!(batch.timestamp, itm::Timestamp::Sync(_))
}
