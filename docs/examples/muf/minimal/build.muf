# ============================================================================
# Minimal Muffinfile — build.muf
# Path: /docs/examples/muf/minimal/build.muf
#
# Goal:
# - Smallest reasonable MUF example that still shows the core concepts:
#   - project metadata
#   - target/profile selection
#   - one bake recipe (build)
#   - one optional clean command
#
# Assumptions (adapt if your repo differs):
# - Sources live in: Src/in/app/
# - Entry file: Src/in/app/main.vit
# - Outputs in: Src/out/bin/
#
# Blocks end with .end
# ============================================================================

project app
  version "0.1.0"
  description "Minimal Muffin MUF example"
  license "MIT"

  dirs
    src_in  "Src/in"
    src_out "Src/out"
  .end

  defaults
    target  "local"
    profile "debug"
  .end
.end

# ----------------------------------------------------------------------------
# Tools (minimal)
# ----------------------------------------------------------------------------
tools
  tool vittec
    kind "compiler"
    bin  "vittec"
  .end
.end

# ----------------------------------------------------------------------------
# Targets (minimal: host)
# ----------------------------------------------------------------------------
targets
  target local
    kind "host"
    triple "auto"
    arch "auto"
    os "auto"
    abi "auto"
  .end
.end

# ----------------------------------------------------------------------------
# Profiles (minimal)
# ----------------------------------------------------------------------------
profiles
  profile debug
    opt "O0"
    debug "on"
    define "MUFFIN_DEBUG=1"
  .end

  profile release
    opt "O3"
    debug "off"
    define "NDEBUG=1"
  .end
.end

# ----------------------------------------------------------------------------
# Build (single recipe)
# ----------------------------------------------------------------------------
bake build
  desc "Compile the minimal app"

  takes
    file "{project.dirs.src_in}/app/main.vit"
  .end

  emits
    file "{project.dirs.src_out}/bin/app.vo"
  .end

  do
    ensure_dir "{project.dirs.src_out}/bin"

    run tool "{tools.vittec.bin}"
      args
        add "--target" "{target}"
        add "--profile" "{profile}"
        add "--in" "{project.dirs.src_in}/app"
        add "--out" "{project.dirs.src_out}/bin/app.vo"
      .end
    .end
  .end
.end

# ----------------------------------------------------------------------------
# Convenience commands
# ----------------------------------------------------------------------------
cmd build
  desc "Default build"
  run bake "build"
.end

cmd clean
  desc "Remove outputs"
  do
    rm_dir "{project.dirs.src_out}"
  .end
.end
