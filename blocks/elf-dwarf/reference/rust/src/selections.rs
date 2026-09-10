//! `selections` — one `[[elf_dwarf.selections]]` entry becomes one
//! filtered walk over an image's statically-allocated variables.
//!
//! Mirrors the README worked example.

use crate::types::{ResolvedType, ScalarKind, resolve_type, static_address};

/// One statically-allocated variable recovered from debug info.
#[derive(Debug, Clone)]
pub struct Global {
    pub name: String,
    /// Absolute link-time address.
    pub address: u64,
    /// Size in bytes, from the variable's type.
    pub byte_size: u64,
    pub kind: ScalarKind,
    pub type_name: Option<String>,
    /// Name of the compilation unit that declared it, when recorded.
    /// Two units can each declare a `static` of the same name, so this
    /// is what tells them apart.
    pub unit_name: Option<String>,
}

/// A selection's filter, resolved from the spec at codegen time.
#[derive(Debug, Clone)]
pub struct Filter {
    /// `name_pattern`, already split on `*` into literal fragments.
    pub fragments: Vec<String>,
    pub leading_wildcard: bool,
    pub trailing_wildcard: bool,
    /// `kinds`; empty means every kind is accepted.
    pub kinds: Vec<ScalarKind>,
    pub max_bytes: Option<u64>,
    /// `include_function_statics` from the owning image.
    pub include_function_statics: bool,
}

impl Filter {
    /// Glob matching over the fragments. Kept deliberately small: the
    /// pattern vocabulary is `*` only, so a full glob crate would be a
    /// dependency earning nothing.
    pub fn matches_name(&self, name: &str) -> bool {
        let mut rest = name;

        for (index, fragment) in self.fragments.iter().enumerate() {
            let first = index == 0;

            match rest.find(fragment.as_str()) {
                Some(at) => {
                    // A fragment before the first `*` must be anchored.
                    if first && !self.leading_wildcard && at != 0 {
                        return false;
                    }
                    rest = &rest[at + fragment.len()..];
                }
                None => return false,
            }
        }

        // A fragment after the last `*` must reach the end.
        self.trailing_wildcard || rest.is_empty() || self.fragments.is_empty()
    }

    pub fn accepts(&self, global: &Global) -> bool {
        if !self.kinds.is_empty() && !self.kinds.contains(&global.kind) {
            return false;
        }

        if let Some(limit) = self.max_bytes {
            if global.byte_size > limit {
                return false;
            }
        }

        self.matches_name(&global.name)
    }
}

/// Walking one compilation unit and pushing every accepted global.
///
/// The DFS visits function-scope `static` variables too. They are
/// statically allocated and perfectly readable, and they appear at a
/// depth below one — which is what `include_function_statics` filters
/// on.
pub fn collect_unit<R: gimli::Reader>(
    dwarf: &gimli::Dwarf<R>,
    unit: &gimli::Unit<R>,
    filter: &Filter,
    out: &mut Vec<Global>,
) -> Result<(), gimli::Error> {
    let unit_name = match unit.name.as_ref() {
        Some(name) => Some(name.to_string_lossy()?.into_owned()),
        None => None,
    };

    let mut cursor: gimli::EntriesCursor<'_, R> = unit.entries();

    while let Some(entry) = cursor.next_dfs()? {
        if entry.tag() != gimli::DW_TAG_variable {
            continue;
        }

        // A declaration is an `extern` with no storage in this image.
        if matches!(
            entry.attr_value(gimli::DW_AT_declaration),
            Some(gimli::AttributeValue::Flag(true))
        ) {
            continue;
        }

        if !filter.include_function_statics && entry.depth() > 1 {
            continue;
        }

        // No single address means it is not statically allocated.
        let address = match static_address(dwarf, unit, entry)? {
            Some(address) => address,
            None => continue,
        };

        let name = match entry.attr_value(gimli::DW_AT_name) {
            Some(value) => dwarf.attr_string(unit, value)?.to_string_lossy()?.into_owned(),
            None => continue,
        };

        let resolved: ResolvedType = match entry.attr_value(gimli::DW_AT_type) {
            Some(gimli::AttributeValue::UnitRef(offset)) => resolve_type(dwarf, unit, offset)?,
            _ => ResolvedType::default(),
        };

        let byte_size = resolved
            .byte_size
            .or_else(|| {
                entry
                    .attr(gimli::DW_AT_byte_size)
                    .and_then(|attr| attr.udata_value())
            })
            .unwrap_or(0);

        let global = Global {
            name,
            address,
            byte_size,
            kind: resolved.kind,
            type_name: resolved.name,
            unit_name: unit_name.clone(),
        };

        if filter.accepts(&global) {
            out.push(global);
        }
    }

    Ok(())
}

/// The whole selection: every unit in the image, filtered.
pub fn collect<R: gimli::Reader>(
    dwarf: &gimli::Dwarf<R>,
    filter: &Filter,
) -> Result<Vec<Global>, gimli::Error> {
    let mut out: Vec<Global> = Vec::new();
    let mut units = dwarf.units();

    while let Some(header) = units.next()? {
        let unit: gimli::Unit<R> = dwarf.unit(header)?;
        collect_unit(dwarf, &unit, filter, &mut out)?;
    }

    Ok(out)
}
