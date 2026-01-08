# ============================================================================
# Muffin Target — aarch64-darwin.muf (MAX)
# Path: /docs/examples/targets/aarch64-darwin.muf
#
# Purpose:
# - Example target definition for macOS on Apple Silicon (aarch64/arm64).
# - Shows Darwin-specific toolchain wiring, sysroot, deployment target, and
#   common flags for clang/ld.
#
# Notes:
# - This is documentation-oriented and may need adaptation to your schema.
# - All blocks end with .end
# ============================================================================

target aarch64-darwin
  name "macOS (Apple Silicon) — aarch64"
  kind "cross"

  # Target identity
  triple "aarch64-apple-darwin"
  arch "aarch64"
  os "darwin"
  abi "apple"

  endian "little"
  pointer_width 64

  # ----------------------------------------------------------------------------
  # Platform specifics
  # ----------------------------------------------------------------------------
  darwin
    # Minimum supported macOS version (used for -mmacosx-version-min / LC_VERSION_MIN)
    deployment_target "11.0"

    # SDK resolution:
    # - "auto" means driver uses xcrun --sdk macosx --show-sdk-path
    # - Or set an absolute path
    sdkroot "auto"

    # Common SDK name
    sdk "macosx"

    # Codesign policy (optional)
    codesign
      enabled "off"
      identity ""
      entitlements ""
    .end
  .end

  # ----------------------------------------------------------------------------
  # Toolchain wiring
  # ----------------------------------------------------------------------------
  toolchain
    # Prefer xcrun on macOS to locate correct clang/ar/ld
    cc "xcrun"
    cxx "xcrun"
    ar "xcrun"
    ranlib "xcrun"
    ld "xcrun"

    # Resolve actual tool names via xcrun
    xcrun
      sdk "{darwin.sdk}"
      cc  "clang"
      cxx "clang++"
      ar  "ar"
      ranlib "ranlib"
      ld  "ld"       # Apple ld (or use "ld.lld" if you ship LLVM)
    .end

    # Response templates (optional; can be shared with host)
    rsp
      cc "@{toolchain_assets}/response_files/clang.rsp.tmpl"
      ar "@{toolchain_assets}/response_files/ar.rsp.tmpl"
      ld "@{toolchain_assets}/response_files/ld64.rsp.tmpl"
    .end
  .end

  # ----------------------------------------------------------------------------
  # Output conventions
  # ----------------------------------------------------------------------------
  output
    root "Src/out"

    bin_dir   "Src/out/bin"
    lib_dir   "Src/out/lib"
    obj_dir   "Src/out/obj"
    cache_dir "Src/out/.cache"

    exe_ext ""      # Mach-O executables have no extension
    obj_ext ".o"
    lib_ext ".a"
    dylib_ext ".dylib"

    symbols
      mode "separate"           # dSYM bundle generation (if enabled by driver)
      dir  "Src/out/symbols"
    .end
  .end

  # ----------------------------------------------------------------------------
  # Default flags (base)
  # ----------------------------------------------------------------------------
  flags
    # clang target selection
    target
      # Example expansion used by driver:
      #   -target aarch64-apple-darwin
      add "-target"
      add "{triple}"
    .end

    # sysroot flags (driver expands sdkroot auto)
    sysroot
      when darwin.sdkroot != ""
      add "-isysroot"
      add "{darwin.sdkroot}"
    .end

    # deployment target
    deployment
      add "-mmacosx-version-min={darwin.deployment_target}"
    .end

    # baseline warnings
    warnings
      add "-Wall"
      add "-Wextra"
      add "-Wpedantic"
    .end
  .end

  # ----------------------------------------------------------------------------
  # Profiles (Darwin overrides)
  # ----------------------------------------------------------------------------
  profiles
    profile debug
      opt "O0"
      debug "on"
      lto "off"

      cflags
        add "-g"
        add "-fno-omit-frame-pointer"
        add "{flags.target.*}"
        add "{flags.sysroot.*}"
        add "{flags.deployment.*}"
        add "{flags.warnings.*}"
      .end

      ldflags
        add "{flags.target.*}"
        add "{flags.sysroot.*}"
        add "{flags.deployment.*}"
      .end
    .end

    profile release
      opt "O3"
      debug "off"
      lto "thin"

      cflags
        add "-O3"
        add "-DNDEBUG"
        add "{flags.target.*}"
        add "{flags.sysroot.*}"
        add "{flags.deployment.*}"
        add "{flags.warnings.*}"
      .end

      ldflags
        add "-O3"
        add "{flags.target.*}"
        add "{flags.sysroot.*}"
        add "{flags.deployment.*}"
      .end
    .end

    profile relwithdebinfo
      opt "O2"
      debug "on"
      lto "thin"

      cflags
        add "-O2"
        add "-g"
        add "-DNDEBUG"
        add "{flags.target.*}"
        add "{flags.sysroot.*}"
        add "{flags.deployment.*}"
        add "{flags.warnings.*}"
      .end

      ldflags
        add "-O2"
        add "{flags.target.*}"
        add "{flags.sysroot.*}"
        add "{flags.deployment.*}"
      .end
    .end
  .end

  # ----------------------------------------------------------------------------
  # Feature gates
  # ----------------------------------------------------------------------------
  features
    feature darwin
      when os == "darwin"
      define "MUFFIN_DARWIN=1"
      define "MUFFIN_APPLE=1"
      define "MUFFIN_AARCH64=1"
    .end
  .end

  # ----------------------------------------------------------------------------
  # Cache policy (example)
  # ----------------------------------------------------------------------------
  cache
    enabled "on"
    key
      include "triple"
      include "darwin.deployment_target"
      include "darwin.sdkroot"
      include "profile"
      include "env:SDKROOT"
      include "env:DEVELOPER_DIR"
    .end
  .end

.end
