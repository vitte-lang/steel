# misc.vit — Muffin (Vitte) — MAX MAX
#
# Ultra-complete misc toolbox used everywhere in Muffin/Steel.
# -----------------------------------------------------------------------------
# Adds:
# - robust parsing helpers (int/bool)
# - stable hashing-friendly encoders
# - argv/env formatting
# - set-like operations on lists
# - lightweight glob matcher (single * and ?)
# - table formatting helpers
# - path ignore predicates (build/dist/tmp/cache/Steel)
#
# Blocks: .end only
# -----------------------------------------------------------------------------

mod muffin/misc

use std/string
use std/result
use muffin/directory

export all

# -----------------------------------------------------------------------------
# String basics
# -----------------------------------------------------------------------------

fn is_empty(s: str) -> bool ret string::len(s) == 0 .end
fn not_empty(s: str) -> bool ret string::len(s) != 0 .end
fn trim(s: str) -> str ret string::trim(s) .end
fn lower(s: str) -> str ret string::lower(s) .end
fn upper(s: str) -> str ret string::upper(s) .end
fn starts_with(s: str, p: str) -> bool ret string::starts_with(s, p) .end
fn ends_with(s: str, suf: str) -> bool ret string::ends_with(s, suf) .end
fn contains(s: str, sub: str) -> bool ret string::index_of(s, sub) >= 0 .end

fn last_index_of(s: str, sub: str) -> i32
  ret string::last_index_of(s, sub)
.end

fn replace_all(s: str, a: str, b: str) -> str
  # naive replace loop
  if a == "" ret s .end
  let mut out: str = ""
  let mut i: i32 = 0
  while i < string::len(s)
    let j: i32 = string::index_of_from(s, a, i)
    if j < 0
      out = out + string::slice(s, i, string::len(s))
      break
    .end
    out = out + string::slice(s, i, j) + b
    i = j + string::len(a)
  .end
  ret out
.end

fn split_ws(s: str) -> list[str]
  let mut out: list[str] = []
  let mut cur: str = ""
  let mut i: i32 = 0
  while i < string::len(s)
    let c: i32 = string::codepoint_at(s, i)
    i = i + 1
    if c == 32 || c == 9 || c == 10 || c == 13
      if string::len(cur) > 0
        out = out + [cur]
        cur = ""
      .end
    else
      cur = cur + string::from_codepoint(c)
    .end
  .end
  if string::len(cur) > 0 out = out + [cur] .end
  ret out
.end

fn split(s: str, sep: str) -> list[str]
  if sep == "" ret [s] .end
  let mut out: list[str] = []
  let mut i: i32 = 0
  while true
    let j: i32 = string::index_of_from(s, sep, i)
    if j < 0
      out = out + [string::slice(s, i, string::len(s))]
      break
    .end
    out = out + [string::slice(s, i, j)]
    i = j + string::len(sep)
  .end
  ret out
.end

fn join(xs: list[str], sep: str) -> str
  let mut out: str = ""
  let mut i: i32 = 0
  while i < len(xs)
    if i > 0 out = out + sep .end
    out = out + xs[i]
    i = i + 1
  .end
  ret out
.end

fn quote(s: str) -> str ret "\"" + s + "\"" .end

# minimal JSON escaping
fn escape_json(s: str) -> str
  let mut out: str = ""
  let mut i: i32 = 0
  while i < string::len(s)
    let c: i32 = string::codepoint_at(s, i)
    if c == 92 out = out + "\\\\"
    elif c == 34 out = out + "\\\""
    elif c == 10 out = out + "\\n"
    elif c == 13 out = out + "\\r"
    elif c == 9 out = out + "\\t"
    else out = out + string::from_codepoint(c)
    .end
    i = i + 1
  .end
  ret out
.end

fn escape_shell(s: str) -> str
  # single-quote strategy: ' -> '"'"'
  if string::len(s) == 0 ret "''" .end
  let mut out: str = "'"
  let mut i: i32 = 0
  while i < string::len(s)
    let c: i32 = string::codepoint_at(s, i)
    if c == 39
      out = out + "'\"'\"'"
    else
      out = out + string::from_codepoint(c)
    .end
    i = i + 1
  .end
  out = out + "'"
  ret out
.end

# -----------------------------------------------------------------------------
# Parsing helpers
# -----------------------------------------------------------------------------

