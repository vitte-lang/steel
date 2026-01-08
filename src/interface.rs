# interface.vit — Muffin (Vitte) — MAX MAX MAX
#
# Interfaces / contracts / ABI for Muffin+Steel runtime wiring.
# -----------------------------------------------------------------------------
# Goals
# - testability: everything side-effectful goes through injected backends
# - determinism: hashing and planning rely on stable interfaces
# - portability: one abstraction layer for Linux/macOS/Windows
# - composability: Steel can embed Muffin resolver and reuse Proc/Fs/Cache/Log
#
# Implementation model (no traits yet):
# - "interface" = struct of function pointers + ctx handle (i64)
# - the ctx is opaque and owned by caller/runtime
#
# Rules
# - no allocation hidden in wrappers unless explicit
# - error types are data-first (kind+message+path+code), no exceptions
# - order-sensitive arrays kept as-is where meaningful (argv)
#
# Blocks: .end only
# -----------------------------------------------------------------------------

mod muffin/interface

use std/string
use std/result

export all

# -----------------------------------------------------------------------------
# Common result/error primitives
# -----------------------------------------------------------------------------

enum IoErrKind
  Io
  NotFound
  Permission
  Invalid
  Unsupported
  AlreadyExists
  Timeout
.end

struct IoError
  kind: IoErrKind
  message: str
  path: str
  code: i32             # errno/GetLastError or mapped code, 0 if unknown
.end

type IoRes[T] = result::Result[T, IoError]

fn io_err(kind: IoErrKind, msg: str, path: str, code: i32) -> IoError
  ret IoError(kind: kind, message: msg, path: path, code: code)
.end

# -----------------------------------------------------------------------------
# Logger interface
# -----------------------------------------------------------------------------

enum LogLevel
  Trace
  Debug
  Info
  Warn
  Error
.end

struct LogRecord
  level: LogLevel
  component: str        # "muffin", "steel", "resolve", "exec", ...
  msg: str
  file: str
  line: i32
  ts_ms: i64
.end

struct LogSink
  ctx: i64
  log: fn(ctx: i64, rec: LogRecord) -> bool
.end

fn log_emit(s: LogSink, level: LogLevel, component: str, msg: str) -> bool
  ret s.log(s.ctx, LogRecord(level: level, component: component, msg: msg, file: "", line: 0, ts_ms: 0))
.end

fn log_emit_at(s: LogSink, level: LogLevel, component: str, msg: str, file: str, line: i32, ts_ms: i64) -> bool
  ret s.log(s.ctx, LogRecord(level: level, component: component, msg: msg, file: file, line: line, ts_ms: ts_ms))
.end

# -----------------------------------------------------------------------------
# Clock interface
# -----------------------------------------------------------------------------

struct Clock
  ctx: i64
  now_ms: fn(ctx: i64) -> i64
  sleep_ms: fn(ctx: i64, ms: i64) -> bool
.end

fn clock_now_ms(c: Clock) -> i64 ret c.now_ms(c.ctx) .end
fn clock_sleep_ms(c: Clock, ms: i64) -> bool ret c.sleep_ms(c.ctx, ms) .end

# -----------------------------------------------------------------------------
# Filesystem interface
# -----------------------------------------------------------------------------

enum EntryKind
  File
  Dir
  Link
  Other
.end

struct FsEntry
  name: str
  path: str
  kind: EntryKind
.end

struct Stat
  kind: EntryKind
  size: i64
  mtime_ms: i64
  mode: i32             # unix perms if applicable
.end

struct Fs
  ctx: i64

  exists: fn(ctx: i64, path: str) -> bool
  is_dir: fn(ctx: i64, path: str) -> bool

  stat: fn(ctx: i64, path: str) -> IoRes[Stat]
  list_dir: fn(ctx: i64, path: str) -> IoRes[list[FsEntry]]

  mkdirs: fn(ctx: i64, path: str) -> IoRes[bool]
  remove_file: fn(ctx: i64, path: str) -> IoRes[bool]
  remove_dir_all: fn(ctx: i64, path: str) -> IoRes[bool]
  rename: fn(ctx: i64, src: str, dst: str) -> IoRes[bool]
  copy_file: fn(ctx: i64, src: str, dst: str) -> IoRes[bool]

  read_text: fn(ctx: i64, path: str) -> IoRes[str]
  write_text: fn(ctx: i64, path: str, text: str) -> IoRes[bool]

  read_bytes: fn(ctx: i64, path: str) -> IoRes[list[u8]]
  write_bytes: fn(ctx: i64, path: str, bytes: list[u8]) -> IoRes[bool]

  mtime_ms: fn(ctx: i64, path: str) -> IoRes[i64]

  sha256_hex: fn(ctx: i64, path: str) -> IoRes[str]
  blake3_hex: fn(ctx: i64, path: str) -> IoRes[str]
