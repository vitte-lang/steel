//! Vitte Library
//!
//! Utility functions for pattern matching and globbing.
//!
//! # Modules
//!
//! - `fnmatch` — Unix-like filename pattern matching (globbing without recursion)
//! - `glob` — Recursive glob pattern matching with ** support

pub mod fnmatch;
pub mod glob;

pub use fnmatch::{fnmatch, FnmatchOptions};
pub use glob::{glob, GlobPattern, GlobOptions};
