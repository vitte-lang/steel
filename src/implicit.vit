# implicit.vit — Muffin (Vitte) — MAX MAX
#
# Implicit rules / inference / normalization / conventions
# -----------------------------------------------------------------------------
# But:
# - rendre "build muffin" et "build steel" opérationnels avec un minimum de config
# - déduire ce qui manque (profile, tool, cwd, outputs, deps implicites, file groups)
# - stabiliser la config résolue (tri, dedup, normalisation de chemins)
# - produire des diagnostics (warn/error) sans exécuter
#
# Pipeline recommandé:
#   parse -> resolve(refs) -> apply_defaults -> apply_implicit -> expand -> hash -> plan(deps) -> exec
#
# Conventions:
# - pas de { } ; blocs .end uniquement
# - déterministe: sort/dedup partout
# - “safe defaults”: pas de shell implicite sauf si cmd fourni explicitement
# -----------------------------------------------------------------------------

mod muffin/implicit

use std/string
use std/result
use muffin/config
use muffin/directory
use muffin/debug
use muffin/externs

export all

# -----------------------------------------------------------------------------
# Errors / Diagnostics
# -----------------------------------------------------------------------------

enum ImpErrKind
  MissingProfile
  MissingToolchain
  UnknownTool
  Invalid
.end

struct ImpError
  kind: ImpErrKind
  message: str
.end

type ImpRes[T] = result::Result[T, ImpError]

fn imp_err(kind: ImpErrKind, msg: str) -> ImpError
  ret ImpError(kind: kind, message: msg)
.end

enum ImpDiagLevel
  Note
  Warn
  Error
.end

struct ImpDiag
  level: ImpDiagLevel
  msg: str
.end

# -----------------------------------------------------------------------------
# Policy (tunable behaviors)
# -----------------------------------------------------------------------------

struct ImplicitPolicy
  # deps conventions
  inject_check_into_build: bool
  inject_build_into_test: bool
  inject_build_into_dist: bool
  inject_check_into_doc: bool

  # steps defaults
  default_cwd_root: bool
  allow_shell_from_cmd: bool          # if cmd != "" then tool becomes "sh"
  force_argv0_exe: bool               # ensure argv[0] is resolved exe for tool steps

  # outputs conventions
  default_build_outputs_dir: bool
  default_dist_outputs_dir: bool
  default_doc_outputs_dir: bool

  # file groups inference
  infer_groups: bool
  infer_src_globs: bool
  max_group_files: i32                # cap to avoid huge workspaces

  # normalize
  normalize_paths: bool
.end

fn policy_default() -> ImplicitPolicy
  ret ImplicitPolicy(
    inject_check_into_build: true,
    inject_build_into_test: true,
    inject_build_into_dist: true,
    inject_check_into_doc: true,

    default_cwd_root: true,
    allow_shell_from_cmd: true,
    force_argv0_exe: true,

    default_build_outputs_dir: true,
    default_dist_outputs_dir: true,
    default_doc_outputs_dir: true,

    infer_groups: true,
    infer_src_globs: true,
    max_group_files: 20000,

    normalize_paths: true
  )
.end

# -----------------------------------------------------------------------------
# Public API
# -----------------------------------------------------------------------------

struct ImplicitReport
  diags: list[ImpDiag]
  changed: bool
.end

fn apply_implicit(r0: Resolved) -> ImpRes[Resolved]
  let pol: ImplicitPolicy = policy_default()
  let rr: ImpRes[tuple[Resolved, ImplicitReport]] = apply_implicit_with(r0, pol)
  if result::is_err(rr) ret rr as Resolved .end
  ret result::Ok((result::unwrap(rr)).0)
.end

