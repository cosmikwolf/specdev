//! `[[mk20dx.interrupts]]` — a non-port vector the module owns, with the
//! flag clear its family requires (prose §4.1 and §4.3).
//!
//! The channel arrives as a parameter because peripheral construction is
//! not this block's to emit (prose §5): project init builds the
//! `PitChannel` and hands it to whichever task or handler owns the
//! vector.

use mk20dx_hal::timer::PitChannel;

/// Handler body for a `[[mk20dx.interrupts]]` entry with
/// `vector = "PIT0"`. The PIT family clears through the channel's
/// `clear_interrupt()`; a handler that returns without clearing re-enters
/// immediately.
///
/// Work stays out of the handler: latch what happened and hand it to
/// ordinary code. Binding this function to the PIT0 vector is the
/// runtime's business.
pub fn on_pit0(tick: &mut PitChannel<0>) {
    tick.clear_interrupt();
    // latch / notify here — an atomic, a channel, or a task wake.
}
