// src/load.rs
//
// Muffin — load (workspace discovery + Muffinfile/build.muf ingestion)
//
// Purpose:
// - Locate a workspace root and Muffinfile/build.muf
// - Load + merge configuration layers into a single in-memory Workspace
// - Provide deterministic precedence + clear diagnostics
//
// This module focuses on:
// - discovery: walk up from cwd to find Muffinfile/build.muf (or use explicit path)
// - reading: uses a small read helper (inline) to read UTF-8 (BOM safe)
// - parsing: stub interface; plug your real parser/lowering pipeline
// - merging: overlays (profile/target/env) applied deterministically
//
// Notes:
// - No external deps.
// - Replace "stub parser" with your actual Muffin AST/IR builder.
// - Designed to be used by CLI commands like `build muffin`, `check`, etc.
//
// Typical usage:
//   let ctx = LoadCtx::from_cwd(".")?.with_profile("debug");
//   let ws = WorkspaceLoader::new().load(&ctx)?;
//
// Integration points:
// - crate::read (if you have it) can replace the inline read helpers.
// - crate::loadapi could wrap this module; here we implement an all-in-one loader.

#![allow(dead_code)]

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

/* ============================== errors/diagnostics ============================== */

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LoadError {
    Io {
        path: PathBuf,
        op: &'static str,
        message: String,
    },
    NotFound {
        what: String,
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
}

impl fmt::Display for LoadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LoadError::Io { path, op, message } => write!(f, "{} {}: {}", op, path.display(), message),
            LoadError::NotFound { what } => write!(f, "not found: {what}"),
            LoadError::Parse { path, message } => write!(f, "parse {}: {}", path.display(), message),
            LoadError::Invalid { message } => write!(f, "invalid: {message}"),
            LoadError::Conflict { key, message } => write!(f, "conflict {key}: {message}"),
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

/* ============================== context ============================== */

#[derive(Debug, Clone)]
pub struct LoadCtx {
    pub cwd: PathBuf,

    /// Optional explicit workspace root.
    pub root_hint: Option<PathBuf>,

    /// Optional explicit muffinfile path.
    pub muffinfile_hint: Option<PathBuf>,

    /// Active profile & target (optional).
    pub profile: Option<String>,
    pub target: Option<String>,

    /// env var prefix for overrides (ex: MUFFIN_)
    pub env_prefix: String,

    /// if true, missing Muffinfile doesn't error (returns empty workspace)
    pub allow_missing: bool,
}

impl LoadCtx {
    pub fn from_cwd(cwd: impl Into<PathBuf>) -> Result<Self, LoadError> {
        Ok(Self {
            cwd: cwd.into(),
            root_hint: None,
            muffinfile_hint: None,
            profile: None,
            target: None,
            env_prefix: "MUFFIN_".to_string(),
            allow_missing: false,
        })
    }

    pub fn with_root(mut self, root: impl Into<PathBuf>) -> Self {
        self.root_hint = Some(root.into());
        self
    }

    pub fn with_muffinfile(mut self, path: impl Into<PathBuf>) -> Self {
        self.muffinfile_hint = Some(path.into());
        self
    }

    pub fn with_profile(mut self, name: impl Into<String>) -> Self {
        self.profile = Some(name.into());
        self
    }

    pub fn with_target(mut self, name: impl Into<String>) -> Self {
        self.target = Some(name.into());
        self
    }

    pub fn with_env_prefix(mut self, p: impl Into<String>) -> Self {
        self.env_prefix = p.into();
        self
    }

    pub fn allow_missing(mut self, yes: bool) -> Self {
        self.allow_missing = yes;
        self
    }
}

/* ============================== workspace model ============================== */

#[derive(Debug, Clone)]
pub struct Workspace {
    pub root: PathBuf,
    pub muffinfile: Option<PathBuf>,

    pub vars: BTreeMap<String, String>,
    pub profiles: BTreeMap<String, Profile>,
    pub targets: BTreeMap<String, Target>,
    pub tools: BTreeMap<String, Tool>,
    pub rules: BTreeMap<String, Rule>,

    pub created_at: SystemTime,
}

impl Workspace {
    pub fn new(root: PathBuf) -> Self {
        Self {
            root,
            muffinfile: None,
            vars: BTreeMap::new(),
            profiles: BTreeMap::new(),
            targets: BTreeMap::new(),
            tools: BTreeMap::new(),
            rules: BTreeMap::new(),
            created_at: SystemTime::now(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Profile {
    pub name: String,
    pub vars: BTreeMap<String, String>,
    pub flags: BTreeSet<String>,
}

#[derive(Debug, Clone)]
pub struct Target {
    pub name: String,
    pub triple: Option<String>,
    pub vars: BTreeMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct Tool {
    pub name: String,
    pub program: String,
    pub args: Vec<String>,
    pub env: BTreeMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct Rule {
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

/* ============================== loader ============================== */

#[derive(Debug, Clone)]
pub struct WorkspaceLoader {
    pub search_filenames: Vec<&'static str>,
    pub allow_overrides: bool,
    pub last_wins: bool,
}

impl Default for WorkspaceLoader {
    fn default() -> Self {
        Self {
            search_filenames: vec!["Muffinfile", "build.muf"],
            allow_overrides: true,
            last_wins: true,
        }
    }
}

impl WorkspaceLoader {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn load(&self, ctx: &LoadCtx) -> Result<Workspace, LoadError> {
        let root = self.resolve_root(ctx)?;
        let mut ws = Workspace::new(root.clone());

        let muffinfile = self.resolve_muffinfile(ctx, &root)?;
        ws.muffinfile = muffinfile.clone();

        // Base layer: from Muffinfile/build.muf (if any)
        if let Some(path) = &muffinfile {
            let text = read_text_utf8(path)?;
            let frag = parse_muffinfile_stub(path, &text)?; // replace with real parser
            self.merge_fragment(&mut ws, frag, "file")?;
        } else if !ctx.allow_missing {
            return Err(LoadError::NotFound {
                what: format!(
                    "workspace muffinfile (searched: {})",
                    self.search_filenames.join(", ")
                ),
            });
        }

        // Env overlay: MUFFIN_VAR_X=... , MUFFIN_PROFILE=... etc.
        let env_frag = load_env_overlay(&ctx.env_prefix);
        self.merge_fragment(&mut ws, env_frag, "env")?;

        // Apply profile/target overlays
        apply_overlays(&mut ws, ctx)?;

        Ok(ws)
    }

    fn resolve_root(&self, ctx: &LoadCtx) -> Result<PathBuf, LoadError> {
        if let Some(r) = &ctx.root_hint {
            return Ok(r.clone());
        }

        // If muffinfile hint provided, root is its parent (best-effort).
        if let Some(p) = &ctx.muffinfile_hint {
            return Ok(p.parent().unwrap_or_else(|| Path::new(".")).to_path_buf());
        }

        // Otherwise: walk up from cwd to find Muffinfile/build.muf
        let mut cur = ctx.cwd.clone();
        loop {
            for name in &self.search_filenames {
                let candidate = cur.join(name);
                if candidate.exists() {
                    return Ok(cur);
                }
            }

            if !cur.pop() {
                break;
            }
        }

        // fallback: cwd as root
        Ok(ctx.cwd.clone())
    }

    fn resolve_muffinfile(&self, ctx: &LoadCtx, root: &Path) -> Result<Option<PathBuf>, LoadError> {
        if let Some(p) = &ctx.muffinfile_hint {
            return Ok(Some(p.clone()));
        }

        for name in &self.search_filenames {
            let candidate = root.join(name);
            if candidate.exists() {
                return Ok(Some(candidate));
            }
        }

        Ok(None)
    }

    fn merge_fragment(&self, ws: &mut Workspace, frag: WorkspaceFragment, source: &str) -> Result<(), LoadError> {
        if ws.muffinfile.is_none() {
            ws.muffinfile = frag.muffinfile.clone();
        }

        merge_kv(&mut ws.vars, frag.vars, self, &format!("{source}.vars"))?;
        merge_map(&mut ws.profiles, frag.profiles, self, &format!("{source}.profiles"))?;
        merge_map(&mut ws.targets, frag.targets, self, &format!("{source}.targets"))?;
        merge_map(&mut ws.tools, frag.tools, self, &format!("{source}.tools"))?;
        merge_map(&mut ws.rules, frag.rules, self, &format!("{source}.rules"))?;
        Ok(())
    }
}

/* ============================== fragments + merge ============================== */

#[derive(Debug, Clone, Default)]
pub struct WorkspaceFragment {
    pub muffinfile: Option<PathBuf>,
    pub vars: BTreeMap<String, String>,
    pub profiles: BTreeMap<String, Profile>,
    pub targets: BTreeMap<String, Target>,
    pub tools: BTreeMap<String, Tool>,
    pub rules: BTreeMap<String, Rule>,
}

fn merge_kv(
    into: &mut BTreeMap<String, String>,
    other: BTreeMap<String, String>,
    pol: &WorkspaceLoader,
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

fn merge_map<T: Clone>(
    into: &mut BTreeMap<String, T>,
    other: BTreeMap<String, T>,
    pol: &WorkspaceLoader,
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

/* ============================== overlays ============================== */

fn apply_overlays(ws: &mut Workspace, ctx: &LoadCtx) -> Result<(), LoadError> {
    if let Some(p) = &ctx.profile {
        let prof = ws.profiles.get(p).ok_or_else(|| LoadError::NotFound {
            what: format!("profile '{p}'"),
        })?;
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
        if let Some(triple) = &tgt.triple {
            ws.vars.insert("target.triple".to_string(), triple.clone());
        }
    }

    Ok(())
}

/* ============================== env overlay ============================== */

fn load_env_overlay(prefix: &str) -> WorkspaceFragment {
    let mut frag = WorkspaceFragment::default();

    // Convention:
    //   {PREFIX}VAR_FOO=bar -> vars["FOO"]="bar"
    //   {PREFIX}PROFILE=debug -> vars["profile"]="debug" (optional)
    //   {PREFIX}TARGET=x86_64 -> vars["target"]="x86_64" (optional)
    let var_prefix = format!("{prefix}VAR_");

    for (k, v) in std::env::vars() {
        if let Some(key) = k.strip_prefix(&var_prefix) {
            frag.vars.insert(key.to_string(), v);
        } else if k == format!("{prefix}PROFILE") {
            frag.vars.insert("profile".to_string(), v);
        } else if k == format!("{prefix}TARGET") {
            frag.vars.insert("target".to_string(), v);
        }
    }

    frag
}

/* ============================== read helpers (utf8) ============================== */

fn read_text_utf8(path: &Path) -> Result<String, LoadError> {
    let bytes = std::fs::read(path).map_err(|e| io_err(path, "read", e))?;
    let bytes = strip_utf8_bom(&bytes);
    let s = std::str::from_utf8(bytes).map_err(|e| LoadError::Parse {
        path: path.to_path_buf(),
        message: format!("invalid utf-8: {e}"),
    })?;
    Ok(s.to_string())
}

fn strip_utf8_bom(bytes: &[u8]) -> &[u8] {
    const BOM: &[u8] = &[0xEF, 0xBB, 0xBF];
    if bytes.starts_with(BOM) {
        &bytes[BOM.len()..]
    } else {
        bytes
    }
}

/* ============================== parser stub ============================== */

/// Stub parser: returns empty fragment.
/// Replace with your real Muffinfile/build.muf parser + lowering.
///
/// Expected responsibilities:
/// - parse declarations: var/profile/target/tool/rule blocks
/// - expand includes/imports
/// - build a WorkspaceFragment with deterministic maps
fn parse_muffinfile_stub(path: &Path, _text: &str) -> Result<WorkspaceFragment, LoadError> {
    // You can keep this as an "empty workspace" for bring-up.
    // Or return Err(Parse{...}) to force wiring the parser.
    Ok(WorkspaceFragment {
        muffinfile: Some(path.to_path_buf()),
        ..Default::default()
    })
}

/* ============================== tests ============================== */

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn env_overlay_loads_vars() {
        std::env::set_var("MUFFIN_VAR_FOO", "bar");
        let frag = load_env_overlay("MUFFIN_");
        assert_eq!(frag.vars.get("FOO").map(|s| s.as_str()), Some("bar"));
    }

    #[test]
    fn merge_conflict_errors_if_disallowed() {
        let loader = WorkspaceLoader {
            allow_overrides: false,
            last_wins: true,
            ..Default::default()
        };

        let mut ws = Workspace::new(PathBuf::from("."));
        ws.vars.insert("A".to_string(), "1".to_string());

        let mut frag = WorkspaceFragment::default();
        frag.vars.insert("A".to_string(), "2".to_string());

        let err = loader.merge_fragment(&mut ws, frag, "x").unwrap_err();
        assert!(matches!(err, LoadError::Conflict { .. }));
    }
}
