

# Muffinfile / build.muf — advanced/targets/local.muf
#
# Objectif:
# - Exemple "MAX" orienté targets : Linux / macOS / Windows / BSD / Solaris
# - Un seul buildfile, configuration gelée en .mff (phase `build muffin`)
# - Construction (phase `Muffin build`) via tools déclarés
# - Conventions multi-langages (C/C++/C#/Rust/Vitte) par abstraction tool
#
# Usage (exemples):
#   build muffin                     # configure (plan par défaut)
#   build muffin -all                # configure tout ce qui est exportable
#   build muffin local               # configure plan local
#   build muffin -release            # force profile release
#   build muffin -D target=x86_64-unknown-linux-gnu
#   Muffin build                     # build depuis Muffinconfig.mff
#
# Introspection:
#   muffin decompile Muffinconfig.mff
#   muffin -why app.exe
#   muffin -graph --format dot

muffin bake 2

# ---------------------------------------------------------------------------
# Store (cache)
# ---------------------------------------------------------------------------
store local_cache
  path "./.muffin/cache"
  mode content
.end

# ---------------------------------------------------------------------------
# Capsule (sandbox)
# - Policy portable (l'enforcement dépend de l'OS)
# ---------------------------------------------------------------------------
capsule hermetic
  env allow ["PATH", "HOME", "TMP", "TEMP", "SystemRoot"]
  fs allow_read  ["./", "/usr", "/bin", "/lib", "/lib64", "/System", "C:/Windows"]
  fs allow_write ["./.muffin", "./.muff", "./build", "./out", "./target"]
  fs deny        ["../", "/etc/shadow", "C:/Windows/System32/config/SAM"]
  net deny
  time stable true
.end

# ---------------------------------------------------------------------------
# Variables globales (targets, conventions d'extensions, dossiers)
# ---------------------------------------------------------------------------
var workspace_root: text = "./"
var out_dir: text = "./out"
var build_dir: text = "./.muffin"
var target: text = "x86_64-unknown-linux-gnu"  # override via -D target=...

# Extensions logiques (résolues par target/profil)
var obj_ext: text = ".o"
var static_ext: text = ".a"
var shared_ext: text = ".so"
var exe_ext: text = ""          # ".exe" sous Windows

# Toolchain (override via -D cc=clang etc.)
var cc: text = "cc"
var cxx: text = "c++"
var ar: text = "ar"
var ld: text = "ld"
var rustc: text = "rustc"
var dotnet: text = "dotnet"
var vittec: text = "vittec"

# Options de compilation (communes)
var c_std: text = "c17"
var cxx_std: text = "c++20"
var c_flags: text = "-Wall -Wextra"
var cxx_flags: text = "-Wall -Wextra"
var ld_flags: text = ""

# ---------------------------------------------------------------------------
# Profiles (defaults + overrides)
# ---------------------------------------------------------------------------
profile debug
  set opt "0"
  set debug true
  set sanitize false
.end

profile release
  set opt "3"
  set debug false
  set sanitize false
.end

profile ci
  set opt "2"
  set debug false
  set sanitize true
.end

# ---------------------------------------------------------------------------
# Tools (déclaratifs)
# ---------------------------------------------------------------------------
# C compiler

tool cc
  exec "${cc}"
  sandbox true
  capsule hermetic
.end

# C++ compiler

tool cxx
  exec "${cxx}"
  sandbox true
  capsule hermetic
.end

# Archiver

tool ar
  exec "${ar}"
  sandbox true
  capsule hermetic
.end

# Linker

tool ld
  exec "${ld}"
  sandbox true
  capsule hermetic
.end

# Rust

tool rustc
  exec "${rustc}"
  sandbox true
  capsule hermetic
.end

# C# (ex: dotnet build) — illustratif

tool dotnet
  exec "${dotnet}"
  sandbox true
  capsule hermetic
.end

# Vitte compiler — illustratif

tool vittec
  exec "${vittec}"
  sandbox true
  capsule hermetic
.end

# ---------------------------------------------------------------------------
# Bakes (DAG)
# ---------------------------------------------------------------------------

# 1) Sources C (glob)

bake c_src
  out files: src.glob
  make files glob "src/c/**/*.c"
  cache content
.end

# 2) Sources C++ (glob)

bake cxx_src
  out files: src.glob
  make files glob "src/cpp/**/*.cpp"
  cache content
.end

# 3) Build objets C (résultat logique: out obj: bin.obj)
#    Note: ici, un seul artefact symbolique; dans un projet réel, on fan-out par fichier.

bake c_obj
  in  src: src.glob
  out obj: bin.obj

  run tool cc
    takes src as "--inputs"
    emits obj as "--out"
    set "--target" target
    set "--std" c_std
    set "--flags" c_flags
  .end

  cache content
  output obj at "${out_dir}/obj/c${obj_ext}"
