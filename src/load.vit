# load.vit — Muffin (Vitte) — MAX
#
# Loader / workspace discovery / manifest ingestion.
# -----------------------------------------------------------------------------
# Responsibilities:
# - find workspace root from a start directory
# - locate the manifest (Muffinfile / build.muf / mod.muf)
# - read source text
# - parse (delegated to parser module)
# - apply defaults + implicit rules
# - produce Resolved config
# - optionally emit .mcf (Muffinconfig) for "build muffin" dry-run verification
#
# This is the entry used by CLI commands:
# - build muffin   : validate + emit .mcf (+print)
# - build steel    : validate + plan + exec (Steel wraps)
#
# Blocks: .end only
# -----------------------------------------------------------------------------

mod muffin/load

use std/string
use std/result
use muffin/externs

use muffin/interface
use muffin/config
use muffin/default
use muffin/implicit
use muffin/debug
use muffin/directory

# parser is assumed:
# - parse_muf(text, path) -> ParseRes[AstFile]
# - lower(ast) -> LowerRes[Config]
use muffin/parser

export all

# -----------------------------------------------------------------------------
# Errors
# -----------------------------------------------------------------------------

enum LoadErrKind
  WorkspaceNotFound
  ManifestNotFound
  Io
  Parse
  Lower
  Resolve
  Emit
.end

struct LoadError
  kind: LoadErrKind
  message: str
  path: str
.end

type LoadRes[T] = result::Result[T, LoadError]

fn load_err(kind: LoadErrKind, msg: str, path: str) -> LoadError
  ret LoadError(kind: kind, message: msg, path: path)
.end

# -----------------------------------------------------------------------------
# Options / Results
# -----------------------------------------------------------------------------

struct LoadOptions
  start_dir: str
  explicit_manifest: str       # if set, bypass discovery
  emit_mcf: bool               # emit Muffinconfig .mcf
  mcf_path: str                # if empty => <root>/.muffin/<profile>.mcf
  validate_only: bool          # stop after resolve/implicit
  verbose: bool
.end

fn load_options_default() -> LoadOptions
  ret LoadOptions(
    start_dir: ".",
    explicit_manifest: "",
    emit_mcf: false,
    mcf_path: "",
    validate_only: false,
    verbose: false
  )
.end

struct Loaded
  root: str
  manifest_path: str
  manifest_text: str
  resolved: Resolved
  mcf_emitted: str             # path or ""
.end

# -----------------------------------------------------------------------------
# Public API
# -----------------------------------------------------------------------------

fn load_workspace(rt: interface::Runtime, opt: LoadOptions) -> LoadRes[Loaded]
  # 1) find root
  let rr_root: interface::IoRes[str] =
    (opt.explicit_manifest != "")
      ? result::Ok(directory::parent_dir(directory::norm_path(opt.explicit_manifest)))
      : rt.ws.find_root(rt.ws.ctx, opt.start_dir)

  if result::is_err(rr_root)
    ret result::Err(load_err(LoadErrKind::WorkspaceNotFound, "workspace root not found", opt.start_dir))
  .end
  let root: str = directory::norm_path(result::unwrap(rr_root))

  # 2) manifest path
  let mp: str =
    (opt.explicit_manifest != "")
      ? directory::norm_path(opt.explicit_manifest)
      : discover_manifest(rt, root)

  if mp == ""
    ret result::Err(load_err(LoadErrKind::ManifestNotFound, "no manifest found in workspace", root))
  .end

  # 3) read manifest text
  let rr_txt: interface::IoRes[str] = rt.ws.read_muf(rt.ws.ctx, mp)
  if result::is_err(rr_txt)
    ret result::Err(load_err(LoadErrKind::Io, "failed to read manifest", mp))
  .end
  let txt: str = result::unwrap(rr_txt)

  # 4) parse
  let rr_ast: parser::ParseRes[parser::AstFile] = parser::parse_muf(txt, mp)
  if result::is_err(rr_ast)
    ret result::Err(load_err(LoadErrKind::Parse, parser::format_parse_error(result::unwrap_err(rr_ast)), mp))
  .end
  let ast: parser::AstFile = result::unwrap(rr_ast)

  # 5) lower to Config (semantic config)
  let rr_cfg: parser::LowerRes[Config] = parser::lower(ast)
  if result::is_err(rr_cfg)
    ret result::Err(load_err(LoadErrKind::Lower, parser::format_lower_error(result::unwrap_err(rr_cfg)), mp))
  .end
  let cfg: Config = result::unwrap(rr_cfg)

  # 6) resolve references to Resolved
  let rr_res: result::Result[Resolved, str] = resolve_config(root, mp, cfg)
  if result::is_err(rr_res)
    ret result::Err(load_err(LoadErrKind::Resolve, result::unwrap_err(rr_res), mp))
  .end
  let mut r: Resolved = result::unwrap(rr_res)

  # 7) apply defaults
  r = default::apply_defaults(r)

  # 8) apply implicit (may fail)
  let rr_imp: implicit::ImpRes[Resolved] = implicit::apply_implicit(r)
  if result::is_err(rr_imp)
    ret result::Err(load_err(LoadErrKind::Resolve, (result::unwrap_err(rr_imp)).message, mp))
  .end
  r = result::unwrap(rr_imp)

  # 9) optionally emit .mcf
  let mut emitted: str = ""
  if opt.emit_mcf
    let p: str = resolve_mcf_path(root, r.selection.profile, opt.mcf_path)
    let rr_emit: LoadRes[str] = emit_mcf(rt, r, p)
    if result::is_err(rr_emit) ret rr_emit .end
    emitted = result::unwrap(rr_emit)
  .end

  ret result::Ok(Loaded(root: root, manifest_path: mp, manifest_text: txt, resolved: r, mcf_emitted: emitted))