fn apply_implicit_with(r0: Resolved, pol: ImplicitPolicy) -> ImpRes[tuple[Resolved, ImplicitReport]]
  let mut r: Resolved = r0
  let mut rep: ImplicitReport = ImplicitReport(diags: [], changed: false)

  # 0) Ensure tool aliases exist (may be needed by step normalization)
  let r_prev_tools: i32 = len(r.tools)
  r = ensure_tool_aliases(r)
  if len(r.tools) != r_prev_tools rep.changed = true .end

  # 1) Selection/profile defaults
  let rr1: ImpRes[tuple[Resolved, bool]] = ensure_selected_profile(r)
  if result::is_err(rr1) ret rr1 as tuple[Resolved, ImplicitReport] .end
  let r1: tuple[Resolved, bool] = result::unwrap(rr1)
  r = r1.0
  if r1.1 rep.changed = true .end

  # 2) Toolchain default check
  let rr_tc: ImpRes[tuple[Resolved, bool]] = ensure_selected_toolchain(r)
  if result::is_err(rr_tc) ret rr_tc as tuple[Resolved, ImplicitReport] .end
  let r2: tuple[Resolved, bool] = result::unwrap(rr_tc)
  r = r2.0
  if r2.1 rep.changed = true .end

  # 3) Targets normalization + deps implicit
  let rr3: ImpRes[tuple[Resolved, ImplicitReport]] = implicit_targets(r, pol, rep)
  if result::is_err(rr3) ret rr3 .end
  let t3: tuple[Resolved, ImplicitReport] = result::unwrap(rr3)
  r = t3.0
  rep = t3.1

  # 4) Packages implicit
  let rr4: ImpRes[tuple[Resolved, ImplicitReport]] = implicit_packages(r, rep)
  if result::is_err(rr4) ret rr4 .end
  let t4: tuple[Resolved, ImplicitReport] = result::unwrap(rr4)
  r = t4.0
  rep = t4.1

  # 5) Steps implicit (cwd/tool/argv/env/io)
  let rr5: ImpRes[tuple[Resolved, ImplicitReport]] = implicit_steps(r, pol, rep)
  if result::is_err(rr5) ret rr5 .end
  let t5: tuple[Resolved, ImplicitReport] = result::unwrap(rr5)
  r = t5.0
  rep = t5.1

  # 6) File groups inference (optional)
  if pol.infer_groups
    let rr6: ImpRes[tuple[Resolved, ImplicitReport]] = implicit_groups(r, pol, rep)
    if result::is_err(rr6) ret rr6 .end
    let t6: tuple[Resolved, ImplicitReport] = result::unwrap(rr6)
    r = t6.0
    rep = t6.1
  .end

  # 7) Normalize: sort/dedup/paths
  let rr7: ImpRes[tuple[Resolved, bool]] = normalize_all(r, pol)
  if result::is_err(rr7) ret rr7 as tuple[Resolved, ImplicitReport] .end
  let t7: tuple[Resolved, bool] = result::unwrap(rr7)
  r = t7.0
  if t7.1 rep.changed = true .end

  ret result::Ok((r, rep))
.end

# -----------------------------------------------------------------------------
# Selection/profile/toolchain
# -----------------------------------------------------------------------------

fn ensure_selected_profile(mut r: Resolved) -> ImpRes[tuple[Resolved, bool]]
  let mut changed: bool = false
  if r.selection.profile == ""
    let i: i32 = find_profile(r.profiles, "debug")
    if i >= 0
      r.selection.profile = "debug"
      changed = true
    elif len(r.profiles) > 0
      r.selection.profile = r.profiles[0].name
      changed = true
    else
      ret result::Err(imp_err(ImpErrKind::MissingProfile, "no profile defined"))
    .end
  .end
  ret result::Ok((r, changed))
.end

fn ensure_selected_toolchain(mut r: Resolved) -> ImpRes[tuple[Resolved, bool]]
  let mut changed: bool = false
  if r.selection.toolchain == ""
    # prefer "vitte" if exists else first
    let i: i32 = find_toolchain(r.toolchains, "vitte")
    if i >= 0
      r.selection.toolchain = "vitte"
      changed = true
    elif len(r.toolchains) > 0
      r.selection.toolchain = r.toolchains[0].name
      changed = true
    else
      ret result::Err(imp_err(ImpErrKind::MissingToolchain, "no toolchain defined"))
    .end
  .end
  ret result::Ok((r, changed))
.end

fn selected_profile(r: Resolved, name: str) -> ImpRes[Profile]
  let p: str = (name != "") ? name : r.selection.profile
  let i: i32 = find_profile(r.profiles, p)
  if i < 0
    ret result::Err(imp_err(ImpErrKind::MissingProfile, "unknown profile: " + p))
  .end
  ret result::Ok(r.profiles[i])
.end

# -----------------------------------------------------------------------------
# Targets
# -----------------------------------------------------------------------------

