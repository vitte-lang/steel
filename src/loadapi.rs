// src/loadapi.rs
//
// Muffin — load API (dynamic provider layer)
//
// Purpose:
// - Centralize "loading" of external configuration / data sources into Muffin runtime:
//   - Muffinfile / build.muf parsing results (AST/IR) -> internal models
//   - plugin/registry metadata -> internal structures
//   - workspace config (global + local) -> merged view
//   - environment overlays
//
// This module defines:
// - LoadApi: a facade used by commands (build, check, graph, etc.)
// - Providers: trait-based sources that can be composed
// - LoadContext: shared inputs (cwd, root, profiles, target)
// - Merge rules with deterministic precedence
// - Diagnostics-friendly error model
//
// No external deps.
//
// Notes:
// - The actual parsing of Muffinfile is not implemented here; plug in your parser.
// - For "max", a small in-memory provider and a file provider stub are included.

#![allow(dead_code)]

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

/* ============================== diagnostics/errors ============================== */

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LoadError {
    Io {
        path: PathBuf,
        op: &'static str,
        message: String,
    },
    Parse {
        path: PathBuf,
        message: String,
    },
    Invalid {
        message: String,
    },
    Conflict {
        key: String,
        message: String,
    },
    NotFound {
        what: String,
    },
    Other {
        message: String,
    },
}

impl fmt::Display for LoadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LoadError::Io { path, op, message } => write!(f, "{} {}: {}", op, path.display(), message),
            LoadError::Parse { path, message } => write!(f, "parse {}: {}", path.display(), message),
            LoadError::Invalid { message } => write!(f, "invalid: {message}"),
            LoadError::Conflict { key, message } => write!(f, "conflict {key}: {message}"),
            LoadError::NotFound { what } => write!(f, "not found: {what}"),
            LoadError::Other { message } => write!(f, "{message}"),
        }
    }
}

impl std::error::Error for LoadError {}

fn io_err(path: &Path, op: &'static str, e: std::io::Error) -> LoadError {
    LoadError::Io {
        path: path.to_path_buf(),
        op,
        message: e.to_string(),
    }
}

/* ============================== core models loaded ============================== */

/// High-level workspace model used by commands.
#[derive(Debug, Clone)]
pub struct WorkspaceModel {
    pub root: PathBuf,
    pub muffinfile: Option<PathBuf>,
    pub profiles: BTreeMap<String, ProfileModel>,
    pub targets: BTreeMap<String, TargetModel>,
    pub vars: BTreeMap<String, String>,
    pub tools: BTreeMap<String, ToolModel>,
    pub rules: BTreeMap<String, RuleModel>,
    pub created_at: SystemTime,
}

