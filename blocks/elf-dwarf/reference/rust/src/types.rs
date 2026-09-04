//! `types` — resolving a variable's static address and its `DW_AT_type`
//! chain.
//!
//! Mirrors the README worked example. These two walks are what turn a
//! DIE into something a reader can act on: an address to fetch and a
//! width and encoding to interpret the fetched bytes with.

/// How a scalar is encoded in memory. `Other` covers structs, arrays,
/// unions and any encoding not classified — deliberately, because a
/// viewer can still show raw bytes for those.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ScalarKind {
    Unsigned,
    Signed,
    Float,
    Bool,
    #[default]
    Other,
}

/// The outcome of following a `DW_AT_type` chain to something terminal.
#[derive(Debug, Clone, Default)]
pub struct ResolvedType {
    pub kind: ScalarKind,
    pub byte_size: Option<u64>,
    pub name: Option<String>,
}

/// Evaluating `DW_AT_location` to a single static address.
///
/// This is the general path and the one to emit. The tempting shortcut —
/// decoding the first operation and matching `Operation::Address` — reads
/// only `DW_OP_addr`, which is what GCC emits. Clang and rustc emitting
/// DWARF 5 use `DW_OP_addrx`, an index into `.debug_addr`, and the
/// shortcut silently returns nothing for every global in those images.
/// A symbol list that is empty for half the toolchains in use, with no
/// error, is the worst possible failure here.
///
/// Any other `Requires*` result means the variable is not statically
/// allocated — a local on the stack or in a register — and is correctly
/// skipped.
pub fn static_address<R: gimli::Reader>(
    dwarf: &gimli::Dwarf<R>,
    unit: &gimli::Unit<R>,
    entry: &gimli::DebuggingInformationEntry<R>,
) -> Result<Option<u64>, gimli::Error> {
    let expression: gimli::Expression<R> = match entry.attr_value(gimli::DW_AT_location) {
        Some(gimli::AttributeValue::Exprloc(expression)) => expression,
        // A location-list reference means the variable moves during
        // execution, so it has no single address to sample.
        _ => return Ok(None),
    };

    let mut evaluation: gimli::Evaluation<R> = expression.evaluation(unit.encoding());
    let mut result = evaluation.evaluate()?;

    loop {
        match result {
            gimli::EvaluationResult::Complete => break,
            // DW_OP_addr. The image is linked, so the link-time address
            // is already final and is handed straight back.
            gimli::EvaluationResult::RequiresRelocatedAddress(address) => {
                result = evaluation.resume_with_relocated_address(address)?;
            }
            // DW_OP_addrx / DW_OP_constx: the address lives in
            // .debug_addr and must be looked up by index.
            gimli::EvaluationResult::RequiresIndexedAddress { index, relocate: _ } => {
                let address = dwarf.address(unit, index)?;
                result = evaluation.resume_with_indexed_address(address)?;
            }
            _ => return Ok(None),
        }
    }

    // `result()` panics if the evaluation did not complete, which the
    // loop above guarantees.
    let pieces: Vec<gimli::Piece<R>> = evaluation.result();

    match pieces.as_slice() {
        [gimli::Piece {
            location: gimli::Location::Address { address },
            ..
        }] => Ok(Some(*address)),
        // More than one piece means the value is split across locations,
        // which a single address cannot describe.
        _ => Ok(None),
    }
}

/// Multiplying out an array's `DW_TAG_subrange_type` children.
///
/// Shows how to read the direct children of a DIE: `entries_at_offset`
/// plus `next_entry`, which stops at the null DIE ending a child list.
/// `next_dfs` would walk past it into the following siblings.
pub fn array_element_count<R: gimli::Reader>(
    unit: &gimli::Unit<R>,
    array: gimli::UnitOffset<R::Offset>,
) -> Result<Option<u64>, gimli::Error> {
    let mut cursor: gimli::EntriesCursor<'_, R> = unit.entries_at_offset(array)?;

    // The first `next_entry` lands on the array DIE itself.
    if !cursor.next_entry()? {
        return Ok(None);
    }

    let mut total: Option<u64> = None;

    loop {
        if !cursor.next_entry()? {
            break;
        }

        let count = {
            let child = match cursor.current() {
                Some(child) => child,
                // `current` is None at the null DIE closing the list.
                None => break,
            };

            if child.tag() != gimli::DW_TAG_subrange_type {
                continue;
            }

            child
                .attr(gimli::DW_AT_count)
                .and_then(|attr| attr.udata_value())
                .or_else(|| {
                    child
                        .attr(gimli::DW_AT_upper_bound)
                        .and_then(|attr| attr.udata_value())
                        .map(|upper| upper + 1)
                })
        };

        match count {
            Some(n) => total = Some(total.unwrap_or(1) * n),
            // A flexible array member has no bound, so the size is
            // genuinely unknown rather than zero.
            None => return Ok(None),
        }
    }

    Ok(total)
}

