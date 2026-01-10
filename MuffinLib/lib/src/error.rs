//! MuffinLib error types and Result alias.

use std::fmt;

/// Structured diagnostic data for CLI rendering.
#[derive(Debug, Clone)]
pub struct Diagnostic {
    pub code: &'static str,
    pub message: String,
    pub help: Vec<String>,
}

/// Error type for the MuffinLib facade crate.
#[derive(Debug)]
pub enum MuffinError {
    /// Generic error message.
    Message(String),
    /// User-facing validation failure.
    ValidationFailed(String),
    /// Execution failure (tools/processes/sandbox).
    ExecutionFailed(String),
    /// CLI usage/command error (option parsing, unknown command).
    InvalidCommand { message: String, help: Vec<String> },
    /// Not found or missing resource.
    NotFound(String),
    /// Internal logic error.
    Internal(String),
    /// IO errors.
    Io(std::io::Error),
}

impl fmt::Display for MuffinError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MuffinError::Message(msg) => write!(f, "{msg}"),
            MuffinError::ValidationFailed(msg) => write!(f, "{msg}"),
            MuffinError::ExecutionFailed(msg) => write!(f, "{msg}"),
            MuffinError::InvalidCommand { message, .. } => write!(f, "{message}"),
            MuffinError::NotFound(msg) => write!(f, "{msg}"),
            MuffinError::Internal(msg) => write!(f, "{msg}"),
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

impl MuffinError {
    /// Build a structured diagnostic for rendering in the CLI.
    pub fn diagnostic(&self) -> Diagnostic {
        match self {
            MuffinError::Message(msg) => Diagnostic {
                code: "E0001",
                message: msg.clone(),
                help: Vec::new(),
            },
            MuffinError::ValidationFailed(msg) => Diagnostic {
                code: "E0100",
                message: msg.clone(),
                help: Vec::new(),
            },
            MuffinError::ExecutionFailed(msg) => Diagnostic {
                code: "E0200",
                message: msg.clone(),
                help: Vec::new(),
            },
            MuffinError::InvalidCommand { message, help } => Diagnostic {
                code: "E0300",
                message: message.clone(),
                help: help.clone(),
            },
            MuffinError::NotFound(msg) => Diagnostic {
                code: "E0404",
                message: msg.clone(),
                help: Vec::new(),
            },
            MuffinError::Internal(msg) => Diagnostic {
                code: "E0500",
                message: msg.clone(),
                help: Vec::new(),
            },
            MuffinError::Io(err) => Diagnostic {
                code: "E0002",
                message: format!("io error: {err}"),
                help: Vec::new(),
            },
        }
    }

    /// Exit code mapping for CLI consumers.
    pub fn exit_code(&self) -> u8 {
        match self {
            MuffinError::InvalidCommand { .. } | MuffinError::ValidationFailed(_) => 2,
            MuffinError::ExecutionFailed(_) => 3,
            MuffinError::Io(_) | MuffinError::NotFound(_) => 4,
            MuffinError::Internal(_) => 5,
            MuffinError::Message(_) => 1,
        }
    }

    /// Render a CLI-friendly error string with codes and optional help.
    pub fn render_cli(&self, prefix: &str) -> String {
        let diag = self.diagnostic();
        let mut s = String::new();
        s.push_str(prefix);
        s.push_str(": error[");
        s.push_str(diag.code);
        s.push_str("]: ");
        s.push_str(&diag.message);
        if !diag.help.is_empty() {
            for h in &diag.help {
                s.push_str("\nhelp: ");
                s.push_str(h);
            }
        }
        s
    }
}

/// Convenience Result alias for MuffinLib consumers.
pub type Result<T> = std::result::Result<T, MuffinError>;
