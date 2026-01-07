# expand.vit — Muffin (Vitte)
#
# Expansion / interpolation engine:
# - variables: ${VAR}, $VAR (env + local)
# - builtin variables: ${workspace.root}, ${paths.build}, ${profile.name}, etc.
# - list expansion for argv templates
# - template strings for cmd lines
# - deterministic, explicit policies (allowed env)
#
# This module is used by:
# - resolver (turns file definitions into resolved steps)
# - executor (render argv/cmd before spawn)
# - Steel integration (same expansion rules)
#
# Security posture:
# - only allow env keys in EnvPolicy.allow
# - do not execute shells; expansion is pure string/list transform
#
# Blocks: .end only
# -----------------------------------------------------------------------------

mod muffin/expand

use std/string
use std/result
use muffin/config
use muffin/debug

export all

# -----------------------------------------------------------------------------
# Errors
# -----------------------------------------------------------------------------

enum ExpErrKind
  Parse
  UnknownVar
  ForbiddenEnv
  Depth
.end

struct ExpError
  kind: ExpErrKind
  message: str
  at: i32
.end

type ExpRes[T] = result::Result[T, ExpError]

fn exp_err(kind: ExpErrKind, msg: str, at: i32) -> ExpError
  ret ExpError(kind: kind, message: msg, at: at)
.end

# -----------------------------------------------------------------------------
# Context
# -----------------------------------------------------------------------------

struct ExpandCtx
  r: Resolved
  profile: Profile
  target: Target
  step: Step

  locals: map[str, str]
  depth_limit: i32
.end

fn ctx_new(r: Resolved, prof: Profile, tgt: Target, st: Step) -> ExpandCtx
  ret ExpandCtx(
    r: r,
    profile: prof,
    target: tgt,
    step: st,
    locals: map_new_str(),
    depth_limit: 32
  )
.end

fn with_local(mut c: ExpandCtx, k: str, v: str) -> ExpandCtx
  c.locals = map_put_str(c.locals, k, v)
  ret c
.end

# -----------------------------------------------------------------------------
# Public API
# -----------------------------------------------------------------------------

fn expand_str(ctx: ExpandCtx, s: str) -> ExpRes[str]
  ret expand_str_depth(ctx, s, 0)
.end

fn expand_argv(ctx: ExpandCtx, argv: list[str]) -> ExpRes[list[str]]
  let mut out: list[str] = []
  let mut i: i32 = 0
  while i < len(argv)
    let rr: ExpRes[str] = expand_str(ctx, argv[i])
    if result::is_err(rr) ret rr as list[str] .end
    let v: str = result::unwrap(rr)
    # split policy: keep as single arg (no shell splitting)
    out = out + [v]
    i = i + 1
  .end
  ret result::Ok(out)
.end

# "argv template" expansion:
# - supports sentinel tokens:
#   - "@{list:group:NAME}" -> expands to each file in group NAME
#   - "@{list:files}" -> expands to target.inputs_files
#   - "@{list:dirs}" -> expands to target.inputs_dirs
# - everything else expands normally and is kept as one arg.
fn expand_argv_template(ctx: ExpandCtx, argv: list[str]) -> ExpRes[list[str]]
  let mut out: list[str] = []
  let mut i: i32 = 0
  while i < len(argv)
    let a0: str = argv[i]
    i = i + 1

    if string::starts_with(a0, "@{") && string::ends_with(a0, "}")
      let rr2: ExpRes[list[str]] = expand_special_list(ctx, a0)
      if result::is_err(rr2) ret rr2 .end
      out = out + result::unwrap(rr2)
      continue
    .end

    let rr: ExpRes[str] = expand_str(ctx, a0)
    if result::is_err(rr) ret rr as list[str] .end
    out = out + [result::unwrap(rr)]
  .end
  ret result::Ok(out)
.end

# -----------------------------------------------------------------------------
# Core expansion (string)
# -----------------------------------------------------------------------------

