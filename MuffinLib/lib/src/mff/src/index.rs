//! MFF index (MAX).
//!
//! The `.mff` file is Muffin's compiled/binary bundle format for reproducible builds.
//! Conceptually, an MFF contains:
//! - a header (magic/version/endianness)
//! - a table-of-contents (TOC) of typed entries
//! - content blobs (compressed or raw)
//! - optional signatures, provenance, and build metadata
//!
//! This `index` module defines the in-memory index model and helpers to:
//! - build an index while writing an MFF
//! - load an index while reading an MFF (without materializing all blobs)
//! - validate basic invariants (offsets, ranges, ordering, duplicates)
//! - resolve entries by path/logical name/type
//!
//! Notes:
//! - This is std-only (no serde). If you need JSON export, do it at a higher layer.
//! - Byte-order handling is delegated to your binary reader/writer helpers.
//! - Compression is represented abstractly; actual codecs live elsewhere.
//!
//! Typical usage (writer):
//!   let mut idx = MffIndex::new(MffVersion::V1);
//!   idx.push_entry(...);
//!   idx.finalize(file_len)?;
//!
//! Typical usage (reader):
//!   let idx = MffIndex::read_from(&mut reader)?;
//!   let e = idx.find_path("src/main.c")?;
//!   reader.seek(SeekFrom::Start(e.offset))?;
//!   ...

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::path::{Path, PathBuf};

pub const MFF_MAGIC: [u8; 4] = *b"MFF\0";

/// Format version for MFF.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum MffVersion {
    V1 = 1,
}

impl MffVersion {
    pub fn as_u32(self) -> u32 {
        self as u32
    }
}

/// Endianness stored in file.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Endian {
    Little,
    Big,
}

/// Compression kind for a blob entry payload.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Compression {
    None,
    /// Deflate (zlib) or raw DEFLATE; codec decided by container flags.
    Deflate,
    /// Zstd.
    Zstd,
    /// LZ4.
    Lz4,
}

/// High-level entry types inside the container.
///
/// Keep this list stable; it is part of the on-disk semantic model.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum EntryKind {
    /// A source file captured into the bundle.
    Source,
    /// A build manifest (e.g. `build.muf`, target files, etc.).
    Manifest,
    /// A resolved dependency lock / snapshot.
    Lock,
    /// A compiled artifact (object/library/exe).
    Artifact,
    /// A toolchain binary or metadata (optional).
    Tool,
    /// A plugin (dynamic module) or plugin metadata.
    Plugin,
    /// Diagnostics / logs captured during compilation.
    Log,
    /// Arbitrary key-value metadata (small).
    Meta,
    /// Signature (public key / signature blob / cert chain).
    Signature,
    /// Reserved/custom types.
    Custom(u32),
}

impl EntryKind {
    pub fn as_u32(self) -> u32 {
        match self {
            EntryKind::Source => 1,
            EntryKind::Manifest => 2,
            EntryKind::Lock => 3,
            EntryKind::Artifact => 4,
            EntryKind::Tool => 5,
            EntryKind::Plugin => 6,
            EntryKind::Log => 7,
            EntryKind::Meta => 8,
            EntryKind::Signature => 9,
            EntryKind::Custom(x) => x,
        }
    }

    pub fn from_u32(v: u32) -> Self {
        match v {
            1 => EntryKind::Source,
            2 => EntryKind::Manifest,
            3 => EntryKind::Lock,
            4 => EntryKind::Artifact,
            5 => EntryKind::Tool,
            6 => EntryKind::Plugin,
            7 => EntryKind::Log,
            8 => EntryKind::Meta,
            9 => EntryKind::Signature,
            x => EntryKind::Custom(x),
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            EntryKind::Source => "source",
            EntryKind::Manifest => "manifest",
            EntryKind::Lock => "lock",
            EntryKind::Artifact => "artifact",
            EntryKind::Tool => "tool",
            EntryKind::Plugin => "plugin",
            EntryKind::Log => "log",
            EntryKind::Meta => "meta",
            EntryKind::Signature => "signature",
            EntryKind::Custom(_) => "custom",
        }
    }
}

/// Entry flags (bitset).
///
/// Keep these stable; they may be persisted in TOC.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct EntryFlags(pub u32);

