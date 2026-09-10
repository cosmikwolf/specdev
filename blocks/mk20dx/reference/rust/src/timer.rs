//! `[[mk20dx.timers]]` — PIT channels, both entry forms (prose §6).

use fugit::ExtU32;
use mk20dx_hal::clocks::Clocks;
use mk20dx_hal::pac;
use mk20dx_hal::timer::{PitChannel, PitExt};

/// The project-init shape: the PIT is split once, and each module takes
/// the channels it declared.
pub fn init(pit: pac::Pit, sim: &pac::Sim, clocks: &Clocks) -> (PitChannel<0>, PitChannel<3>) {
    let pit = pit.split(sim, clocks);

    // name = "sequencer_tick", channel = 0, period_us = 600 — started at
    // init; enable_interrupt() because the module also declares the PIT0
    // vector (see vector_interrupt.rs for the handler half).
    let mut sequencer_tick: PitChannel<0> = pit.ch0;
    sequencer_tick.start(600u32.micros());
    sequencer_tick.enable_interrupt();

    // name = "time_driver", channel = 3, no period_us — handed over
    // unstarted; the module's own code owns its timing.
    let time_driver: PitChannel<3> = pit.ch3;

    (sequencer_tick, time_driver)
}