fn implicit_targets(mut r: Resolved, pol: ImplicitPolicy, mut rep: ImplicitReport) -> ImpRes[tuple[Resolved, ImplicitReport]]
  let mut i: i32 = 0
  while i < len(r.targets)
    let mut t: Target = r.targets[i]
    let before: Target = t

    # default kind
    if t.kind == "" t.kind = "build" .end

    # default profile per-target
    if t.profile == "" t.profile = r.selection.profile .end

    # implicit deps conventions
    if pol.inject_check_into_build && t.name == "build"
      if !contains_str(t.deps, "check")
        t.deps = t.deps + ["check"]
        rep.changed = true
        rep.diags = rep.diags + [ImpDiag(level: ImpDiagLevel::Note, msg: "implicit dep: build -> check")]
      .end
    .end

    if pol.inject_build_into_test && t.name == "test"
      if !contains_str(t.deps, "build")
        t.deps = t.deps + ["build"]
        rep.changed = true
        rep.diags = rep.diags + [ImpDiag(level: ImpDiagLevel::Note, msg: "implicit dep: test -> build")]
      .end
    .end

    if pol.inject_build_into_dist && t.name == "dist"
      if !contains_str(t.deps, "build")
        t.deps = t.deps + ["build"]
        rep.changed = true
        rep.diags = rep.diags + [ImpDiag(level: ImpDiagLevel::Note, msg: "implicit dep: dist -> build")]
      .end
    .end

    if pol.inject_check_into_doc && t.name == "doc"
      if !contains_str(t.deps, "check")
        t.deps = t.deps + ["check"]
        rep.changed = true
        rep.diags = rep.diags + [ImpDiag(level: ImpDiagLevel::Note, msg: "implicit dep: doc -> check")]
      .end
    .end

    # Empty target -> phony
    if len(t.steps) == 0 && (t.kind == "build" || t.kind == "test" || t.kind == "doc" || t.kind == "dist")
      t.kind = "phony"
      rep.diags = rep.diags + [ImpDiag(level: ImpDiagLevel::Warn, msg: "target '" + t.name + "' has no steps; forcing kind=phony")]
      rep.changed = true
    .end

    # Default outputs dirs by target kind/name
    if pol.default_build_outputs_dir && (t.name == "build" || t.kind == "build")
      if len(t.outputs_dirs) == 0
        t.outputs_dirs = [join_path(r.paths.root, r.paths.build)]
        rep.changed = true
      .end
    .end
    if pol.default_dist_outputs_dir && (t.name == "dist" || t.kind == "dist")
      if len(t.outputs_dirs) == 0
        t.outputs_dirs = [join_path(r.paths.root, r.paths.dist)]
        rep.changed = true
      .end
    .end
    if pol.default_doc_outputs_dir && (t.name == "doc" || t.kind == "doc")
      if len(t.outputs_dirs) == 0
        t.outputs_dirs = [join_path(r.paths.root, r.paths.doc)]
        rep.changed = true
      .end
    .end

    # store
    r.targets[i] = t
    if !target_eq(before, t) rep.changed = true .end

    i = i + 1
  .end

  ret result::Ok((r, rep))
.end

fn target_eq(a: Target, b: Target) -> bool
  # minimal equality (fast heuristic)
  if a.name != b.name ret false .end
  if a.kind != b.kind ret false .end
  if a.profile != b.profile ret false .end
  if a.package != b.package ret false .end
  if len(a.deps) != len(b.deps) ret false .end
  if len(a.steps) != len(b.steps) ret false .end
  ret true
.end

# -----------------------------------------------------------------------------
# Packages
# -----------------------------------------------------------------------------

fn implicit_packages(mut r: Resolved, mut rep: ImplicitReport) -> ImpRes[tuple[Resolved, ImplicitReport]]
  # If targets reference packages that don't exist, create placeholders.
  let mut i: i32 = 0
  while i < len(r.targets)
    let t: Target = r.targets[i]
    i = i + 1
    if t.package == "" continue .end
    if find_package(r.packages, t.package) >= 0 continue .end

    debug::warn(debug::Cat::Resolve, "implicit package created: " + t.package)
    rep.diags = rep.diags + [ImpDiag(level: ImpDiagLevel::Warn, msg: "implicit package created: " + t.package)]
    rep.changed = true

    let p: Package = Package(
      name: t.package,
      version: "0.0.0",
      root: r.paths.root,
      deps: [],
      meta: map_new_str()
    )
    r.packages = r.packages + [p]
  .end

  r.packages = sort_packages(r.packages)
  ret result::Ok((r, rep))
