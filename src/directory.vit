# directory.vit — Muffin (Vitte)
#
# Directory / FS helpers:
# - list / walk directories
# - glob-lite (file patterns) for Muffin discovery
# - ensure dirs, copy, remove, mtime, checksum helpers (thin wrappers)
# - deterministic ordering (sorted)
#
# Notes:
# - keep filesystem interactions centralized (auditability)
# - avoid hidden behavior; caller decides recursion, excludes, follow symlinks
# - when in doubt: return lists sorted + normalized
#
# Blocks: .end only
# -----------------------------------------------------------------------------

mod muffin/directory

use std/string
use std/result
use std/io

export all

# -----------------------------------------------------------------------------
# Errors
# -----------------------------------------------------------------------------

enum FsErrKind
  Io
  NotFound
  NotDir
  NotFile
  InvalidPattern
.end

struct FsError
  kind: FsErrKind
  message: str
  path: str
.end

type FsRes[T] = result::Result[T, FsError]

fn fs_err(kind: FsErrKind, msg: str, path: str) -> FsError
  ret FsError(kind: kind, message: msg, path: path)
.end

# -----------------------------------------------------------------------------
# Path normalize (small, no ".." elimination)
# -----------------------------------------------------------------------------

fn is_sep(c: i32) -> bool
  ret c == 47 || c == 92
.end

fn norm_path(p: str) -> str
  let mut out: str = ""
  let mut i: i32 = 0
  let mut prev_sep: bool = false
  while i < string::len(p)
    let c: i32 = string::codepoint_at(p, i)
    i = i + 1
    if is_sep(c)
      if !prev_sep out = out + "/" .end
      prev_sep = true
      continue
    .end
    out = out + string::from_codepoint(c)
    prev_sep = false
  .end
  while string::len(out) > 1 && string::ends_with(out, "/")
    out = string::slice(out, 0, string::len(out) - 1)
  .end
  if out == "" ret "." .end
  ret out
.end

fn join_path(a: str, b: str) -> str
  if a == "" ret norm_path(b) .end
  if b == "" ret norm_path(a) .end
  let aa: str = norm_path(a)
  let bb: str = norm_path(b)
  if string::ends_with(aa, "/") ret norm_path(aa + bb) .end
  ret norm_path(aa + "/" + bb)
.end

# -----------------------------------------------------------------------------
# Sorting (deterministic)
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

fn dedup_sorted(xs: list[str]) -> list[str]
  if len(xs) == 0 ret [] .end
  let mut out: list[str] = [xs[0]]
  let mut i: i32 = 1
  while i < len(xs)
    if xs[i] != xs[i - 1] out = out + [xs[i]] .end
    i = i + 1
  .end
  ret out
.end

fn finalize_paths(xs: list[str]) -> list[str]
  let mut tmp: list[str] = []
  let mut i: i32 = 0
  while i < len(xs)
    tmp = tmp + [norm_path(xs[i])]
    i = i + 1
  .end
  tmp = sort_str(tmp)
  tmp = dedup_sorted(tmp)
  ret tmp
.end

# -----------------------------------------------------------------------------
# Directory listing
# -----------------------------------------------------------------------------

struct DirEntry
  name: str
  path: str
  kind: str      # "file" | "dir" | "link" | "other"
.end

fn list_dir(path: str) -> FsRes[list[DirEntry]]
  let p: str = norm_path(path)
  if !fs_exists(p)
    ret result::Err(fs_err(FsErrKind::NotFound, "not found", p))
  .end
  if !fs_is_dir(p)
    ret result::Err(fs_err(FsErrKind::NotDir, "not a directory", p))
  .end

  let rr: result::Result[list[DirEntry], str] = fs_list_dir(p)
  if result::is_err(rr)
    ret result::Err(fs_err(FsErrKind::Io, "list_dir failed", p))
  .end

  let mut es: list[DirEntry] = result::unwrap(rr)
  # stable sort by entry.path
  es = sort_entries(es)
  ret result::Ok(es)
.end

fn sort_entries(xs: list[DirEntry]) -> list[DirEntry]
  let mut a: list[DirEntry] = xs
  let mut i: i32 = 0
  while i < len(a)
    let mut j: i32 = i + 1
    while j < len(a)
      if a[j].path < a[i].path
        let t: DirEntry = a[i]
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
# Walk (recursive)
# -----------------------------------------------------------------------------

