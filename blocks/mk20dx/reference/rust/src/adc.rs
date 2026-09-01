//! `[[mk20dx.adcs]]` — an ADC with construction-time configuration and
//! the calibration that makes it real (prose §9).

use mk20dx_hal::adc::{Adc, AdcExt, Averaging, Resolution};
use mk20dx_hal::clocks::Clocks;
use mk20dx_hal::pac;

/// name = "cv_inputs", instance = 1, resolution = "bits12",
/// averaging = "x4", calibrate defaulting to true. The instance marker
/// is named by module path — `adc::Adc1` — because the PAC peripheral
/// type is ALSO called `Adc1`. `calibrate` returns a Result and is
/// handled, never swallowed.
pub fn init(adc1: pac::Adc1, clocks: &Clocks, sim: &pac::Sim) -> Adc<mk20dx_hal::adc::Adc1> {
    let mut cv_inputs = adc1.adc(clocks, sim);
    cv_inputs.set_resolution(Resolution::Bits12);
    cv_inputs.set_averaging(Averaging::Avg4);
    cv_inputs.calibrate().expect("ADC calibration failed");
    cv_inputs
}
