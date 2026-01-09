//! Prelude (prelude.rs) — MAX.
//!
//! Purpose:
//! - Provide a single glob-import path for common MuffinLib types.
//! - Keep it conservative enough to avoid name conflicts, but ergonomic.
//!
//! Usage:
//! ```rust
//! use muffinlib::prelude::*;
//! ```
//!
//! Notes:
//! - This file assumes the crate root re-exports modules/types as in `lib.rs`.
//! - If some modules are feature-gated, you can gate exports here too.

pub use crate::error::{MuffinError, Result};

// --- span ---
pub use crate::span::{
    FileError, FileId, LineCol, Location, SourceFile, Span, SpanRange,
};

// --- store ---
pub use crate::store::{
    Cas, CasConfig, CasError, Digest, DigestAlgo, EntryKind, GcError, GcOptions, GcReport,
    IndexEntry, IndexError, StoreIndex,
};

// --- tool ---
pub use crate::tool::{
    ToolError, ToolOutput, ToolRunner, ToolSpec, ToolStatus,
};

// --- platform ---
pub use crate::platform::Platform;
