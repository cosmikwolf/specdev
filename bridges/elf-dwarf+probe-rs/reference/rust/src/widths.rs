//! `widths` — the DWARF-type to probe-access-width mapping.
//!
//! This is the decision the bridge exists for. The elf-dwarf block knows
//! a variable is a 2-byte signed integer and knows nothing about reading
//! memory; the probe-rs block knows how to read 2 bytes and nothing about
//! what they mean. Only the pairing knows the former should choose the
//! latter.

/// Mirrors the elf-dwarf block's `ScalarKind`. Restated here rather than
/// imported: the bridge's reference compiles against both blocks' pinned
/// APIs, not against their emitted code.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScalarKind {
    Unsigned,
    Signed,
    Float,
    Bool,
    Other,
}

/// The seam on the debug-info side: a DWARF base-type encoding becomes a
/// classification the width mapping can act on.
///
/// This reads gimli's own `DW_ATE_*` constants rather than a transcribed
/// integer table, which is what keeps the mapping honest if the set ever
/// grows. An unrecognised encoding classifies as `Other` and is handled
/// by the `aggregates` policy — never guessed at.
pub fn kind_from_encoding(encoding: gimli::DwAte) -> ScalarKind {
    match encoding {
        gimli::DW_ATE_boolean => ScalarKind::Bool,
        gimli::DW_ATE_float => ScalarKind::Float,
        gimli::DW_ATE_signed | gimli::DW_ATE_signed_char => ScalarKind::Signed,
        gimli::DW_ATE_unsigned | gimli::DW_ATE_unsigned_char => ScalarKind::Unsigned,
        _ => ScalarKind::Other,
    }
}

/// A recovered variable, as the elf-dwarf side hands it over.
#[derive(Clone, Debug)]
pub struct Recovered {
    pub name: String,
    pub address: u64,
    pub byte_size: u64,
    pub kind: ScalarKind,
}

/// How one variable is read, and how its bytes are then interpreted.
///
/// Width and interpretation are carried together deliberately. A `u32`
/// and an `f32` are the same four-byte read and different numbers, so a
/// plan that recorded only the width would have thrown away exactly the
/// half of the type that the debug info was consulted for.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Access {
    Word8,
    Word16,
    Word32,
    Word64,
    Bytes(usize),
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Interpret {
    Unsigned,
    Signed,
    Float,
    Bool,
    Raw,
}

#[derive(Clone, Debug)]
pub struct Plan {
    pub name: String,
    pub address: u64,
    pub access: Access,
    pub interpret: Interpret,
}

/// The `aggregates` binding field.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Aggregates {
    Skip,
    Bytes,
}

/// `width_from = "type"` — derive the access from the recovered type.
///
/// Returns `None` when the variable must not be sampled: an aggregate
/// under `aggregates = "skip"`, or a scalar whose recorded size is not a
/// width the probe can read.
///
/// A size that is not 1, 2, 4 or 8 is refused rather than rounded up.
/// Rounding would read neighbouring bytes and present them as part of
/// the value, which is a wrong number that looks like a right one.
pub fn access_from_type(recovered: &Recovered, aggregates: Aggregates) -> Option<Plan> {
    let (access, interpret) = match recovered.kind {
        ScalarKind::Other => match aggregates {
            Aggregates::Skip => return None,
            // One transaction, never a series of words: a struct read as
            // four separate words can tear across a write from the
            // running target.
            Aggregates::Bytes => (
                Access::Bytes(usize::try_from(recovered.byte_size).ok()?),
                Interpret::Raw,
            ),
        },
        ScalarKind::Bool => (Access::Word8, Interpret::Bool),
        ScalarKind::Float => match recovered.byte_size {
            4 => (Access::Word32, Interpret::Float),
            8 => (Access::Word64, Interpret::Float),
            _ => return None,
        },
        ScalarKind::Signed | ScalarKind::Unsigned => {
            let access = match recovered.byte_size {
                1 => Access::Word8,
                2 => Access::Word16,
                4 => Access::Word32,
                8 => Access::Word64,
                _ => return None,
            };
            let interpret = if recovered.kind == ScalarKind::Signed {
                Interpret::Signed
            } else {
                Interpret::Unsigned
            };
            (access, interpret)
        }
    };

    Some(Plan {
        name: recovered.name.clone(),
        address: recovered.address,
        access,
        interpret,
    })
}

/// `width_from = "sampler_default"` — every variable read at the
/// sampler's width, ignoring the recovered type.
///
/// The interpretation is still taken from the type: the sampler has an
/// opinion about transfer width, not about whether the bytes are a float.
pub fn access_from_sampler_default(recovered: &Recovered, default: Access) -> Option<Plan> {
    let interpret = match recovered.kind {
        ScalarKind::Signed => Interpret::Signed,
        ScalarKind::Unsigned => Interpret::Unsigned,
        ScalarKind::Float => Interpret::Float,
        ScalarKind::Bool => Interpret::Bool,
        ScalarKind::Other => Interpret::Raw,
    };

    Some(Plan {
        name: recovered.name.clone(),
        address: recovered.address,
        access: default,
        interpret,
    })
}

/// Turning a completed read back into a number.
///
/// Sign extension is the reason `Interpret` is carried at all: a `-1i16`
/// arrives as `0xFFFF`, and widening it without extending gives 65535.
pub fn interpret_scalar(raw: u64, access: Access, interpret: Interpret) -> f64 {
    match interpret {
        Interpret::Unsigned => raw as f64,
        Interpret::Bool => {
            if raw != 0 {
                1.0
            } else {
                0.0
            }
        }
        Interpret::Signed => match access {
            Access::Word8 => (raw as u8 as i8) as f64,
            Access::Word16 => (raw as u16 as i16) as f64,
            Access::Word32 => (raw as u32 as i32) as f64,
            Access::Word64 => (raw as i64) as f64,
            Access::Bytes(_) => raw as f64,
        },
        Interpret::Float => match access {
            Access::Word32 => f32::from_bits(raw as u32) as f64,
            Access::Word64 => f64::from_bits(raw),
            _ => raw as f64,
        },
        Interpret::Raw => raw as f64,
    }
}
