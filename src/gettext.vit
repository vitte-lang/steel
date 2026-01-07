# gettext.vit — Muffin (Vitte)
#
# i18n / messages layer (gettext-like, but minimal):
# - load .po / .pot (subset) OR flat key=value catalog
# - plural rules (minimal: n != 1)
# - domain + locale selection
# - fallback chain: locale -> language -> "C"
#
# Rationale:
# - Muffin needs stable user-facing messages for CLI, diagnostics, docs.
# - Keep it contemporary and small, not a full gettext runtime.
#
# Integration points:
# - debug.vit can call gettext for localized headings if enabled.
# - commands.vit can choose locale from env LANG/LC_ALL.
#
# Blocks: .end only
# -----------------------------------------------------------------------------

mod muffin/gettext

use std/string
use std/result
use std/io

export all

# -----------------------------------------------------------------------------
# Errors
# -----------------------------------------------------------------------------

enum I18nErrKind
  Io
  Parse
.end

struct I18nError
  kind: I18nErrKind
  message: str
  file: str
  line: i32
.end

type I18nRes[T] = result::Result[T, I18nError]

fn i18n_err(kind: I18nErrKind, msg: str, file: str, line: i32) -> I18nError
  ret I18nError(kind: kind, message: msg, file: file, line: line)
.end

# -----------------------------------------------------------------------------
# Catalog model
# -----------------------------------------------------------------------------

struct Entry
  id: str
  str0: str            # singular translation
  str1: str            # plural translation (optional)
.end

struct Catalog
  locale: str
  domain: str
  entries: map[str, Entry]
.end

fn catalog_new(locale: str, domain: str) -> Catalog
  ret Catalog(locale: locale, domain: domain, entries: map_new_entry())
.end

fn put_entry(mut c: Catalog, e: Entry) -> Catalog
  c.entries = map_put_entry(c.entries, e.id, e)
  ret c
.end

fn has_entry(c: Catalog, id: str) -> bool
  ret map_has_entry(c.entries, id)
.end

fn get_entry(c: Catalog, id: str) -> Entry
  ret map_get_entry(c.entries, id)
.end

# -----------------------------------------------------------------------------
# Runtime context
# -----------------------------------------------------------------------------

struct I18n
  base_dir: str        # e.g. "i18n"
  domain: str          # e.g. "muffin"
  locale: str          # resolved locale, e.g. "fr_FR"
  fallback: list[str]  # ["fr_FR", "fr", "C"]
  cat: Catalog
  enabled: bool
.end

fn i18n_default() -> I18n
  ret I18n(
    base_dir: "i18n",
    domain: "muffin",
    locale: "C",
    fallback: ["C"],
    cat: catalog_new("C", "muffin"),
    enabled: false
  )
.end

fn detect_locale() -> str
  # order: LC_ALL > LC_MESSAGES > LANG
  let lc_all: str = env_get("LC_ALL")
  if lc_all != "" ret normalize_locale(lc_all) .end
  let lc_msg: str = env_get("LC_MESSAGES")
  if lc_msg != "" ret normalize_locale(lc_msg) .end
  let lang: str = env_get("LANG")
  if lang != "" ret normalize_locale(lang) .end
  ret "C"
.end

fn normalize_locale(raw: str) -> str
  # accept "fr_FR.UTF-8" -> "fr_FR"
  let s: str = raw
  let cut: i32 = index_of_any(s, [".", "@"])
  if cut >= 0
    ret string::slice(s, 0, cut)
  .end
  ret s
.end

fn build_fallback(locale: str) -> list[str]
  let loc: str = locale
  if loc == "" || loc == "C" || loc == "POSIX"
    ret ["C"]
  .end
  # "fr_FR" -> ["fr_FR","fr","C"]
  let lang: str = language_part(loc)
  if lang != "" && lang != loc
    ret [loc, lang, "C"]
  .end
  ret [loc, "C"]
.end

fn language_part(locale: str) -> str
  let i: i32 = string::index_of(locale, "_")
  if i < 0
    # "fr"
    return locale
  .end
  return string::slice(locale, 0, i)
.end

# -----------------------------------------------------------------------------
# Public API
# -----------------------------------------------------------------------------

fn init(base_dir: str, domain: str, locale0: str) -> I18n
  let mut i: I18n = i18n_default()
  i.base_dir = base_dir
  i.domain = domain

  let loc: str = (locale0 != "") ? normalize_locale(locale0) : detect_locale()
  i.locale = loc
  i.fallback = build_fallback(loc)

  let rr: I18nRes[Catalog] = load_best_catalog(i.base_dir, i.domain, i.fallback)
  if result::is_ok(rr)
    i.cat = result::unwrap(rr)
    i.enabled = true
  else
    # no catalog -> disabled (fallback to msgid)
    i.enabled = false
  .end

  ret i
.end

fn tr(i: I18n, msgid: str) -> str
  if !i.enabled ret msgid .end
  if has_entry(i.cat, msgid)
    let e: Entry = get_entry(i.cat, msgid)
    if e.str0 != "" ret e.str0 .end
  .end
  ret msgid
