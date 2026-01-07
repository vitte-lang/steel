# hash.vit — Muffin (Vitte) — MAX
#
# Hashing / fingerprints / content digests:
# - stable hashing for paths + file content + argv + env
# - used for cache keys (Steel/cache) and incremental builds
# - algorithm negotiation: sha256 default, optionally blake3 if provided by runtime
#
# Design:
# - pure functions, deterministic
# - no hidden I/O unless explicitly requested (hash_file*)
# - canonical encoding for structured data (KV, lists)
#
# Blocks: .end only
# -----------------------------------------------------------------------------

mod muffin/hash

use std/string
use std/result
use muffin/externs

export all

# -----------------------------------------------------------------------------
# Errors
# -----------------------------------------------------------------------------

enum HashErrKind
  Io
  Algo
.end

struct HashError
  kind: HashErrKind
  message: str
  path: str
.end

type HashRes[T] = result::Result[T, HashError]

fn hash_err(kind: HashErrKind, msg: str, path: str) -> HashError
  ret HashError(kind: kind, message: msg, path: path)
.end

# -----------------------------------------------------------------------------
# Algorithm
# -----------------------------------------------------------------------------

enum HashAlgo
  Sha256
  Blake3
.end

fn algo_to_str(a: HashAlgo) -> str
  if a == HashAlgo::Sha256 ret "sha256" .end
  ret "blake3"
.end

fn parse_algo(s0: str) -> HashAlgo
  let s: str = string::lower(s0)
  if s == "blake3" || s == "b3" ret HashAlgo::Blake3 .end
  ret HashAlgo::Sha256
.end

fn default_algo() -> HashAlgo
  # env override
  let a: str = env_get("MUFFIN_HASH")
  if a != "" ret parse_algo(a) .end
  ret HashAlgo::Sha256
.end

# -----------------------------------------------------------------------------
# Digest type
# -----------------------------------------------------------------------------

struct Digest
  algo: HashAlgo
  hex: str
.end

fn digest(algo: HashAlgo, hex: str) -> Digest
  ret Digest(algo: algo, hex: hex)
.end

fn digest_str(d: Digest) -> str
  ret algo_to_str(d.algo) + ":" + d.hex
.end

# -----------------------------------------------------------------------------
# Canonical encoding helpers
# -----------------------------------------------------------------------------
# We encode data as UTF-8 text, with separators that are unambiguous:
# - items are length-prefixed: <len>:<bytes>
# - sequences start with "L" and maps start with "M"
# - map entries sorted by key, each entry: key + value (both length-prefixed)

fn enc_str(s: str) -> str
  ret externs::i32_to_str(string::len(s)) + ":" + s
.end

fn enc_bool(b: bool) -> str
  ret b ? "1" : "0"
.end

fn enc_i64(x: i64) -> str
  ret i64_to_str(x)
.end

fn enc_list_str(xs: list[str]) -> str
  let mut out: str = "L" + externs::i32_to_str(len(xs)) + ":"
  let mut i: i32 = 0
  while i < len(xs)
    out = out + enc_str(xs[i])
    i = i + 1
  .end
  ret out
.end

fn enc_map_str(m: map[str, str]) -> str
  # sort keys for stability
  let ks: list[str] = sort_str(map_keys_str(m))
  let mut out: str = "M" + externs::i32_to_str(len(ks)) + ":"
  let mut i: i32 = 0
  while i < len(ks)
    let k: str = ks[i]
    let v: str = map_get_str(m, k)
    out = out + enc_str(k) + enc_str(v)
    i = i + 1
  .end
  ret out
.end

# -----------------------------------------------------------------------------
# Core hashing (string -> digest)
# -----------------------------------------------------------------------------

fn hash_text(algo: HashAlgo, text: str) -> HashRes[Digest]
  if algo == HashAlgo::Sha256
    let rr: result::Result[str, str] = sys_sha256_hex(text)
    if result::is_err(rr)
      ret result::Err(hash_err(HashErrKind::Algo, "sha256 not available", ""))
    .end
    ret result::Ok(digest(algo, result::unwrap(rr)))
  .end

  # blake3
  let rr2: result::Result[str, str] = sys_blake3_hex(text)
  if result::is_err(rr2)
    ret result::Err(hash_err(HashErrKind::Algo, "blake3 not available", ""))
  .end
  ret result::Ok(digest(algo, result::unwrap(rr2)))
.end

fn hash_concat(algo: HashAlgo, parts: list[str]) -> HashRes[Digest]
  # join with canonical encoding (length-prefix to avoid collisions)
  let mut buf: str = "P" + externs::i32_to_str(len(parts)) + ":"
  let mut i: i32 = 0
  while i < len(parts)
    buf = buf + enc_str(parts[i])
    i = i + 1
  .end
  ret hash_text(algo, buf)
.end

# -----------------------------------------------------------------------------
# High-level fingerprints
# -----------------------------------------------------------------------------

fn hash_argv(algo: HashAlgo, argv: list[str]) -> HashRes[Digest]
  let payload: str = "argv:" + enc_list_str(argv)
  ret hash_text(algo, payload)
.end

