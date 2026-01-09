//! MFF reader (MAX).
//!
//! High-level streaming reader for `.mff` bundles.
//!
//! Responsibilities:
//! - validate header (magic/version/endian)
//! - read TOC/index
//! - provide random access to entry payloads (seek+read)
//! - convenience extractors (to files/bytes/string)
//! - optional transparent decompression (delegated to feature gates / stubs)
//!
//! This module is std-only and does not depend on external codecs.
//! Compression handling is represented via an abstraction; actual codec support
//! can be added behind features (zstd/lz4/flate2).
//!
//! Expected binary layout (conceptual, adaptable to your actual format):
//!   [Header]
//!   [TOC section: count + entries]
//!   [Blobs...]
//!
//! NOTE: If your existing on-disk format differs, adapt `read_header` and `read_index`
//! to match the real layout.

use std::fs::File;
use std::io::{self, Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};

use super::index::{
    Compression, Endian, EntryFlags, EntryId, EntryKind, IndexError, MffEntry, MffIndex, MffVersion,
    MFF_MAGIC,
};

#[derive(Debug)]
pub enum ReadError {
    Io(io::Error),
    Index(IndexError),
    InvalidMagic([u8; 4]),
    UnsupportedVersion(u32),
    UnsupportedEndian(u8),
    Corrupt(&'static str),
}

impl std::fmt::Display for ReadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ReadError::Io(e) => write!(f, "io: {e}"),
            ReadError::Index(e) => write!(f, "index: {e}"),
            ReadError::InvalidMagic(m) => write!(f, "invalid magic: {:?}", m),
            ReadError::UnsupportedVersion(v) => write!(f, "unsupported version: {v}"),
            ReadError::UnsupportedEndian(e) => write!(f, "unsupported endian tag: {e}"),
            ReadError::Corrupt(s) => write!(f, "corrupt: {s}"),
        }
    }
}

impl std::error::Error for ReadError {}

impl From<io::Error> for ReadError {
    fn from(e: io::Error) -> Self {
        ReadError::Io(e)
    }
}

impl From<IndexError> for ReadError {
    fn from(e: IndexError) -> Self {
        ReadError::Index(e)
    }
}

/// Header as read from file (in-memory).
#[derive(Debug, Clone)]
pub struct MffHeader {
    pub magic: [u8; 4],
    pub version: MffVersion,
    pub endian: Endian,
    pub toc_offset: u64,
    pub toc_size: u64,
}

impl MffHeader {
    pub fn is_valid(&self) -> bool {
        self.magic == MFF_MAGIC
    }
}

/// Reader options.
#[derive(Debug, Clone)]
pub struct ReaderOptions {
    /// If true, validate index ranges against file length.
    pub validate_ranges: bool,
    /// If true, validate overlap in index.
    pub validate_overlap: bool,
    /// If true, allow reading entries with compression != None but do NOT decompress (raw bytes).
    pub allow_compressed_raw: bool,
    /// Max allowed TOC entries for sanity.
    pub max_entries: u32,
}

impl Default for ReaderOptions {
    fn default() -> Self {
        Self {
            validate_ranges: true,
            validate_overlap: true,
            allow_compressed_raw: true,
            max_entries: 1_000_000,
        }
    }
}

/// High-level MFF reader over any Seek+Read.
pub struct MffReader<R: Read + Seek> {
    r: R,
    pub opts: ReaderOptions,
    pub header: MffHeader,
    pub index: MffIndex,
    file_len: u64,
}

impl MffReader<File> {
    pub fn open(path: impl AsRef<Path>) -> Result<Self, ReadError> {
        let f = File::open(path)?;
        Self::new(f, ReaderOptions::default())
    }

    pub fn open_with(path: impl AsRef<Path>, opts: ReaderOptions) -> Result<Self, ReadError> {
        let f = File::open(path)?;
        Self::new(f, opts)
    }
}

impl<R: Read + Seek> MffReader<R> {
    pub fn new(mut r: R, opts: ReaderOptions) -> Result<Self, ReadError> {
        let file_len = r.seek(SeekFrom::End(0))?;
        r.seek(SeekFrom::Start(0))?;

        let header = read_header(&mut r)?;
        if !header.is_valid() {
            return Err(ReadError::InvalidMagic(header.magic));
        }

        let mut index = read_index(&mut r, &header, file_len, &opts)?;
        index.file_len = Some(file_len);

        if opts.validate_ranges || opts.validate_overlap {
            index.validate()?;
        }
        index.finalize(file_len)?;

        Ok(Self {
            r,
            opts,
            header,
            index,
            file_len,
        })
    }

