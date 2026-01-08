# ============================================================================
# Muffinfile — build.muf (MAX)
# Path: /build.muf
#
# Purpose:
# - Single entry build file for a Muffin workspace
# - Declares project metadata, targets, profiles, tool wiring
# - Defines bake recipes that generate:
#   - per-folder .muff configs
#   - Src/out/lib/*.va (static library artifacts)
#   - Src/out/bin/*.vo (object/compiled artifacts)
#   - and on Windows: Src/out/bin/*.exe
#
# Conventions used (doc-oriented):
# - All blocks end with .end
# - Paths are workspace-relative unless absolute
# - "auto" means resolved by Muffin at runtime
# ============================================================================

# ----------------------------------------------------------------------------
# 0) Workspace / project
# ----------------------------------------------------------------------------
project muffin
  version "0.1.0"
  description "Muffin build workspace for Vitte projects"
  license "MIT"

  # Root folders (override if your layout differs)
  dirs
    src_in  "Src/in"
    src_out "Src/out"
    toolchain "toolchain"
    docs "docs"
  .end

  # Default selections
  defaults
    target  "local"
    profile "debug"
  .end
.end

# ----------------------------------------------------------------------------
# 1) Global variables
# ----------------------------------------------------------------------------
vars
  # Resolve toolchain assets (response templates, etc.)
  set toolchain_assets = "{project.dirs.toolchain}/assets"

  # Common output dirs
  set out_root   = "{project.dirs.src_out}"
  set out_bin    = "{project.dirs.src_out}/bin"
  set out_lib    = "{project.dirs.src_out}/lib"
  set out_obj    = "{project.dirs.src_out}/obj"
  set out_cache  = "{project.dirs.src_out}/.cache"

  # Config emission: per-program folder
  set cfg_name   = ".muff"         # your per-directory config name
  set cfg_rel_in = "{project.dirs.src_in}"

  # Naming patterns (tokens resolved by Muffin)
  set name_lib   = "compilation_{folder}_generate_.va"
  set name_obj   = "compilation_{folder}_generate_.vo"
  set name_exe   = "compilation_{folder}_generate_.exe"
.end

# ----------------------------------------------------------------------------
# 2) Toolchain templates & tools
# ----------------------------------------------------------------------------
tools
  tool clang
    kind "cc"
    bin  "clang"
    rsp  "{vars.toolchain_assets}/reponse_files/clang.rsp.tmpl"
  .end

  tool ar
    kind "ar"
    bin  "ar"
    rsp  "{vars.toolchain_assets}/reponse_files/ar.rsp.tmpl"
  .end

  tool lld
    kind "ld"
    bin  "ld.lld"
    rsp  "{vars.toolchain_assets}/reponse_files/lld.rsp.tmpl"
  .end

  tool vittec
    kind "compiler"
    bin  "vittec"
    # Optional: pinned version, lookup path, etc.
    # version "0.1.x"
  .end
.end

# ----------------------------------------------------------------------------
# 3) Targets
# ----------------------------------------------------------------------------
targets
  # Host target (auto-detect)
  use "docs/examples/muf/advanced/targets/local.muf"
.end

# ----------------------------------------------------------------------------
# 4) Profiles
# ----------------------------------------------------------------------------
profiles
  profile debug
    opt "O0"
    debug "on"
    define "MUFFIN_DEBUG=1"
    cflags add "-g"
    cflags add "-Wall"
    cflags add "-Wextra"
    cflags add "-Wpedantic"
  .end

  profile release
    opt "O3"
    debug "off"
    define "NDEBUG=1"
    cflags add "-O3"
    cflags add "-DNDEBUG"
  .end
.end

# ----------------------------------------------------------------------------
# 5) Inputs discovery
# ----------------------------------------------------------------------------
discover
  # Programs are folders inside Src/in/*
  programs
    root "{vars.cfg_rel_in}"
    pattern "**/*"         # folder scan
    kind "folder"

    # A folder is considered a "program folder" if it contains one of these
    # (adapt to your actual source entry layout)
    entry_any
      file "Src/program/lib.vit"
      file "src/program/lib.vit"
      file "main.vit"
    .end
  .end