fn parse_bool(s0: str) -> result::Result[bool, str]
  let s: str = lower(trim(s0))
  if s == "1" || s == "true" || s == "yes" || s == "on" ret result::Ok(true) .end
  if s == "0" || s == "false" || s == "no" || s == "off" ret result::Ok(false) .end
  ret result::Err("invalid bool: " + s0)
.end

fn parse_i32(s0: str) -> result::Result[i32, str]
  let s: str = trim(s0)
  if s == "" ret result::Err("empty int") .end
  let mut neg: bool = false
  let mut i: i32 = 0
  if string::starts_with(s, "-")
    neg = true
    i = 1
  .end
  let mut n: i32 = 0
  while i < string::len(s)
    let c: i32 = string::codepoint_at(s, i)
    if c < 48 || c > 57 ret result::Err("invalid int: " + s0) .end
    n = n * 10 + (c - 48)
    i = i + 1
  .end
  if neg n = -n .end
  ret result::Ok(n)
.end

fn parse_i64(s0: str) -> result::Result[i64, str]
  let s: str = trim(s0)
  if s == "" ret result::Err("empty int") .end
  let mut neg: bool = false
  let mut i: i32 = 0
  if string::starts_with(s, "-")
    neg = true
    i = 1
  .end
  let mut n: i64 = 0
  while i < string::len(s)
    let c: i32 = string::codepoint_at(s, i)
    if c < 48 || c > 57 ret result::Err("invalid int: " + s0) .end
    n = n * 10 + (c - 48) as i64
    i = i + 1
  .end
  if neg n = -n .end
  ret result::Ok(n)
.end

# -----------------------------------------------------------------------------
# Path helpers
# -----------------------------------------------------------------------------

fn norm_path(p: str) -> str ret directory::norm_path(p) .end
fn join_path(a: str, b: str) -> str ret directory::join_path(a, b) .end
fn parent_dir(p: str) -> str ret directory::parent_dir(p) .end
fn basename(p: str) -> str ret directory::basename(p) .end

fn ext(p: str) -> str
  let b: str = basename(p)
  let i: i32 = string::last_index_of(b, ".")
  if i < 0 ret "" .end
  ret string::slice(b, i, string::len(b))
.end

fn strip_ext(p: str) -> str
  let b: str = basename(p)
  let i: i32 = string::last_index_of(b, ".")
  if i < 0 ret p .end
  let dir: str = parent_dir(p)
  let stem: str = string::slice(b, 0, i)
  if dir == "" || dir == "." ret stem .end
  ret join_path(dir, stem)
.end

fn path_is_under(p: str, dir: str) -> bool
  let np: str = norm_path(p)
  let nd: str = norm_path(dir)
  if nd == "" ret false .end
  if string::starts_with(np, nd)
    # require boundary (/ or end)
    if string::len(np) == string::len(nd) ret true .end
    let c: i32 = string::codepoint_at(np, string::len(nd))
    if c == 47 || c == 92 ret true .end
  .end
  ret false
.end

fn path_ignore_default(root: str, p: str, build: str, dist: str, tmp: str, cache: str, steel: str) -> bool
  let pb: str = norm_path(join_path(root, build))
  let pd: str = norm_path(join_path(root, dist))
  let pt: str = norm_path(join_path(root, tmp))
  let pc: str = norm_path(join_path(root, cache))
  let ps: str = norm_path(join_path(root, steel))
  if path_is_under(p, pb) ret true .end
  if path_is_under(p, pd) ret true .end
  if path_is_under(p, pt) ret true .end
  if path_is_under(p, pc) ret true .end
  if path_is_under(p, ps) ret true .end
  ret false
.end

# -----------------------------------------------------------------------------
# List/set helpers
# -----------------------------------------------------------------------------

fn contains_str(xs: list[str], x: str) -> bool
  let mut i: i32 = 0
  while i < len(xs)
    if xs[i] == x ret true .end
    i = i + 1
  .end
  ret false
.end

fn filter_nonempty(xs: list[str]) -> list[str]
  let mut out: list[str] = []
  let mut i: i32 = 0
  while i < len(xs)
    let s: str = trim(xs[i])
    if s != "" out = out + [s] .end
    i = i + 1
  .end
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

fn sort_dedup(xs: list[str]) -> list[str]
  let a: list[str] = sort_str(filter_nonempty(xs))
  ret dedup_sorted(a)