fn expand_str_depth(ctx: ExpandCtx, s: str, depth: i32) -> ExpRes[str]
  if depth > ctx.depth_limit
    ret result::Err(exp_err(ExpErrKind::Depth, "expansion depth limit", 0))
  .end

  let mut out: str = ""
  let mut i: i32 = 0
  while i < string::len(s)
    let c: i32 = string::codepoint_at(s, i)

    if c == 36 # '$'
      if i + 1 >= string::len(s)
        out = out + "$"
        i = i + 1
        continue
      .end

      let n: i32 = string::codepoint_at(s, i + 1)

      if n == 123 # '{'
        # ${ ... }
        let rr: ExpRes[tuple[str, i32]] = parse_braced_var(s, i + 2)
        if result::is_err(rr) ret rr as str .end
        let pair: tuple[str, i32] = result::unwrap(rr)
        let key: str = pair.0
        let next_i: i32 = pair.1

        let vv: ExpRes[str] = resolve_var(ctx, key, i)
        if result::is_err(vv) ret vv .end

        let val: str = result::unwrap(vv)
        # recursive expand inside value
        let rr2: ExpRes[str] = expand_str_depth(ctx, val, depth + 1)
        if result::is_err(rr2) ret rr2 .end

        out = out + result::unwrap(rr2)
        i = next_i
        continue
      .end

      # $VAR
      if is_ident_start(n)
        let rr: tuple[str, i32] = parse_bare_var(s, i + 1)
        let key: str = rr.0
        let next_i: i32 = rr.1

        let vv: ExpRes[str] = resolve_var(ctx, key, i)
        if result::is_err(vv) ret vv .end

        let val: str = result::unwrap(vv)
        let rr2: ExpRes[str] = expand_str_depth(ctx, val, depth + 1)
        if result::is_err(rr2) ret rr2 .end

        out = out + result::unwrap(rr2)
        i = next_i
        continue
      .end

      # "$" not followed by var
      out = out + "$"
      i = i + 1
      continue
    .end

    out = out + string::from_codepoint(c)
    i = i + 1
  .end

  ret result::Ok(out)
.end

fn parse_braced_var(s: str, start: i32) -> ExpRes[tuple[str, i32]]
  let mut i: i32 = start
  let mut key: str = ""
  while i < string::len(s)
    let c: i32 = string::codepoint_at(s, i)
    if c == 125 # '}'
      return result::Ok((string::trim(key), i + 1))
    .end
    key = key + string::from_codepoint(c)
    i = i + 1
  .end
  ret result::Err(exp_err(ExpErrKind::Parse, "unterminated ${...}", start))
.end

fn parse_bare_var(s: str, start: i32) -> tuple[str, i32]
  let mut i: i32 = start
  let mut key: str = ""
  while i < string::len(s)
    let c: i32 = string::codepoint_at(s, i)
    if !is_ident_continue(c)
      break
    .end
    key = key + string::from_codepoint(c)
    i = i + 1
  .end
  ret (key, i)
.end

# -----------------------------------------------------------------------------
# Variable resolution
# -----------------------------------------------------------------------------

fn resolve_var(ctx: ExpandCtx, key: str, at: i32) -> ExpRes[str]
  # locals first
  if map_has_str(ctx.locals, key)
    ret result::Ok(map_get_str(ctx.locals, key))
  .end

  # builtins (structured namespace)
  let rr: ExpRes[str] = resolve_builtin(ctx, key, at)
  if result::is_ok(rr) ret rr .end

  # env: only if allowed
  if env_allowed(ctx.r.env, key)
    let v: str = env_get(key)
    ret result::Ok(v)
  .end

  # explicit rejection for env-like tokens
  if looks_like_env(key)
    ret result::Err(exp_err(ExpErrKind::ForbiddenEnv, "env var not allowed: " + key, at))
  .end

  ret result::Err(exp_err(ExpErrKind::UnknownVar, "unknown variable: " + key, at))
.end

fn resolve_builtin(ctx: ExpandCtx, key: str, at: i32) -> ExpRes[str]
  # workspace.*
  if key == "workspace.name" ret result::Ok(ctx.r.workspace.name) .end
  if key == "workspace.root" ret result::Ok(ctx.r.workspace.root) .end
  if key == "workspace.file" ret result::Ok(ctx.r.workspace.file) .end
  if key == "workspace.emit" ret result::Ok(ctx.r.workspace.emit) .end

  # paths.*
  if key == "paths.root" ret result::Ok(ctx.r.paths.root) .end
  if key == "paths.build" ret result::Ok(ctx.r.paths.build) .end
  if key == "paths.dist" ret result::Ok(ctx.r.paths.dist) .end
  if key == "paths.tmp" ret result::Ok(ctx.r.paths.tmp) .end
  if key == "paths.cache" ret result::Ok(ctx.r.paths.cache) .end
  if key == "paths.steel" ret result::Ok(ctx.r.paths.steel) .end
  if key == "paths.src" ret result::Ok(ctx.r.paths.src) .end
  if key == "paths.doc" ret result::Ok(ctx.r.paths.doc) .end

  # host.*
  if key == "host.os" ret result::Ok(ctx.r.host.os) .end
  if key == "host.arch" ret result::Ok(ctx.r.host.arch) .end
  if key == "host.triple" ret result::Ok(ctx.r.host.triple) .end
  if key == "host.endian" ret result::Ok(ctx.r.host.endian) .end

  # selection.*
  if key == "selection.profile" ret result::Ok(ctx.r.selection.profile) .end
  if key == "selection.target" ret result::Ok(ctx.r.selection.target) .end

  # profile.*
  if key == "profile.name" ret result::Ok(ctx.profile.name) .end
  if key == "profile.opt" ret result::Ok(ctx.profile.opt) .end
  if key == "profile.debug" ret result::Ok(ctx.profile.debug ? "true" : "false") .end
  if key == "profile.lto" ret result::Ok(ctx.profile.lto ? "true" : "false") .end

  # target.*
  if key == "target.name" ret result::Ok(ctx.target.name) .end
  if key == "target.kind" ret result::Ok(ctx.target.kind) .end
  if key == "target.package" ret result::Ok(ctx.target.package) .end
  if key == "target.profile" ret result::Ok(ctx.target.profile) .end

  # step.*
  if key == "step.name" ret result::Ok(ctx.step.name) .end
  if key == "step.tool" ret result::Ok(ctx.step.tool) .end
  if key == "step.cwd" ret result::Ok(ctx.step.cwd) .end

  # fingerprint
  if key == "fingerprint.algo" ret result::Ok(ctx.r.fingerprint.algo) .end
  if key == "fingerprint.value" ret result::Ok(ctx.r.fingerprint.value) .end

  ret result::Err(exp_err(ExpErrKind::UnknownVar, "unknown builtin: " + key, at))
