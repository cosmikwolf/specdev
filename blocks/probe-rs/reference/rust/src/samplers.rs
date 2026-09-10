//! `samplers` — one `[[probe_rs.samplers]]` entry becomes one periodic
//! read loop against a running core.
//!
//! Mirrors the README worked example. The address list is a runtime input:
//! this block does not model where addresses come from.

use std::error::Error;
use std::time::{Duration, Instant};

use probe_rs::{MemoryInterface, Session};

/// A single read request within a pass. `width` selects which
/// `MemoryInterface` call is used.
#[derive(Clone, Copy, Debug)]
pub struct Read {
    pub address: u64,
    pub width: Width,
}

/// The `default_width` enum, one variant per schema value.
#[derive(Clone, Copy, Debug)]
pub enum Width {
    U8,
    U16,
    U32,
    U64,
    Bytes(usize),
}

/// One sampled value, widened to `u64` so a single channel carries every
/// scalar width. An aggregate keeps its bytes.
#[derive(Clone, Debug)]
pub enum Sample {
    Scalar(u64),
    Bytes(Vec<u8>),
}

/// One read, dispatched on width.
///
/// Every scalar accessor takes a `u64` address and returns the value
/// directly. `read` fills a caller-owned byte slice instead, which is the
/// only correct choice for an aggregate: a struct read as four separate
/// words can tear across a write from the running target, while one
/// `read` is a single transaction.
pub fn read_one(core: &mut probe_rs::Core<'_>, request: Read) -> Result<Sample, probe_rs::Error> {
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
    Ok(sample)
}

/// One sampling pass with `on_error = "skip"`.
///
/// A failed read is dropped and the pass continues. This is the default
/// because one bad address must not stop the other signals — a viewer
/// whose whole display freezes because a single symbol moved is worse than
/// one with a gap in one trace.
///
/// `max_batch` chunks the address list so one oversized request cannot
/// stall the interval.
pub fn sample_pass_skip(
    session: &mut Session,
    core_index: usize,
    requests: &[Read],
    max_batch: usize,
    out: &mut Vec<Option<Sample>>,
) -> Result<(), probe_rs::Error> {
    let mut core = session.core(core_index)?;
    out.clear();

    for chunk in requests.chunks(max_batch) {
        for &request in chunk {
            out.push(read_one(&mut core, request).ok());
        }
    }

    Ok(())
}

/// One sampling pass with `on_error = "abort"`: the first failure
/// propagates and the loop stops.
pub fn sample_pass_abort(
    session: &mut Session,
    core_index: usize,
    requests: &[Read],
    out: &mut Vec<Sample>,
) -> Result<(), probe_rs::Error> {
    let mut core = session.core(core_index)?;
    out.clear();

    for &request in requests {
        out.push(read_one(&mut core, request)?);
    }

    Ok(())
}

/// The `interval_ms` deadline loop.
///
/// The interval is a floor, not a guarantee. A pass that overruns simply
/// runs late; the next deadline is computed from the previous deadline, so
/// a single slow pass does not shift every later one. Never busy-wait —
/// the probe transport is the bottleneck and spinning steals the CPU the
/// consumer needs.
pub fn run_for(
    session: &mut Session,
    core_index: usize,
    requests: &[Read],
    interval_ms: u64,
    max_batch: usize,
    passes: usize,
) -> Result<Vec<Vec<Option<Sample>>>, Box<dyn Error>> {
    let interval = Duration::from_millis(interval_ms);
    let mut deadline = Instant::now();
    let mut collected: Vec<Vec<Option<Sample>>> = Vec::with_capacity(passes);

    for _ in 0..passes {
        let mut pass: Vec<Option<Sample>> = Vec::with_capacity(requests.len());
        sample_pass_skip(session, core_index, requests, max_batch, &mut pass)?;
        collected.push(pass);

        deadline += interval;
        if let Some(remaining) = deadline.checked_duration_since(Instant::now()) {
            std::thread::sleep(remaining);
        } else {
            // The pass overran its slot. Re-base rather than trying to
            // catch up, which would run passes back-to-back with no gap.
            deadline = Instant::now();
        }
    }

    Ok(collected)
}