.end

# -----------------------------------------------------------------------------
# Manifest discovery
# -----------------------------------------------------------------------------

fn discover_manifest(rt: interface::Runtime, root: str) -> str
  # order of preference:
  # - Muffinfile
  # - build.muf
  # - mod.muf
  # - muffin.muf
  let cands: list[str] = [
    directory::join_path(root, "Muffinfile"),
    directory::join_path(root, "build.muf"),
    directory::join_path(root, "mod.muf"),
    directory::join_path(root, "muffin.muf")
  ]

  let mut i: i32 = 0
  while i < len(cands)
    let p: str = cands[i]
    if rt.fs.exists(rt.fs.ctx, p) return directory::norm_path(p) .end
    i = i + 1
  .end

  return ""
.end

# -----------------------------------------------------------------------------
# Resolve config -> resolved
# -----------------------------------------------------------------------------
# This is a "glue" layer; real project may have a dedicated resolver module.
# Here we only:
# - fill workspace + paths
# - merge config sections into resolved shape
# - keep it deterministic

fn resolve_config(root: str, manifest_path: str, cfg: Config) -> result::Result[Resolved, str]
  let mut r: Resolved = resolved_empty()

  r.workspace.name = cfg.workspace.name
  r.workspace.root = root
  r.workspace.file = manifest_path
  r.workspace.emit = directory::join_path(root, ".muffin")

  r.paths.root = root
  r.paths.build = (cfg.paths.build != "") ? cfg.paths.build : "build"
  r.paths.dist = (cfg.paths.dist != "") ? cfg.paths.dist : "dist"
  r.paths.tmp = (cfg.paths.tmp != "") ? cfg.paths.tmp : ".muffin/tmp"
  r.paths.cache = (cfg.paths.cache != "") ? cfg.paths.cache : ".muffin/cache"
  r.paths.doc = (cfg.paths.doc != "") ? cfg.paths.doc : "doc"
  r.paths.src = (cfg.paths.src != "") ? cfg.paths.src : "src"
  r.paths.steel = (cfg.paths.steel != "") ? cfg.paths.steel : "Steel"

  r.selection.profile = cfg.selection.profile
  r.selection.target = cfg.selection.target
  r.selection.toolchain = cfg.selection.toolchain

  r.profiles = cfg.profiles
  r.toolchains = cfg.toolchains
  r.tools = cfg.tools

  r.targets = cfg.targets
  r.packages = cfg.packages
  r.file_groups = cfg.file_groups

  r.env = cfg.env
  r.fingerprint.algo = cfg.fingerprint.algo
  r.fingerprint.value = cfg.fingerprint.value

  ret result::Ok(r)
.end

# -----------------------------------------------------------------------------
# Emit .mcf (Muffinconfig)
# -----------------------------------------------------------------------------

fn resolve_mcf_path(root: str, profile: str, override: str) -> str
  if override != "" return directory::norm_path(override) .end
  let dir: str = directory::join_path(root, ".muffin")
  let file: str = (profile != "") ? ("Muffinconfig." + profile + ".mcf") : "Muffinconfig.mcf"
  return directory::join_path(dir, file)
.end

fn emit_mcf(rt: interface::Runtime, r: Resolved, path: str) -> LoadRes[str]
  # ensure dir
  let dir: str = directory::parent_dir(path)
  let rr_mk: interface::IoRes[bool] = rt.fs.mkdirs(rt.fs.ctx, dir)
  if result::is_err(rr_mk)
    ret result::Err(load_err(LoadErrKind::Emit, "failed to create .muffin dir", dir))
  .end

  let txt: str = render_mcf(r)
  let rr_w: interface::IoRes[bool] = rt.ws.write_mcf(rt.ws.ctx, path, txt)
  if result::is_err(rr_w)
    ret result::Err(load_err(LoadErrKind::Emit, "failed to write .mcf", path))
  .end
  ret result::Ok(path)
.end

fn render_mcf(r: Resolved) -> str
  # simple canonical text format (line-based, stable)
  # This file is meant to be consumed by Steel or debug tooling.
  let mut out: str = ""
  out = out + "mcf 1\n"
  out = out + "workspace.root=" + r.workspace.root + "\n"
  out = out + "workspace.file=" + r.workspace.file + "\n"
  out = out + "selection.profile=" + r.selection.profile + "\n"
  out = out + "selection.target=" + r.selection.target + "\n"
  out = out + "selection.toolchain=" + r.selection.toolchain + "\n"
  out = out + "paths.build=" + r.paths.build + "\n"
  out = out + "paths.dist=" + r.paths.dist + "\n"
  out = out + "paths.tmp=" + r.paths.tmp + "\n"
  out = out + "paths.cache=" + r.paths.cache + "\n"
  out = out + "paths.steel=" + r.paths.steel + "\n"
  out = out + "paths.src=" + r.paths.src + "\n"
  out = out + "fingerprint.algo=" + r.fingerprint.algo + "\n"
  out = out + "fingerprint.value=" + r.fingerprint.value + "\n"

  # counts
  out = out + "profiles.count=" + externs::i32_to_str(len(r.profiles)) + "\n"
  out = out + "targets.count=" + externs::i32_to_str(len(r.targets)) + "\n"
  out = out + "tools.count=" + externs::i32_to_str(len(r.tools)) + "\n"
  out = out + "groups.count=" + externs::i32_to_str(len(r.file_groups)) + "\n"

  ret out
.end

# -----------------------------------------------------------------------------
# Externs (resolved construction + stringify)
# -----------------------------------------------------------------------------

extern fn resolved_empty() -> Resolved
extern fn len[T](xs: list[T]) -> i32