.end

# wrappers
fn fs_exists(fs: Fs, path: str) -> bool ret fs.exists(fs.ctx, path) .end
fn fs_is_dir(fs: Fs, path: str) -> bool ret fs.is_dir(fs.ctx, path) .end
fn fs_stat(fs: Fs, path: str) -> IoRes[Stat] ret fs.stat(fs.ctx, path) .end
fn fs_list_dir(fs: Fs, path: str) -> IoRes[list[FsEntry]] ret fs.list_dir(fs.ctx, path) .end
fn fs_mkdirs(fs: Fs, path: str) -> IoRes[bool] ret fs.mkdirs(fs.ctx, path) .end
fn fs_remove_file(fs: Fs, path: str) -> IoRes[bool] ret fs.remove_file(fs.ctx, path) .end
fn fs_remove_dir_all(fs: Fs, path: str) -> IoRes[bool] ret fs.remove_dir_all(fs.ctx, path) .end
fn fs_rename(fs: Fs, src: str, dst: str) -> IoRes[bool] ret fs.rename(fs.ctx, src, dst) .end
fn fs_copy_file(fs: Fs, src: str, dst: str) -> IoRes[bool] ret fs.copy_file(fs.ctx, src, dst) .end
fn fs_read_text(fs: Fs, path: str) -> IoRes[str] ret fs.read_text(fs.ctx, path) .end
fn fs_write_text(fs: Fs, path: str, text: str) -> IoRes[bool] ret fs.write_text(fs.ctx, path, text) .end
fn fs_read_bytes(fs: Fs, path: str) -> IoRes[list[u8]] ret fs.read_bytes(fs.ctx, path) .end
fn fs_write_bytes(fs: Fs, path: str, b: list[u8]) -> IoRes[bool] ret fs.write_bytes(fs.ctx, path, b) .end
fn fs_mtime_ms(fs: Fs, path: str) -> IoRes[i64] ret fs.mtime_ms(fs.ctx, path) .end

# -----------------------------------------------------------------------------
# Process runner interface
# -----------------------------------------------------------------------------

enum ProcErrKind
  Spawn
  Wait
  Timeout
  Kill
.end

struct ProcError
  kind: ProcErrKind
  message: str
  code: i32
.end

type ProcRes[T] = result::Result[T, ProcError]

fn proc_err(kind: ProcErrKind, msg: str, code: i32) -> ProcError
  ret ProcError(kind: kind, message: msg, code: code)
.end

enum ProcStream
  Inherit
  Null
  Pipe
  File
.end

struct ProcIo
  kind: ProcStream
  path: str
.end

fn proc_io_inherit() -> ProcIo ret ProcIo(kind: ProcStream::Inherit, path: "") .end
fn proc_io_null() -> ProcIo ret ProcIo(kind: ProcStream::Null, path: "") .end
fn proc_io_pipe() -> ProcIo ret ProcIo(kind: ProcStream::Pipe, path: "") .end
fn proc_io_file(path: str) -> ProcIo ret ProcIo(kind: ProcStream::File, path: path) .end

struct ProcOptions
  cwd: str
  env: map[str, str]
  stdin: ProcIo
  stdout: ProcIo
  stderr: ProcIo
  timeout_ms: i64          # 0 = none
.end

fn proc_options_default() -> ProcOptions
  ret ProcOptions(
    cwd: "",
    env: map_new_str(),
    stdin: proc_io_inherit(),
    stdout: proc_io_inherit(),
    stderr: proc_io_inherit(),
    timeout_ms: 0
  )
.end

struct ProcResult
  exit_code: i32
  duration_ms: i64
  stdout: str              # if stdout=Pipe (else "")
  stderr: str              # if stderr=Pipe (else "")
.end

struct Proc
  ctx: i64

  run: fn(ctx: i64, argv: list[str], opt: ProcOptions) -> ProcRes[ProcResult]