.end

fn union_str(a: list[str], b: list[str]) -> list[str]
  ret sort_dedup(a + b)
.end

fn intersect_str(a: list[str], b: list[str]) -> list[str]
  let bb: list[str] = sort_dedup(b)
  let aa: list[str] = sort_dedup(a)
  let mut out: list[str] = []
  let mut i: i32 = 0
  while i < len(aa)
    if contains_str(bb, aa[i]) out = out + [aa[i]] .end
    i = i + 1
  .end
  ret out
.end

fn diff_str(a: list[str], b: list[str]) -> list[str]
  let bb: list[str] = sort_dedup(b)
  let aa: list[str] = sort_dedup(a)
  let mut out: list[str] = []
  let mut i: i32 = 0
  while i < len(aa)
    if !contains_str(bb, aa[i]) out = out + [aa[i]] .end
    i = i + 1
  .end
  ret out
.end

fn slice_str(xs: list[str], a: i32, b: i32) -> list[str]
  let mut out: list[str] = []
  let mut i: i32 = a
  while i < b
    out = out + [xs[i]]
    i = i + 1
  .end
  ret out
.end

# argv order-sensitive dedup
fn dedup_keep_order(xs: list[str]) -> list[str]
  let mut out: list[str] = []
  let mut seen: list[str] = []
  let mut i: i32 = 0
  while i < len(xs)
    let s: str = trim(xs[i])
    i = i + 1
    if s == "" continue .end
    if contains_str(seen, s) continue .end
    seen = seen + [s]
    out = out + [s]
  .end
  ret out
.end

# -----------------------------------------------------------------------------
# Map helpers (string map)
# -----------------------------------------------------------------------------

fn map_get_or(m: map[str, str], k: str, fallback: str) -> str
  if map_has_str(m, k) ret map_get_str(m, k) .end
  ret fallback
.end

fn map_sorted_keys(m: map[str, str]) -> list[str]
  ret sort_str(map_keys_str(m))
.end

fn map_merge(mut a: map[str, str], b: map[str, str]) -> map[str, str]
  let ks: list[str] = map_sorted_keys(b)
  let mut i: i32 = 0
  while i < len(ks)
    let k: str = ks[i]
    a = map_put_str(a, k, map_get_str(b, k))
    i = i + 1
  .end
  ret a
.end

fn map_to_env_lines(m: map[str, str]) -> list[str]
  let ks: list[str] = map_sorted_keys(m)
  let mut out: list[str] = []
  let mut i: i32 = 0
  while i < len(ks)
    let k: str = ks[i]
    out = out + [k + "=" + map_get_str(m, k)]
    i = i + 1
  .end
  ret out
.end

# -----------------------------------------------------------------------------
# Glob matcher (simple)
# -----------------------------------------------------------------------------
# Supports:
# - '*' matches any sequence
# - '?' matches one char
# No character classes. Deterministic DP.

fn glob_match(pattern: str, text: str) -> bool
  let p: str = pattern
  let t: str = text
  let pn: i32 = string::len(p)
  let tn: i32 = string::len(t)

  # dp[i][j] is not available; compress to two rows using maps
  let mut prev: list[bool] = bools(tn + 1, false)
  let mut cur: list[bool] = bools(tn + 1, false)

  prev[0] = true

  let mut i: i32 = 0
  while i < pn
    cur = bools(tn + 1, false)
    let pc: i32 = string::codepoint_at(p, i)

    # dp[i+1][0]
    if pc == 42 # '*'
      cur[0] = prev[0]
    else
      cur[0] = false
    .end

    let mut j: i32 = 0
    while j < tn
      let tc: i32 = string::codepoint_at(t, j)
      let v: bool =
        (pc == 42) ? (cur[j] || prev[j + 1]) :
        (pc == 63) ? prev[j] :
        (pc == tc) ? prev[j] :
        false
      cur[j + 1] = v
      j = j + 1
    .end

    prev = cur
    i = i + 1
  .end

  ret prev[tn]
.end

fn bools(n: i32, v: bool) -> list[bool]
  let mut out: list[bool] = []
  let mut i: i32 = 0
  while i < n
    out = out + [v]
    i = i + 1
  .end
  ret out
.end

# -----------------------------------------------------------------------------
# Human formatting
# -----------------------------------------------------------------------------

