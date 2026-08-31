//! `[[mk20dx.pins]]` — every mode the schema admits, emitted the way the
//! codegen prose says to (§2 and §3 of `../../codegen/rust/README.md`).

use mk20dx_hal::gpio::{Alternate, GpioExt, Input, Output, Pin, PullUp, PushPull};
use mk20dx_hal::pac;

/// Pins a module stores, annotated with their full types so a wrong mode
/// fails at the field rather than at first use (prose §3). The port is a
/// `char` const generic (quoted, uppercase) and the pin number a `u8`.
pub struct OwnedPins {
    /// `mode = "push_pull_output"`
    pub status_led: Pin<'C', 5, Output<PushPull>>,
    /// `mode = "pull_up_input"`
    pub button: Pin<'D', 5, Input<PullUp>>,
    /// `mode = "alternate"`, `alt_fn = 2`. The mux index is a TYPE
    /// parameter. The value 2 here is the reference's compile vehicle,
    /// not a pin-mux fact — a real spec supplies the index from the
    /// part's pin-mux table.
    pub spi_clk: Pin<'D', 1, Alternate<2>>,
}

/// The project-init shape: each port is split ONCE, with all three
/// arguments (the port, its `Pt` block, and `&sim` for the clock gate),
/// and every pin the modules own is moved into its mode from that one
/// split result.
pub fn init(dp: pac::Peripherals) -> OwnedPins {
    let pins_b = dp.portb.split(dp.ptb, &dp.sim);
    let pins_c = dp.portc.split(dp.ptc, &dp.sim);
    let pins_d = dp.portd.split(dp.ptd, &dp.sim);

    // `mode = "open_drain_output"`
    let _bus_line = pins_b.pb1.into_open_drain_output();
    // `mode = "floating_input"`
    let _sense = pins_d.pd4.into_floating_input();
    // `mode = "pull_down_input"`
    let _strap = pins_d.pd6.into_pull_down_input();

    OwnedPins {
        status_led: pins_c.pc5.into_push_pull_output(),
        button: pins_d.pd5.into_pull_up_input(),
        spi_clk: pins_d.pd1.into_alternate::<2>(),
    }
}
