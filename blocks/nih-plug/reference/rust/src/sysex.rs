//! `sysex` — the `SysExMessage` impl emitted for a `[nih_plug.midi.sysex]`
//! subtable (README "SysEx message handling"): a raw-frame message type whose
//! `Buffer` is `[u8; max_message_len]`, with `from_buffer` / `to_buffer`.
//! Type-checks the emitted skeleton against pinned nih-plug; the buffer size
//! (4104) is the Yamaha DX7 32-voice bank-dump length used in the prose.
use nih_plug::prelude::*;

#[derive(Debug, Clone, PartialEq)]
pub struct Dx7Dump {
    /// Raw frame, 0xF0 … 0xF7 inclusive.
    pub raw: [u8; 4104], // = sysex.max_message_len
    pub len: usize,
}

impl SysExMessage for Dx7Dump {
    // From sysex.max_message_len — must be a const array length.
    type Buffer = [u8; 4104];

    fn from_buffer(buffer: &[u8]) -> Option<Self> {
        // Cheaply recognize + copy the frame; the real DX7 decode lives in
        // the engine crate, off the audio thread.
        let mut raw = [0u8; 4104];
        let len = buffer.len().min(raw.len());
        raw[..len].copy_from_slice(&buffer[..len]);
        Some(Self { raw, len })
    }

    fn to_buffer(self) -> (Self::Buffer, usize) {
        (self.raw, self.len)
    }
}
