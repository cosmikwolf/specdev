//! Shared parameter struct the per-widget examples bind against — the
//! `#[derive(Params)]` struct the nih-plug block's prose owns. The bridge
//! composes against it; here it provides one parameter of each kind the widget
//! examples need.
use nih_plug::prelude::*;

#[derive(Enum, PartialEq, Clone, Copy)]
pub enum FilterKind {
    Low,
    High,
}

#[derive(Params)]
pub struct RefParams {
    #[id = "gain"]
    pub gain: FloatParam,
    #[id = "steps"]
    pub steps: IntParam,
    #[id = "on"]
    pub on: BoolParam,
    #[id = "filter"]
    pub filter: EnumParam<FilterKind>,
}
