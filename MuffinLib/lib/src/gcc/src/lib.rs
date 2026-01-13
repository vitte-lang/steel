//! GCC / Clang driver integration for Muffin.
//!
//! Responsabilités :
//! - détection du compilateur C (gcc / clang)
//! - construction des arguments (compile / link)
//! - exécution du pipeline C via Muffin

pub mod detect;
pub mod args;
pub mod driver;

// Re-exports publics (API stable pour Muffin)

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