.end

fn trn(i: I18n, msgid_singular: str, msgid_plural: str, n: i64) -> str
  if !i.enabled
    ret (n == 1) ? msgid_singular : msgid_plural
  .end

  # entry key: singular msgid
  if has_entry(i.cat, msgid_singular)
    let e: Entry = get_entry(i.cat, msgid_singular)
    if n == 1
      if e.str0 != "" ret e.str0 .end
      ret msgid_singular
    .end
    if e.str1 != "" ret e.str1 .end
    # if no plural translation, fallback to plural msgid
    ret msgid_plural
  .end

  ret (n == 1) ? msgid_singular : msgid_plural
.end

# -----------------------------------------------------------------------------
# Loading catalogs
# -----------------------------------------------------------------------------
# Layout:
#   {base_dir}/{locale}/{domain}.po
#   {base_dir}/{locale}/{domain}.cat   (flat key=value)
#
# choose first existing in fallback list.

fn load_best_catalog(base_dir: str, domain: str, locales: list[str]) -> I18nRes[Catalog]
  let mut i: i32 = 0
  while i < len(locales)
    let loc: str = locales[i]
    i = i + 1

    let p1: str = join_path(join_path(base_dir, loc), domain + ".po")
    if fs_exists(p1)
      let rr: I18nRes[Catalog] = load_po(p1, loc, domain)
      if result::is_ok(rr) ret rr .end
    .end

    let p2: str = join_path(join_path(base_dir, loc), domain + ".cat")
    if fs_exists(p2)
      let rr2: I18nRes[Catalog] = load_cat(p2, loc, domain)
      if result::is_ok(rr2) ret rr2 .end
    .end
  .end

  ret result::Err(i18n_err(I18nErrKind::Io, "no catalog found", "", 0))
.end

# -----------------------------------------------------------------------------
# .cat format: key=value (UTF-8), '#' comment
# -----------------------------------------------------------------------------

fn load_cat(path: str, locale: str, domain: str) -> I18nRes[Catalog]
  let rr: result::Result[str, str] = fs_read_text(path)
  if result::is_err(rr)
    ret result::Err(i18n_err(I18nErrKind::Io, "read failed", path, 0))
  .end
  let txt: str = result::unwrap(rr)

  let mut c: Catalog = catalog_new(locale, domain)
  let lines: list[str] = string::split_lines(txt)

  let mut ln: i32 = 0
  while ln < len(lines)
    let raw: str = lines[ln]
    let line_no: i32 = ln + 1
    ln = ln + 1

    let s: str = string::trim(raw)
    if s == "" continue .end
    if string::starts_with(s, "#") continue .end

    let eq: i32 = string::index_of(s, "=")
    if eq < 0
      ret result::Err(i18n_err(I18nErrKind::Parse, "expected key=value", path, line_no))
    .end

    let k: str = string::trim(string::slice(s, 0, eq))
    let v: str = string::trim(string::slice(s, eq + 1, string::len(s)))
    c = put_entry(c, Entry(id: k, str0: unescape_cat(v), str1: ""))
  .end

  ret result::Ok(c)
.end

fn unescape_cat(v: str) -> str
  # supports \n, \t, \", \\
  let mut out: str = ""
  let mut i: i32 = 0
  while i < string::len(v)
    let c: i32 = string::codepoint_at(v, i)
    if c == 92 && i + 1 < string::len(v)
      let e: i32 = string::codepoint_at(v, i + 1)
      if e == 110 out = out + "\n"
      elif e == 116 out = out + "\t"
      elif e == 34 out = out + "\""
      elif e == 92 out = out + "\\"
      else out = out + string::from_codepoint(e)
      .end
      i = i + 2
      continue
    .end
    out = out + string::from_codepoint(c)
    i = i + 1
  .end
  ret out
.end

# -----------------------------------------------------------------------------
# .po subset loader
# -----------------------------------------------------------------------------
# Supported subset:
#   msgid "..."
#   msgid_plural "..."
#   msgstr "..."
#   msgstr[0] "..."
#   msgstr[1] "..."
# Continuation lines: "..." appended
# Comments ignored.
#
# No contexts, no flags.

