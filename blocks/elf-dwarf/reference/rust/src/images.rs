//! `images` — one `[[elf_dwarf.images]]` entry becomes one ELF load path.
//!
//! Mirrors the README worked example. Loading is deliberately two-stage:
//! an owning value holds the section bytes and a borrowing value indexes
//! into them, so the borrowed `Dwarf` cannot outlive its storage.

use std::borrow::Cow;

// `Object` and `ObjectSection` are TRAITS and must be in scope for
// `File::section_by_name` and `Section::uncompressed_data` to resolve.
use object::{Object, ObjectSection};

/// The `endian` enum, one variant per schema value.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Endian {
    Auto,
    Little,
    Big,
}

/// Stage one: parse the container and read the endianness off it.
///
/// `object::File<'data, R = &'data [u8]>` takes a lifetime and a reader
/// type, the second defaulted — so the annotation names both. The two
/// must be the **same** lifetime: elided separately they do not unify,
/// and the borrow checker rejects the signature.
pub fn parse<'a>(bytes: &'a [u8]) -> Result<object::File<'a, &'a [u8]>, object::Error> {
    object::File::parse(bytes)
}

/// `endian = "auto"` reads the container's own declaration, which is
/// right for every real toolchain output. The explicit variants exist
/// for a stripped or hand-assembled image whose header lies.
pub fn endianness<'a>(
    object_file: &object::File<'a, &'a [u8]>,
    declared: Endian,
) -> gimli::RunTimeEndian {
    match declared {
        Endian::Little => gimli::RunTimeEndian::Little,
        Endian::Big => gimli::RunTimeEndian::Big,
        Endian::Auto => {
            if object_file.is_little_endian() {
                gimli::RunTimeEndian::Little
            } else {
                gimli::RunTimeEndian::Big
            }
        }
    }
}

/// Stage two: hand gimli each debug section's bytes.
///
/// A **missing** section must come back as an empty slice, not an error.
/// Debug info is routinely partial — an image built without
/// `.debug_addr`, or without line tables, is still perfectly readable —
/// and failing the load over an absent section rejects images that work.
pub fn load_sections<'data>(
    object_file: &'data object::File<'data, &'data [u8]>,
) -> Result<gimli::DwarfSections<Cow<'data, [u8]>>, gimli::Error> {
    gimli::DwarfSections::load(|id: gimli::SectionId| -> Result<Cow<'data, [u8]>, gimli::Error> {
        Ok(match object_file.section_by_name(id.name()) {
            Some(section) => section
                .uncompressed_data()
                .unwrap_or(Cow::Borrowed(&[][..])),
            None => Cow::Borrowed(&[][..]),
        })
    })
}

/// Stage three: borrow the owned bytes as readers.
///
/// `Dwarf<R>` takes one parameter; `EndianSlice<'input, Endian>` takes a
/// lifetime and an endianity type. The returned `Dwarf` borrows from
/// `sections`, so `sections` must outlive it.
pub fn borrow_sections<'a>(
    sections: &'a gimli::DwarfSections<Cow<'a, [u8]>>,
    endian: gimli::RunTimeEndian,
) -> gimli::Dwarf<gimli::EndianSlice<'a, gimli::RunTimeEndian>> {
    sections.borrow(|section| gimli::EndianSlice::new(section, endian))
}

/// Iterating compilation units.
///
/// `dwarf.units()` yields a header iterator with an **inherent** `next`
/// returning `Result<Option<_>>` — it is not a `std` iterator, so
/// `for header in dwarf.units()` does not compile.
pub fn unit_count<R: gimli::Reader>(dwarf: &gimli::Dwarf<R>) -> Result<usize, gimli::Error> {
    let mut units = dwarf.units();
    let mut count = 0usize;

    while let Some(header) = units.next()? {
        let _unit: gimli::Unit<R> = dwarf.unit(header)?;
        count += 1;
    }

    Ok(count)
}

/// Whether the image is linked.
///
/// In an unlinked object file every variable's address is a relocation
/// placeholder — normally zero — so a symbol table read from one is
/// uniformly wrong rather than obviously broken. Checking the container
/// kind up front turns that into a clear refusal.
pub fn is_linked<'a>(object_file: &object::File<'a, &'a [u8]>) -> bool {
    matches!(
        object_file.kind(),
        object::ObjectKind::Executable | object::ObjectKind::Dynamic
    )
}
