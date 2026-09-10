//! `[[mk20dx.uarts]]` — a UART wired with the module's own alternate
//! pins, split into halves (prose §8).

use mk20dx_hal::clocks::Clocks;
use mk20dx_hal::gpio::{Alternate, Pin};
use mk20dx_hal::pac;
use mk20dx_hal::time::U32Ext;
use mk20dx_hal::uart::{Config, Rx, Tx, Uart2, UartExt};

/// name = "midi", instance = 2, baudrate = 31250, 8N1 default (no
/// parity / word_length builder calls). Split because TX and RX go to
/// different owners.
pub fn init(
    uart2: pac::Uart2,
    midi_tx: Pin<'D', 3, Alternate<3>>,
    midi_rx: Pin<'D', 2, Alternate<3>>,
    clocks: &Clocks,
    sim: &pac::Sim,
) -> (Tx<Uart2>, Rx<Uart2>) {
    let midi = uart2.serial(midi_tx, midi_rx, Config::new(31_250u32.Hz()), clocks, sim);
    midi.split()
}