/// Following `DW_AT_type` to something terminal.
///
/// The loop is bounded: malformed or cyclic debug info must not hang a
/// viewer that is only trying to populate a symbol list.
pub fn resolve_type<R: gimli::Reader>(
    dwarf: &gimli::Dwarf<R>,
    unit: &gimli::Unit<R>,
    start: gimli::UnitOffset<R::Offset>,
) -> Result<ResolvedType, gimli::Error> {
    let mut offset = start;

    for _ in 0..32 {
        // `Unit::entry` returns an OWNED DIE, so it does not conflict
        // with a cursor borrowing the same unit.
        let entry: gimli::DebuggingInformationEntry<R> = unit.entry(offset)?;

        let byte_size = entry
            .attr(gimli::DW_AT_byte_size)
            .and_then(|attr| attr.udata_value());

        let name = match entry.attr_value(gimli::DW_AT_name) {
            Some(value) => {
                let reader = dwarf.attr_string(unit, value)?;
                Some(reader.to_string_lossy()?.into_owned())
            }
            None => None,
        };

        match entry.tag() {
            gimli::DW_TAG_base_type => {
                let encoding = match entry.attr_value(gimli::DW_AT_encoding) {
                    Some(gimli::AttributeValue::Encoding(encoding)) => encoding,
                    _ => gimli::DwAte(0),
                };

                let kind = match encoding {
                    gimli::DW_ATE_boolean => ScalarKind::Bool,
                    gimli::DW_ATE_float => ScalarKind::Float,
                    gimli::DW_ATE_signed | gimli::DW_ATE_signed_char => ScalarKind::Signed,
                    gimli::DW_ATE_unsigned | gimli::DW_ATE_unsigned_char => ScalarKind::Unsigned,
                    _ => ScalarKind::Other,
                };

                return Ok(ResolvedType {
                    kind,
                    byte_size,
                    name,
                });
            }

            // A pointer is an address-wide unsigned integer.
            gimli::DW_TAG_pointer_type => {
                return Ok(ResolvedType {
                    kind: ScalarKind::Unsigned,
                    byte_size: byte_size.or(Some(u64::from(unit.encoding().address_size))),
                    name,
                });
            }

            // An enum classifies by its underlying type when one is
            // recorded, and is unsigned otherwise.
            gimli::DW_TAG_enumeration_type => {
                let underlying = match entry.attr_value(gimli::DW_AT_type) {
                    Some(gimli::AttributeValue::UnitRef(next)) => {
                        Some(resolve_type(dwarf, unit, next)?)
                    }
                    _ => None,
                };

                return Ok(ResolvedType {
                    kind: underlying
                        .as_ref()
                        .map_or(ScalarKind::Unsigned, |resolved| resolved.kind),
                    byte_size: byte_size
                        .or_else(|| underlying.as_ref().and_then(|resolved| resolved.byte_size)),
                    name,
                });
            }

            // An array carries NO byte size of its own: it is the
            // element size times the product of the subrange counts.
            gimli::DW_TAG_array_type => {
                let element = match entry.attr_value(gimli::DW_AT_type) {
                    Some(gimli::AttributeValue::UnitRef(next)) => {
                        Some(resolve_type(dwarf, unit, next)?)
                    }
                    _ => None,
                };

                let count = array_element_count(unit, offset)?;
                let element_size = element.as_ref().and_then(|resolved| resolved.byte_size);

                let size = byte_size.or(match (element_size, count) {
                    (Some(each), Some(n)) => Some(each * n),
                    _ => None,
                });

                return Ok(ResolvedType {
                    kind: ScalarKind::Other,
                    byte_size: size,
                    name: element
                        .and_then(|resolved| resolved.name)
                        .map(|inner| format!("{inner}[{}]", count.unwrap_or(0))),
                });
            }

            // Transparent wrappers: keep following.
            gimli::DW_TAG_typedef
            | gimli::DW_TAG_const_type
            | gimli::DW_TAG_volatile_type
            | gimli::DW_TAG_restrict_type
            | gimli::DW_TAG_atomic_type => match entry.attr_value(gimli::DW_AT_type) {
                Some(gimli::AttributeValue::UnitRef(next)) => {
                    offset = next;
                    continue;
                }
                // `const void` and friends terminate here.
                _ => {
                    return Ok(ResolvedType {
                        kind: ScalarKind::Other,
                        byte_size,
                        name,
                    });
                }
            },

            // Aggregates: keep the size, do not claim a scalar kind.
            _ => {
                return Ok(ResolvedType {
                    kind: ScalarKind::Other,
                    byte_size,
                    name,
                });
            }
        }
    }

    Ok(ResolvedType::default())
}
