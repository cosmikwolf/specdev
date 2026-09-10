//! `sampled_selections` — the whole handoff, in the order emitted code
//! runs it: recover the symbols, plan the reads, then execute a pass.
//!
//! Mirrors the README worked example.

use std::error::Error;

use probe_rs::{MemoryInterface, Session};

use crate::widths::{Access, Aggregates, Interpret, Plan, Recovered, access_from_type};

/// The `on_missing` binding field.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OnMissing {
    Empty,
    Error,
}

/// Stage one, on the elf-dwarf side: a selection's recovered variables
/// become a read plan.
///
/// A variable the mapping refuses is dropped here rather than at read
/// time, so the plan holds only reads that can actually be issued.
pub fn plan(
    recovered: &[Recovered],
    aggregates: Aggregates,
    on_missing: OnMissing,
) -> Result<Vec<Plan>, Box<dyn Error>> {
    let plans: Vec<Plan> = recovered
        .iter()
        .filter_map(|variable| access_from_type(variable, aggregates))
        .collect();

    if plans.is_empty() && on_missing == OnMissing::Error {
        return Err("selection yielded no samplable variables".into());
    }

    Ok(plans)
}

/// One sampled value, carried with the interpretation its type implies.
#[derive(Clone, Debug)]
pub enum Sample {
    Scalar { raw: u64, interpret: Interpret },
    Bytes(Vec<u8>),
}

/// Stage two, on the probe-rs side: execute one pass over the plan.
///
/// The plan's `Access` selects the `MemoryInterface` call. Everything
/// here is an ordinary probe-rs read — the bridge's whole contribution
/// happened before this function, in choosing which read to issue.
pub fn read_one(core: &mut probe_rs::Core<'_>, item: &Plan) -> Result<Sample, probe_rs::Error> {
    let sample = match item.access {
        Access::Word8 => Sample::Scalar {
            raw: core.read_word_8(item.address)? as u64,
            interpret: item.interpret,
        },
        Access::Word16 => Sample::Scalar {
            raw: core.read_word_16(item.address)? as u64,
            interpret: item.interpret,
        },
        Access::Word32 => Sample::Scalar {
            raw: core.read_word_32(item.address)? as u64,
            interpret: item.interpret,
        },
        Access::Word64 => Sample::Scalar {
            raw: core.read_word_64(item.address)?,
            interpret: item.interpret,
        },
        Access::Bytes(len) => {
            let mut buffer: Vec<u8> = vec![0; len];
            core.read(item.address, &mut buffer)?;
            Sample::Bytes(buffer)
        }
    };

    Ok(sample)
}

/// One full pass, honouring the sampler's `max_batch` and its
/// `on_error = "skip"` policy.
pub fn sample_pass(
    session: &mut Session,
    core_index: usize,
    plans: &[Plan],
    max_batch: usize,
    out: &mut Vec<Option<Sample>>,
) -> Result<(), probe_rs::Error> {
    let mut core = session.core(core_index)?;
    out.clear();

    for chunk in plans.chunks(max_batch) {
        for item in chunk {
            out.push(read_one(&mut core, item).ok());
        }
    }

    Ok(())
}

/// Re-planning after the image reloads.
///
/// This is the case the pairing exists to make correct. Addresses move
/// on every rebuild, so a plan outlives its image only by accident. A
/// reload must rebuild the plan wholesale rather than patch it: a
/// variable can change size or vanish, not merely move, and a patched
/// plan would keep reading the old width at the new address.
pub fn replan(
    recovered: &[Recovered],
    aggregates: Aggregates,
    on_missing: OnMissing,
    existing: &mut Vec<Plan>,
) -> Result<(), Box<dyn Error>> {
    let rebuilt = plan(recovered, aggregates, on_missing)?;
    *existing = rebuilt;
    Ok(())
}
