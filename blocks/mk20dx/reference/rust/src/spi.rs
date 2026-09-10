//! `[[mk20dx.spi]]` — an SPI bus wired with the module's own alternate
//! pins (prose §7).

use mk20dx_hal::clocks::Clocks;
use mk20dx_hal::gpio::{Alternate, Pin};
use mk20dx_hal::pac;
use mk20dx_hal::spi::{Config, Spi, SpiExt};
use mk20dx_hal::time::U32Ext;

/// name = "main_bus", instance = 0, baudrate_hz = 15000000, mode 0.
///
/// The pins are this module's `pins` entries (`spi_sck` / `spi_mosi` /
/// `spi_miso`, ALT2 on the mk20d7 pin-mux table). The per-instance pin
/// traits make a wrong pin fail this call. The instance marker is named
/// by module path — `spi::Spi0` — because the PAC peripheral type is
/// ALSO called `Spi0`.
pub fn init(
    spi0: pac::Spi0,
    spi_sck: Pin<'C', 5, Alternate<2>>,
    spi_mosi: Pin<'C', 6, Alternate<2>>,
    spi_miso: Pin<'C', 7, Alternate<2>>,
    clocks: &Clocks,
    sim: &pac::Sim,
) -> Spi<mk20dx_hal::spi::Spi0> {
    spi0.spi(
        spi_sck,
        spi_mosi,
        spi_miso,
        Config::new(15_000_000u32.Hz()),
        clocks,
        sim,
    )
}