fn load_po(path: str, locale: str, domain: str) -> I18nRes[Catalog]
  let rr: result::Result[str, str] = fs_read_text(path)
  if result::is_err(rr)
    ret result::Err(i18n_err(I18nErrKind::Io, "read failed", path, 0))
  .end
  let txt: str = result::unwrap(rr)
  let lines: list[str] = string::split_lines(txt)

  let mut c: Catalog = catalog_new(locale, domain)

  let mut cur_id: str = ""
  let mut cur_pl: str = ""
  let mut cur_s0: str = ""
  let mut cur_s1: str = ""

  let mut mode: str = "" # "id" "pl" "s0" "s1"

  fn flush(mut c: Catalog, id: str, s0: str, s1: str) -> Catalog
    if id == "" return c .end
    return put_entry(c, Entry(id: id, str0: s0, str1: s1))
  .end

  let mut ln: i32 = 0
  while ln < len(lines)
    let raw: str = lines[ln]
    let line_no: i32 = ln + 1
    ln = ln + 1

    let s: str = string::trim(raw)
    if s == "" 
      # blank -> end of entry
      c = flush(c, cur_id, cur_s0, cur_s1)
      cur_id = ""
      cur_pl = ""
      cur_s0 = ""
      cur_s1 = ""
      mode = ""
      continue
    .end
    if string::starts_with(s, "#")
      continue
    .end

    if string::starts_with(s, "msgid ")
      # flush previous
      c = flush(c, cur_id, cur_s0, cur_s1)
      cur_id = parse_po_quoted(path, line_no, string::slice(s, 6, string::len(s)))?
      cur_pl = ""
      cur_s0 = ""
      cur_s1 = ""
      mode = "id"
      continue
    .end

    if string::starts_with(s, "msgid_plural ")
      cur_pl = parse_po_quoted(path, line_no, string::slice(s, 13, string::len(s)))?
      mode = "pl"
      continue
    .end

    if string::starts_with(s, "msgstr ")
      cur_s0 = parse_po_quoted(path, line_no, string::slice(s, 7, string::len(s)))?
      mode = "s0"
      continue
    .end

    if string::starts_with(s, "msgstr[0] ")
      cur_s0 = parse_po_quoted(path, line_no, string::slice(s, 10, string::len(s)))?
      mode = "s0"
      continue
    .end

    if string::starts_with(s, "msgstr[1] ")
      cur_s1 = parse_po_quoted(path, line_no, string::slice(s, 10, string::len(s)))?
      mode = "s1"
      continue
    .end

    # continuation line: "..."
    if string::starts_with(s, "\"")
      let frag: str = parse_po_quoted(path, line_no, s)?
      if mode == "id" cur_id = cur_id + frag
      elif mode == "pl" cur_pl = cur_pl + frag
      elif mode == "s0" cur_s0 = cur_s0 + frag
      elif mode == "s1" cur_s1 = cur_s1 + frag
      .end
      continue
    .end

    # unknown line
    ret result::Err(i18n_err(I18nErrKind::Parse, "unknown .po directive", path, line_no))
  .end

  c = flush(c, cur_id, cur_s0, cur_s1)
  ret result::Ok(c)
.end

# Vitte doesn't have "?" operator in our canonical phrase set, so we implement as helper returning Res.
fn parse_po_quoted(path: str, line: i32, s: str) -> I18nRes[str]
  let t: str = string::trim(s)
  if string::len(t) < 2 || !string::starts_with(t, "\"") || !string::ends_with(t, "\"")
    ret result::Err(i18n_err(I18nErrKind::Parse, "expected quoted string", path, line))
  .end
  let inner: str = string::slice(t, 1, string::len(t) - 1)
  ret result::Ok(unescape_po(inner))
.end

fn unescape_po(v: str) -> str
  let mut out: str = ""
  let mut i: i32 = 0
  while i < string::len(v)
    let c: i32 = string::codepoint_at(v, i)
    if c == 92 && i + 1 < string::len(v)
      let e: i32 = string::codepoint_at(v, i + 1)
      if e == 110 out = out + "\n"
      elif e == 116 out = out + "\t"
      elif e == 34 out = out + "\""
      elif e == 92 out = out + "\\"
      else out = out + string::from_codepoint(e)
      .end
      i = i + 2
      continue
    .end
    out = out + string::from_codepoint(c)
    i = i + 1
  .end
  ret out
.end

# -----------------------------------------------------------------------------
# Small string/path helpers
# -----------------------------------------------------------------------------

fn index_of_any(s: str, chars: list[str]) -> i32
  let mut best: i32 = -1
  let mut i: i32 = 0
  while i < len(chars)
    let c: str = chars[i]
    i = i + 1
    let j: i32 = string::index_of(s, c)
    if j >= 0
      if best < 0 || j < best best = j .end
    .end
  .end
  ret best
.end

fn join_path(a: str, b: str) -> str
  if a == "" ret b .end
  if b == "" ret a .end
  if string::ends_with(a, "/") ret a + b .end
  ret a + "/" + b
.end

# -----------------------------------------------------------------------------
# Externs
# -----------------------------------------------------------------------------

extern fn env_get(name: str) -> str

extern fn fs_exists(path: str) -> bool
extern fn fs_read_text(path: str) -> result::Result[str, str]

extern fn map_new_entry() -> map[str, Entry]
extern fn map_put_entry(m: map[str, Entry], k: str, v: Entry) -> map[str, Entry]
extern fn map_get_entry(m: map[str, Entry], k: str) -> Entry
extern fn map_has_entry(m: map[str, Entry], k: str) -> bool

extern fn len[T](xs: list[T]) -> i32