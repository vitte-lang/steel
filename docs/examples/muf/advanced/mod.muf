# ============================================================================
# Muffin Module Manifest — mod.muf (MAX)
# Path: /mod.muf
#
# Purpose:
# - Defines a reusable Muffin "module" (package) metadata + build contract
# - Declares sources, outputs, targets/profiles compatibility, dependencies
# - Designed to be consumed by:
#     build muffin
#     build muffin --dep <module>
#
# Notes:
# - This is a doc-grade “max” manifest: includes many optional fields.
# - Adapt keywords to your exact Muffin schema if your parser differs.
# - All blocks end with .end
# ============================================================================

module muffin
  # Identity
  name        "muffin"
  namespace   "muffin"
  version     "0.1.0"
  description "Muffin build system module (toolchain + config + examples)"
  license     "MIT"

  # Publisher / origin (optional)
  publisher
    name  "muffin"
    email "dev@example.invalid"
    url   "https://example.invalid/muffin"
  .end

  # Repository / provenance (optional)
  source
    vcs   "git"
    repo  "https://example.invalid/muffin.git"
    ref   "main"
    # commit "..."
  .end

  # Semver constraints for host tools (optional)
  requires
    muffin ">=0.1.0"
    vittec ">=0.1.0"
  .end

  # Tags for registry indexing (optional)
  tags
    add "build"
    add "toolchain"
    add "vitte"
    add "docs"
  .end

  # --------------------------------------------------------------------------
  # Layout
  # --------------------------------------------------------------------------
  layout
    root "."
    docs "docs"
    toolchain "toolchain"
    examples "docs/examples"
  .end

  # --------------------------------------------------------------------------
  # Targets / platforms compatibility
  # --------------------------------------------------------------------------
  platforms
    # If empty => all platforms allowed
    allow "windows-x86_64-msvc"
    allow "windows-x86_64-gnu"
    allow "linux-x86_64-gnu"
    allow "linux-x86_64-musl"
    allow "darwin-aarch64"
    allow "darwin-x86_64"
    allow "freebsd-x86_64"
  .end

  # --------------------------------------------------------------------------
  # Build contract (inputs/outputs)
  # --------------------------------------------------------------------------
  build
    # What this module provides (logical artifacts)
    provides
      artifact "toolchain-assets"
      artifact "docs-site"
      artifact "targets"
      artifact "examples"
    .end

    # Source groups (for hashing / caching)
    sources
      group toolchain_assets
        root "toolchain/assets"
        glob "**/*"
      .end

      group docs
        root "docs"
        glob "**/*"
        exclude "**/site/**"
        exclude "**/.cache/**"
      .end

      group examples
        root "docs/examples"
        glob "**/*"
      .end
    .end

    # Default outputs (module packaging outputs, not app build outputs)
    outputs
      dir  "dist"
      file "dist/muffin-toolchain-assets.zip"
      file "dist/muffin-docs-site.zip"
      file "dist/muffin-targets.zip"
      file "dist/muffin-examples.zip"
      file "dist/SBOM.spdx.json"      # optional
      file "dist/checksums.sha256"    # optional
    .end

    # Build steps (module-level bakes)
    # These can be invoked by the workspace build.muf or by `muffin pack`.
    bakes
      bake pack_toolchain_assets
        takes
          root "toolchain/assets"
          glob "**/*"
        .end
        emits
          file "dist/muffin-toolchain-assets.zip"
        .end
        do
          ensure_dir "dist"
          zip "dist/muffin-toolchain-assets.zip"
            add_dir "toolchain/assets"
          .end
        .end
      .end

      bake pack_docs_site
        takes
          root "docs"
          glob "**/*"
        .end
        emits
          file "dist/muffin-docs-site.zip"
        .end
        do
          ensure_dir "dist"
          zip "dist/muffin-docs-site.zip"
            add_dir "docs"
          .end
        .end
      .end

      bake pack_targets
        takes
          root "toolchain/targets"
          glob "**/*.muf"
        .end
        emits
          file "dist/muffin-targets.zip"
        .end
        do
          ensure_dir "dist"
          zip "dist/muffin-targets.zip"
            add_dir "toolchain/targets"
          .end
        .end
      .end

      bake pack_examples
        takes
          root "docs/examples"
          glob "**/*"
        .end
        emits
          file "dist/muffin-examples.zip"
        .end
        do
          ensure_dir "dist"
          zip "dist/muffin-examples.zip"
            add_dir "docs/examples"
          .end
        .end
      .end

      bake sbom
        takes
          root "."
          glob "mod.muf"
          glob "build.muf"
          glob "toolchain/**"
          glob "docs/**"
        .end
        emits
          file "dist/SBOM.spdx.json"
        .end
        do
          ensure_dir "dist"
          # Tool is illustrative; replace by your SBOM generator.
          run tool "muffin-sbom"
            args add "--format" "spdx-json"
            args add "--out" "dist/SBOM.spdx.json"
          .end
        .end
      .end

      bake checksums
        takes
          root "dist"
          glob "*.zip"
          glob "SBOM.spdx.json"
        .end
        emits
          file "dist/checksums.sha256"
        .end
        do
          run tool "sha256sum"
            args add "dist/*"
            stdout "dist/checksums.sha256"
          .end
        .end
      .end
    .end

    # Default high-level build target for module packaging
    default_bake "pack_docs_site"
  .end

  # --------------------------------------------------------------------------
  # Dependencies (module-level)
  # --------------------------------------------------------------------------
  deps
    # Example dependency blocks (optional)
    # dep vitte_std
    #   name "vitte-stdlib"
    #   version ">=0.1.0"
    #   source "registry"
    # .end
  .end

  # --------------------------------------------------------------------------
  # Verification / policy (optional)
  # --------------------------------------------------------------------------
  policy
    reproducible "on"
    sandbox
      fs "read-write"
      net "off"
      env "inherit"
      time "allow"
    .end

    signatures
      required "off"
      # keyring "keys/"
    .end
  .end
.end
