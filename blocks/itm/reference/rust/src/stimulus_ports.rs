//! `stimulus_ports` — one `[[itm.stimulus_ports]]` entry becomes one
//! payload decoder for `TracePacket::Instrumentation`.
//!
//! Mirrors the README worked example. Firmware writes to the port; the
//! decoder sees a packet carrying the port number and raw bytes.

use itm::TracePacket;

/// The `payload` enum, one variant per schema value.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Payload {
    Text,
    U8,
    U16,
    U32,
    I8,
    I16,
    I32,
    F32,
    Bytes,
}

/// A decoded stimulus-port value.
#[derive(Clone, Debug, PartialEq)]
pub enum PortValue {
    /// Complete lines only. A `text` payload can split across packets,
    /// so a partial tail is held back rather than emitted.
    Lines(Vec<String>),
    Unsigned(u64),
    Signed(i64),
    Float(f32),
    Bytes(Vec<u8>),
}

/// Per-port accumulator. A `text` port needs one of these to hold the
/// partial line between packets; the scalar ports carry no state but
/// share the type so a port table is uniform.
#[derive(Default)]
pub struct PortBuffer {
    partial: String,
}

/// Routing a packet to its port.
///
/// `Instrumentation` is the only variant a stimulus port produces. The
/// `port` field is how a packet is attributed, so a port declared in the
/// spec but never written by firmware simply never matches — which is
/// not an error and must not be reported as one.
pub fn port_of(packet: &TracePacket) -> Option<u8> {
    match packet {
        TracePacket::Instrumentation { port, .. } => Some(*port),
        _ => None,
    }
}

/// Decoding one packet's payload at the declared width.
///
/// Every scalar is little-endian, which is the byte order every
/// Cortex-M writes. A payload shorter than the declared width means the
/// firmware and the spec disagree; return `None` rather than padding,
/// because a silently zero-extended value is worse than a missing one.
pub fn decode(
    packet: &TracePacket,
    payload: Payload,
    buffer: &mut PortBuffer,
) -> Option<PortValue> {
    let bytes: &[u8] = match packet {
        TracePacket::Instrumentation { payload: data, .. } => data,
        _ => return None,
    };

    let value = match payload {
        Payload::Text => {
            buffer.partial.push_str(&String::from_utf8_lossy(bytes));
            let mut lines: Vec<String> =
                buffer.partial.lines().map(|line| line.to_string()).collect();

            // A trailing byte that is not a newline is an incomplete
            // line: hold it back for the next packet.
            if !buffer.partial.ends_with('\n') {
                let tail = lines.pop().unwrap_or_default();
                buffer.partial = tail;
            } else {
                buffer.partial.clear();
            }

            PortValue::Lines(lines)
        }
        Payload::U8 => PortValue::Unsigned(u8::from_le_bytes(bytes.try_into().ok()?) as u64),
        Payload::U16 => PortValue::Unsigned(u16::from_le_bytes(bytes.try_into().ok()?) as u64),
        Payload::U32 => PortValue::Unsigned(u32::from_le_bytes(bytes.try_into().ok()?) as u64),
        Payload::I8 => PortValue::Signed(i8::from_le_bytes(bytes.try_into().ok()?) as i64),
        Payload::I16 => PortValue::Signed(i16::from_le_bytes(bytes.try_into().ok()?) as i64),
        Payload::I32 => PortValue::Signed(i32::from_le_bytes(bytes.try_into().ok()?) as i64),
        Payload::F32 => PortValue::Float(f32::from_le_bytes(bytes.try_into().ok()?)),
        Payload::Bytes => PortValue::Bytes(bytes.to_vec()),
    };

    Some(value)
}

/// `Overflow` is the one protocol packet a port consumer must not
/// ignore: it means the target produced trace data faster than the link
/// carried it, so packets were lost. Emitted code surfaces this rather
/// than dropping it, because the resulting gap is otherwise invisible.
pub fn is_overflow(packet: &TracePacket) -> bool {
    matches!(packet, TracePacket::Overflow)
}