.end

# 4) Build objets C++

bake cxx_obj
  in  src: src.glob
  out obj: bin.obj

  run tool cxx
    takes src as "--inputs"
    emits obj as "--out"
    set "--target" target
    set "--std" cxx_std
    set "--flags" cxx_flags
  .end

  cache content
  output obj at "${out_dir}/obj/cpp${obj_ext}"
.end

# 5) Archive statique (lib)

bake static_lib
  in  c:   bin.obj
  in  cpp: bin.obj
  out lib: lib.static

  run tool ar
    takes c   as "--in"
    takes cpp as "--in"
    emits lib as "--out"
    set "--mode" "static"
  .end

  cache content
  output lib at "${out_dir}/lib/libsample${static_ext}"
.end

# 6) Exécutable (link)

bake app
  in  lib: lib.static
  out exe: bin.exe

  run tool ld
    takes lib as "--in"
    emits exe as "--out"
    set "--target" target
    set "--flags" ld_flags
  .end

  cache content
  output exe at "${out_dir}/bin/app${exe_ext}"
.end

# 7) Rust (illustratif) — compile crate vers binaire

bake rust_app
  out exe: bin.exe

  run tool rustc
    emits exe as "--out"
    set "--target" target
    set "--crate" "bin"
    set "--manifest" "Cargo.toml"
    set "--profile" "${profile}"
  .end

  cache content
  output exe at "${out_dir}/bin/rust_app${exe_ext}"
.end

# 8) C# (illustratif) — build via dotnet

bake cs_app
  out exe: bin.exe

  run tool dotnet
    emits exe as "--out"
    set "--cmd" "build"
    set "--target" target
    set "--project" "src/cs/App.csproj"
    set "--configuration" "${profile}"
  .end

  cache content
  output exe at "${out_dir}/bin/cs_app${exe_ext}"
.end

# 9) Vitte (illustratif) — compile vers artefacts vitte

bake vitte_lib
  out lib: lib.static

  run tool vittec
    emits lib as "--out"
    set "--target" target
    set "--mode" "static"
    set "--src" "src/vitte"
  .end

  cache content
  output lib at "${out_dir}/lib/vitte_lib${static_ext}"
.end

# ---------------------------------------------------------------------------
# Wiring (connexion des outputs vers inputs)
# ---------------------------------------------------------------------------

wire c_src.files   -> c_obj.src
wire cxx_src.files -> cxx_obj.src

wire c_obj.obj   -> static_lib.c
wire cxx_obj.obj -> static_lib.cpp

wire static_lib.lib -> app.lib

# ---------------------------------------------------------------------------
# Exports (buildables / -all)
# ---------------------------------------------------------------------------

export app.exe
export static_lib.lib
export rust_app.exe
export cs_app.exe
export vitte_lib.lib

# ---------------------------------------------------------------------------
# Plans
# ---------------------------------------------------------------------------

plan default
  run exports
.end

plan local
  run app.exe
.end

plan all
  run exports
.end

# ---------------------------------------------------------------------------
# Switch (CLI mapping) — flags ergonomiques cross-platform
# ---------------------------------------------------------------------------

switch
  # profils
  flag "-debug"   set profile "debug"
  flag "-release" set profile "release"
  flag "-ci"      set profile "ci"

  # targets usuels
  flag "--linux-x64"    set target "x86_64-unknown-linux-gnu"
  flag "--linux-arm64"  set target "aarch64-unknown-linux-gnu"

  flag "--macos-x64"    set target "x86_64-apple-darwin"
  flag "--macos-arm64"  set target "aarch64-apple-darwin"

  flag "--win-x64-msvc" set target "x86_64-pc-windows-msvc"
  flag "--win-x64-gnu"  set target "x86_64-pc-windows-gnu"

  flag "--bsd-x64"      set target "x86_64-unknown-freebsd"
  flag "--solaris-x64"  set target "x86_64-unknown-solaris"

  # plans
  flag "-all"    set plan "all"
  flag "-local"  set plan "local"

  # introspection (exécution directe)
  flag "-graph"  run exports
.end

# ---------------------------------------------------------------------------
# Global setters (defaults)
# ---------------------------------------------------------------------------

set profile "debug"
set plan "default"

# ---------------------------------------------------------------------------
# Notes:
# - Les valeurs ${profile} et ${...} sont illustratives : si ton implémentation n'a pas
#   d'expansion de variables dans string_lit, utilise `-D` et des valeurs littérales.
# - Les tools (cc/ld/ar/rustc/dotnet/vittec) sont volontairement génériques.
# - Dans un vrai build, `c_obj` et `cxx_obj` sont généralement fan-out par fichier.
# - Les extensions (obj/lib/exe) peuvent être fixées via profile/target dans resolve.