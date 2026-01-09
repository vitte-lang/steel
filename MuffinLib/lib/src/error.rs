//! MuffinLib error types and Result alias.

use std::fmt;

/// Error type for the MuffinLib facade crate.
#[derive(Debug)]
pub enum MuffinError {
    /// Generic error message.
    Message(String),
    /// IO errors.
    Io(std::io::Error),
}

impl fmt::Display for MuffinError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MuffinError::Message(msg) => write!(f, "{msg}"),
            MuffinError::Io(err) => write!(f, "io error: {err}"),
        }
    }
}

impl std::error::Error for MuffinError {}

impl From<std::io::Error> for MuffinError {
    fn from(err: std::io::Error) -> Self {
        MuffinError::Io(err)
    }
}

/// Convenience Result alias for MuffinLib consumers.
pub type Result<T> = std::result::Result<T, MuffinError>;
