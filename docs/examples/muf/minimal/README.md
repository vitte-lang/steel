# Minimal Muffin MUF Example

This directory contains the **smallest practical Muffin setup**. It is intended as a starting point to understand MUF without advanced features (multi-targets, caches, response files, per-folder configs).

---

## Files

- `build.muf`  
  Minimal workspace build file. Defines:
  - project metadata
  - one host target (`local`)
  - two profiles (`debug`, `release`)
  - one build recipe

- `mod.muf`  
  Minimal module manifest. Defines:
  - module identity
  - required tools
  - source ownership for packaging / hashing

---

## Assumed layout

```text
Src/
├─ in/
│  └─ app/
│     └─ main.vit
└─ out/
   └─ bin/
