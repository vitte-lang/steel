//! Muffin: Declarative configuration layer for Vitte build system
//!
//! Muffin parses, validates, and resolves a workspace configuration (packages, profiles,
//! toolchains, targets), then generates a stable configuration artifact `Muffinconfig.mcfg`.
//!
//! # Architecture
//!
//! The architecture follows a "Freeze then Build" principle:
//! - **Phase 1**: Configuration (validation + resolution)
//! - **Phase 2**: Construction (DAG execution via Steel)
//!
//! # Modules
//!
//! - `parser` — Lexical and syntactic analysis of Muffin files
//! - `validator` — Coherence checking and constraint validation
//! - `resolver` — Profile inheritance, variable interpolation, dependency resolution
//! - `generator` — Serialization to Muffinconfig.mcfg and exports
//! - `model` — Core data structures (Workspace, Package, Profile, Target, Toolchain)
//! - `interface` — Runtime abstraction and CLI interface
//! - `commands` — CLI command implementations

// ============================================================================
// PARSER MODULE
// ============================================================================

/// Lexical and syntactic analysis of Muffin files
pub mod parser {
    pub use crate::arscan;      // Lexer/tokenizer
    pub use crate::read;        // Block-oriented parser
}

// ============================================================================
// VALIDATOR MODULE
// ============================================================================

/// Validation: coherence checking, references resolution, constraint validation
pub mod validator {
    pub use crate::config;           // Configuration coherence
    pub use crate::dependancies;     // Dependency graph validation
    pub use crate::target_file;      // Target specification validation
}

// ============================================================================
// RESOLVER MODULE
// ============================================================================

/// Resolution: profile inheritance, variable expansion, implicit rule resolution
pub mod resolver {
    pub use crate::variable;         // Variable interpolation and scope
    pub use crate::expand;           // Macro and variable expansion
    pub use crate::implicit;         // Implicit rule resolution
    pub use crate::default;          // Default value application
}

// ============================================================================
// GENERATOR MODULE
// ============================================================================

/// Generation: Muffinconfig.mcfg serialization and export utilities
pub mod generator {
    pub use crate::interface;        // Runtime interface (abstract I/O)
    pub use crate::output;           // Output formatting and serialization
}

// ============================================================================
// MODEL MODULE
// ============================================================================

/// Core data structures: Workspace, Package, Profile, Target, Toolchain
pub mod model {
    pub use crate::muffinint;        // Muffin internal API
    pub use crate::def_target_file;  // Target file definitions
    pub use crate::rule;             // Rule model
}

// ============================================================================
// RUNTIME & INTERFACE MODULE
// ============================================================================

/// Runtime abstractions: filesystem, process, logging
pub mod runtime {
    pub use crate::os;               // OS-specific implementations
    pub use crate::posixos;          // POSIX compliance layer
    pub use crate::job;              // Job/process management
    pub use crate::debug;            // Debug and logging utilities
}

// ============================================================================
// CLI & COMMANDS MODULE
// ============================================================================

/// Command-line interface and command implementations
pub mod cli {
    pub use crate::commands;         // Command implementations
    pub use crate::interface;        // CLI interface
}

// ============================================================================
// UTILITIES MODULE
// ============================================================================

/// Utility functions and helpers
pub mod utils {
    pub use crate::misc;             // Miscellaneous utilities
    pub use crate::hash;             // Hash utilities
    pub use crate::strcache;         // String cache
    pub use crate::directory;        // Directory utilities
    pub use crate::vpath;            // Virtual path resolution
}

// ============================================================================
// VERSION MODULE
// ============================================================================

/// Version and metadata
pub mod metadata {
    pub use crate::version;          // Version information
    pub use crate::warning;          // Warning utilities
}

// ============================================================================
// LEGACY/PLATFORM-SPECIFIC MODULES
// ============================================================================

/// Platform-specific implementations (VMS, remote execution, etc.)
pub mod platform {
    pub use crate::vms_exit;
    pub use crate::vms_export_symbol;
    pub use crate::vms_progname;
    pub use crate::vmsdir;
    pub use crate::vmsfunctions;
    pub use crate::vmsify;
    pub use crate::vmsjobs;
    pub use crate::posixos;
    pub use crate::remote_cstms;
    pub use crate::remote_stub;
}

// ============================================================================
// RE-EXPORTS FOR CONVENIENCE
// ============================================================================

pub use crate::loadapi::*;     // Main public API (parse, resolve, emit)
pub use crate::build_muf::*;   // Build muffin command

pub mod config;
pub mod compiler;
pub mod validator;

pub use config::Config;
pub use compiler::Compiler;
pub use validator::Validator;
