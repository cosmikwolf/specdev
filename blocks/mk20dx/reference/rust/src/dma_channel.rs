//! `[[mk20dx.dma_channels]]` — the one split, a wired channel, and an
//! unwired one (prose §10).

use mk20dx_hal::dma::{DmaChannel, DmaExt, DmaSource};
use mk20dx_hal::pac;

/// The project-init shape: the controller is split once; each module
/// takes the channels it declared.
pub fn init(dma: pac::Dma, dmamux: pac::Dmamux, sim: &pac::Sim) -> (DmaChannel<1>, DmaChannel<2>) {
    let dma = dma.split(dmamux, sim);

    // name = "display_flush", channel = 1, source = "SPI0_TX" — the
    // source string is the HAL constant's name, verbatim.
    let mut display_flush: DmaChannel<1> = dma.ch1;
    display_flush.set_source(DmaSource::SPI0_TX);

    // name = "scratch_copy", channel = 2, no source — software-triggered,
    // handed over unwired.
    let scratch_copy: DmaChannel<2> = dma.ch2;

    (display_flush, scratch_copy)
}
