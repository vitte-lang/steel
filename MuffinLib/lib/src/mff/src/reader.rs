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
    Compression, Endian, EntryId, EntryKind, IndexError, MffEntry, MffIndex, MffVersion, MFF_MAGIC,
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
        let e = self.entry_by_path(path)?;
        let rel = e
            .path
            .as_ref()
            .ok_or_else(|| ReadError::Corrupt("entry has no path"))?;
        let out = out_dir.as_ref().join(rel);
        if let Some(parent) = out.parent() {
            std::fs::create_dir_all(parent)?;
        }
        self.extract_entry_to(e.id, &out)?;
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
/*
   This is a concrete minimal layout used by this generated reader.
   If your real format differs, edit these functions to match it.

   Header (f
