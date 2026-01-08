# Advanced Muffin examples (MUF)

This folder contains **advanced** MUF examples that go beyond “hello world” build files. The goal is to provide patterns you can copy into real projects: multi-target setup, profile tuning, toolchain wiring (response files), caching strategy, and per-folder configuration generation.

> MUF files are **line-oriented**, **block-based**, and every block terminates with `.end`.  
> These examples are written to be readable first, strict second. Adapt names to your Muffin schema if needed.

---

## Contents

- `targets/`
  - `local.muf` — Host/auto target: resolves OS/arch/ABI at runtime, wires toolchain + response templates, and defines output conventions.
- (Optional future additions)
  - `targets/windows-x86_64-msvc.muf`
  - `targets/linux-x86_64-gnu.muf`
  - `targets/aarch64-darwin.muf`
  - `profiles/` (debug/release/relwithdebinfo)
  - `recipes/` (workspaces, monorepo builds, docs builds, packaging)

---

## 1) How these examples are intended to be used

Typical workflow:

1. Put a **workspace** build file at repo root:
   - `build.muf`
2. Put reusable module metadata in:
   - `mod.muf`
3. Import targets/profiles from this folder:
   - `docs/examples/muf/advanced/targets/local.muf`

A minimal pattern is:

```muf
targets
  use "docs/examples/muf/advanced/targets/local.muf"
.end