.end

fn proc_run(p: Proc, argv: list[str], opt: ProcOptions) -> ProcRes[ProcResult]
  ret p.run(p.ctx, argv, opt)
.end

# -----------------------------------------------------------------------------
# Cache interface
# -----------------------------------------------------------------------------

enum CacheErrKind
  Io
  Miss
  Corrupt
  Unsupported
.end

struct CacheError
  kind: CacheErrKind
  message: str
  key: str
.end

type CacheRes[T] = result::Result[T, CacheError]

fn cache_err(kind: CacheErrKind, msg: str, key: str) -> CacheError
  ret CacheError(kind: kind, message: msg, key: key)
.end

struct CacheMeta
  key: str
  size: i64
  created_ms: i64
  algo: str              # hash algo, e.g. "sha256"
.end

struct Cache
  ctx: i64

  has: fn(ctx: i64, key: str) -> CacheRes[bool]
  get: fn(ctx: i64, key: str) -> CacheRes[list[u8]]
  put: fn(ctx: i64, key: str, value: list[u8]) -> CacheRes[bool]
  remove: fn(ctx: i64, key: str) -> CacheRes[bool]
  meta: fn(ctx: i64, key: str) -> CacheRes[CacheMeta]
.end

fn cache_has(c: Cache, key: str) -> CacheRes[bool] ret c.has(c.ctx, key) .end
fn cache_get(c: Cache, key: str) -> CacheRes[list[u8]] ret c.get(c.ctx, key) .end
fn cache_put(c: Cache, key: str, v: list[u8]) -> CacheRes[bool] ret c.put(c.ctx, key, v) .end
fn cache_remove(c: Cache, key: str) -> CacheRes[bool] ret c.remove(c.ctx, key) .end
fn cache_meta(c: Cache, key: str) -> CacheRes[CacheMeta] ret c.meta(c.ctx, key) .end

# -----------------------------------------------------------------------------
# Workspace provider interface
# -----------------------------------------------------------------------------

struct WorkspaceIO
  ctx: i64

  # locate root + read manifest(s)
  find_root: fn(ctx: i64, start: str) -> IoRes[str]
  read_muf: fn(ctx: i64, path: str) -> IoRes[str]

  # emit config (.mcf)
  write_mcf: fn(ctx: i64, path: str, text: str) -> IoRes[bool]

  # optional: diagnostics output directory
  ensure_out_dir: fn(ctx: i64, path: str) -> IoRes[bool]
.end

# -----------------------------------------------------------------------------
# Terminal / UI interface (optional)
# -----------------------------------------------------------------------------

enum AnsiMode
  Auto
  Always
  Never
.end

struct Term
  ctx: i64
  is_tty: fn(ctx: i64) -> bool
  width: fn(ctx: i64) -> i32
  ansi_mode: fn(ctx: i64) -> AnsiMode
.end

# -----------------------------------------------------------------------------
# Runtime bundle
# -----------------------------------------------------------------------------

struct Runtime
  fs: Fs
  proc: Proc
  clock: Clock
  log: LogSink
  cache: Cache
  ws: WorkspaceIO
  term: Term
.end

fn runtime(fs: Fs, proc: Proc, clock: Clock, log: LogSink, cache: Cache, ws: WorkspaceIO, term: Term) -> Runtime
  ret Runtime(fs: fs, proc: proc, clock: clock, log: log, cache: cache, ws: ws, term: term)
.end

# -----------------------------------------------------------------------------
# Default implementations (delegating to extern syscalls)
# -----------------------------------------------------------------------------

fn default_log_sink() -> LogSink
  ret LogSink(ctx: 0, log: sys_log)
.end

fn default_clock() -> Clock
  ret Clock(ctx: 0, now_ms: sys_now_ms, sleep_ms: sys_sleep_ms)
.end

fn default_fs() -> Fs
  ret Fs(
    ctx: 0,
    exists: sys_fs_exists,
    is_dir: sys_fs_is_dir,
    stat: sys_fs_stat,
    list_dir: sys_fs_list_dir,
    mkdirs: sys_fs_mkdirs,
    remove_file: sys_fs_remove_file,
    remove_dir_all: sys_fs_remove_dir_all,
    rename: sys_fs_rename,
    copy_file: sys_fs_copy_file,
    read_text: sys_fs_read_text,
    write_text: sys_fs_write_text,
    read_bytes: sys_fs_read_bytes,
    write_bytes: sys_fs_write_bytes,
    mtime_ms: sys_fs_mtime_ms,
    sha256_hex: sys_fs_sha256_hex,
    blake3_hex: sys_fs_blake3_hex
  )