impl EntryFlags {
    pub const NONE: EntryFlags = EntryFlags(0);
    pub const COMPRESSED: EntryFlags = EntryFlags(1 << 0);
    pub const ENCRYPTED: EntryFlags = EntryFlags(1 << 1);
    pub const SIGNED: EntryFlags = EntryFlags(1 << 2);
    pub const EXECUTABLE: EntryFlags = EntryFlags(1 << 3);
    pub const READONLY: EntryFlags = EntryFlags(1 << 4);

    pub fn contains(self, other: EntryFlags) -> bool {
        (self.0 & other.0) == other.0
    }

    pub fn union(self, other: EntryFlags) -> EntryFlags {
        EntryFlags(self.0 | other.0)
    }
}

/// A stable identifier for an entry (hash of kind+path/logical+size).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct EntryId(pub u64);

/// One TOC entry in the index.
#[derive(Debug, Clone)]
pub struct MffEntry {
    pub id: EntryId,
    pub kind: EntryKind,

    /// Path inside bundle (normalized unix-style) if applicable.
    pub path: Option<String>,

    /// Logical name (e.g. "target:x86_64-linux-gnu", "toolchain:clang").
    pub logical: Option<String>,

    /// Offset of the stored payload in the file.
    pub offset: u64,
    /// Size of stored payload (compressed size if compressed).
    pub stored_size: u64,
    /// Original size (uncompressed). Equals stored_size if not compressed.
    pub original_size: u64,

    pub flags: EntryFlags,
    pub compression: Compression,

    /// Content hash (e.g. blake3/sha256) in hex; optional.
    pub content_hash: Option<String>,
    /// Optional build provenance hash (config graph hash).
    pub provenance_hash: Option<String>,

    /// Arbitrary metadata.
    pub meta: BTreeMap<String, String>,
}

impl MffEntry {
    pub fn new(kind: EntryKind) -> Self {
        Self {
            id: EntryId(0),
            kind,
            path: None,
            logical: None,
            offset: 0,
            stored_size: 0,
            original_size: 0,
            flags: EntryFlags::NONE,
            compression: Compression::None,
            content_hash: None,
            provenance_hash: None,
            meta: BTreeMap::new(),
        }
    }

    pub fn with_path(mut self, p: impl AsRef<Path>) -> Self {
        self.path = Some(normalize_bundle_path(p));
        self
    }

    pub fn with_logical(mut self, s: impl Into<String>) -> Self {
        self.logical = Some(s.into());
        self
    }

    pub fn with_offset(mut self, offset: u64) -> Self {
        self.offset = offset;
        self
    }

    pub fn with_sizes(mut self, stored: u64, original: u64) -> Self {
        self.stored_size = stored;
        self.original_size = original;
        self
    }

    pub fn with_flags(mut self, flags: EntryFlags) -> Self {
        self.flags = flags;
        self
    }

    pub fn with_compression(mut self, c: Compression) -> Self {
        self.compression = c;
        if c != Compression::None {
            self.flags = self.flags.union(EntryFlags::COMPRESSED);
        }
        self
    }

    pub fn with_hash(mut self, hex: impl Into<String>) -> Self {
        self.content_hash = Some(hex.into());
        self
    }

    pub fn with_provenance(mut self, hex: impl Into<String>) -> Self {
        self.provenance_hash = Some(hex.into());
        self
    }

    pub fn meta(mut self, k: impl Into<String>, v: impl Into<String>) -> Self {
        self.meta.insert(k.into(), v.into());
        self
    }

    pub fn range(&self) -> std::ops::Range<u64> {
        self.offset..(self.offset.saturating_add(self.stored_size))
    }
}

/// Full in-memory index.
#[derive(Debug, Clone)]
pub struct MffIndex {
    pub version: MffVersion,
    pub endian: Endian,

    /// Optional top-level metadata (container-level).
    pub meta: BTreeMap<String, String>,

    /// TOC entries.
    pub entries: Vec<MffEntry>,

    /// Optional file length (known after finalize/read).
    pub file_len: Option<u64>,

    /// Cached lookup maps (built on demand).
    cache: IndexCache,
}

#[derive(Debug, Clone, Default)]
struct IndexCache {
    built: bool,
    by_id: BTreeMap<EntryId, usize>,
    by_path: BTreeMap<String, Vec<usize>>,
    by_logical: BTreeMap<String, Vec<usize>>,
    by_kind: BTreeMap<EntryKind, Vec<usize>>,
}

#[derive(Debug)]
pub enum IndexError {
    Msg(String),
    DuplicateId(EntryId),
    DuplicateKey(String),
    OutOfBounds { id: EntryId, offset: u64, size: u64, file_len: u64 },
    Overlap { a: EntryId, b: EntryId },
    InvalidPath(String),
}

