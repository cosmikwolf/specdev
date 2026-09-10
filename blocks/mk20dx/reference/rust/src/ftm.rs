//! `[[mk20dx.ftms]]` — both entry forms: running PWM, and manual
//! ownership handed over stopped (prose §11).

use mk20dx_hal::clocks::Clocks;
use mk20dx_hal::pac;
use mk20dx_hal::pwm::{FtmChannel, FtmExt, Ftm0, Ftm1Parts};
use mk20dx_hal::time::U32Ext;

/// name = "gate_pwm", instance = 0, frequency_hz = 20000, with
/// channels = [{ name = "gate_1", channel = 2 }]. `pwm()` starts the
/// counter; each declared channel still needs its `set_pwm()` to
/// produce output, at duty 0 until the module says otherwise. Duty on a
/// single channel is `set_value` (the CnV compare against the timer's
/// modulo) — `set_duty` exists only on the combine-mode channel PAIR,
/// which this block does not model.
pub fn init_pwm(ftm0: pac::Ftm0, clocks: &Clocks, sim: &pac::Sim) -> FtmChannel<Ftm0, 2> {
    let gate_channels = ftm0.pwm(20_000u32.Hz(), clocks, sim);
    let mut gate_1 = gate_channels.ch2;
    gate_1.set_pwm();
    gate_1.set_value(0);
    gate_1
}

/// name = "trigger_clock", instance = 1, no frequency_hz — manual
/// configuration ownership. Split parts, counter stopped; the module's
/// own code takes it from there.
pub fn init_manual(ftm1: pac::Ftm1, clocks: &Clocks, sim: &pac::Sim) -> Ftm1Parts {
    ftm1.split(clocks, sim)
}
