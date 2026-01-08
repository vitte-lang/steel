// src/muffinint.rs
//
// Muffin — "muffinint" (internal integer utilities)
//
// Purpose:
// - Provide small, dependency-free integer helpers used across Muffin:
//   - parsing ints with strict rules (no locale, no underscores by default)
//   - safe conversions (checked / saturating / clamped)
//   - human-friendly formatting (bytes, durations) when needed by CLI/output
//   - stable hashing helpers for numeric keys
//
// Naming:
// - "muffinint" is an internal util module; adapt to your crate layout.
//
// Notes:
// - Prefer explicit behavior: return Result with clear errors.
// - No allocations on hot paths unless formatting/humanization is requested.

#![allow(dead_code)]

use std::fmt;
use std::num::{IntErrorKind, ParseIntError};
use std::time::Duration;

/* ============================== errors ============================== */

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IntParseError {
    Empty,
    Invalid {
        input: String,
        reason: String,
    },
    Overflow {
        input: String,
    },
}

impl fmt::Display for IntParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            IntParseError::Empty => write!(f, "empty integer"),
            IntParseError::Invalid { input, reason } => write!(f, "invalid integer '{input}': {reason}"),
            IntParseError::Overflow { input } => write!(f, "integer overflow '{input}'"),
        }
    }
}

impl std::error::Error for IntParseError {}

/* ============================== strict parsing ============================== */

/// Parse i64 with strict rules:
/// - trims whitespace
/// - accepts optional leading +/-
—/// - base 10 only (unless `0x`/`0b`/`0o` prefixes are present if `allow_prefix_base` = true)
/// - rejects underscores unless allow_underscores
pub fn parse_i64_strict(
    s: &str,
    allow_prefix_base: bool,
    allow_underscores: bool,
) -> Result<i64, IntParseError> {
    let t = s.trim();
