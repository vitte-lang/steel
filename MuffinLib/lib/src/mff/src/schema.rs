//! MFF schema constants + binary layout definitions (schema.rs).
//!
//! This module centralizes the on-disk schema identifiers and numeric tags used by
//! Muffin File Format (MFF). The intent is to keep *all* magic values and tag
//! numbers in one place so reader/writer stay consistent.
//!
//! If you change these, you are changing the file format. Add new versions
//! instead of breaking existing tags.

use super::index::{Compression, Endian, EntryKind, MffVersion, MFF_MAGIC};

/// Global schema name for tooling (not necessarily stored on-disk).
pub const MFF_SCHEMA_NAME: &str = "muffin.mff";

/// Current schema version string (tooling-facing, not the on-disk `MffVersion`).
pub const MFF_SCHEMA_VERSION: &str = "1.0";

/// Fixed header size in bytes for the minimal layout.
pub const MFF_HEADER_SIZE: u64 = 32;

/// Endian tag values stored in header.
pub const MFF_ENDIAN_LE: u8 = 1;
pub const MFF_ENDIAN_BE: u8 = 2;

/// Compression tag values stored in TOC entries.
pub const MFF_COMP_NONE: u32 = 0;
pub const MFF_COMP_DEFLATE: u32 = 1;
pub const MFF_COMP_ZSTD: u32 = 2;
pub const MFF_COMP_LZ4: u32 = 3;

/// Entry flags bit positions (must match `EntryFlags`).
pub const MFF_FLAG_COMPRESSED: u32 = 1 << 0;
pub const MFF_FLAG_ENCRYPTED: u32 = 1 << 1;
pub const MFF_FLAG_SIGNED: u32 = 1 << 2;
pub const MFF_FLAG_EXECUTABLE: u32 = 1 << 3;
pub const MFF_FLAG_READONLY: u32 = 1 << 4;

/// TOC section sanity limits (defensive parsing).
pub const MFF_MAX_TOC_ENTRIES_DEFAULT: u32 = 1_000_000;
pub const MFF_MAX_STRING_U16: usize = u16::MAX as usize;

/// Minimal conceptual layout notes (kept in code for single-source-of-truth).
///
/// Header (fixed 32 bytes):
///   0..4   magic "MFF\0"
///   4..8   u32 version
///   8      u8 endian (1=LE,2=BE)
///   9..16  reserved (7 bytes)
///   16..24 u64 toc_offset
///   24..32 u64 toc_size
///
/// TOC (at toc_offset):
///   u32 entry_count
///   repeated entry_count times:
///     u32 kind
///     u32 flags
///     u32 compression
///     u32 reserved
///     u64 offset
///     u64 stored_size
///     u64 original_size
///     u16 path_len, bytes (utf8)
///     u16 logical_len, bytes (utf8)
///     u16 hash_len, bytes (utf8 hex)
///     u16 prov_len, bytes (utf8 hex)
///     u32 meta_count
///       repeated meta_count:
///         u16 klen, bytes
///         u16 vlen, bytes
pub const _MFF_LAYOUT_DOC: &str = "see module doc";

/// Convert on-disk endian tag to `Endian`.
pub fn endian_from_tag(tag: u8) -> Option<Endian> {
    match tag {
        MFF_ENDIAN_LE => Some(Endian::Little),
        MFF_ENDIAN_BE => Some(Endian::Big),
        _ => None,
    }
}

/// Convert `Endian` to on-disk tag.
pub fn endian_to_tag(e: Endian) -> u8 {
    match e {
        Endian::Little => MFF_ENDIAN_LE,
        Endian::Big => MFF_ENDIAN_BE,
    }
}

/// Convert on-disk compression tag to `Compression`.
pub fn compression_from_tag(tag: u32) -> Option<Compression> {
    match tag {
        MFF_COMP_NONE => Some(Compression::None),
        MFF_COMP_DEFLATE => Some(Compression::Deflate),
        MFF_COMP_ZSTD => Some(Compression::Zstd),
        MFF_COMP_LZ4 => Some(Compression::Lz4),
        _ => None,
    }
}

/// Convert `Compression` to on-disk tag.
pub fn compression_to_tag(c: Compression) -> u32 {
    match c {
        Compression::None => MFF_COMP_NONE,
        Compression::Deflate => MFF_COMP_DEFLATE,
        Compression::Zstd => MFF_COMP_ZSTD,
        Compression::Lz4 => MFF_COMP_LZ4,
    }
}

/// On-disk numeric tag for `EntryKind`.
pub fn entry_kind_to_tag(k: EntryKind) -> u32 {
    k.as_u32()
}

/// `EntryKind` from on-disk numeric tag.
pub fn entry_kind_from_tag(tag: u32) -> EntryKind {
    EntryKind::from_u32(tag)
}

/// Validate the header magic.
pub fn is_valid_magic(m: [u8; 4]) -> bool {
    m == MFF_MAGIC
}

/// On-disk version tag from `MffVersion`.
pub fn version_to_tag(v: MffVersion) -> u32 {
    v.as_u32()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn endian_tags_roundtrip() {
        assert_eq!(endian_from_tag(endian_to_tag(Endian::Little)), Some(Endian::Little));
        assert_eq!(endian_from_tag(endian_to_tag(Endian::Big)), Some(Endian::Big));
    }

    #[test]
    fn compression_tags_roundtrip() {
        for c in [Compression::None, Compression::Deflate, Compression::Zstd, Compression::Lz4] {
            assert_eq!(compression_from_tag(compression_to_tag(c)), Some(c));
        }
    }

    #[test]
    fn magic_matches() {
        assert!(is_valid_magic(MFF_MAGIC));
    }
}
