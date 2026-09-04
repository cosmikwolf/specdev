//! `data_watches` — one `[[itm.data_watches]]` entry becomes one
//! payload decoder for the DWT data-trace packets of one comparator.
//!
//! Mirrors the README worked example. A comparator is armed by the
//! debugger, not by firmware, so these packets appear with no
//! target-side instrumentation at all.

use itm::{MemoryAccessType, TracePacket};

/// The `value_type` enum, one variant per schema value.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ValueType {
    U8,
    U16,
    U32,
    I8,
    I16,
    I32,
    F32,
}

/// The direction of the access that produced a `value` packet.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Access {
    Read,
    Write,
}

/// A decoded data-trace event, one variant per `emits` value.
#[derive(Clone, Debug, PartialEq)]
pub enum WatchEvent {
    Unsigned { access: Access, value: u64 },
    Signed { access: Access, value: i64 },
    Float { access: Access, value: f32 },
    Address(Vec<u8>),
    ProgramCounter(u32),
}

/// Attributing a packet to a comparator.
///
/// All three data-trace variants carry the comparator number, which is
/// the only link back to the spec entry — the packet does not name the
/// address it matched. A comparator number that never appears means the
/// debugger armed a different unit than the spec declares.
pub fn comparator_of(packet: &TracePacket) -> Option<u8> {
    match packet {
        TracePacket::DataTraceValue { comparator, .. } => Some(*comparator),
        TracePacket::DataTraceAddress { comparator, .. } => Some(*comparator),
        TracePacket::DataTracePC { comparator, .. } => Some(*comparator),
        _ => None,
    }
}

/// `emits = "value"` — decoding the accessed data.
///
/// The payload arrives as bytes whose width the packet does not
/// describe, so `value_type` must match the width the comparator was
/// armed with. A length mismatch returns `None`: the alternative is a
/// plausible wrong number, which in a viewer is worse than a gap.
///
/// `access_type` distinguishes a read from a write. A watch that fires
/// on both sees each value twice, once per direction, and code that
/// ignores the direction will double-count.
pub fn decode_value(packet: &TracePacket, value_type: ValueType) -> Option<WatchEvent> {
    let (access_type, bytes) = match packet {
        TracePacket::DataTraceValue {
            access_type, value, ..
        } => (access_type, value),
        _ => return None,
    };

    let access = match access_type {
        MemoryAccessType::Read => Access::Read,
        MemoryAccessType::Write => Access::Write,
    };

    let event = match value_type {
        ValueType::U8 => WatchEvent::Unsigned {
            access,
            value: u8::from_le_bytes(bytes.as_slice().try_into().ok()?) as u64,
        },
        ValueType::U16 => WatchEvent::Unsigned {
            access,
            value: u16::from_le_bytes(bytes.as_slice().try_into().ok()?) as u64,
        },
        ValueType::U32 => WatchEvent::Unsigned {
            access,
            value: u32::from_le_bytes(bytes.as_slice().try_into().ok()?) as u64,
        },
        ValueType::I8 => WatchEvent::Signed {
            access,
            value: i8::from_le_bytes(bytes.as_slice().try_into().ok()?) as i64,
        },
        ValueType::I16 => WatchEvent::Signed {
            access,
            value: i16::from_le_bytes(bytes.as_slice().try_into().ok()?) as i64,
        },
        ValueType::I32 => WatchEvent::Signed {
            access,
            value: i32::from_le_bytes(bytes.as_slice().try_into().ok()?) as i64,
        },
        ValueType::F32 => WatchEvent::Float {
            access,
            value: f32::from_le_bytes(bytes.as_slice().try_into().ok()?),
        },
    };

    Some(event)
}

/// `emits = "address"` — the accessed address, for a comparator matching
/// a range rather than one location. The payload is handed out
/// undecoded: its width is the comparator's business.
pub fn decode_address(packet: &TracePacket) -> Option<WatchEvent> {
    match packet {
        TracePacket::DataTraceAddress { data, .. } => Some(WatchEvent::Address(data.clone())),
        _ => None,
    }
}

/// `emits = "pc"` — the program counter of the accessing instruction.
/// Answers "who touched this" rather than "what did it become".
pub fn decode_pc(packet: &TracePacket) -> Option<WatchEvent> {
    match packet {
        TracePacket::DataTracePC { pc, .. } => Some(WatchEvent::ProgramCounter(*pc)),
        _ => None,
    }
}