.end

fn sort_packages(xs: list[Package]) -> list[Package]
  let mut a: list[Package] = xs
  let mut i: i32 = 0
  while i < len(a)
    let mut j: i32 = i + 1
    while j < len(a)
      if a[j].name < a[i].name
        let t: Package = a[i]
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
# Steps / Tools / Env
# -----------------------------------------------------------------------------

fn implicit_steps(mut r: Resolved, pol: ImplicitPolicy, mut rep: ImplicitReport) -> ImpRes[tuple[Resolved, ImplicitReport]]
  let mut i: i32 = 0
  while i < len(r.targets)
    let mut t: Target = r.targets[i]

    # validate profile exists
    let pi: i32 = find_profile(r.profiles, t.profile)
    if pi < 0
      ret result::Err(imp_err(ImpErrKind::MissingProfile, "target " + t.name + " refers to missing profile " + t.profile))
    .end
    let prof: Profile = r.profiles[pi]

    let mut sidx: i32 = 0
    while sidx < len(t.steps)
      let mut s: Step = t.steps[sidx]

      # default step name
      if s.name == "" s.name = t.name + ".step" + externs::i32_to_str(sidx) .end

      # default cwd
      if pol.default_cwd_root && s.cwd == ""
        s.cwd = r.paths.root
        rep.changed = true
      .end

      # cmd -> tool sh (only if allowed)
      if pol.allow_shell_from_cmd && s.tool == "" && s.cmd != ""
        let sh: str = resolve_tool(r, "sh")
        if sh == ""
          # fallback: literal "sh"
          s.tool = "sh"
          s.argv = ["sh", "-lc", s.cmd]
        else
          s.tool = "sh"
          s.argv = [sh, "-lc", s.cmd]
        .end
        s.cmd = ""
        rep.diags = rep.diags + [ImpDiag(level: ImpDiagLevel::Note, msg: "step '" + s.name + "': cmd -> sh -lc")]
        rep.changed = true
      .end

      # tool normalization
      if s.tool != ""
        let exe: str = resolve_tool(r, s.tool)
        if exe == ""
          ret result::Err(imp_err(ImpErrKind::UnknownTool, "unknown tool: " + s.tool + " (step " + s.name + ")"))
        .end

        if len(s.argv) == 0
          s.argv = [exe]
          rep.changed = true
        else
          if pol.force_argv0_exe
            s.argv[0] = exe
            rep.changed = true
          else
            # legacy placeholders
            if s.argv[0] == "{tool}" || s.argv[0] == "$tool"
              s.argv[0] = exe
              rep.changed = true
            .end
          .end
        .end
      .end

      # env: inject profile + target info (namespaced to avoid collision)
      s.env = map_put_str(s.env, "MUFFIN_PROFILE", prof.name)
      s.env = map_put_str(s.env, "MUFFIN_OPT", prof.opt)
      s.env = map_put_str(s.env, "MUFFIN_TARGET", t.name)
      s.env = map_put_str(s.env, "MUFFIN_KIND", t.kind)
      s.env = map_put_str(s.env, "MUFFIN_ROOT", r.paths.root)
      rep.changed = true

      # IO inference (lightweight):
      # - if target has inputs_* and step inputs empty => inherit
      if len(s.inputs) == 0
        s.inputs = sort_dedup(t.inputs_files + t.inputs_dirs)
      .end
      # - if target has outputs_* and step outputs empty => inherit
      if len(s.outputs) == 0
        s.outputs = sort_dedup(t.outputs_files + t.outputs_dirs)
      .end

      # normalize lists
      s.argv = sort_dedup_keep_order(s.argv)    # argv order is meaningful
      s.inputs = sort_dedup(s.inputs)
      s.outputs = sort_dedup(s.outputs)

      t.steps[sidx] = s
      sidx = sidx + 1
    .end

    r.targets[i] = t
    i = i + 1
  .end

  ret result::Ok((r, rep))
.end

fn resolve_tool(r: Resolved, name: str) -> str
  let mut i: i32 = 0
  while i < len(r.tools)
    if r.tools[i].name == name
      return r.tools[i].exe
    .end
    i = i + 1
  .end
  return ""
.end