impl WorkspaceModel {
    pub fn new(root: PathBuf) -> Self {
        Self {
            root,
            muffinfile: None,
            profiles: BTreeMap::new(),
            targets: BTreeMap::new(),
            vars: BTreeMap::new(),
            tools: BTreeMap::new(),
            rules: BTreeMap::new(),
            created_at: SystemTime::now(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ProfileModel {
    pub name: String,
    pub vars: BTreeMap<String, String>,
    pub flags: BTreeSet<String>,
}

#[derive(Debug, Clone)]
pub struct TargetModel {
    pub name: String,
    pub triple: Option<String>,
    pub vars: BTreeMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct ToolModel {
    pub name: String,
    pub program: String,
    pub args: Vec<String>,
    pub env: BTreeMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct RuleModel {
    pub name: String,
    pub phony: bool,
    pub inputs: Vec<PathBuf>,
    pub outputs: Vec<PathBuf>,
    pub deps: Vec<String>,
    pub tool: Option<String>,
    pub argv: Vec<String>,
    pub env: BTreeMap<String, String>,
    pub tags: BTreeSet<String>,
    pub meta: BTreeMap<String, String>,
}

/* ============================== load context ============================== */

#[derive(Debug, Clone)]
pub struct LoadContext {
    pub cwd: PathBuf,
    pub root_hint: Option<PathBuf>,
    pub muffinfile_hint: Option<PathBuf>,

    pub profile: Option<String>,
    pub target: Option<String>,

    /// Strictness knobs
    pub allow_missing_muffinfile: bool,
}

impl LoadContext {
    pub fn new(cwd: PathBuf) -> Self {
        Self {
            cwd,
            root_hint: None,
            muffinfile_hint: None,
            profile: None,
            target: None,
            allow_missing_muffinfile: false,
        }
    }
}

/* ============================== provider API ============================== */

pub trait LoadProvider: Send + Sync {
    fn name(&self) -> &'static str;

    /// Provide partial workspace fragments. Merge order defines precedence.
    fn load(&self, ctx: &LoadContext) -> Result<WorkspaceFragment, LoadError>;
}

#[derive(Debug, Clone, Default)]
pub struct WorkspaceFragment {
    pub muffinfile: Option<PathBuf>,
    pub profiles: BTreeMap<String, ProfileModel>,
    pub targets: BTreeMap<String, TargetModel>,
    pub vars: BTreeMap<String, String>,
    pub tools: BTreeMap<String, ToolModel>,
    pub rules: BTreeMap<String, RuleModel>,
}

/* ============================== merge semantics ============================== */

#[derive(Debug, Clone)]
pub struct MergePolicy {
    /// If true, later fragments override earlier ones on key collisions.
    pub last_wins: bool,
    /// If false, collisions produce LoadError::Conflict.
    pub allow_overrides: bool,
}

impl Default for MergePolicy {
    fn default() -> Self {
        Self {
            last_wins: true,
            allow_overrides: true,
        }
    }
}

fn merge_maps<T: Clone>(
    into: &mut BTreeMap<String, T>,
    other: BTreeMap<String, T>,
    pol: &MergePolicy,
    scope: &str,
) -> Result<(), LoadError> {
    for (k, v) in other {
        if into.contains_key(&k) {
            if !pol.allow_overrides {
                return Err(LoadError::Conflict {
                    key: format!("{scope}.{k}"),
                    message: "duplicate key".to_string(),
                });
            }
            if pol.last_wins {
                into.insert(k, v);
            }
        } else {
            into.insert(k, v);
        }
    }
    Ok(())
}

fn merge_kv(
    into: &mut BTreeMap<String, String>,
    other: BTreeMap<String, String>,
    pol: &MergePolicy,
    scope: &str,
) -> Result<(), LoadError> {
    merge_maps(into, other, pol, scope)
}

/* ============================== facade ============================== */

pub struct LoadApi {
    providers: Vec<Box<dyn LoadProvider>>,
    merge: MergePolicy,
}

impl LoadApi {
    pub fn new() -> Self {
        Self {
            providers: Vec::new(),
            merge: MergePolicy::default(),
        }
    }

    pub fn with_merge_policy(mut self, pol: MergePolicy) -> Self {
        self.merge = pol;
        self
    }

    pub fn register(mut self, p: Box<dyn LoadProvider>) -> Self {
        self.providers.push(p);
        self.providers.sort_by_key(|x| x.name());
        self
    }

    pub fn load_workspace(&self, ctx: &LoadContext) -> Result<WorkspaceModel, LoadError> {
        let root = resolve_root(ctx)?;
        let mut ws = WorkspaceModel::new(root);

        for p in &self.providers {
            let frag = p.load(ctx)?;
            if ws.muffinfile.is_none() {
                ws.muffinfile = frag.muffinfile.clone();
            }

            merge_maps(&mut ws.profiles, frag.profiles, &self.merge, "profiles")?;
            merge_maps(&mut ws.targets, frag.targets, &self.merge, "targets")?;
            merge_kv(&mut ws.vars, frag.vars, &self.merge, "vars")?;
            merge_maps(&mut ws.tools, frag.tools, &self.merge, "tools")?;
            merge_maps(&mut ws.rules, frag.rules, &self.merge, "rules")?;
        }

        // Apply profile/target overlays if present
        apply_overlays(&mut ws, ctx)?;

        Ok(ws)
    }
}

fn resolve_root(ctx: &LoadContext) -> Result<PathBuf, LoadError> {
    if let Some(r) = &ctx.root_hint {
        return Ok(r.clone());
    }
    // fallback: cwd as root
    Ok(ctx.cwd.clone())
}

/* ============================== overlays ============================== */

fn apply_overlays(ws: &mut WorkspaceModel, ctx: &LoadContext) -> Result<(), LoadError> {
    if let Some(p) = &ctx.profile {
        let prof = ws.profiles.get(p).ok_or_else(|| LoadError::NotFound {
            what: format!("profile '{p}'"),
        })?;
        // merge vars last-wins
        for (k, v) in &prof.vars {
            ws.vars.insert(k.clone(), v.clone());
        }
    }

    if let Some(t) = &ctx.target {
        let tgt = ws.targets.get(t).ok_or_else(|| LoadError::NotFound {
            what: format!("target '{t}'"),
        })?;
        for (k, v) in &tgt.vars {
            ws.vars.insert(k.clone(), v.clone());
        }
        // expose triple as var
        if let Some(triple) = &tgt.triple {
            ws.vars.insert("target.triple".to_string(), triple.clone());
        }
    }

    Ok(())
}

/* ============================== providers ============================== */

/// Provider: in-memory (tests, embedding).
pub struct MemoryProvider {
    frag: WorkspaceFragment,
}

impl MemoryProvider {
    pub fn new(frag: WorkspaceFragment) -> Self {
        Self { frag }
    }
}

impl LoadProvider for MemoryProvider {
    fn name(&self) -> &'static str {
        "memory"
    }

    fn load(&self, _ctx: &LoadContext) -> Result<WorkspaceFragment, LoadError> {
        Ok(self.frag.clone())
    }
}

/// Provider: environment variables overlay.
pub struct EnvProvider {
    pub prefix: String,
}

impl EnvProvider {
    pub fn new(prefix: &str) -> Self {
        Self {
            prefix: prefix.to_string(),
        }
    }
}

impl LoadProvider for EnvProvider {
    fn name(&self) -> &'static str {
        "env"
    }

    fn load(&self, _ctx: &LoadContext) -> Result<WorkspaceFragment, LoadError> {
        let mut frag = WorkspaceFragment::default();
        for (k, v) in std::env::vars() {
            if let Some(key) = k.strip_prefix(&self.prefix) {
                // Example: MUFFIN_VAR_FOO=bar -> vars["FOO"]="bar"
                frag.vars.insert(key.to_string(), v);
            }
        }
        Ok(frag)
    }
}

/// Provider: file loader stub (Muffinfile/build.muf).
/// This expects a single file path and returns a Parse error until you hook a parser.
pub struct MuffinfileProvider {
    pub path: PathBuf,
}

impl MuffinfileProvider {
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }
}

impl LoadProvider for MuffinfileProvider {
    fn name(&self) -> &'static str {
        "muffinfile"
    }

    fn load(&self, ctx: &LoadContext) -> Result<WorkspaceFragment, LoadError> {
        let path = if let Some(p) = &ctx.muffinfile_hint {
            p.clone()
        } else {
            self.path.clone()
        };

        if !path.exists() {
            if ctx.allow_missing_muffinfile {
                return Ok(WorkspaceFragment {
                    muffinfile: None,
                    ..Default::default()
                });
            }
            return Err(LoadError::NotFound {
                what: format!("muffinfile {}", path.display()),
            });
        }

        let _bytes = std::fs::read(&path).map_err(|e| io_err(&path, "read", e))?;

        // Hook your real parser here:
        // let ast = crate::parser::parse_muffinfile(&bytes)?;
        // let frag = crate::lowering::lower(ast)?;
        // Ok(frag)

        Err(LoadError::Parse {
            path,
            message: "parser not wired (stub)".to_string(),
        })
    }
}

/* ============================== tests ============================== */

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn merges_providers_last_wins() {
        let mut frag1 = WorkspaceFragment::default();
        frag1.vars.insert("A".to_string(), "1".to_string());

        let mut frag2 = WorkspaceFragment::default();
        frag2.vars.insert("A".to_string(), "2".to_string());

        let api = LoadApi::new()
            .with_merge_policy(MergePolicy {
                last_wins: true,
                allow_overrides: true,
            })
            .register(Box::new(MemoryProvider::new(frag1)))
            .register(Box::new(MemoryProvider::new(frag2)));

        let ctx = LoadContext::new(PathBuf::from("."));
        let ws = api.load_workspace(&ctx).unwrap();
        assert_eq!(ws.vars.get("A").map(|s| s.as_str()), Some("2"));
    }

    #[test]
    fn applies_profile_overlay() {
        let mut frag = WorkspaceFragment::default();
        frag.profiles.insert(
            "debug".to_string(),
            ProfileModel {
                name: "debug".to_string(),
                vars: {
                    let mut m = BTreeMap::new();
                    m.insert("OPT".to_string(), "0".to_string());
                    m
                },
                flags: BTreeSet::new(),
            },
        );

        let api = LoadApi::new().register(Box::new(MemoryProvider::new(frag)));
        let mut ctx = LoadContext::new(PathBuf::from("."));
        ctx.profile = Some("debug".to_string());

        let ws = api.load_workspace(&ctx).unwrap();
        assert_eq!(ws.vars.get("OPT").map(|s| s.as_str()), Some("0"));
    }
}
