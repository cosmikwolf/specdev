//! A pin's `interrupt` field — trigger configuration plus the
//! one-handler-per-port dispatch, with the write-back-what-you-read flag
//! clear (prose §4.2).

use mk20dx_hal::gpio::PortBPins;
use mk20dx_hal::pac;

/// Trigger config, emitted where the pins are configured. Takes the split
/// result rather than the port peripheral: the port was already consumed
/// by the project-init split (see `pins::init`), so the PCR writes go
/// through the PAC's raw pointer.
pub fn init(pins_b: PortBPins) {
    // `interrupt = "either_edge"` on two encoder pins, port B.
    let _encoder_a = pins_b.pb0.into_pull_up_input();
    let _encoder_b = pins_b.pb19.into_pull_up_input();

    // IRQC values per the prose's trigger table: 1001 rising, 1010
    // falling, 1011 either edge, 1100 level high, 1000 level low; 0001 /
    // 0010 / 0011 are the dma_* variants (a DMA request — no handler).
    //
    // ISF in the same register is write-1-to-clear, so the modify writes
    // 0 to it explicitly — writing back the 1 it read would clear a
    // pending flag as a side effect of configuring the trigger.
    let portb = unsafe { &*pac::Portb::PTR };
    portb.pcr(0).modify(|_, w| w.irqc()._1011().isf()._0());
    portb.pcr(19).modify(|_, w| w.irqc()._1011().isf()._0());
}

/// The PORTB handler body — ONE handler for the whole port, dispatching
/// by flag. Every interrupting pin on port B raises this same vector.
///
/// How this function is attached to the PORTB vector is the concurrency
/// runtime's business and deliberately not shown here (prose §4.3).
pub fn on_portb() {
    let portb = unsafe { &*pac::Portb::PTR };
    let isfr = portb.isfr().read().bits();

    if isfr & (1 << 0) != 0 {
        // the pin declared at port = "B", pin = 0
    }
    if isfr & (1 << 19) != 0 {
        // the pin declared at port = "B", pin = 19
    }

    // Write back exactly the value read (write-1-to-clear): a flag set
    // between the read and this write keeps its bit and is serviced next
    // time instead of being lost. Never a blanket clear.
    // SAFETY: ISFR is w1c; writing back the read value clears exactly the
    // flags observed.
    portb.isfr().write(|w| unsafe { w.bits(isfr) });
}