impl fmt::Display for IndexError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            IndexError::Msg(s) => write!(f, "{s}"),
            IndexError::DuplicateId(id) => write!(f, "duplicate entry id: {:?}", id),
            IndexError::DuplicateKey(k) => write!(f, "duplicate key: {k}"),
            IndexError::OutOfBounds { id, offset, size, file_len } => {
                write!(f, "entry {:?} out of bounds: off={} size={} file_len={}", id, offset, size, file_len)
            }
            IndexError::Overlap { a, b } => write!(f, "entries overlap: {:?} and {:?}", a, b),
            IndexError::InvalidPath(p) => write!(f, "invalid path: {p}"),
        }
    }
}

impl std::error::Error for IndexError {}

fn ierr(msg: impl Into<String>) -> IndexError {
    IndexError::Msg(msg.into())
}

impl MffIndex {
    pub fn new(version: MffVersion) -> Self {
        Self {
            version,
            endian: Endian::Little,
            meta: BTreeMap::new(),
            entries: Vec::new(),
            file_len: None,
            cache: IndexCache::default(),
        }
    }

    pub fn with_endian(mut self, e: Endian) -> Self {
        self.endian = e;
        self
    }

    pub fn meta(mut self, k: impl Into<String>, v: impl Into<String>) -> Self {
        self.meta.insert(k.into(), v.into());
        self
    }

    pub fn push_entry(&mut self, mut e: MffEntry) -> EntryId {
        // Compute ID deterministically.
        e.id = compute_entry_id(&e);
        let id = e.id;
        self.entries.push(e);
        self.cache.built = false;
        id
    }

    pub fn entries_len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn finalize(&mut self, file_len: u64) -> Result<(), IndexError> {
        self.file_len = Some(file_len);

        // Validate ranges + overlaps + duplicates
        self.validate()?;
        self.rebuild_cache();
        Ok(())
    }

    pub fn validate(&self) -> Result<(), IndexError> {
        let file_len = self.file_len.unwrap_or(u64::MAX);

        // ensure ids unique
        let mut ids = BTreeSet::new();
        for e in &self.entries {
            if !ids.insert(e.id) {
                return Err(IndexError::DuplicateId(e.id));
            }
            // validate path is normalized if present
            if let Some(p) = &e.path {
                if !is_normalized_bundle_path(p) {
                    return Err(IndexError::InvalidPath(p.clone()));
                }
            }
        }

        // bounds check + overlap check
        // sort by offset then compare ranges
        let mut tmp: Vec<&MffEntry> = self.entries.iter().collect();
        tmp.sort_by_key(|e| (e.offset, e.stored_size, e.id.0));

        for e in &tmp {
            if e.offset.saturating_add(e.stored_size) > file_len {
                return Err(IndexError::OutOfBounds {
                    id: e.id,
                    offset: e.offset,
                    size: e.stored_size,
                    file_len,
                });
            }
        }

        for w in tmp.windows(2) {
            let a = w[0];
            let b = w[1];
            if a.range().end > b.range().start {
                return Err(IndexError::Overlap { a: a.id, b: b.id });
            }
        }

        Ok(())
    }

    fn rebuild_cache(&mut self) {
        let mut c = IndexCache::default();

        for (i, e) in self.entries.iter().enumerate() {
            c.by_id.insert(e.id, i);
            c.by_kind.entry(e.kind).or_default().push(i);

            if let Some(p) = &e.path {
                c.by_path.entry(p.clone()).or_default().push(i);
            }
            if let Some(l) = &e.logical {
                c.by_logical.entry(l.clone()).or_default().push(i);
            }
        }

        c.built = true;
        self.cache = c;
    }

    fn ensure_cache(&mut self) {
        if !self.cache.built {
            self.rebuild_cache();
        }
    }

    pub fn get(&self, id: EntryId) -> Option<&MffEntry> {
        self.cache.by_id.get(&id).and_then(|&i| self.entries.get(i))
    }

    pub fn get_mut(&mut self, id: EntryId) -> Option<&mut MffEntry> {
        let i = *self.cache.by_id.get(&id)?;
        self.entries.get_mut(i)
    }

