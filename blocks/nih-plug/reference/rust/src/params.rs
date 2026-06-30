//! `params` — the `#[derive(Params)]` struct + the per-kind range constructors
//! (README "Parameter struct emission" + "Range constructors per kind"): the
//! four parameter kinds (float/int/bool/enum) with FloatRange/IntRange,
//! skew_factor, with_unit, and SmoothingStyle.
use nih_plug::prelude::*;

#[derive(Enum, PartialEq)]
pub enum FilterType {
    Lowpass,
    Highpass,
}

#[derive(Params)]
pub struct MyPluginParams {
    #[id = "gain"]
    pub gain: FloatParam,
    #[id = "voices"]
    pub voices: IntParam,
    #[id = "bypass"]
    pub bypass: BoolParam,
    #[id = "filter"]
    pub filter_type: EnumParam<FilterType>,
}

impl Default for MyPluginParams {
    fn default() -> Self {
        Self {
            gain: FloatParam::new(
                "Gain",
                0.0,
                FloatRange::Skewed {
                    min: -30.0,
                    max: 30.0,
                    factor: FloatRange::skew_factor(-2.0),
                },
            )
            .with_unit(" dB")
            .with_smoother(SmoothingStyle::Logarithmic(50.0)),

            voices: IntParam::new("Voice Count", 8, IntRange::Linear { min: 1, max: 32 }),

            bypass: BoolParam::new("Bypass", false),

            filter_type: EnumParam::new("Filter", FilterType::Lowpass),
        }
    }
}
