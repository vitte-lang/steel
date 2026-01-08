# Appendix

# Appendix

Cette annexe regroupe les éléments de référence : conventions, formats, tables de compatibilité, exemples d’artefacts et checklists.

---

## A) Conventions de nommage

### A.1. Fichiers

- **Buildfiles** : `Muffinfile`, `build.muf`, `main.muff`, `master.muff`
- **Binaire de compilation** : `Muffinconfig.mff`
- **Répertoires d’artefacts** : `./.muffin/` ou `./.muff/` (selon configuration)

### A.2. Types logiques (recommandés)

Muffin manipule des **types logiques** au niveau des ports ; l’extension réelle dépend du target.

- `src.glob` : liste de chemins (inputs)
- `src.file` : chemin fichier unique
- `bin.obj` : objet compilé (Unix: `.o`, Windows: `.obj`)
- `lib.static` : archive statique (Unix: `.a`, Windows: `.lib`)
- `lib.shared` : bibliothèque partagée (Unix: `.so`/`.dylib`, Windows: `.dll`)
- `bin.exe` : exécutable (Windows: `.exe`, Unix: sans extension)
- `pkg.archive` : artefact packaging (ex: `.tar.gz`, `.zip`, `.pkg`, `.msi`)

---

## B) Table des extensions (référence)

### B.1. Objets

- Unix : `*.o`
- Windows : `*.obj`

### B.2. Archives statiques

- Unix : `*.a`
- Windows : `*.lib`

### B.3. Partagées

- Linux/BSD/Solaris : `*.so`
- macOS : `*.dylib`
- Windows : `*.dll`

### B.4. Exécutables

- Windows : `*.exe`
- Unix/macOS/BSD/Solaris : (sans extension)

---

## C) Targets (triples)

Muffin encode les plateformes en triples `arch-vendor-os-abi`.

### C.1. Exemples usuels

- Linux x86_64 : `x86_64-unknown-linux-gnu`
- Linux arm64 : `aarch64-unknown-linux-gnu`
- macOS x86_64 : `x86_64-apple-darwin`
- macOS arm64 : `aarch64-apple-darwin`
- Windows x86_64 (MSVC) : `x86_64-pc-windows-msvc`
- Windows x86_64 (GNU) : `x86_64-pc-windows-gnu`
- FreeBSD x86_64 : `x86_64-unknown-freebsd`
- Solaris x86_64 : `x86_64-unknown-solaris`

### C.2. Mapping target → extensions

Recommandation : garder des types logiques (`bin.obj`, `lib.static`, `bin.exe`) et résoudre l’extension finale via target.

- `bin.obj` → `.o` (Unix) / `.obj` (Windows)
- `lib.static` → `.a` (Unix) / `.lib` (Windows)
- `lib.shared` → `.so` (Linux/BSD/Solaris) / `.dylib` (macOS) / `.dll` (Windows)
- `bin.exe` → `""` (Unix) / `.exe` (Windows)

---

## D) Profiles (référence)

Profils recommandés :

- `debug` : `opt=0`, symboles, checks
- `release` : `opt=3`, stripping possible
- `ci` : `opt=2`, logs réduits, checks adaptés

Recommandation : figer dans `.mff` les valeurs effectives du profil (pas seulement son nom).

---

## E) Cache / Invalidation

### E.1. Modes de cache

- `content` : content-addressed (recommandé)
- `mtime` : compat (moins déterministe)
- `off` : désactivé

### E.2. Clé d’invalidation (concept)

Une clé d’invalidation typique est le hash de :

- inputs (contenu ou metadata selon mode)
- expansion des globs
- toolchain (binaire + version)
- arguments/flags
- target/profile
- capsule/policy

### E.3. Causes fréquentes de rebuild

- fichier input modifié
- liste de fichiers modifiée (glob)
- toolchain/version changée
- flags changés
- policy capsule changée
- target/profile changés

---

## F) Capsule (policy) — modèles

### F.1. Capsule hermétique (recommandée)

```text
capsule hermetic
  env allow ["PATH", "HOME", "TMP", "TEMP", "SystemRoot"]
  fs allow_read  ["./", "/usr", "/bin", "/lib", "/lib64", "/System", "C:/Windows"]
  fs allow_write ["./.muffin", "./.muff", "./out", "./build", "./target"]
  fs deny        ["../"]
  net deny
  time stable true
.end
```

### F.2. Capsule dev (plus permissive)

```text
capsule dev
  env allow ["*"]
  fs allow_read  ["./"]
  fs allow_write ["./"]
  net allow
  time stable false
.end
```

Notes : l’enforcement dépend des capacités OS (best-effort possible).

---

## G) Switch (mapping CLI)

Exemple minimal :

```text
switch
  flag "-debug"   set profile "debug"
  flag "-release" set profile "release"
  flag "--linux-x64" set target "x86_64-unknown-linux-gnu"
  flag "--win-x64-msvc" set target "x86_64-pc-windows-msvc"
  flag "-all" set plan "all"
.end
```

---

## H) Checklist — buildfile “complet”

Un buildfile complet inclut idéalement :

- `store` (cache) + mode (`content`)
- `capsule` (policy) et association aux tools
- `profile` (debug/release/ci)
- `tool` déclaratifs (compile/link/archive/test/package)
- `bake` (ports typés) + `wire`
- `export` (cibles publiques)
- `plan` (scénarios)
- `switch` (mapping CLI)

---

## I) Exemples d’artefacts

### I.1. Projet C (exemple)

- Compilation : `*.c` → `*.o` (ou `*.obj` sur Windows)
- Archive : `*.o` → `lib*.a` (ou `*.lib`)
- Link : `*.o + *.a` → binaire final

### I.2. Projet C++ (exemple)

- Compilation : `*.cpp` → `*.o` / `*.obj`
- Link : objets → `bin.exe`

### I.3. Projet Rust (exemple)

- `Cargo.toml` + sources → binaire / lib via `rustc`/`cargo` (encapsulé en `tool`)

### I.4. Projet C# (exemple)

- `*.csproj` → binaire via `dotnet` (encapsulé en `tool`)

---

## J) Commandes (rappel)

### J.1. Configure

```text
build muffin
build muffin <plan>
build muffin -all
build muffin -debug
build muffin -release
build muffin -ci
build muffin -D KEY=VALUE
```

### J.2. Build

```text
Muffin build
Muffin build --plan <name>
Muffin build --mff <path>
Muffin build -j <n>
Muffin build --dry-run
Muffin build --no-cache
```

### J.3. Introspection

```text
muffin decompile Muffinconfig.mff
muffin why <artifact>
muffin graph --format dot
muffin doctor --tools
muffin clean --scope cache
```