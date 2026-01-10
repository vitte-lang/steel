
# Target schema (generated)

Ce document décrit le **schéma Target** de Muffin : identifiants de plateformes (host/target), normalisation, capacités, et conventions de résolution (toolchains, sysroot, formats binaires).

Objectif : une représentation **uniforme** (machines anciennes/récentes, OS variés), afin que le build reste reproductible et portable.

---

## 1) Concepts

### 1.1 Host vs Target

- **host** : plateforme sur laquelle Muffin s’exécute (machine actuelle).
- **target** : plateforme pour laquelle Muffin produit des artefacts.

Une exécution est donc :

- native : `host == target`
- cross : `host != target`

### 1.2 Target ID

Muffin utilise un identifiant canonique (convention) :

- `arch-vendor-os-abi`

Exemples :

- `x86_64-unknown-linux-gnu`
- `aarch64-unknown-linux-gnu`
- `x86_64-apple-darwin`
- `aarch64-apple-darwin`
- `x86_64-pc-windows-msvc`
- `x86_64-unknown-freebsd`
- `x86_64-unknown-openbsd`
- `x86_64-unknown-netbsd`
- `x86_64-unknown-dragonfly`
- `x86_64-unknown-solaris`

> Le format est inspiré des “triples”, mais Muffin conserve ses propres règles de normalisation.

---

## 2) Schéma logique

### 2.1 Modèle `TargetSpec`

Structure (logique) :

- `id: string` (canonical)
- `arch: Arch`
- `vendor: Vendor`
- `os: Os`
- `abi: Abi`
- `endian: Endian`
- `pointer_width: int` (16/32/64)
- `cpu: CpuModel` (optionnel)
- `features: list<string>` (optionnel)
- `atomic: AtomicSupport`
- `tls: TlsSupport`
- `float: FloatSupport`
- `exe_format: ExeFormat`
- `lib_formats: LibFormats`
- `object_format: ObjectFormat`
- `archive_format: ArchiveFormat`
- `path: PathConventions`
- `env: RuntimeEnv`
- `toolchain: ToolchainHints`

Ces champs peuvent provenir :

- d’un catalogue interne (targets connus)
- d’un `target.json` (extension optionnelle)
- d’overrides CLI/vars (`-D target=...` + options associées)

---

## 3) Dictionnaires

### 3.1 `Arch`

Valeurs usuelles (non exhaustif) :

- `x86_64`
- `x86` (i686)
- `aarch64`
- `armv7`
- `armv6`
- `riscv64`
- `riscv32`
- `mips64`
- `mips32`
- `powerpc64`
- `powerpc32`
- `sparc64`

### 3.2 `Vendor`

- `unknown`
- `pc`
- `apple`
- `ibm`
- `sun`

### 3.3 `Os`

- `linux`
- `windows`
- `darwin`
- `freebsd`
- `openbsd`
- `netbsd`
- `dragonfly`
- `solaris`

### 3.4 `Abi`

- `gnu`
- `musl`
- `msvc`
- `android`
- `gnueabihf`
- `eabi`
- `eabihf`

### 3.5 `Endian`

- `little`
- `big`

### 3.6 `ObjectFormat`

- `elf`
- `macho`
- `coff` (PE/COFF)

### 3.7 `ArchiveFormat`

- `ar`
- `lib` (Windows COFF archive)

### 3.8 `ExeFormat`

- `elf`
- `macho`
- `pe`

---

## 4) Conventions fichiers (extensions)

### 4.1 Objets

- Linux/BSD/Solaris : `*.o`
- Windows (COFF) : `*.obj`

### 4.2 Bibliothèques

- static (Unix-like) : `libNAME.a`
- shared (Unix-like) : `libNAME.so` (Linux), `libNAME.dylib` (macOS)
- static (Windows) : `NAME.lib`
- shared (Windows) : `NAME.dll` + import lib `NAME.lib` (optionnel)