fn hash_env_subset(algo: HashAlgo, keys: list[str]) -> HashRes[Digest]
  # keys sorted; missing env -> empty
  let ks: list[str] = sort_str(keys)
  let mut m: map[str, str] = map_new_str()
  let mut i: i32 = 0
  while i < len(ks)
    let k: str = ks[i]
    m = map_put_str(m, k, env_get(k))
    i = i + 1
  .end
  let payload: str = "env:" + enc_map_str(m)
  ret hash_text(algo, payload)
.end

fn hash_paths(algo: HashAlgo, paths: list[str]) -> HashRes[Digest]
  # normalize slash + sort
  let ps: list[str] = sort_str(norm_paths(paths))
  let payload: str = "paths:" + enc_list_str(ps)
  ret hash_text(algo, payload)
.end

fn hash_step_key(algo: HashAlgo, tool: str, argv: list[str], cwd: str, env: map[str, str], inputs: list[str], outputs: list[str]) -> HashRes[Digest]
  let payload: str =
    "step:" +
    enc_str(tool) +
    enc_str(cwd) +
    enc_list_str(argv) +
    enc_map_str(env) +
    enc_list_str(sort_str(norm_paths(inputs))) +
    enc_list_str(sort_str(norm_paths(outputs)))
  ret hash_text(algo, payload)
.end

fn hash_target_key(algo: HashAlgo, name: str, kind: str, deps: list[str], steps_keys: list[Digest]) -> HashRes[Digest]
  let mut sk: list[str] = []
  let mut i: i32 = 0
  while i < len(steps_keys)
    sk = sk + [digest_str(steps_keys[i])]
    i = i + 1
  .end
  sk = sort_str(sk)

  let payload: str =
    "target:" +
    enc_str(name) +
    enc_str(kind) +
    enc_list_str(sort_str(deps)) +
    enc_list_str(sk)

  ret hash_text(algo, payload)
.end

# -----------------------------------------------------------------------------
# File hashing
# -----------------------------------------------------------------------------

fn hash_file_hex(algo: HashAlgo, path: str) -> HashRes[Digest]
  if !fs_exists(path)
    ret result::Err(hash_err(HashErrKind::Io, "file not found", path))
  .end

  if algo == HashAlgo::Sha256
    let rr: result::Result[str, str] = fs_sha256_hex(path)
    if result::is_err(rr)
      ret result::Err(hash_err(HashErrKind::Io, "sha256 file failed", path))
    .end
    ret result::Ok(digest(algo, result::unwrap(rr)))
  .end

  let rr2: result::Result[str, str] = fs_blake3_hex(path)
  if result::is_err(rr2)
    ret result::Err(hash_err(HashErrKind::Algo, "blake3 file not available", path))
  .end
  ret result::Ok(digest(algo, result::unwrap(rr2)))
.end

fn hash_files_aggregate(algo: HashAlgo, paths: list[str]) -> HashRes[Digest]
  # stable: sort normalized paths; aggregate "path + digest"
  let ps: list[str] = sort_str(norm_paths(paths))
  let mut parts: list[str] = []
  let mut i: i32 = 0
  while i < len(ps)
    let p: str = ps[i]
    let d: HashRes[Digest] = hash_file_hex(algo, p)
    if result::is_err(d) ret d .end
    parts = parts + [enc_str(p) + enc_str(result::unwrap(d).hex)]
    i = i + 1
  .end
  ret hash_concat(algo, parts)
.end

# -----------------------------------------------------------------------------
# Utilities
# -----------------------------------------------------------------------------

fn norm_paths(xs: list[str]) -> list[str]
  let mut out: list[str] = []
  let mut i: i32 = 0
  while i < len(xs)
    out = out + [norm_path(xs[i])]
    i = i + 1
  .end
  ret out
.end

fn norm_path(p: str) -> str
  # slash normalize + trim trailing (except root)
  let mut out: str = ""
  let mut i: i32 = 0
  let mut prev_sep: bool = false
  while i < string::len(p)
    let c: i32 = string::codepoint_at(p, i)
    i = i + 1
    if c == 92 || c == 47
      if !prev_sep out = out + "/" .end
      prev_sep = true
    else
      out = out + string::from_codepoint(c)
      prev_sep = false
    .end
  .end
  while string::len(out) > 1 && string::ends_with(out, "/")
    out = string::slice(out, 0, string::len(out) - 1)
  .end
  if out == "" ret "." .end
  ret out
.end

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

extern fn env_get(name: str) -> str

extern fn fs_exists(path: str) -> bool
extern fn fs_sha256_hex(path: str) -> result::Result[str, str]
extern fn fs_blake3_hex(path: str) -> result::Result[str, str]

extern fn sys_sha256_hex(text: str) -> result::Result[str, str]
extern fn sys_blake3_hex(text: str) -> result::Result[str, str]

extern fn map_new_str() -> map[str, str]
extern fn map_put_str(m: map[str, str], k: str, v: str) -> map[str, str]
extern fn map_get_str(m: map[str, str], k: str) -> str
extern fn map_keys_str(m: map[str, str]) -> list[str]

extern fn i64_to_str(x: i64) -> str
extern fn len[T](xs: list[T]) -> i32
