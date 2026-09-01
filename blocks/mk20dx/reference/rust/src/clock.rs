//! `[mk20dx.clock]` — the one-per-project clock-tree preset (prose §5).

use mk20dx_hal::clocks::{ClockSpeed, Clocks, McgExt};
use mk20dx_hal::pac;

/// `speed = "mhz120"` on the mk20d7 part this reference compiles for.
/// Consumes MCG and OSC, borrows Sim; the frozen `Clocks` is what every
/// peripheral constructor below takes by reference.
pub fn init(mcg: pac::Mcg, osc: pac::Osc, sim: &pac::Sim) -> Clocks {
    mcg.constrain().freeze_at(ClockSpeed::Mhz120, osc, sim)
}