### 4.3 Exécutables

- Unix-like : `NAME`
- Windows : `NAME.exe`

### 4.4 Conventions de paths

- séparateur : `/` (POSIX) vs `\` (Windows)
- extension exécutable : `".exe"` uniquement sur Windows
- case sensitivity : OS-dependent

---

## 5) Runtime / environnement

### 5.1 `RuntimeEnv`

Champs (convention) :

- `c_runtime: "glibc" | "musl" | "msvcrt" | "ucrt" | "bionic" | "unknown"`
- `dynamic_linker: string` (optionnel)
- `sysroot: string` (optionnel)
- `pkg_config: bool` (optionnel)

### 5.2 TLS / Atomics / Float

`AtomicSupport` (convention) :

- `none|partial|full`

`TlsSupport` :

- `none|emulated|native`

`FloatSupport` :

- `soft|hard|mixed`

---

## 6) Toolchain (hints)

### 6.1 `ToolchainHints`

Muffin n’impose pas un compilateur unique : il décrit des **hints** et des points de résolution.

Champs :

- `cc: string` (ex: `"cc"`, `"clang"`, `"x86_64-w64-mingw32-gcc"`)
- `cxx: string`
- `ar: string`
- `ld: string`
- `strip: string`
- `ranlib: string`
- `rc: string` (Windows resources)
- `link_mode: "static"|"dynamic"|"auto"`

Résolution :

1) overrides CLI (`-D cc=...`) / vars
2) `tool` déclarés dans le buildfile
3) détection host (PATH)
4) fallback catalogue

### 6.2 Sysroot

- Linux cross : `--sysroot` + headers/libs du target
- macOS cross : SDK + `xcrun` (policy)
- Windows cross : toolchain + CRT (MSVC/UCRT ou MinGW)
- BSD/Solaris : sysroot (ports/pkg), ou toolchain native sur la machine cible

---

## 7) Normalisation et alias

### 7.1 Canonicalisation

Entrées acceptées (exemples) :

- alias : `linux-x64`, `macos-arm64`, `windows-x64`
- triple partiel : `x86_64-linux-gnu`
- triple complet : `x86_64-unknown-linux-gnu`

Règles de canonicalisation (convention) :

- vendor absent → `unknown` (sauf Windows → `pc`, macOS → `apple`)
- abi absent → valeur par défaut selon OS :
  - linux → `gnu` (ou `musl` si policy)
  - windows → `msvc`
  - darwin → `darwin`
  - *bsd/solaris → `unknown`

### 7.2 Alias recommandés

- `linux-x64` → `x86_64-unknown-linux-gnu`
- `linux-arm64` → `aarch64-unknown-linux-gnu`
- `macos-x64` → `x86_64-apple-darwin`
- `macos-arm64` → `aarch64-apple-darwin`
- `windows-x64` → `x86_64-pc-windows-msvc`
- `freebsd-x64` → `x86_64-unknown-freebsd`
- `openbsd-x64` → `x86_64-unknown-openbsd`
- `netbsd-x64` → `x86_64-unknown-netbsd`
- `solaris-x64` → `x86_64-unknown-solaris`

---

## 8) Matrice plateforme (conventions)

### 8.1 Linux

- `object_format`: `elf`
- `exe_format`: `elf`
- `static_lib`: `libNAME.a`
- `shared_lib`: `libNAME.so`
- `crt`: `glibc` ou `musl`

Targets usuels :

- `x86_64-unknown-linux-gnu`
- `aarch64-unknown-linux-gnu`
- `x86_64-unknown-linux-musl`
- `aarch64-unknown-linux-musl`

### 8.2 macOS (Darwin)

- `object_format`: `macho`
- `exe_format`: `macho`
- `static_lib`: `libNAME.a`
- `shared_lib`: `libNAME.dylib`
- `tooling`: `xcrun` / SDK (policy)

Targets usuels :

- `x86_64-apple-darwin`
- `aarch64-apple-darwin`

### 8.3 Windows

- `object_format`: `coff`
- `exe_format`: `pe`
- `static_lib`: `NAME.lib`
- `shared_lib`: `NAME.dll` (+ import lib `NAME.lib`)

Targets usuels :

- `x86_64-pc-windows-msvc`
- `x86_64-pc-windows-gnu` (si support MinGW)

### 8.4 BSD

- `object_format`: `elf` (général)
- `exe_format`: `elf`
- `static_lib`: `libNAME.a`
- `shared_lib`: `libNAME.so`

Targets usuels :

- `x86_64-unknown-freebsd`
- `x86_64-unknown-openbsd`
- `x86_64-unknown-netbsd`
- `x86_64-unknown-dragonfly`

### 8.5 Solaris

- `object_format`: `elf`
- `exe_format`: `elf`
- `static_lib`: `libNAME.a`
- `shared_lib`: `libNAME.so`

Target usuel :

- `x86_64-unknown-solaris`

---

## 9) Déclaration dans le manifest (convention)

Dans `manifest.muf` :

- `[targets.<id>]` : options/hints

Exemple :

```text
[targets.x86_64-unknown-linux-gnu]
enabled = true
link = "dynamic"

