# Muffinconfig.mcf (exemple)
#
# Fichier généré par `build muffin`.
# Objectif: représenter une configuration *résolue* et stable, directement consommable par le runner.
#
# Propriétés attendues:
# - stable/déterministe (ordre trié, chemins normalisés)
# - explicite (aucune déduction à refaire côté runner)
# - portable (pas de dépendance à l’expansion shell)

mcf 1

# ---------------------------------------------------------------------------
# Host / platform
# ---------------------------------------------------------------------------

host
  os "linux"                    # linux | macos | windows
  arch "x86_64"                 # x86_64 | aarch64 | ...
  triple "x86_64-unknown-linux-gnu"
  endian "little"               # little | big
.end

# ---------------------------------------------------------------------------
# Workspace
# ---------------------------------------------------------------------------

workspace
  name "vitte"
  root "/path/to/repo"
  file "muffin"                 # fichier source ayant produit ce .mcf
  format "muf"
  version "2"
.end

paths
  root "/path/to/repo"
  dist "dist"
  build "build"
  cache "build/cache"
  tmp "build/tmp"
.end

# ---------------------------------------------------------------------------
# Selection (résolution des defaults)
# ---------------------------------------------------------------------------

selection
  profile "debug"
  target "syntax_smoke"
.end

# ---------------------------------------------------------------------------
# Profiles
# ---------------------------------------------------------------------------

profiles
  profile debug
    opt "0"
    debug true
    features []
  .end

  profile release
    opt "3"
    debug false
    features []
  .end
.end

# ---------------------------------------------------------------------------
# Packages (résolus)
# ---------------------------------------------------------------------------

packages
  package vitte_beryl
    kind "lib"
    version "0.1.0"
    src_dir "lingua/syntax/vitte_beryl/src"

    # Exemple de deps (optionnel)
    deps
      # dep "std" version ">=0.1"  (si registry)
    .end
  .end
.end

# ---------------------------------------------------------------------------
# Toolchains (résolues)
# ---------------------------------------------------------------------------

toolchains
  toolchain vitte
    # Ces chemins peuvent être absolus ou relatifs à `paths.root`.
    # Ils sont résolus au moment de la génération.
    compiler "build/x86_64-unknown-linux-gnu/stage0/bin/vittec"
    build_driver "build/x86_64-unknown-linux-gnu/stage0/bin/build"
    muffin "build/x86_64-unknown-linux-gnu/stage0/bin/muffin"

    # Flags résolus (conventions)
    cflags []
    ldflags []
  .end
.end

# ---------------------------------------------------------------------------
# Env whitelist (ce que l'exécution est autorisée à lire)
# ---------------------------------------------------------------------------

env
  # Valeurs injectées / autorisées
  var "PATH" "/usr/bin:/bin"
  var "PORT" "/dev/ttyACM0"

  # Option: marque les variables qui affectent la signature incrémentale
  affects_fingerprint
    name "PATH"
    name "PORT"
  .end
.end

# ---------------------------------------------------------------------------
# File discovery (résolution des patterns / globs)
# ---------------------------------------------------------------------------

# Note: ces listes sont déjà “figées”.
# Le runner ne doit pas refaire rglob/glob, il consomme cette liste.

files
  group "vitte_beryl_src"
    file "lingua/syntax/vitte_beryl/src/lib.vit"
    file "lingua/syntax/vitte_beryl/src/tests/lib.vit"
  .end
.end

# ---------------------------------------------------------------------------
# Targets (résolues)
# ---------------------------------------------------------------------------

targets
  target syntax_smoke
    kind "test"
    package "vitte_beryl"
    profile "debug"

    inputs
      # group référence une liste figée ci-dessus
      group "vitte_beryl_src"

      # Entrées additionnelles possibles
    .end

    outputs
      dir "dist/syntax"
    .end

    steps
      step check
        tool "vitte.compiler"
        argv
          arg "check"
          arg "lingua/syntax/vitte_beryl/src/tests/lib.vit"
          arg "--out"
          arg "dist/syntax"
        .end
      .end

      step run
        tool "vitte.compiler"
        argv
          arg "run"
          arg "lingua/syntax/vitte_beryl/src/tests/lib.vit"
          arg "--out"
          arg "dist/syntax"
        .end
      .end
    .end

    # Dépendances symboliques (convention)
    deps []
  .end
.end

# ---------------------------------------------------------------------------
# Tool aliases (pour le runner)
# ---------------------------------------------------------------------------

# Permet d’adresser les outils par rôle.
# Exemple: tool "vitte.compiler" => toolchains.vitte.compiler

tool_aliases
  alias "vitte.compiler" "toolchains.vitte.compiler"
  alias "vitte.build_driver" "toolchains.vitte.build_driver"
  alias "vitte.muffin" "toolchains.vitte.muffin"
.end

# ---------------------------------------------------------------------------
# Fingerprint (optionnel)
# ---------------------------------------------------------------------------

# Hash global de la config résolue.
# Utilisé par CI pour invalider proprement.

fingerprint
  algo "sha256"
  value "0000000000000000000000000000000000000000000000000000000000000000"
.end
