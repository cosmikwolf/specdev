//! `sessions` — one `[[probe_rs.sessions]]` entry becomes one attach path.
//!
//! Mirrors the README worked example. Types are annotated explicitly so the
//! compiler checks them.

use std::error::Error;
use std::str::FromStr;

use probe_rs::probe::DebugProbeSelector;
use probe_rs::probe::list::Lister;
use probe_rs::{Permissions, Session};

/// `attach_mode = "attach"`, with an optional `probe_selector` and
/// `speed_khz`.
///
/// The three steps are always the same and always in this order: choose a
/// probe, configure the wire, then attach. Speed must be set on the `Probe`
/// — once `attach` consumes it there is no longer a handle to set it on.
pub fn attach(
    chip: &str,
    probe_selector: Option<&str>,
    speed_khz: Option<u32>,
    allow_erase_all: bool,
) -> Result<Session, Box<dyn Error>> {
    let lister = Lister::new();

    let mut probe = match probe_selector {
        Some(selector) => lister.open(DebugProbeSelector::from_str(selector)?)?,
        None => {
            let probes = lister.list_all();
            let info = probes.first().ok_or("no debug probe found")?;
            info.open()?
        }
    };

    if let Some(khz) = speed_khz {
        // Returns the speed actually applied, which may be lower than the
        // request. Discarding it is fine; logging it is better.
        let _applied: u32 = probe.set_speed(khz)?;
    }

    let permissions: Permissions = if allow_erase_all {
        Permissions::new().allow_erase_all()
    } else {
        Permissions::new()
    };

    let session: Session = probe.attach(chip, permissions)?;
    Ok(session)
}

/// `attach_mode = "attach_under_reset"`.
///
/// Same shape, different terminal call. This is the only way onto a target
/// whose firmware disables the debug pins or sleeps immediately after boot.
pub fn attach_under_reset(
    chip: &str,
    probe_selector: Option<&str>,
    speed_khz: Option<u32>,
) -> Result<Session, Box<dyn Error>> {
    let lister = Lister::new();

    let mut probe = match probe_selector {
        Some(selector) => lister.open(DebugProbeSelector::from_str(selector)?)?,
        None => {
            let probes = lister.list_all();
            let info = probes.first().ok_or("no debug probe found")?;
            info.open()?
        }
    };

    if let Some(khz) = speed_khz {
        let _applied: u32 = probe.set_speed(khz)?;
    }

    let session: Session = probe.attach_under_reset(chip, Permissions::new())?;
    Ok(session)
}

/// Enumerating probes for a picker UI, without attaching to any of them.
///
/// `list_all` performs no I/O against the target and is safe to call while a
/// session is open elsewhere.
pub fn list_probes() -> Vec<String> {
    let lister = Lister::new();
    lister
        .list_all()
        .iter()
        .map(|info| info.to_string())
        .collect()
}
