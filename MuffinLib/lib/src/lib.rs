//! C:\Users\gogin\Documents\GitHub\muffin\MuffinLib\lib\src\lib.rs — MAX.
//!
//! MuffinLib: core utility library for Muffin toolchain.
//!
//! This crate is intentionally dependency-light and std-only by default.
//! It aggregates and re-exports foundational modules used across the Muffin
//! compiler/build ecosystem.
//!
//! Modules (expected):
//! - error     : unified error type
//! - span      : SourceFile/Span/Pos primitives
//! - diag      : diagnostics (optional, if present)
//! - platform  : OS helpers (optional, if present)
//! - store     : CAS + index + GC (optional, if present)
//! - tool      : tool runner + probe (optional, if present)
//! - llib      : curated facade + prelude (optional)
//!
//! Keep `lib.rs` as the stable entrypoint; internal reorgs should not break downstream.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod error;

pub type Result<T> = crate::error::Result<T>;
pub use crate::error::MuffinError;

// Optional modules: include if these folders exist inside this crate.
pub mod span;
pub mod store;
pub mod tool;
pub mod platform;

// Optional convenience facade.
pub mod llib;

// --- Common ergonomic re-exports (keep stable) ---
pub use span::{FileError, FileId, LineCol, Location, Pos, PosLC, Range, SourceFile, Span, SpanRange};
pub use store::{
    Cas, CasConfig, CasError, Digest, DigestAlgo, EntryKind, GcError, GcOptions, GcReport,
    IndexEntry, IndexError, StoreIndex,
};
pub use tool::{
    ProbeError, ToolCandidate, ToolError, ToolOutput, ToolProbe, ToolRunner, ToolSpec, ToolStatus,
};

// If `platform` module defines `Platform` (recommended), re-export it.
// If not present, remove these lines.
pub use platform::Platform;

/// MuffinLib version string (compile-time).
pub const MUFFINLIB_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Build profile info (best-effort).
pub fn build_profile() -> &'static str {
    if cfg!(debug_assertions) {
        "debug"
    } else {
        "release"
    }
}

/// Minimal library info string.
pub fn info_string() -> String {
    format!(
        "muffinlib {} ({}/{})",
        MUFFINLIB_VERSION,
        env!("CARGO_CFG_TARGET_OS"),
        env!("CARGO_CFG_TARGET_ARCH")
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn info_non_empty() {
        assert!(!info_string().is_empty());
    }

    #[test]
    fn profile_known() {
        let p = build_profile();
        assert!(p == "debug" || p == "release");
    }
}
