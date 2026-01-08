//! C:\Users\gogin\Documents\GitHub\muffin\MuffinLib\lib\src\lib.rs — MAX.
//!
//! MuffinLib: foundational library for Muffin toolchain.
//!
//! This crate aggregates low-level, reusable building blocks:
//! - error: unified error + Result alias
//! - span: file/span/pos primitives (used by diag, parsers, etc.)
//! - diag: diagnostics rendering (if present in workspace)
//! - platform: OS helpers (if present in workspace)
//! - store: CAS + index + GC (if present in workspace)
//! - tool: process execution + probing (if present in workspace)
//!
//! Keep this crate dependency-light. Prefer re-exporting subcrates/modules.
//! If your workspace uses separate crates per folder, adapt `pub use` accordingly.

pub mod error;

pub type Result<T> = crate::error::Result<T>;
pub use crate::error::MuffinError;

// --- Optional in-crate modules (if this crate directly contains these trees) ---
pub mod span;
pub mod store;
pub mod tool;
pub mod platform;

// --- Optional re-exports (if you have separate crates instead) ---
// pub use muffin_span as span;
// pub use muffin_store as store;
// pub use muffin_tool as tool;
// pub use muffin_platform as platform;

// Common re-exports for ergonomics
pub use span::{FileId, LineCol, Location, Pos, PosLC, Range, SourceFile, Span, SpanRange};
pub use store::{Cas, CasConfig, Digest, DigestAlgo, StoreIndex};
pub use tool::{ToolOutput, ToolRunner, ToolSpec, ToolStatus};

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
    fn profile_is_known() {
        let p = build_profile();
        assert!(p == "debug" || p == "release");
    }
}