    pub fn file_len(&self) -> u64 {
        self.file_len
    }

    pub fn inner_mut(&mut self) -> &mut R {
        &mut self.r
    }

    pub fn inner(&self) -> &R {
        &self.r
    }

    /// Locate an entry by normalized path.
    pub fn entry_by_path(&mut self, path: impl AsRef<Path>) -> Result<&MffEntry, ReadError> {
        Ok(self.index.find_path(path)?)
    }

    /// Locate an entry by logical name.
    pub fn entry_by_logical(&mut self, logical: &str) -> Result<&MffEntry, ReadError> {
        Ok(self.index.find_logical(logical)?)
    }

    /// Get all entries of a kind.
    pub fn entries_by_kind(&mut self, kind: EntryKind) -> Vec<&MffEntry> {
        self.index.find_kind(kind)
    }

    /// Read stored payload bytes (compressed if compressed).
    pub fn read_entry_stored(&mut self, id: EntryId) -> Result<Vec<u8>, ReadError> {
        let e = self
            .index
            .get(id)
            .ok_or_else(|| ReadError::Corrupt("entry id not found"))?
            .clone();

        self.read_range(e.offset, e.stored_size)
    }

    /// Read payload bytes (decompressed if supported and requested).
    ///
    /// If compression is not supported and `allow_compressed_raw` is true,
    /// returns stored bytes as-is.
    pub fn read_entry(&mut self, id: EntryId) -> Result<Vec<u8>, ReadError> {
        let e = self
            .index
            .get(id)
            .ok_or_else(|| ReadError::Corrupt("entry id not found"))?
            .clone();

        let stored = self.read_range(e.offset, e.stored_size)?;

        match e.compression {
            Compression::None => Ok(stored),
            _ => {
                if self.opts.allow_compressed_raw {
                    // Return stored bytes (caller can decompress).
                    Ok(stored)
                } else {
                    Err(ReadError::Corrupt("compressed entry but raw not allowed"))
                }
            }
        }
    }

    /// Read payload as UTF-8 string (after `read_entry()`).
    pub fn read_entry_string(&mut self, id: EntryId) -> Result<String, ReadError> {
        let bytes = self.read_entry(id)?;
        let s = String::from_utf8(bytes).map_err(|_| ReadError::Corrupt("utf8 decode failed"))?;
        Ok(s)
    }

    /// Extract entry payload to a file.
    pub fn extract_entry_to(
        &mut self,
        id: EntryId,
        out_path: impl AsRef<Path>,
    ) -> Result<(), ReadError> {
        let bytes = self.read_entry(id)?;
        std::fs::write(out_path, bytes)?;
        Ok(())
    }

    /// Extract an entry referenced by path to a directory, preserving relative path.
    pub fn extract_path_to_dir(
        &mut self,
        path: impl AsRef<Path>,
        out_dir: impl AsRef<Path>,
    ) -> Result<PathBuf, ReadError> {
        let (id, rel) = {
            let e = self.entry_by_path(path)?;
            let rel = e
                .path
                .as_ref()
                .ok_or_else(|| ReadError::Corrupt("entry has no path"))?
                .clone();
            (e.id, rel)
        };
        let out = out_dir.as_ref().join(rel);
        if let Some(parent) = out.parent() {
            std::fs::create_dir_all(parent)?;
        }
        self.extract_entry_to(id, &out)?;
        Ok(out)
    }

    /// Read a byte range from the file.
    pub fn read_range(&mut self, offset: u64, size: u64) -> Result<Vec<u8>, ReadError> {
        if offset.saturating_add(size) > self.file_len {
            return Err(ReadError::Corrupt("range out of bounds"));
        }

        self.r.seek(SeekFrom::Start(offset))?;
        let mut buf = vec![0u8; size as usize];
        self.r.read_exact(&mut buf)?;
        Ok(buf)
    }
}

/* ------------------------------ Binary layout ----------------------------- */

