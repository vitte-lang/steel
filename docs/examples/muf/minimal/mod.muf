# ============================================================================
# Minimal Muffin Module Manifest — mod.muf
# Path: /docs/examples/muf/minimal/mod.muf
#
# Goal:
# - Smallest reasonable module manifest:
#   - identity (name/version)
#   - minimal compatibility
#   - minimal build contract (what files belong to the module)
#
# Blocks end with .end
# ============================================================================

module app
  name        "app"
  namespace   "examples"
  version     "0.1.0"
  description "Minimal Muffin MUF example module"
  license     "MIT"

  # Optional: tool requirements
  requires
    muffin ">=0.1.0"
    vittec ">=0.1.0"
  .end

  # What belongs to the module (for packaging / hashing)
  sources
    group src
      root "Src/in/app"
      glob "**/*"
    .end
    group build
      root "."
      file "build.muf"
      file "mod.muf"
    .end
  .end

  # Optional: default outputs when packaging this module
  outputs
    dir  "dist"
    file "dist/app-src.zip"
  .end
.end
