use std::fmt;

#[derive(Debug)]
pub enum FlanError {
    ConfigNotFound,
    ValidationFailed(String),
    CompilationFailed(String),
    IoError(std::io::Error),
}

impl fmt::Display for FlanError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            FlanError::ConfigNotFound => write!(f, "FlanConfig not found"),
            FlanError::ValidationFailed(msg) => write!(f, "Validation failed: {}", msg),
            FlanError::CompilationFailed(msg) => write!(f, "Compilation failed: {}", msg),
            FlanError::IoError(e) => write!(f, "I/O error: {}", e),
        }
    }
}

impl From<std::io::Error> for FlanError {
    fn from(err: std::io::Error) -> Self {
        FlanError::IoError(err)
    }
}

impl std::error::Error for FlanError {}
