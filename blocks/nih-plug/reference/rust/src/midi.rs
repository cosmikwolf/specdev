//! `midi` — the MIDI-level consts + the CC / pitch-bend event drain
//! (README "MIDI input / output levels"): `input_level = "midi_ccs"` →
//! `MidiConfig::MidiCCs`, and the `NoteEvent::MidiCC` / `MidiPitchBend`
//! match arms. Type-checks the level const + event variant field names
//! against pinned nih-plug (CC/pitch-bend arrive only at MidiCCs).
use nih_plug::prelude::*;

// From input_level = "midi_ccs" / output_level = "none".
pub const MIDI_INPUT: MidiConfig = MidiConfig::MidiCCs;
pub const MIDI_OUTPUT: MidiConfig = MidiConfig::None;

/// The `process()` event drain for a `midi_ccs`-level plugin. `S` is the
/// plugin's SysExMessage type; `()` when it has none.
pub fn drain(event: NoteEvent<()>) {
    match event {
        NoteEvent::MidiCC {
            timing,
            channel,
            cc,
            value,
        } => {
            // value is normalized to [0, 1]; nih_plug::midi::control_change
            // lists named CC numbers.
            let _ = (timing, channel, cc, value);
        }
        NoteEvent::MidiPitchBend {
            timing,
            channel,
            value,
        } => {
            let _ = (timing, channel, value);
        }
        NoteEvent::NoteOn { .. } => {}
        _ => {}
    }
}