.end

# ----------------------------------------------------------------------------
# 6) Generation: per-folder .muff configuration
# ----------------------------------------------------------------------------
bake gen_config
  desc "Generate per-program .muff config in each Src/in/<folder>/"

  takes
    root "{vars.cfg_rel_in}"
    glob "**/*"
  .end

  emits
    # Emits a config file in each discovered folder
    file "{vars.cfg_rel_in}/{folder}/{vars.cfg_name}"
  .end

  do
    # Ensure output dirs exist (logical operation; actual mkdir depends on Muffin runtime)
    ensure_dir "{vars.out_root}"
    ensure_dir "{vars.out_bin}"
    ensure_dir "{vars.out_lib}"
    ensure_dir "{vars.out_obj}"
    ensure_dir "{vars.out_cache}"

    # Write config for this folder
    write_file "{vars.cfg_rel_in}/{folder}/{vars.cfg_name}"
      content """
# Auto-generated by Muffin (gen_config)
# Folder: {folder}

inputs
  # Core sources (example list from your description)
  add "Src/program/lib.vit"
  add "error.vit"
  add "read.vit"
  add "output.rs"
.end

outputs
  # Static library + compilation artifact
  lib "{vars.out_lib}/{vars.name_lib}"
  obj "{vars.out_bin}/{vars.name_obj}"

  # Windows executable optional (resolved by target.os)
  exe "{vars.out_bin}/{vars.name_exe}"
.end

meta
  target "{target}"
  profile "{profile}"
.end
"""
    .end
  .end
.end

# ----------------------------------------------------------------------------
# 7) Build: compile each folder to .vo + .va (+ .exe on Windows)
# ----------------------------------------------------------------------------
bake build_all
  desc "Compile all discovered program folders"

  depends
    bake "gen_config"
  .end

  takes
    root "{vars.cfg_rel_in}"
    # Treat each program folder as a unit
    glob "**/*"
  .end

  emits
    file "{vars.out_lib}/{vars.name_lib}"
    file "{vars.out_bin}/{vars.name_obj}"
    # Conditional exe on Windows
    file "{vars.out_bin}/{vars.name_exe}"
  .end

  do
    # 1) Compile Vitte sources into an intermediate object (.vo)
    run tool "{tools.vittec.bin}"
      args
        add "--target" "{target}"
        add "--profile" "{profile}"
        add "--in" "{vars.cfg_rel_in}/{folder}"
        add "--out" "{vars.out_bin}/{vars.name_obj}"
      .end
    .end

    # 2) Archive static library (.va) from the produced objects
    run tool "{tools.ar.bin}"
      args
        add "rcs"
        add "{vars.out_lib}/{vars.name_lib}"
        add "{vars.out_bin}/{vars.name_obj}"
      .end
    .end

    # 3) Link exe on Windows (optional)
    when os == "windows"
      run tool "{tools.lld.bin}"
        args
          add "-o"
          add "{vars.out_bin}/{vars.name_exe}"
          add "{vars.out_bin}/{vars.name_obj}"
          # add libs here if needed
        .end
      .end
    .end
  .end
.end

# ----------------------------------------------------------------------------
# 8) Convenience commands (aliases)
# ----------------------------------------------------------------------------
cmd build
  desc "Default build (debug/local)"
  run bake "build_all"
.end

cmd build_all
  desc "Build all programs (explicit)"
  run bake "build_all"
.end

cmd gen
  desc "Generate per-folder configs only"
  run bake "gen_config"
.end

cmd clean
  desc "Remove outputs"
  do
    rm_dir "{vars.out_root}"
  .end
.end

# ----------------------------------------------------------------------------
# 9) Suggested usage (docs)
# ----------------------------------------------------------------------------
# build muffin
# build muffin -all
# build muffin -debug
#
# Example overrides:
#   build muffin --target local --profile release
#   build muffin --target local --profile relwithdebinfo
# ----------------------------------------------------------------------------
