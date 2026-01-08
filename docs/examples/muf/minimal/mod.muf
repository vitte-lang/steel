

# mod.muf — minimal
#
# Objectif : exemple minimal, lisible, portable.
# - `build muffin` : configure → génère `Muffinconfig.mff`
# - `Muffin build` : build → exécute le plan depuis `.mff`
#
# Remarque : cet exemple évite volontairement les variations OS/arch.

muffin bake 2

# ---------------------------------------------------------------------------
# Store (cache) — minimal
# ---------------------------------------------------------------------------
store cache
  path "./.muffin/cache"
  mode content
.end

# ---------------------------------------------------------------------------
# Capsule — minimal (policy hermétique soft)
# ---------------------------------------------------------------------------
capsule dev
  env allow ["PATH", "HOME", "TMP", "TEMP", "SystemRoot"]
  fs allow_read  ["./"]
  fs allow_write ["./.muffin", "./.muff", "./out", "./build"]
  net deny
  time stable true
.end

# ---------------------------------------------------------------------------
# Variables
# ---------------------------------------------------------------------------
var out_dir: text = "./out"
var cc: text = "cc"

# ---------------------------------------------------------------------------
# Profile
# ---------------------------------------------------------------------------
profile debug
  set opt "0"
  set debug true
.end

# ---------------------------------------------------------------------------
# Tool (C compiler) — minimal
# ---------------------------------------------------------------------------
tool cc
  exec "${cc}"
  sandbox true
  capsule dev
.end

# ---------------------------------------------------------------------------
# Bakes
# ---------------------------------------------------------------------------

# 1) Inputs : sources C
bake src
  out files: src.glob
  make files glob "src/**/*.c"
  cache content
.end

# 2) Compile : produit un objet (type logique)
bake obj
  in  src: src.glob
  out obj: bin.obj

  run tool cc
    takes src as "--inputs"
    emits obj as "--out"
    set "--flags" "-Wall -Wextra"
  .end

  cache content
  output obj at "${out_dir}/obj/app.o"
.end

# ---------------------------------------------------------------------------
# Wiring
# ---------------------------------------------------------------------------
wire src.files -> obj.src

# ---------------------------------------------------------------------------
# Export / Plan
# ---------------------------------------------------------------------------
export obj.obj

plan default
  run exports
.end

set profile "debug"
set plan "default"