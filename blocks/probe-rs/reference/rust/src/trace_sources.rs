//! `trace_sources` — one `[[probe_rs.trace_sources]]` entry becomes one
//! trace-sink setup plus the reader that drains it.
//!
//! Mirrors the README worked example. The bytes leave here undecoded: what
//! they mean is a packet-protocol concern and lives elsewhere.

use std::error::Error;
use std::io::Read;

use probe_rs::Session;
use probe_rs::architecture::arm::component::TraceSink;
use probe_rs::architecture::arm::swo::SwoConfig;

/// `sink = "swo"`.
///
/// `SwoConfig::new` takes the frequency of the clock feeding the trace
/// unit, in Hz — on most parts the core clock. The prescaler producing the
/// requested baud is derived from it, so a wrong value yields a garbled
/// stream rather than an error. Every setter is a consuming builder
/// method: reassign the result, never call one for its side effect.
pub fn setup_swo(
    session: &mut Session,
    core_index: usize,
    tpiu_clk_hz: u32,
    baud: Option<u32>,
    manchester: bool,
    continuous_formatting: bool,
) -> Result<(), Box<dyn Error>> {
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
    Ok(())
}

/// `sink = "tpiu"`. Identical configuration, different sink variant. On
/// parts that do not distinguish the two this is the same hardware.
pub fn setup_tpiu(
    session: &mut Session,
    core_index: usize,
    tpiu_clk_hz: u32,
    baud: Option<u32>,
) -> Result<(), Box<dyn Error>> {
    let mut config: SwoConfig = SwoConfig::new(tpiu_clk_hz);
    if let Some(rate) = baud {
        config = config.set_baud(rate);
    }
    session.setup_tracing(core_index, TraceSink::Tpiu(config))?;
    Ok(())
}

/// `sink = "trace_memory"`. The on-chip buffer carries no clock or baud
/// configuration, so `TraceSink::TraceMemory` is a unit variant.
pub fn setup_trace_memory(session: &mut Session, core_index: usize) -> Result<(), Box<dyn Error>> {
    session.setup_tracing(core_index, TraceSink::TraceMemory)?;
    Ok(())
}

/// Draining a streaming sink (`swo` / `tpiu`).
///
/// `swo_reader` borrows the session mutably and yields a `std::io::Read`.
/// Emitted code returns the reader to its caller rather than parsing here:
/// what the bytes mean is not this block's concern.
///
/// Call this often enough to keep the probe's buffer clear. An SWO
/// overflow is silent — the target drops packets and nothing reports it.
pub fn drain_swo(session: &mut Session, buffer: &mut [u8]) -> Result<usize, Box<dyn Error>> {
    let mut reader = session.swo_reader()?;
    let read: usize = reader.read(buffer)?;
    Ok(read)
}

/// Draining `trace_memory`. Polled in blocks rather than streamed: the
/// on-chip buffer is read back whole, so there is no `Read` to hand out.
pub fn drain_trace_memory(session: &mut Session) -> Result<Vec<u8>, Box<dyn Error>> {
    let data: Vec<u8> = session.read_trace_data()?;
    Ok(data)
}

/// `data_watch_units` — arm one DWT comparator per listed unit.
///
/// Each armed comparator emits a trace packet whenever its watched address
/// is accessed, with no polling and no target-side instrumentation.
/// Hardware provides four at most and typically fewer, so an over-long
/// list fails at the probe rather than being silently truncated.
pub fn arm_data_watches(
    session: &mut Session,
    watches: &[(usize, u32)],
) -> Result<(), Box<dyn Error>> {
    for &(unit, address) in watches {
        session.add_swv_data_trace(unit, address)?;
    }
    Ok(())
}

/// Releasing comparators, and then the trace unit itself. Emitted teardown
/// runs in this order: a disarm after the unit is disabled has nothing to
/// act on.
pub fn teardown(
    session: &mut Session,
    core_index: usize,
    units: &[usize],
) -> Result<(), Box<dyn Error>> {
    for &unit in units {
        session.remove_swv_data_trace(unit)?;
    }
    session.disable_swv(core_index)?;
    Ok(())
}