fn human_bytes(n: i64) -> str
  if n < 1024 ret i64_to_str(n) + " B" .end
  let kb: i64 = n / 1024
  if kb < 1024 ret i64_to_str(kb) + " KiB" .end
  let mb: i64 = kb / 1024
  if mb < 1024 ret i64_to_str(mb) + " MiB" .end
  let gb: i64 = mb / 1024
  ret i64_to_str(gb) + " GiB"
.end

fn human_ms(ms: i64) -> str
  if ms < 1000 ret i64_to_str(ms) + " ms" .end
  let s: i64 = ms / 1000
  if s < 60 ret i64_to_str(s) + " s" .end
  let m: i64 = s / 60
  let rem: i64 = s - m * 60
  ret i64_to_str(m) + " min " + i64_to_str(rem) + " s"
.end

# -----------------------------------------------------------------------------
# Stable encoders (hash/debug)
# -----------------------------------------------------------------------------

fn enc_str(s: str) -> str
  ret i32_to_str(string::len(s)) + ":" + s
.end

fn enc_list(xs: list[str]) -> str
  let mut out: str = "L" + i32_to_str(len(xs)) + ":"
  let mut i: i32 = 0
  while i < len(xs)
    out = out + enc_str(xs[i])
    i = i + 1
  .end
  ret out
.end

fn enc_map(m: map[str, str]) -> str
  let ks: list[str] = map_sorted_keys(m)
  let mut out: str = "M" + i32_to_str(len(ks)) + ":"
  let mut i: i32 = 0
  while i < len(ks)
    let k: str = ks[i]
    out = out + enc_str(k) + enc_str(map_get_str(m, k))
    i = i + 1
  .end
  ret out
.end

# -----------------------------------------------------------------------------
# Table formatting (for CLI)
# -----------------------------------------------------------------------------

fn pad_right(s: str, n: i32) -> str
  let m: i32 = string::len(s)
  if m >= n ret s .end
  let mut out: str = s
  let mut i: i32 = 0
  while i < (n - m)
    out = out + " "
    i = i + 1
  .end
  ret out
.end

fn render_table(headers: list[str], rows: list[list[str]]) -> str
  # compute widths
  let mut w: list[i32] = []
  let mut i: i32 = 0
  while i < len(headers)
    w = w + [string::len(headers[i])]
    i = i + 1
  .end

  let mut r: i32 = 0
  while r < len(rows)
    let row: list[str] = rows[r]
    let mut c: i32 = 0
    while c < len(row)
      if c < len(w)
        let n: i32 = string::len(row[c])
        if n > w[c] w[c] = n .end
      .end
      c = c + 1
    .end
    r = r + 1
  .end

  let mut out: str = ""
  # header
  i = 0
  while i < len(headers)
    out = out + pad_right(headers[i], w[i])
    if i + 1 < len(headers) out = out + "  " .end
    i = i + 1
  .end
  out = out + "\n"

  # underline
  i = 0
  while i < len(headers)
    out = out + pad_right(repeat("-", w[i]), w[i])
    if i + 1 < len(headers) out = out + "  " .end
    i = i + 1
  .end
  out = out + "\n"

  # rows
  r = 0
  while r < len(rows)
    let row: list[str] = rows[r]
    let mut c: i32 = 0
    while c < len(headers)
      let cell: str = (c < len(row)) ? row[c] : ""
      out = out + pad_right(cell, w[c])
      if c + 1 < len(headers) out = out + "  " .end
      c = c + 1
    .end
    out = out + "\n"
    r = r + 1
  .end

  ret out
.end

fn repeat(s: str, n: i32) -> str
  let mut out: str = ""
  let mut i: i32 = 0
  while i < n
    out = out + s
    i = i + 1
  .end
  ret out
.end

# -----------------------------------------------------------------------------
# Externs
# -----------------------------------------------------------------------------

extern fn len[T](xs: list[T]) -> i32
extern fn i32_to_str(x: i32) -> str
extern fn i64_to_str(x: i64) -> str

extern fn map_has_str(m: map[str, str], k: str) -> bool
extern fn map_get_str(m: map[str, str], k: str) -> str
extern fn map_put_str(m: map[str, str], k: str, v: str) -> map[str, str]
extern fn map_keys_str(m: map[str, str]) -> list[str]