    pub fn find_path(&mut self, path: impl AsRef<Path>) -> Result<&MffEntry, IndexError> {
        self.ensure_cache();
        let key = normalize_bundle_path(path);
        let Some(idxs) = self.cache.by_path.get(&key) else {
            return Err(ierr(format!("path not found: {key}")));
        };
        let i = *idxs.first().unwrap();
        Ok(&self.entries[i])
    }

    pub fn find_logical(&mut self, logical: &str) -> Result<&MffEntry, IndexError> {
        self.ensure_cache();
        let Some(idxs) = self.cache.by_logical.get(logical) else {
            return Err(ierr(format!("logical not found: {logical}")));
        };
        let i = *idxs.first().unwrap();
        Ok(&self.entries[i])
    }

    pub fn find_kind(&mut self, kind: EntryKind) -> Vec<&MffEntry> {
        self.ensure_cache();
        self.cache
            .by_kind
            .get(&kind)
            .into_iter()
            .flat_map(|v| v.iter().map(|&i| &self.entries[i]))
            .collect()
    }

    pub fn iter(&self) -> impl Iterator<Item = &MffEntry> {
        self.entries.iter()
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut MffEntry> {
        self.cache.built = false;
        self.entries.iter_mut()
    }
}

/* ------------------------------ Path utils -------------------------------- */

/// Normalize a path to a canonical bundle path:
/// - UTF-8, forward slashes
/// - no drive letters
/// - no `..` segments (best effort)
pub fn normalize_bundle_path(p: impl AsRef<Path>) -> String {
    let p = p.as_ref();

    let mut parts: Vec<String> = Vec::new();
    for comp in p.components() {
        use std::path::Component;
        match comp {
            Component::Prefix(_) => {
                // drop Windows prefixes
            }
            Component::RootDir => {
                // ignore leading '/'
            }
            Component::CurDir => {}
            Component::ParentDir => {
                parts.pop();
            }
            Component::Normal(s) => {
                parts.push(s.to_string_lossy().to_string());
            }
        }
    }
    parts.join("/")
}

fn is_normalized_bundle_path(s: &str) -> bool {
    // cheap checks
    if s.contains('\\') {
        return false;
    }
    if s.starts_with('/') {
        return false;
    }
    if s.contains("..") {
        // reject "a/../b" and "../"
        // (note: false positives on literal ".." segment are intended)
        return false;
    }
    true
}

/* ------------------------------ ID hashing -------------------------------- */

fn compute_entry_id(e: &MffEntry) -> EntryId {
    // std-only hashing: stable-ish within same rust version.
    use std::hash::{Hash, Hasher};

    let mut h = std::collections::hash_map::DefaultHasher::new();
    e.kind.as_u32().hash(&mut h);
    e.path.hash(&mut h);
    e.logical.hash(&mut h);
    e.stored_size.hash(&mut h);
    e.original_size.hash(&mut h);
    e.flags.0.hash(&mut h);
    e.compression.hash(&mut h);
    if let Some(x) = &e.content_hash {
        x.hash(&mut h);
    }
    EntryId(h.finish())
}

/* -------------------------------- Tests ----------------------------------- */

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_path_windows_like() {
        let p = PathBuf::from(r"C:\Users\vince\proj\src\main.c");
        let n = normalize_bundle_path(&p);
        assert!(n.contains("Users"));
        assert!(!n.contains('\\'));
        assert!(!n.starts_with('/'));
    }

    #[test]
    fn index_validate_overlap() {
        let mut idx = MffIndex::new(MffVersion::V1);
        idx.file_len = Some(100);

        let e1 = MffEntry::new(EntryKind::Source)
            .with_path("src/a.c")
            .with_offset(10)
            .with_sizes(20, 20);

        let e2 = MffEntry::new(EntryKind::Source)
            .with_path("src/b.c")
            .with_offset(25) // overlaps with e1 [10..30)
            .with_sizes(10, 10);

        idx.push_entry(e1);
        idx.push_entry(e2);

        let r = idx.validate();
        assert!(matches!(r, Err(IndexError::Overlap { .. })));
    }

    #[test]
    fn find_by_path() {
        let mut idx = MffIndex::new(MffVersion::V1);
        idx.file_len = Some(1000);

        let e = MffEntry::new(EntryKind::Manifest)
            .with_path("build.muf")
            .with_offset(100)
            .with_sizes(10, 10);

        idx.push_entry(e);
        idx.finalize(1000).unwrap();

        let f = idx.find_path("build.muf").unwrap();
        assert_eq!(f.kind, EntryKind::Manifest);
    }
}