struct WalkOptions
  recursive: bool
  follow_symlinks: bool
  include_files: bool
  include_dirs: bool
  max_depth: i32       # -1 infinite
  excludes: list[str]  # path prefixes (normalized) relative or absolute
.end

fn walk_options_default() -> WalkOptions
  ret WalkOptions(
    recursive: true,
    follow_symlinks: false,
    include_files: true,
    include_dirs: false,
    max_depth: -1,
    excludes: [".git", "Steel/cache", "build/tmp"]
  )
.end

fn is_excluded(excludes: list[str], p: str) -> bool
  let np: str = norm_path(p)
  let mut i: i32 = 0
  while i < len(excludes)
    let ex: str = norm_path(excludes[i])
    if ex != "" && string::starts_with(np, ex)
      ret true
    .end
    i = i + 1
  .end
  ret false
.end

fn walk(path: str, opt: WalkOptions) -> FsRes[list[str]]
  let root: str = norm_path(path)
  if !fs_exists(root)
    ret result::Err(fs_err(FsErrKind::NotFound, "not found", root))
  .end
  if !fs_is_dir(root)
    ret result::Err(fs_err(FsErrKind::NotDir, "not a directory", root))
  .end

  let mut out: list[str] = []
  let rr: FsRes[bool] = walk_impl(root, opt, 0, out)
  if result::is_err(rr)
    ret result::Err(result::unwrap_err(rr))
  .end
  out = finalize_paths(out)
  ret result::Ok(out)
.end

fn walk_impl(path: str, opt: WalkOptions, depth: i32, mut out: list[str]) -> FsRes[bool]
  if opt.max_depth >= 0 && depth > opt.max_depth
    ret result::Ok(true)
  .end

  if is_excluded(opt.excludes, path)
    ret result::Ok(true)
  .end

  let rr: FsRes[list[DirEntry]] = list_dir(path)
  if result::is_err(rr) ret result::Err(result::unwrap_err(rr)) .end
  let entries: list[DirEntry] = result::unwrap(rr)

  let mut i: i32 = 0
  while i < len(entries)
    let e: DirEntry = entries[i]
    i = i + 1

    if is_excluded(opt.excludes, e.path)
      continue
    .end

    if e.kind == "dir"
      if opt.include_dirs
        out = out + [e.path]
      .end
      if opt.recursive
        let r2: FsRes[bool] = walk_impl(e.path, opt, depth + 1, out)
        if result::is_err(r2) ret result::Err(result::unwrap_err(r2)) .end
        out = out # explicit
      .end
    elif e.kind == "file"
      if opt.include_files
        out = out + [e.path]
      .end
    elif e.kind == "link"
      # follow symlink only if requested; treat as file otherwise
      if opt.follow_symlinks
        if fs_is_dir(e.path)
          if opt.include_dirs out = out + [e.path] .end
          if opt.recursive
            let r2: FsRes[bool] = walk_impl(e.path, opt, depth + 1, out)
            if result::is_err(r2) ret result::Err(result::unwrap_err(r2)) .end
          .end
        else
          if opt.include_files out = out + [e.path] .end
        .end
      else
        if opt.include_files out = out + [e.path] .end
      .end
    else
      # ignore "other"
    .end
  .end

  ret result::Ok(true)
.end

# -----------------------------------------------------------------------------
# Glob-lite matcher
# -----------------------------------------------------------------------------
# Pattern subset:
# - '*' matches any sequence (not including '/')
# - '**' matches any sequence (including '/')
# - '?' matches one char (not '/')
# - suffix matching common: "*.vit", "src/**", "Steel/**"
#
# Deterministic: filtering only; no filesystem expansion here.

enum PatTokKind
  Lit
  Star
  DStar
  QMark
.end

struct PatTok
  kind: PatTokKind
  lit: str
.end

fn tok_lit(s: str) -> PatTok ret PatTok(kind: PatTokKind::Lit, lit: s) .end
fn tok_star() -> PatTok ret PatTok(kind: PatTokKind::Star, lit: "") .end
fn tok_dstar() -> PatTok ret PatTok(kind: PatTokKind::DStar, lit: "") .end
fn tok_q() -> PatTok ret PatTok(kind: PatTokKind::QMark, lit: "") .end

