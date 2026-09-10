//! `[[mk20dx.pins]]` — every mode the schema admits, emitted the way the
//! codegen prose says to (§2 and §3 of `../../codegen/rust/README.md`).

use embedded_hal::digital::{InputPin, OutputPin, StatefulOutputPin};
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

    let mut status_led = pins_c.pc5.into_push_pull_output();
    let mut button = pins_d.pd5.into_pull_up_input();

    // Driving the handles goes through embedded-hal's digital traits,
    // which the block contributes (the HAL implements them without
    // re-exporting them). The error is infallible on this chip; .ok() /
    // .unwrap() at the call site is the accepted form.
    status_led.set_high().ok();
    status_led.toggle().ok();
    let _pressed = button.is_low().unwrap();

    OwnedPins {
        status_led,
        button,
        spi_clk: pins_d.pd1.into_alternate::<2>(),
    }
}
