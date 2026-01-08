# Examples

This section provides copy/paste-ready examples for **Muffin**.

- **MUF examples**: build files and manifests (`.muf`)
- **Targets**: platform target definitions (`.muf`)
- **RSP templates**: response-file templates for toolchain commands (`.rsp.tmpl`)

If you are new to MUF, start with **Minimal**, then move to **Advanced**.

---

## MUF

### Minimal

Smallest practical project:

- `muf/minimal/build.muf`
- `muf/minimal/mod.muf`
- `muf/minimal/README.md`

Recommended when you want to understand:

- blocks + `.end`
- `project`, `targets`, `profiles`
- one `bake build` recipe

### Advanced

More realistic patterns:

- `muf/advanced/README.md`
- `muf/advanced/targets/local.muf`

Recommended when you want:

- reusable targets
- response files (Windows command line limits)
- per-folder config generation (ex: `.muff`)
- multi-profile tuning + feature gates
- caching strategy

---

## Targets

Pre-made target definitions you can import into your build:

- `targets/aarch64-darwin.muf`
- `targets/x86_64-linux-gnu.muf`

Pattern:

```muf
targets
  use "docs/examples/targets/x86_64-linux-gnu.muf"
.end
