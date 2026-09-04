//! `stream` — joining the probe's SWO reader to the packet decoder, and
//! reconciling the timestamp clock between the two blocks.
//!
//! Mirrors the README worked example.

use std::error::Error;

use itm::{Decoder, DecoderOptions, LocalTimestampOptions, TimestampsConfiguration};
use probe_rs::Session;
use probe_rs::architecture::arm::component::TraceSink;
use probe_rs::architecture::arm::swo::SwoConfig;

/// The `timestamp_clock` binding field.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TimestampClock {
    TraceSource,
    Decoder,
}

/// The clock reconciliation, which is half the reason this bridge exists.
///
/// The probe-rs side needs the trace unit's input frequency to compute
/// the SWO prescaler. The itm side needs the same frequency to turn
/// timestamp ticks into seconds. They are two independently-typed
/// numbers that must agree, and nothing in either block can notice when
/// they do not: a mismatch yields a perfectly well-formed packet stream
/// on a time axis that is wrong by exactly their ratio.
pub fn resolve_clock_hz(
    source: TimestampClock,
    tpiu_clk_hz: u32,
    decoder_clock_hz: Option<u32>,
) -> Result<u32, Box<dyn Error>> {
    match source {
        TimestampClock::TraceSource => Ok(tpiu_clk_hz),
        TimestampClock::Decoder => {
            decoder_clock_hz.ok_or_else(|| "decoder clock_hz is required".into())
        }
    }
}

/// Setting up the sink, using the same frequency that will drive the
/// decoder. Emitting these two from one value is the point.
pub fn setup_swo(
    session: &mut Session,
    core_index: usize,
    tpiu_clk_hz: u32,
    baud: Option<u32>,
) -> Result<(), Box<dyn Error>> {
    let mut config: SwoConfig = SwoConfig::new(tpiu_clk_hz);

    if let Some(rate) = baud {
        config = config.set_baud(rate);
    }

    config = config.set_mode_uart().set_continuous_formatting(false);

    session.setup_tracing(core_index, TraceSink::Swo(config))?;
    Ok(())
}

/// The join itself, in `mode = "singles"`.
///
/// `session.swo_reader()` yields a `SwoReader<'_>` that implements
/// `std::io::Read`, and `Decoder::new` accepts any `R: Read`. The two
/// compose directly — this function existing and compiling is the proof
/// that the pairing needs no adapter crate.
///
/// The decoder borrows the session for as long as it lives, so the
/// session cannot be used for memory reads while a decoder is open.
/// Emitted code therefore takes the reader inside the trace thread and
/// keeps memory sampling on a separate session.
pub fn decode_singles(
    session: &mut Session,
    ignore_eof: bool,
    limit: usize,
) -> Result<Vec<itm::TracePacket>, Box<dyn Error>> {
    let reader = session.swo_reader()?;
    let decoder = Decoder::new(reader, DecoderOptions { ignore_eof });

    let mut collected: Vec<itm::TracePacket> = Vec::new();

    for item in decoder.singles().take(limit) {
        match item {
            Ok(packet) => collected.push(packet),
            // Per-packet, not fatal: a trace link drops bytes whenever
            // the target outruns the probe.
            Err(_) => continue,
        }
    }

    Ok(collected)
}

/// The join in `mode = "timestamped"`, driven by the reconciled clock.
pub fn decode_timestamped(
    session: &mut Session,
    clock_hz: u32,
    lts_prescaler: LocalTimestampOptions,
    limit: usize,
) -> Result<Vec<itm::TimestampedTracePackets>, Box<dyn Error>> {
    let reader = session.swo_reader()?;
    let decoder = Decoder::new(reader, DecoderOptions { ignore_eof: true });

    let iter = decoder.timestamps(TimestampsConfiguration {
        clock_frequency: clock_hz,
        lts_prescaler,
        expect_malformed: true,
    });

    let mut collected: Vec<itm::TimestampedTracePackets> = Vec::new();

    for item in iter.take(limit) {
        match item {
            Ok(batch) => collected.push(batch),
            Err(_) => continue,
        }
    }

    Ok(collected)
}
