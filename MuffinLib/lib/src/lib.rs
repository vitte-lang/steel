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
#![allow(missing_docs)]

pub mod error;
pub mod prelude;

pub type Result<T> = crate::error::Result<T>;
pub use crate::error::MuffinError;

// Facade: wire module roots to the shared source trees.
#[path = "diag/src/mod.rs"]
pub mod diag;
#[path = "graph/src/mod.rs"]
pub mod graph;
#[path = "mff/src/mod.rs"]
pub mod mff;
#[path = "path/src/mod.rs"]
pub mod path;
#[path = "platform/src/mod.rs"]
pub mod platform;
#[path = "span/src/mod.rs"]
pub mod span;
#[path = "store/src/mod.rs"]
pub mod store;
#[path = "tool/src/mod.rs"]
pub mod tool;

// MUF v4.1 lexer/parser/AST (Bracket + Dot Ops).
#[path = "muffin/src/ast.rs"]
pub mod muf_ast;
#[path = "muffin/src/lexer.rs"]
pub mod muf_lexer;
#[path = "muffin/src/parser.rs"]
pub mod muf_parser;

// --- Common ergonomic re-exports (keep stable) ---
pub use span::{FileError, FileId, LineCol, Location, SourceFile, Span, SpanRange};
pub use store::{
    Cas, CasConfig, CasError, Digest, DigestAlgo, EntryKind, GcError, GcOptions, GcReport,
    IndexEntry, IndexError, StoreIndex,
};
pub use tool::{ToolError, ToolOutput, ToolRunner, ToolSpec, ToolStatus};
pub use muf_ast::{
    Atom as MufAtom, Block as MufBlock, BlockItem as MufBlockItem, MufFile, Number as MufNumber,
    Pos as MufPos, RefPath as MufRefPath, Span as MufSpan,
};
pub use muf_lexer::{LexError as MufLexError, Lexer as MufLexer, Token as MufToken, TokenKind as MufTokenKind};
pub use muf_parser::{parse_muf, ParseError as MufParseError};

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
    let os = option_env!("CARGO_CFG_TARGET_OS").unwrap_or("unknown");
    let arch = option_env!("CARGO_CFG_TARGET_ARCH").unwrap_or("unknown");
    format!(
        "muffinlib {} ({}/{})",
        MUFFINLIB_VERSION,
        os,
        arch
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
