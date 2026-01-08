# ============================================================================
# Muffinfile (example) — advanced/targets/local.muf
# Path: /docs/examples/muf/advanced/targets/local.muf
#
# Purpose:
# - Demonstrate an "advanced" local target definition for Muffin.
# - Show multi-profile flags, platform detection hooks, cache keys, and
#   host toolchain wiring (clang/ar/ld).
#
# Notes:
# - This is documentation-oriented: it shows structure + intent.
# - Adapt variable names to your actual Muffin schema if they differ.
# - All blocks end with .end
# ============================================================================

# ----------------------------------------------------------------------------
# target "local"
# ----------------------------------------------------------------------------
target local
  # Human-readable name
  name "Local (host)"
  kind "host"

  # Target triple-like identifier (host computed)
  # "auto" means: resolve at runtime from host OS/arch
  triple "auto"

  # CPU / ABI knobs (host computed unless overridden)
  arch "auto"          # x86_64 | aarch64 | riscv64 | ...
  os "auto"            # windows | linux | darwin | freebsd | ...
  abi "auto"           # gnu | musl | msvc | ...

  # Endianness (derived from arch, but can be pinned)
  endian "auto"        # little | big

  # ----------------------------------------------------------------------------
  # Toolchain wiring
  # ----------------------------------------------------------------------------
  toolchain
    cc "clang"
    cxx "clang++"
    ar "ar"
    ranlib "ranlib"
    ld "ld.lld"

    # Optional: explicit paths (useful in CI or non-standard installs)
    # cc_path "/usr/bin/clang"
    # ar_path "/usr/bin/ar"

    # Response file templates (Windows-friendly command lines)
    rsp
      cc "@{toolchain_assets}/response_files/clang.rsp.tmpl"
      ar "@{toolchain_assets}/response_files/ar.rsp.tmpl"
      ld "@{toolchain_assets}/response_files/lld.rsp.tmpl"
    .end
  .end

  # ----------------------------------------------------------------------------
  # Output conventions
  # ----------------------------------------------------------------------------
  output
    # Root output directory for this target
    root "Src/out"

    # Artifact layout
    bin_dir  "Src/out/bin"
    lib_dir  "Src/out/lib"
    obj_dir  "Src/out/obj"
    cache_dir "Src/out/.cache"

    # File extensions (host-dependent defaults)
    exe_ext  "auto"   # ".exe" on windows, "" elsewhere
    obj_ext  "auto"   # ".obj" on windows, ".o" elsewhere
    lib_ext  "auto"   # ".lib" on windows, ".a" elsewhere

    # Naming pattern (tokens: {project}, {module}, {profile}, {hash}, ...)
    name_pattern "{project}_{module}_{profile}"

    # Debug symbols policy
    symbols
      # "separate" => .pdb/.dSYM/etc. in symbols dir
      mode "auto"   # inline | separate | strip | auto
      dir  "Src/out/symbols"
    .end
  .end

  # ----------------------------------------------------------------------------
  # Profiles
  # ----------------------------------------------------------------------------
  profiles
    profile debug
      opt "O0"
      debug "on"
      lto "off"
      sanitize "off"
      define "MUFFIN_DEBUG=1"
      define "VITTE_ASSERT=1"

      cflags
        add "-g"
        add "-fno-omit-frame-pointer"
        add "-Wall"
        add "-Wextra"
        add "-Wpedantic"
      .end

      ldflags
        add "-g"
      .end
    .end

    profile release
      opt "O3"
      debug "off"
      lto "thin"
      sanitize "off"
      define "NDEBUG=1"

      cflags
        add "-O3"
        add "-DNDEBUG"
      .end

      ldflags
        add "-O3"
      .end
    .end

    profile relwithdebinfo
      opt "O2"
      debug "on"
      lto "thin"
      sanitize "off"
      define "NDEBUG=1"

      cflags
        add "-O2"
        add "-g"
        add "-DNDEBUG"
      .end
    .end
  .end

  # ----------------------------------------------------------------------------
  # Feature gates / platform switches
  # ----------------------------------------------------------------------------
  features
    # Portable baseline
    feature posix
      when os in ["linux","darwin","freebsd"]
      define "MUFFIN_POSIX=1"
      cflags add "-D_POSIX_C_SOURCE=200809L"
    .end

    feature windows
      when os == "windows"
      define "MUFFIN_WINDOWS=1"
      cflags add "-DWIN32_LEAN_AND_MEAN"
      cflags add "-DNOMINMAX"
    .end

    feature darwin
      when os == "darwin"
      define "MUFFIN_DARWIN=1"
      # Example: unify SDK root if needed
      # env "SDKROOT" "/Applications/Xcode.app/Contents/Developer/Platforms/MacOSX.platform/Developer/SDKs/MacOSX.sdk"
    .end
  .end

  # ----------------------------------------------------------------------------
  # Caching & determinism
  # ----------------------------------------------------------------------------
  cache
    enabled "on"
    # Strong cache key composition:
    # - tool versions
    # - target triple
    # - profile
    # - env overrides
    key
      include "toolchain.cc"
      include "toolchain.ld"
      include "triple"
      include "profile"
      include "env:SDKROOT"
      include "env:CC"
      include "env:CFLAGS"
    .end

    # Optional: remote cache (HTTP/S3/etc.)
    remote
      enabled "off"
      kind "http"
      url "https://cache.example.invalid/muffin"
      # auth "token"
    .end
  .end

  # ----------------------------------------------------------------------------
  # Environment overrides (opt-in)
  # ----------------------------------------------------------------------------
  env
    # Allow users to override tools via env
    allow_override "on"

    # Canonical env vars
    map "CC"     -> "toolchain.cc"
    map "CXX"    -> "toolchain.cxx"
    map "AR"     -> "toolchain.ar"
    map "RANLIB" -> "toolchain.ranlib"
    map "LD"     -> "toolchain.ld"

    map "CFLAGS"   -> "profiles.*.cflags"
    map "LDFLAGS"  -> "profiles.*.ldflags"
  .end

.end
