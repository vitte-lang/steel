use std::fmt;

#[derive(Debug)]
pub enum MuffinError {
    ConfigNotFound,
    ValidationFailed(String),
    CompilationFailed(String),
    IoError(std::io::Error),
}

impl fmt::Display for MuffinError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            MuffinError::ConfigNotFound => write!(f, "Muffinfile not found"),
            MuffinError::ValidationFailed(msg) => write!(f, "Validation failed: {}", msg),
            MuffinError::CompilationFailed(msg) => write!(f, "Compilation failed: {}", msg),
            MuffinError::IoError(e) => write!(f, "I/O error: {}", e),
        }
    }
}

impl From<std::io::Error> for MuffinError {
    fn from(err: std::io::Error) -> Self {
        MuffinError::IoError(err)
    }
}

impl std::error::Error for MuffinError {}