fn compile_pattern(pat: str) -> FsRes[list[PatTok]]
  let p: str = pat
  if p == "" ret result::Err(fs_err(FsErrKind::InvalidPattern, "empty pattern", pat)) .end

  let mut toks: list[PatTok] = []
  let mut buf: str = ""
  let mut i: i32 = 0
  while i < string::len(p)
    let c: i32 = string::codepoint_at(p, i)

    if c == 42 # '*'
      if buf != ""
        toks = toks + [tok_lit(buf)]
        buf = ""
      .end
      # check double star
      if i + 1 < string::len(p) && string::codepoint_at(p, i + 1) == 42
        toks = toks + [tok_dstar()]
        i = i + 2
      else
        toks = toks + [tok_star()]
        i = i + 1
      .end
      continue
    .end

    if c == 63 # '?'
      if buf != ""
        toks = toks + [tok_lit(buf)]
        buf = ""
      .end
      toks = toks + [tok_q()]
      i = i + 1
      continue
    .end

    buf = buf + string::from_codepoint(c)
    i = i + 1
  .end

  if buf != "" toks = toks + [tok_lit(buf)] .end
  ret result::Ok(toks)
.end

fn match_pattern(pat: str, path: str) -> bool
  let rr: FsRes[list[PatTok]] = compile_pattern(pat)
  if result::is_err(rr) ret false .end
  let toks: list[PatTok] = result::unwrap(rr)
  ret match_toks(toks, norm_path(path), 0, 0)
.end

fn match_toks(toks: list[PatTok], s: str, ti: i32, si: i32) -> bool
  if ti == len(toks) && si == string::len(s)
    ret true
  .end
  if ti == len(toks)
    ret false
  .end

  let t: PatTok = toks[ti]

  if t.kind == PatTokKind::Lit
    if string::starts_with_at(s, t.lit, si)
      ret match_toks(toks, s, ti + 1, si + string::len(t.lit))
    .end
    ret false
  .end

  if t.kind == PatTokKind::QMark
    if si >= string::len(s) ret false .end
    let c: i32 = string::codepoint_at(s, si)
    if c == 47 ret false .end
    ret match_toks(toks, s, ti + 1, si + 1)
  .end

  if t.kind == PatTokKind::Star
    # consume 0..N but not '/'
    let mut k: i32 = si
    while true
      if match_toks(toks, s, ti + 1, k) ret true .end
      if k >= string::len(s) break .end
      let c: i32 = string::codepoint_at(s, k)
      if c == 47 break .end
      k = k + 1
    .end
    ret false
  .end

  # DStar
  let mut k: i32 = si
  while true
    if match_toks(toks, s, ti + 1, k) ret true .end
    if k >= string::len(s) break .end
    k = k + 1
  .end
  ret false
.end

fn filter_by_patterns(paths: list[str], includes: list[str], excludes: list[str]) -> list[str]
  let mut out: list[str] = []
  let mut i: i32 = 0
  while i < len(paths)
    let p: str = norm_path(paths[i])
    i = i + 1

    if len(excludes) > 0 && any_match(excludes, p)
      continue
    .end
    if len(includes) == 0
      out = out + [p]
      continue
    .end
    if any_match(includes, p)
      out = out + [p]
    .end
  .end
  out = finalize_paths(out)
  ret out
.end

fn any_match(pats: list[str], path: str) -> bool
  let mut i: i32 = 0
  while i < len(pats)
    if match_pattern(pats[i], path) ret true .end
    i = i + 1
  .end
  ret false
.end

# -----------------------------------------------------------------------------
# Ensure dirs / file ops
# -----------------------------------------------------------------------------

fn ensure_dir(path: str) -> FsRes[bool]
  let p: str = norm_path(path)
  if fs_exists(p)
    if fs_is_dir(p) ret result::Ok(true) .end
    ret result::Err(fs_err(FsErrKind::NotDir, "path exists but is not a dir", p))
  .end

  let rr: result::Result[bool, str] = fs_mkdirs(p)
  if result::is_err(rr)
    ret result::Err(fs_err(FsErrKind::Io, "mkdirs failed", p))
  .end
  ret result::Ok(true)