fn read_header<R: Read>(r: &mut R) -> Result<MffHeader, ReadError> {
    let mut magic = [0u8; 4];
    r.read_exact(&mut magic)?;

    let version = read_u32(r)?;
    let endian_tag = read_u8(r)?;
    let mut reserved = [0u8; 7];
    r.read_exact(&mut reserved)?;

    let toc_offset = read_u64(r)?;
    let toc_size = read_u64(r)?;

    let version = match version {
        1 => MffVersion::V1,
        v => return Err(ReadError::UnsupportedVersion(v)),
    };

    let endian = match endian_tag {
        1 => Endian::Little,
        2 => Endian::Big,
        e => return Err(ReadError::UnsupportedEndian(e)),
    };

    Ok(MffHeader {
        magic,
        version,
        endian,
        toc_offset,
        toc_size,
    })
}

fn read_index<R: Read + Seek>(
    r: &mut R,
    header: &MffHeader,
    file_len: u64,
    opts: &ReaderOptions,
) -> Result<MffIndex, ReadError> {
    if header.toc_offset == 0 || header.toc_size == 0 {
        return Err(ReadError::Corrupt("toc not present"));
    }

    r.seek(SeekFrom::Start(header.toc_offset))?;
    let count = read_u32(r)?;
    if count > opts.max_entries {
        return Err(ReadError::Corrupt("toc entry count too large"));
    }

    let mut index = MffIndex::new(header.version).with_endian(header.endian);
    index.file_len = Some(file_len);

    for _ in 0..count {
        let kind = EntryKind::from_u32(read_u32(r)?);
        let flags = read_u32(r)?;
        let compression = read_compression(r)?;
        let _reserved = read_u32(r)?;

        let offset = read_u64(r)?;
        let stored_size = read_u64(r)?;
        let original_size = read_u64(r)?;

        let path = read_opt_string_u16(r)?;
        let logical = read_opt_string_u16(r)?;
        let content_hash = read_opt_string_u16(r)?;
        let provenance_hash = read_opt_string_u16(r)?;

        let meta_count = read_u32(r)?;
        let mut meta = std::collections::BTreeMap::new();
        for _ in 0..meta_count {
            let k = read_string_u16(r)?;
            let v = read_string_u16(r)?;
            meta.insert(k, v);
        }

        let mut e = MffEntry::new(kind)
            .with_offset(offset)
            .with_sizes(stored_size, original_size)
            .with_flags(EntryFlags(flags))
            .with_compression(compression);
        e.path = path;
        e.logical = logical;
        e.content_hash = content_hash;
        e.provenance_hash = provenance_hash;
        e.meta = meta;

        index.push_entry(e);
    }

    Ok(index)
}

fn read_u8<R: Read>(r: &mut R) -> Result<u8, ReadError> {
    let mut b = [0u8; 1];
    r.read_exact(&mut b)?;
    Ok(b[0])
}

fn read_u16<R: Read>(r: &mut R) -> Result<u16, ReadError> {
    let mut b = [0u8; 2];
    r.read_exact(&mut b)?;
    Ok(u16::from_le_bytes(b))
}

fn read_u32<R: Read>(r: &mut R) -> Result<u32, ReadError> {
    let mut b = [0u8; 4];
    r.read_exact(&mut b)?;
    Ok(u32::from_le_bytes(b))
}

fn read_u64<R: Read>(r: &mut R) -> Result<u64, ReadError> {
    let mut b = [0u8; 8];
    r.read_exact(&mut b)?;
    Ok(u64::from_le_bytes(b))
}

fn read_string_u16<R: Read>(r: &mut R) -> Result<String, ReadError> {
    let len = read_u16(r)? as usize;
    if len == 0 {
        return Ok(String::new());
    }
    let mut buf = vec![0u8; len];
    r.read_exact(&mut buf)?;
    String::from_utf8(buf).map_err(|_| ReadError::Corrupt("utf8 decode failed"))
}

fn read_opt_string_u16<R: Read>(r: &mut R) -> Result<Option<String>, ReadError> {
    let len = read_u16(r)? as usize;
    if len == 0 {
        return Ok(None);
    }
    let mut buf = vec![0u8; len];
    r.read_exact(&mut buf)?;
    let s = String::from_utf8(buf).map_err(|_| ReadError::Corrupt("utf8 decode failed"))?;
    Ok(Some(s))
}

fn read_compression<R: Read>(r: &mut R) -> Result<Compression, ReadError> {
    match read_u32(r)? {
        0 => Ok(Compression::None),
        1 => Ok(Compression::Deflate),
        2 => Ok(Compression::Zstd),
        3 => Ok(Compression::Lz4),
        _ => Err(ReadError::Corrupt("unknown compression tag")),
    }
}