.end

fn looks_like_env(key: str) -> bool
  # heuristique: uppercase + underscores
  let mut up: bool = true
  let mut i: i32 = 0
  while i < string::len(key)
    let c: i32 = string::codepoint_at(key, i)
    i = i + 1
    if c == 95 continue .end
    if c >= 48 && c <= 57 continue .end
    if c >= 65 && c <= 90 continue .end
    up = false
  .end
  ret up
.end

fn env_allowed(envp: EnvPolicy, key: str) -> bool
  let mut i: i32 = 0
  while i < len(envp.allow)
    if envp.allow[i] == key ret true .end
    i = i + 1
  .end
  ret false
.end

# -----------------------------------------------------------------------------
# Special list expansion
# -----------------------------------------------------------------------------

fn expand_special_list(ctx: ExpandCtx, token: str) -> ExpRes[list[str]]
  # token format: "@{list:...}"
  # inside is "list:TYPE(:NAME)?"
  let inner: str = string::slice(token, 2, string::len(token) - 1)
  let parts: list[str] = split_colon(inner)

  if len(parts) < 2 || parts[0] != "list"
    ret result::Err(exp_err(ExpErrKind::Parse, "invalid list token: " + token, 0))
  .end

  let kind: str = parts[1]

  if kind == "files"
    return result::Ok(ctx.target.inputs_files)
  .end

  if kind == "dirs"
    return result::Ok(ctx.target.inputs_dirs)
  .end

  if kind == "groups" || kind == "group"
    if len(parts) < 3
      ret result::Err(exp_err(ExpErrKind::Parse, "missing group name in: " + token, 0))
    .end
    let gname: str = parts[2]
    let gi: i32 = find_group(ctx.r.file_groups, gname)
    if gi < 0
      ret result::Err(exp_err(ExpErrKind::UnknownVar, "unknown file group: " + gname, 0))
    .end
    return result::Ok(ctx.r.file_groups[gi].files)
  .end

  ret result::Err(exp_err(ExpErrKind::Parse, "unknown list kind: " + kind, 0))
.end

fn split_colon(s: str) -> list[str]
  let mut out: list[str] = []
  let mut cur: str = ""
  let mut i: i32 = 0
  while i < string::len(s)
    let c: i32 = string::codepoint_at(s, i)
    i = i + 1
    if c == 58
      out = out + [cur]
      cur = ""
    else
      cur = cur + string::from_codepoint(c)
    .end
  .end
  out = out + [cur]
  ret out
.end

# -----------------------------------------------------------------------------
# Identifier helpers
# -----------------------------------------------------------------------------

fn is_ident_start(c: i32) -> bool
  if c == 95 ret true .end
  if c >= 65 && c <= 90 ret true .end
  if c >= 97 && c <= 122 ret true .end
  ret false
.end

fn is_ident_continue(c: i32) -> bool
  if is_ident_start(c) ret true .end
  if c >= 48 && c <= 57 ret true .end
  ret false
.end

# -----------------------------------------------------------------------------
# Externs
# -----------------------------------------------------------------------------

extern fn env_get(name: str) -> str

extern fn map_new_str() -> map[str, str]
extern fn map_put_str(m: map[str, str], k: str, v: str) -> map[str, str]
extern fn map_get_str(m: map[str, str], k: str) -> str
extern fn map_has_str(m: map[str, str], k: str) -> bool

extern fn len[T](xs: list[T]) -> i32