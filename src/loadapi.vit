# loadapi.vit — Muffin (Vitte)
#
# Public "load API" consumed by:
# - CLI "build muffin" (validate + emit .mcf)
# - Steel "build steel" (validate + plan + exec)
# - IDE tooling (read config, list targets, print graph)
#
# This is a thin façade over muffin/load.vit with stable signatures.
# -----------------------------------------------------------------------------
# Blocks: .end only
# -----------------------------------------------------------------------------

mod muffin/loadapi

use std/string
use std/result

use muffin/interface
use muffin/load
use muffin/config
use muffin/debug

export all

# -----------------------------------------------------------------------------
# Errors
# -----------------------------------------------------------------------------

enum ApiErrKind
  Load
  Invalid
.end

struct ApiError
  kind: ApiErrKind
  message: str
.end

type ApiRes[T] = result::Result[T, ApiError]

fn api_err(kind: ApiErrKind, msg: str) -> ApiError
  ret ApiError(kind: kind, message: msg)
.end

fn map_load_err(e: load::LoadError) -> ApiError
  ret api_err(ApiErrKind::Load, load_err_to_string(e))
.end

fn load_err_to_string(e: load::LoadError) -> str
  ret load_kind_to_str(e.kind) + ": " + e.message + " [" + e.path + "]"
.end

fn load_kind_to_str(k: load::LoadErrKind) -> str
  if k == load::LoadErrKind::WorkspaceNotFound ret "workspace" .end
  if k == load::LoadErrKind::ManifestNotFound ret "manifest" .end
  if k == load::LoadErrKind::Io ret "io" .end
  if k == load::LoadErrKind::Parse ret "parse" .end
  if k == load::LoadErrKind::Lower ret "lower" .end
  if k == load::LoadErrKind::Resolve ret "resolve" .end
  if k == load::LoadErrKind::Emit ret "emit" .end
  ret "load"
.end

# -----------------------------------------------------------------------------
# API options
# -----------------------------------------------------------------------------

struct LoadApiOptions
  start_dir: str
  manifest: str          # optional explicit manifest path
  profile: str           # optional override selection.profile
  target: str            # optional override selection.target
  toolchain: str         # optional override selection.toolchain

  emit_mcf: bool
  mcf_path: str

  validate_only: bool
  verbose: bool
.end

fn loadapi_options_default() -> LoadApiOptions
  ret LoadApiOptions(
    start_dir: ".",
    manifest: "",
    profile: "",
    target: "",
    toolchain: "",
    emit_mcf: false,
    mcf_path: "",
    validate_only: false,
    verbose: false
  )
.end

# -----------------------------------------------------------------------------
# Public entrypoints
# -----------------------------------------------------------------------------

struct LoadApiResult
  root: str
  manifest_path: str
  resolved: Resolved
  mcf_path: str
.end

fn load_workspace(rt: interface::Runtime, opt: LoadApiOptions) -> ApiRes[LoadApiResult]
  let mut lo: load::LoadOptions = load::load_options_default()

  lo.start_dir = opt.start_dir
  lo.explicit_manifest = opt.manifest
  lo.emit_mcf = opt.emit_mcf
  lo.mcf_path = opt.mcf_path
  lo.validate_only = opt.validate_only
  lo.verbose = opt.verbose

  let rr: load::LoadRes[load::Loaded] = load::load_workspace(rt, lo)
  if result::is_err(rr)
    ret result::Err(map_load_err(result::unwrap_err(rr)))
  .end

  let mut ld: load::Loaded = result::unwrap(rr)

  # Apply selection overrides (API-level), without rerunning implicit:
  # Intended for Steel to set target/profile at runtime.
  if opt.profile != "" ld.resolved.selection.profile = opt.profile .end
  if opt.target != "" ld.resolved.selection.target = opt.target .end
  if opt.toolchain != "" ld.resolved.selection.toolchain = opt.toolchain .end

  ret result::Ok(LoadApiResult(
    root: ld.root,
    manifest_path: ld.manifest_path,
    resolved: ld.resolved,
    mcf_path: ld.mcf_emitted
  ))
.end

# Build Muffin config only (CLI "build muffin")
fn validate_and_emit(rt: interface::Runtime, start_dir: str) -> ApiRes[str]
  let mut opt: LoadApiOptions = loadapi_options_default()
  opt.start_dir = start_dir
  opt.emit_mcf = true
  opt.validate_only = true

  let rr: ApiRes[LoadApiResult] = load_workspace(rt, opt)
  if result::is_err(rr) ret rr as str .end
  ret result::Ok(result::unwrap(rr).mcf_path)
.end

# List targets (for CLI "build muffin --list")
fn list_targets(rt: interface::Runtime, start_dir: str) -> ApiRes[list[str]]
  let mut opt: LoadApiOptions = loadapi_options_default()
  opt.start_dir = start_dir
  opt.validate_only = true

  let rr: ApiRes[LoadApiResult] = load_workspace(rt, opt)
  if result::is_err(rr) ret rr as list[str] .end

  let r: Resolved = result::unwrap(rr).resolved
  let mut out: list[str] = []
  let mut i: i32 = 0
  while i < len(r.targets)
    out = out + [r.targets[i].name]
    i = i + 1
  .end
  out = sort_str(out)
  ret result::Ok(out)
.end

# -----------------------------------------------------------------------------
# Utilities
# -----------------------------------------------------------------------------

fn sort_str(xs: list[str]) -> list[str]
  let mut a: list[str] = xs
  let mut i: i32 = 0
  while i < len(a)
    let mut j: i32 = i + 1
    while j < len(a)
      if a[j] < a[i]
        let t: str = a[i]
        a[i] = a[j]
        a[j] = t
      .end
      j = j + 1
    .end
    i = i + 1
  .end
  ret a
.end

# -----------------------------------------------------------------------------
# Externs
# -----------------------------------------------------------------------------

extern fn len[T](xs: list[T]) -> i32