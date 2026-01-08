//! Capsule — policy sandbox cross-platform.
//!
//! A **capsule** is a hermetic execution policy used by Muffin to run tools
//! deterministically across OS/arch.
//!
//! Responsibilities:
//! - Express a sandbox policy (env/fs/net/time) in a portable representation.
//! - Provide a uniform interface to apply the policy on the current platform.
//! - Offer helpers to validate, normalize, and render policies (debug / decompile).
//!
//! Design goals:
//! - **std-only** and minimal dependencies.
//! - **Portable**: policy model is OS-agnostic; backends implement enforcement.
//! - **Composable**: allow layering defaults + overrides.
//!
//! Notes:
//! - Policy enforcement may be best-effort depending on OS capabilities.
//! - The policy model is stable; platform modules are behind `cfg` gates.

pub mod policy;
pub mod sandbox;

pub mod platform;

pub use policy::{
    CapsuleDecl, CapsulePolicy, EnvPolicy, FsPolicy, NetPolicy, TimePolicy,
};

pub use sandbox::{
    Sandbox, SandboxBackend, SandboxError, SandboxResult,
};

/// Current platform kind used by capsule backends.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum PlatformKind {
    Linux,
    Macos,
    Windows,
    Bsd,
    Solaris,
    Other,
}

impl PlatformKind {
    pub fn detect() -> Self {
        #[cfg(target_os = "linux")]
        return PlatformKind::Linux;

        #[cfg(target_os = "macos")]
        return PlatformKind::Macos;

        #[cfg(target_os = "windows")]
        return PlatformKind::Windows;

        #[cfg(any(
            target_os = "freebsd",
            target_os = "openbsd",
            target_os = "netbsd",
            target_os = "dragonfly"
        ))]
        return PlatformKind::Bsd;

        // There is no official Rust target_os="solaris" stable in all toolchains,
        // but keep the enum variant for completeness.
        #[cfg(any(target_os = "illumos"))]
        return PlatformKind::Solaris;

        PlatformKind::Other
    }

    pub fn as_str(self) -> &'static str {
        match self {
            PlatformKind::Linux => "linux",
            PlatformKind::Macos => "macos",
            PlatformKind::Windows => "windows",
            PlatformKind::Bsd => "bsd",
            PlatformKind::Solaris => "solaris",
            PlatformKind::Other => "other",
        }
    }
}

/// A resolved capsule instance ready for execution.
///
/// This is the minimal contract between configuration (decl) and execution (sandbox).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Capsule {
    pub name: String,
    pub policy: CapsulePolicy,
}

impl Capsule {
    pub fn new(name: impl Into<String>, policy: CapsulePolicy) -> Self {
        Self { name: name.into(), policy }
    }

    /// Returns true if the policy aims for hermetic behavior.
    pub fn is_hermetic(&self) -> bool {
        // conservative heuristic; exact semantics live in policy.rs
        self.policy.net.is_deny() && self.policy.time.is_stable()
    }
}

/// Merge `base` with `overlay` (overlay wins).
///
/// Use this to apply defaults/profiles/overrides.
pub fn merge_policy(base: &CapsulePolicy, overlay: &CapsulePolicy) -> CapsulePolicy {
    let mut out = base.clone();
    out.merge_from(overlay);
    out
}

/// Validate a policy (syntax/shape constraints).
///
/// Returns a list of human readable issues (empty means ok).
///
/// `policy.rs` owns the strict rules; this is a convenience wrapper.
pub fn validate_policy(policy: &CapsulePolicy) -> Vec<String> {
    policy.validate()
}

/// Create a sandbox backend for the current platform.
///
/// The backend choice may depend on policy needs and OS capabilities.
pub fn default_backend() -> Box<dyn SandboxBackend> {
    platform::default_backend()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn platform_detect_smoke() {
        let p = PlatformKind::detect();
        assert!(!p.as_str().is_empty());
    }

    #[test]
    fn capsule_merge_smoke() {
        // Minimal policy constructors are expected in policy.rs.
        // If policy.rs implements Default, this will compile.
        let base = CapsulePolicy::default();
        let mut overlay = CapsulePolicy::default();
        // Force at least one difference if APIs exist.
        overlay.net = NetPolicy::deny();

        let merged = merge_policy(&base, &overlay);
        assert_eq!(merged.net, NetPolicy::deny());
    }
}