# -----------------------------------------------------------------------------
# File groups inference
# -----------------------------------------------------------------------------

fn implicit_groups(mut r: Resolved, pol: ImplicitPolicy, mut rep: ImplicitReport) -> ImpRes[tuple[Resolved, ImplicitReport]]
  # If groups already exist, keep them; we only add missing conventional groups:
  # - src.vit, src.vitte, include, doc, tests (best effort)
  let root: str = r.paths.root

  # discover candidates (walk, then filter)
  let mut opt: directory::WalkOptions = directory::walk_options_default()
  opt.recursive = true
  opt.include_files = true
  opt.include_dirs = false
  opt.max_depth = 64

  let rr: directory::FsRes[list[str]] = directory::walk(root, opt)
  if result::is_err(rr)
    rep.diags = rep.diags + [ImpDiag(level: ImpDiagLevel::Warn, msg: "group inference skipped: walk failed")]
    return result::Ok((r, rep))
  .end

  let mut files: list[str] = result::unwrap(rr)
  if pol.max_group_files > 0 && len(files) > pol.max_group_files
    rep.diags = rep.diags + [ImpDiag(level: ImpDiagLevel::Warn, msg: "group inference capped: too many files (" + externs::i32_to_str(len(files)) + ")")]
    # keep first N deterministically (already sorted by directory.finalize_paths)
    files = slice_str(files, 0, pol.max_group_files)
  .end

  # compute group lists
  let src_exts: list[str] = [".vit", ".vitte"]
  let doc_exts: list[str] = [".md", ".txt", ".texi"]
  let test_markers: list[str] = ["/tests/", "/test/"]

  let mut g_src: list[str] = []
  let mut g_doc: list[str] = []
  let mut g_tests: list[str] = []

  let mut i: i32 = 0
  while i < len(files)
    let p: str = files[i]
    i = i + 1

    if pol.infer_src_globs && has_any_suffix(p, src_exts)
      # ignore build artifacts under build/dist/tmp/cache/Steel
      if is_generated_path(r, p) continue .end
      g_src = g_src + [p]
      continue
    .end

    if has_any_suffix(p, doc_exts)
      if is_generated_path(r, p) continue .end
      g_doc = g_doc + [p]
      continue
    .end

    if contains_any_substr(p, test_markers)
      if is_generated_path(r, p) continue .end
      g_tests = g_tests + [p]
      continue
    .end
  .end

  g_src = sort_dedup(g_src)
  g_doc = sort_dedup(g_doc)
  g_tests = sort_dedup(g_tests)

  # ensure groups exist/merged
  let before_n: i32 = len(r.file_groups)
  r = ensure_group(r, "src.files", g_src)
  r = ensure_group(r, "doc.files", g_doc)
  r = ensure_group(r, "tests.files", g_tests)
  if len(r.file_groups) != before_n rep.changed = true .end

  ret result::Ok((r, rep))
.end

fn is_generated_path(r: Resolved, p: str) -> bool
  let root: str = r.paths.root
  let build: str = directory::norm_path(directory::join_path(root, r.paths.build))
  let dist: str = directory::norm_path(directory::join_path(root, r.paths.dist))
  let tmp: str = directory::norm_path(directory::join_path(root, r.paths.tmp))
  let cache: str = directory::norm_path(directory::join_path(root, r.paths.cache))
  let steel: str = directory::norm_path(directory::join_path(root, r.paths.steel))

  let np: str = directory::norm_path(p)
  if string::starts_with(np, build) ret true .end
  if string::starts_with(np, dist) ret true .end
  if string::starts_with(np, tmp) ret true .end
  if string::starts_with(np, cache) ret true .end
  if string::starts_with(np, steel) ret true .end
  ret false
.end

fn ensure_group(mut r: Resolved, name: str, files: list[str]) -> Resolved
  let gi: i32 = find_group(r.file_groups, name)
  if gi < 0
    r.file_groups = r.file_groups + [FileGroup(name: name, files: files)]
    return r
  .end
  let mut g: FileGroup = r.file_groups[gi]
  g.files = sort_dedup(g.files + files)
  r.file_groups[gi] = g
  return r
.end

# -----------------------------------------------------------------------------
# Normalize all (stable ordering)
# -----------------------------------------------------------------------------

