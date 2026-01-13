//! GCC / Clang driver integration for Flan.
//!
//! Responsabilités :
//! - détection du compilateur C (gcc / clang)
//! - construction des arguments (compile / link)
//! - exécution du pipeline C via Flan

pub mod detect;
pub mod args;
pub mod driver;

// Re-exports publics (API stable pour Flan)

pub use detect::{
    detect_cc,
    CcKind,
    CcTool,
    DetectError,
};

pub use args::{
    GccArgs,
    GccMode,
    CStd,
};

pub use driver::{
    GccDriver,
    CBuildConfig,
    CompileUnit,
    LinkUnit,
    DriverError,
};