[targets.x86_64-pc-windows-msvc]
enabled = true
link = "dynamic"
```

Note : le manifest décrit des **préférences** ; le buildfile reste la source de vérité de l’exécution.

---

## 10) Overrides CLI (convention)

### 10.1 Sélection target

```bash
muffin configure --target x86_64-unknown-linux-gnu
muffin configure -D target=x86_64-unknown-linux-gnu
```

### 10.2 Toolchain

```bash
muffin configure -D cc=clang -D ar=llvm-ar
muffin configure -D sysroot=/opt/sysroots/linux-x64
```

### 10.3 Modes de link

```bash
muffin configure -D link=static
muffin configure -D link=dynamic
```

---

## 11) Validation (règles)

### 11.1 Cohérence du triple

- `arch/os` doivent être reconnus (ou explicitement “unknown”).
- `abi` doit être compatible avec `os` (ex: `msvc` attendu sur `windows`).

### 11.2 Formats

- `os=darwin` → `object_format=macho`
- `os=windows` → `object_format=coff`, `exe_format=pe`
- `os=linux|bsd|solaris` → `object_format=elf`

### 11.3 Extensions

- `windows` → `exe_ext=.exe`, `obj_ext=.obj`, `lib_ext=.lib`
- sinon → `exe_ext=""`, `obj_ext=.o`, `lib_ext=.a`

### 11.4 Déterminisme

- canonicalisation stable
- tri stable des `features`
- sérialisation stable du `TargetSpec` dans le `.mff`

---

## 12) Représentation dans le `.mff` (convention)

Le `.mff` doit encapsuler :

- `host_target: TargetSpec`
- `selected_target: TargetSpec`
- `resolved_toolchain: ToolchainHints` (après résolution)
- `sysroot` / SDK info (si applicable)

Plus un hash d’intégrité sur le contenu normalisé.

---

## 13) Exemple complet (catalogue local)

```text
# Canonical target example (logical)

id = "x86_64-unknown-linux-gnu"
arch = "x86_64"
vendor = "unknown"
os = "linux"
abi = "gnu"
endian = "little"
pointer_width = 64
object_format = "elf"
exe_format = "elf"
archive_format = "ar"
lib_formats = { static = "a", shared = "so" }
path = { sep = "/", exe_ext = "", obj_ext = "o" }

toolchain = {
  cc = "cc",
  cxx = "c++",
  ar = "ar",
  ld = "ld",
  strip = "strip",
  link_mode = "auto"
}
```

---

## 14) Compatibilité et évolution

- Ajouts : append-only si possible.
- Breaking change : bump de version (catalogue / `.mff` format).
- Targets inconnus : error (ou “custom target” via extension future).

