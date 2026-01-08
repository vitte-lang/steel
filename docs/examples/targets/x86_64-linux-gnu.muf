# ============================================================================
# Muffin Target — x86_64-linux-gnu.muf (MAX)
# Path: /docs/examples/targets/x86_64-linux-gnu.muf
#
# Purpose:
# - Example target definition for Linux x86_64 with GNU userland (glibc).
# - Shows GNU-oriented toolchain wiring (clang/ld.lld or gcc/ld), sysroot hooks,
#   baseline flags, and profile tuning.
#
# Notes:
# - This is documentation-oriented and may need adaptation to your schema.
# - All blocks end with .end
# ============================================================================

target x86_64-linux-gnu
  name "Linux (glibc) — x86_64"
  kind "cross"

  # Target identity
  triple "x86_64-unknown-linux-gnu"
  arch "x86_64"
  os "linux"
  abi "gnu"

  endian "little"
  pointer_width 64

  # ----------------------------------------------------------------------------
  # Platform specifics
  # ----------------------------------------------------------------------------
  linux
    # Optional sysroot: "" means use host sysroot
    sysroot ""

    # Optional: minimum kernel/GLIBC policy (informational unless driver enforces)
    min_kernel "4.14"
    libc "glibc"
  .end

  # ----------------------------------------------------------------------------
  # Toolchain wiring
  # ----------------------------------------------------------------------------
  toolchain
    # Default to clang toolchain; swap to gcc if you prefer
    cc "clang"
    cxx "clang++"
    ar "ar"
    ranlib "ranlib"
    ld "ld.lld"

    # Response templates (optional)
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
    root "Src/out"

    bin_dir   "Src/out/bin"
    lib_dir   "Src/out/lib"
    obj_dir   "Src/out/obj"
    cache_dir "Src/out/.cache"

    exe_ext ""       # ELF executables have no extension
    obj_ext ".o"
    lib_ext ".a"
    so_ext  ".so"

    symbols
      mode "inline"  # DWARF in binaries/objects
      dir  "Src/out/symbols"
    .end
  .end

  # ----------------------------------------------------------------------------
  # Default flags (base)
  # ----------------------------------------------------------------------------
  flags
    # clang target selection (useful when cross compiling)
    target
      add "-target"
      add "{triple}"
    .end

    # Optional sysroot
    sysroot
      when linux.sysroot != ""
      add "--sysroot={linux.sysroot}"
    .end

    # Baseline warnings + hardening
    warnings
      add "-Wall"
      add "-Wextra"
      add "-Wpedantic"
      add "-Wshadow"
      add "-Wconversion"
    .end

    hardening
      add "-fstack-protector-strong"
      add "-D_FORTIFY_SOURCE=2"
      add "-fno-plt"
    .end

    # Linker defaults (LLD)
    lld
      add "-fuse-ld=lld"
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

      cflags
        add "-g"
        add "-fno-omit-frame-pointer"
        add "{flags.target.*}"
        add "{flags.sysroot.*}"
        add "{flags.warnings.*}"
        add "{flags.hardening.*}"
      .end

      ldflags
        add "{flags.target.*}"
        add "{flags.sysroot.*}"
        add "{flags.lld.*}"
      .end
    .end

    profile release
      opt "O3"
      debug "off"
      lto "thin"
      sanitize "off"

      cflags
        add "-O3"
        add "-DNDEBUG"
        add "{flags.target.*}"
        add "{flags.sysroot.*}"
        add "{flags.warnings.*}"
        add "{flags.hardening.*}"
      .end

      ldflags
        add "-O3"
        add "{flags.target.*}"
        add "{flags.sysroot.*}"
        add "{flags.lld.*}"
      .end
    .end

    profile relwithdebinfo
      opt "O2"
      debug "on"
      lto "thin"
      sanitize "off"

      cflags
        add "-O2"
        add "-g"
        add "-DNDEBUG"
        add "{flags.target.*}"
        add "{flags.sysroot.*}"
        add "{flags.warnings.*}"
        add "{flags.hardening.*}"
      .end

      ldflags
        add "-O2"
        add "{flags.target.*}"
        add "{flags.sysroot.*}"
        add "{flags.lld.*}"
      .end
    .end
  .end

  # ----------------------------------------------------------------------------
  # Feature gates
  # ----------------------------------------------------------------------------
  features
    feature linux
      when os == "linux"
      define "MUFFIN_LINUX=1"
      define "MUFFIN_GNU=1"
      define "MUFFIN_X86_64=1"
    .end
  .end

  # ----------------------------------------------------------------------------
  # Cache policy (example)
  # ----------------------------------------------------------------------------
  cache
    enabled "on"
    key
      include "triple"
      include "linux.sysroot"
      include "profile"
      include "env:CC"
      include "env:CFLAGS"
      include "env:LDFLAGS"
    .end
  .end

.end
