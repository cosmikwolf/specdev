//! `armed_watches` — arming a declared comparator on the target, and
//! routing its packets back to the watch that declared it.
//!
//! Mirrors the README worked example. This is the other half of the
//! pairing: the itm block says what comparator 2 *means*, the probe-rs
//! block can arm comparator 2 against an address, and only the binding
//! knows they are the same comparator.

use std::error::Error;

use itm::TracePacket;
use probe_rs::Session;

/// The `arm` binding field.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Arm {
    OnStart,
    Manual,
}

/// One binding, resolved at codegen time from the two endpoints.
#[derive(Clone, Debug)]
pub struct Binding {
    /// From the itm `DataWatch`.
    pub comparator: u8,
    /// From the itm `DataWatch`, used to route decoded packets.
    pub watch_id: &'static str,
    pub arm: Arm,
}

/// ITMPR-001, checked at codegen time and re-asserted here.
///
/// The comparator a watch declares must be one the trace source
/// actually arms. Neither block can see the other's number: the itm
/// block has a `comparator` field and the probe-rs block has a
/// `data_watch_units` list, and a disagreement produces a decoder
/// waiting forever for packets from a comparator nobody armed — no
/// error, no packets, no explanation.
pub fn comparator_is_armed(comparator: u8, data_watch_units: &[usize]) -> bool {
    data_watch_units.contains(&usize::from(comparator))
}

/// `arm = "on_start"` — arm every bound comparator as the trace source
/// comes up.
///
/// `add_swv_data_trace` takes the unit number and a `u32` address. The
/// address is a runtime value: where it comes from is not this bridge's
/// concern.
pub fn arm_on_start(
    session: &mut Session,
    bindings: &[(Binding, u32)],
) -> Result<(), Box<dyn Error>> {
    for (binding, address) in bindings {
        if binding.arm != Arm::OnStart {
            continue;
        }
        session.add_swv_data_trace(usize::from(binding.comparator), *address)?;
    }
    Ok(())
}

/// `arm = "manual"` — the application arms the comparator later, once it
/// knows the address. Same call, deferred.
pub fn arm_now(session: &mut Session, binding: &Binding, address: u32) -> Result<(), Box<dyn Error>> {
    session.add_swv_data_trace(usize::from(binding.comparator), address)?;
    Ok(())
}

/// Routing a decoded packet back to the watch that declared its
/// comparator.
///
/// The packet carries the comparator number and nothing else
/// identifying: it does not name the address it matched. The binding is
/// the only thing that maps that number back to a meaning.
pub fn route<'a>(packet: &TracePacket, bindings: &'a [Binding]) -> Option<&'a Binding> {
    let comparator = match packet {
        TracePacket::DataTraceValue { comparator, .. } => *comparator,
        TracePacket::DataTraceAddress { comparator, .. } => *comparator,
        TracePacket::DataTracePC { comparator, .. } => *comparator,
        _ => return None,
    };

    bindings
        .iter()
        .find(|binding| binding.comparator == comparator)
}

/// Releasing comparators, then the trace unit. Order matters: a disarm
/// issued after the unit is disabled has nothing to act on.
pub fn teardown(
    session: &mut Session,
    core_index: usize,
    bindings: &[Binding],
) -> Result<(), Box<dyn Error>> {
    for binding in bindings {
        session.remove_swv_data_trace(usize::from(binding.comparator))?;
    }
    session.disable_swv(core_index)?;
    Ok(())
}
