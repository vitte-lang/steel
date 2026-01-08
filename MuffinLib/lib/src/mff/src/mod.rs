//! Muffin File Format (MFF) module root.
//!
//! `.mff` is Muffin's compiled/binary bundle format intended for reproducible builds.
//!
//! Layers (recommended):
//! - `index`: in-memory index model (TOC entries, validation, lookups)
//! - `codec`: binary read/write helpers (LE/BE, varints, strings, sections)
//! - `reader`: high-level streaming reader (read index, access blobs, extract)
//! - `writer`: high-level writer (build index, write TOC, write blobs)
//! - `verify`: signature/provenance verification (optional)
//!
//! This file re-exports the public surface used by the rest of MuffinLib.
//!
//! NOTE:
//! - The code generated here assumes `index.rs` exists (already generated).
//! - `codec/reader/writer/verify` are declared even if not yet implemented;
//!   remove or comment out if the files are not present yet.

pub mod index;

// Optional modules (enable as files exist).
// pub mod codec;
// pub mod reader;
// pub mod writer;
// pub mod verify;

pub use index::{
    normalize_bundle_path, Compression, Endian, EntryFlags, EntryId, EntryKind, IndexError, MffEntry,
    MffIndex, MffVersion, MFF_MAGIC, GRAPH_JSON_SCHEMA_VERSION,
};

/// Schema version re-export for downstream tooling that stores graph JSON alongside MFF.
/// If you don't embed graph JSON in MFF, you can remove this.
pub use crate::graph::json::GRAPH_JSON_SCHEMA_VERSION;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn magic_is_constant() {
        assert_eq!(MFF_MAGIC, *b"MFF\0");
    }

    #[test]
    fn index_roundtrip_build() {
        let mut idx = MffIndex::new(MffVersion::V1);
        idx.file_len = Some(128);

        let e = MffEntry::new(EntryKind::Meta)
            .with_logical("meta:hello")
            .with_offset(16)
            .with_sizes(8, 8);

        idx.push_entry(e);
        idx.finalize(128).unwrap();

        assert_eq!(idx.entries_len(), 1);
        let f = idx.find_logical("meta:hello").unwrap();
        assert_eq!(f.kind, EntryKind::Meta);
    }
}