.end

fn remove_file(path: str) -> FsRes[bool]
  let p: str = norm_path(path)
  if !fs_exists(p) ret result::Ok(true) .end
  if fs_is_dir(p) ret result::Err(fs_err(FsErrKind::NotFile, "is a directory", p)) .end
  let rr: result::Result[bool, str] = fs_remove_file(p)
  if result::is_err(rr)
    ret result::Err(fs_err(FsErrKind::Io, "remove_file failed", p))
  .end
  ret result::Ok(true)
.end

fn remove_dir_all(path: str) -> FsRes[bool]
  let p: str = norm_path(path)
  if !fs_exists(p) ret result::Ok(true) .end
  if !fs_is_dir(p) ret result::Err(fs_err(FsErrKind::NotDir, "not a dir", p)) .end
  let rr: result::Result[bool, str] = fs_remove_dir_all(p)
  if result::is_err(rr)
    ret result::Err(fs_err(FsErrKind::Io, "remove_dir_all failed", p))
  .end
  ret result::Ok(true)
.end

fn copy_file(src: str, dst: str) -> FsRes[bool]
  let s: str = norm_path(src)
  let d: str = norm_path(dst)
  if !fs_exists(s) ret result::Err(fs_err(FsErrKind::NotFound, "src not found", s)) .end
  let rr: result::Result[bool, str] = fs_copy_file(s, d)
  if result::is_err(rr)
    ret result::Err(fs_err(FsErrKind::Io, "copy failed", s + " -> " + d))
  .end
  ret result::Ok(true)
.end

fn read_text(path: str) -> FsRes[str]
  let p: str = norm_path(path)
  let rr: result::Result[str, str] = fs_read_text(p)
  if result::is_err(rr)
    ret result::Err(fs_err(FsErrKind::Io, "read failed", p))
  .end
  ret result::Ok(result::unwrap(rr))
.end

fn write_text(path: str, text: str) -> FsRes[bool]
  let p: str = norm_path(path)
  let rr: result::Result[bool, str] = fs_write_text(p, text)
  if result::is_err(rr)
    ret result::Err(fs_err(FsErrKind::Io, "write failed", p))
  .end
  ret result::Ok(true)
.end

fn mtime_ms(path: str) -> FsRes[i64]
  let p: str = norm_path(path)
  if !fs_exists(p) ret result::Err(fs_err(FsErrKind::NotFound, "not found", p)) .end
  let rr: result::Result[i64, str] = fs_mtime_ms(p)
  if result::is_err(rr)
    ret result::Err(fs_err(FsErrKind::Io, "mtime failed", p))
  .end
  ret result::Ok(result::unwrap(rr))
.end

fn sha256_file_hex(path: str) -> FsRes[str]
  let p: str = norm_path(path)
  if !fs_exists(p) ret result::Err(fs_err(FsErrKind::NotFound, "not found", p)) .end
  let rr: result::Result[str, str] = fs_sha256_hex(p)
  if result::is_err(rr)
    ret result::Err(fs_err(FsErrKind::Io, "sha256 failed", p))
  .end
  ret result::Ok(result::unwrap(rr))
.end

# -----------------------------------------------------------------------------
# Externs
# -----------------------------------------------------------------------------

extern fn fs_exists(path: str) -> bool
extern fn fs_is_dir(path: str) -> bool

extern fn fs_list_dir(path: str) -> result::Result[list[DirEntry], str]

extern fn fs_mkdirs(path: str) -> result::Result[bool, str]
extern fn fs_remove_file(path: str) -> result::Result[bool, str]
extern fn fs_remove_dir_all(path: str) -> result::Result[bool, str]
extern fn fs_copy_file(src: str, dst: str) -> result::Result[bool, str]

extern fn fs_read_text(path: str) -> result::Result[str, str]
extern fn fs_write_text(path: str, text: str) -> result::Result[bool, str]

extern fn fs_mtime_ms(path: str) -> result::Result[i64, str]
extern fn fs_sha256_hex(path: str) -> result::Result[str, str]

extern fn len[T](xs: list[T]) -> i32