.end

fn default_proc() -> Proc
  ret Proc(ctx: 0, run: sys_proc_run)
.end

fn default_cache() -> Cache
  ret Cache(ctx: 0, has: sys_cache_has, get: sys_cache_get, put: sys_cache_put, remove: sys_cache_remove, meta: sys_cache_meta)
.end

fn default_workspace_io() -> WorkspaceIO
  ret WorkspaceIO(ctx: 0, find_root: sys_ws_find_root, read_muf: sys_ws_read_muf, write_mcf: sys_ws_write_mcf, ensure_out_dir: sys_ws_ensure_out_dir)
.end

fn default_term() -> Term
  ret Term(ctx: 0, is_tty: sys_term_is_tty, width: sys_term_width, ansi_mode: sys_term_ansi_mode)
.end

fn default_runtime() -> Runtime
  ret runtime(default_fs(), default_proc(), default_clock(), default_log_sink(), default_cache(), default_workspace_io(), default_term())
.end

# -----------------------------------------------------------------------------
# Externs (system backends)
# -----------------------------------------------------------------------------

extern fn map_new_str() -> map[str, str]

extern fn sys_log(ctx: i64, rec: LogRecord) -> bool
extern fn sys_now_ms(ctx: i64) -> i64
extern fn sys_sleep_ms(ctx: i64, ms: i64) -> bool

extern fn sys_fs_exists(ctx: i64, path: str) -> bool
extern fn sys_fs_is_dir(ctx: i64, path: str) -> bool
extern fn sys_fs_stat(ctx: i64, path: str) -> IoRes[Stat]
extern fn sys_fs_list_dir(ctx: i64, path: str) -> IoRes[list[FsEntry]]
extern fn sys_fs_mkdirs(ctx: i64, path: str) -> IoRes[bool]
extern fn sys_fs_remove_file(ctx: i64, path: str) -> IoRes[bool]
extern fn sys_fs_remove_dir_all(ctx: i64, path: str) -> IoRes[bool]
extern fn sys_fs_rename(ctx: i64, src: str, dst: str) -> IoRes[bool]
extern fn sys_fs_copy_file(ctx: i64, src: str, dst: str) -> IoRes[bool]
extern fn sys_fs_read_text(ctx: i64, path: str) -> IoRes[str]
extern fn sys_fs_write_text(ctx: i64, path: str, text: str) -> IoRes[bool]
extern fn sys_fs_read_bytes(ctx: i64, path: str) -> IoRes[list[u8]]
extern fn sys_fs_write_bytes(ctx: i64, path: str, bytes: list[u8]) -> IoRes[bool]
extern fn sys_fs_mtime_ms(ctx: i64, path: str) -> IoRes[i64]
extern fn sys_fs_sha256_hex(ctx: i64, path: str) -> IoRes[str]
extern fn sys_fs_blake3_hex(ctx: i64, path: str) -> IoRes[str]

extern fn sys_proc_run(ctx: i64, argv: list[str], opt: ProcOptions) -> ProcRes[ProcResult]

extern fn sys_cache_has(ctx: i64, key: str) -> CacheRes[bool]
extern fn sys_cache_get(ctx: i64, key: str) -> CacheRes[list[u8]]
extern fn sys_cache_put(ctx: i64, key: str, value: list[u8]) -> CacheRes[bool]
extern fn sys_cache_remove(ctx: i64, key: str) -> CacheRes[bool]
extern fn sys_cache_meta(ctx: i64, key: str) -> CacheRes[CacheMeta]

extern fn sys_ws_find_root(ctx: i64, start: str) -> IoRes[str]
extern fn sys_ws_read_muf(ctx: i64, path: str) -> IoRes[str]
extern fn sys_ws_write_mcf(ctx: i64, path: str, text: str) -> IoRes[bool]
extern fn sys_ws_ensure_out_dir(ctx: i64, path: str) -> IoRes[bool]

extern fn sys_term_is_tty(ctx: i64) -> bool
extern fn sys_term_width(ctx: i64) -> i32
extern fn sys_term_ansi_mode(ctx: i64) -> AnsiMode