fn normalize_all(mut r: Resolved, pol: ImplicitPolicy) -> ImpRes[tuple[Resolved, bool]]
  let mut changed: bool = false

  # targets
  let mut i: i32 = 0
  while i < len(r.targets)
    let mut t: Target = r.targets[i]
    let before: Target = t

    if pol.normalize_paths
      t.inputs_files = norm_paths(t.inputs_files)
      t.inputs_dirs = norm_paths(t.inputs_dirs)
      t.outputs_files = norm_paths(t.outputs_files)
      t.outputs_dirs = norm_paths(t.outputs_dirs)
    .end

    t.deps = sort_dedup(t.deps)
    t.inputs_groups = sort_dedup(t.inputs_groups)
    t.inputs_files = sort_dedup(t.inputs_files)
    t.inputs_dirs = sort_dedup(t.inputs_dirs)
    t.outputs_files = sort_dedup(t.outputs_files)
    t.outputs_dirs = sort_dedup(t.outputs_dirs)

    # steps
    let mut sidx: i32 = 0
    while sidx < len(t.steps)
      let mut s: Step = t.steps[sidx]
      if pol.normalize_paths
        s.cwd = directory::norm_path(s.cwd)
        s.inputs = norm_paths(s.inputs)
        s.outputs = norm_paths(s.outputs)
      .end
      s.inputs = sort_dedup(s.inputs)
      s.outputs = sort_dedup(s.outputs)
      # argv order kept; only trim empties
      s.argv = trim_argv(s.argv)
      t.steps[sidx] = s
      sidx = sidx + 1
    .end

    r.targets[i] = t
    if !target_eq(before, t) changed = true .end
    i = i + 1
  .end

  # packages
  r.packages = sort_packages(r.packages)

  # groups
  let mut g: i32 = 0
  while g < len(r.file_groups)
    let mut fg: FileGroup = r.file_groups[g]
    if pol.normalize_paths
      fg.files = norm_paths(fg.files)
    .end
    fg.files = sort_dedup(fg.files)
    r.file_groups[g] = fg
    g = g + 1
  .end

  ret result::Ok((r, changed))
.end

# -----------------------------------------------------------------------------
# Helpers
# -----------------------------------------------------------------------------

fn contains_str(xs: list[str], x: str) -> bool
  let mut i: i32 = 0
  while i < len(xs)
    if xs[i] == x ret true .end
    i = i + 1
  .end
  ret false
.end

fn sort_dedup(xs: list[str]) -> list[str]
  let mut a: list[str] = []
  let mut i: i32 = 0
  while i < len(xs)
    let s: str = string::trim(xs[i])
    if s != "" a = a + [s] .end
    i = i + 1
  .end
  a = sort_str(a)
  a = dedup_sorted(a)
  ret a
.end

fn sort_dedup_keep_order(xs: list[str]) -> list[str]
  # argv is order-sensitive: keep first occurrence, trim empties
  let mut out: list[str] = []
  let mut seen: list[str] = []
  let mut i: i32 = 0
  while i < len(xs)
    let s: str = string::trim(xs[i])
    i = i + 1
    if s == "" continue .end
    if contains_str(seen, s) continue .end
    seen = seen + [s]
    out = out + [s]
  .end
  ret out
.end

fn trim_argv(xs: list[str]) -> list[str]
  let mut out: list[str] = []
  let mut i: i32 = 0
  while i < len(xs)
    let s: str = string::trim(xs[i])
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

fn norm_paths(xs: list[str]) -> list[str]
  let mut out: list[str] = []
  let mut i: i32 = 0
  while i < len(xs)
    out = out + [directory::norm_path(xs[i])]
    i = i + 1
  .end
  ret out
.end

fn has_any_suffix(s: str, sufs: list[str]) -> bool
  let mut i: i32 = 0
  while i < len(sufs)
    if string::ends_with(s, sufs[i]) ret true .end
    i = i + 1
  .end
  ret false
.end

fn contains_any_substr(s: str, subs: list[str]) -> bool
  let mut i: i32 = 0
  while i < len(subs)
    if string::index_of(s, subs[i]) >= 0 ret true .end
    i = i + 1
  .end
  ret false
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

# -----------------------------------------------------------------------------
# Externs
# -----------------------------------------------------------------------------

extern fn map_new_str() -> map[str, str]
extern fn map_put_str(m: map[str, str], k: str, v: str) -> map[str, str]
extern fn len[T](xs: list[T]) -> i32
