

# Muffin — Exemples avancés (`.muf` / `.muff`)

Ce dossier contient des buildfiles *avancés* destinés à illustrer les patterns complets de Muffin : targets multi-OS/arch, sandbox (`capsule`), cache (`store`), graph typé (bakes/ports/wires), plans, mapping CLI (`switch`), introspection (`decompile`, `-why`, `-graph`) et orchestration **multi-langages**.

> Rappel :
> - `build muffin` = **configure** (parse/validate/resolve → génération du binaire de compilation `Muffinconfig.mff`)
> - `Muffin build` = **build** (lecture `.mff` → exécution du DAG via tools déclarés)

---

## Index des exemples

### Targets / plateformes

- `targets/local.muf`
  - Buildfile *MAX* : Linux / macOS / Windows / BSD / Solaris
  - `switch` riche : flags de target + profils + plans
  - `store` content-addressed + `capsule` hermétique
  - DAG complet : sources → objets → archive → link + exemples Rust/C#/Vitte

### Organisation recommandée

- Un buildfile par thème (targets, cache, sandbox, packaging).
- Un buildfile “root” (ex: `master.muff`) qui agrège des `main.muff` de sous-dossiers.

---

## Modèle mental (avancé)

### 1) Tout est un graphe

- `bake` : unité de calcul (nœud du DAG)
- `in` / `out` : ports typés
- `wire` : lien explicite `out → in`
- `export` : sortie buildable (cible publique)
- `plan` : scénario d’exécution (ce que lance la commande)

### 2) Séparation configuration / exécution

- Les globs, paths et options sont **résolus** pendant `build muffin`.
- Le résultat est figé dans `Muffinconfig.mff` : binaire de compilation portable, normalisé, outillable.
- `Muffin build` exécute ensuite le DAG de manière déterministe.

### 3) Multi-langages via tools

Muffin n’est pas lié à un langage : il orchestre des **tools déclaratifs**.

- C/C++ : compile → link → archive
- Rust : compile crate → link (ou `rustc` direct)
- C# : build via `dotnet` (illustratif)
- Vitte : compile vers artefacts spécifiques (optionnel)

---

## Conventions cross-platform

### Targets (triples)

Exemples usuels :

- `x86_64-unknown-linux-gnu`
- `aarch64-unknown-linux-gnu`
- `x86_64-apple-darwin`
- `aarch64-apple-darwin`
- `x86_64-pc-windows-msvc`
- `x86_64-pc-windows-gnu`
- `x86_64-unknown-freebsd`
- `x86_64-unknown-solaris`

### Artefacts (exemples)

- Objets : `*.o` (Unix), `*.obj` (Windows)
- Archives statiques : `*.a` (Unix), `*.lib` (Windows)
- Partagées : `*.so` / `*.dylib` / `*.dll`
- Exécutables : sans extension (Unix), `*.exe` (Windows)
- Artefacts Vitte (si utilisés) : `*.vo` / `*.va`

> Pattern recommandé : garder des **types logiques** côté Muffin (`bin.obj`, `lib.static`, `bin.exe`) et résoudre l’extension via target.

---

## Usage (avancé)

### Configure

```text
# Plan par défaut (génère Muffinconfig.mff)
build muffin

# Plan nommé
build muffin local

# Tout préparer (exports)
build muffin -all

# Profil
build muffin -debug
build muffin -release
build muffin -ci

# Target
build muffin -D target=x86_64-unknown-linux-gnu
build muffin --macos-arm64
build muffin --win-x64-msvc
```

### Build

```text
# Exécute le build depuis Muffinconfig.mff
Muffin build

# Build d’un plan spécifique (si supporté par l’impl)
Muffin build --plan local
```

### Introspection

```text
# Reconstituer l’architecture du build
muffin decompile Muffinconfig.mff

# Vue normalisée à partir d’un buildfile
muffin decompile targets/local.muf

# Comprendre pourquoi un artefact rebuild
muffin -why out/bin/app

# Export du graphe
muffin -graph --format dot
muffin -graph --format json
```

---

## Patterns avancés

### Pattern A — “fan-out objets” (compilation par fichier)

Dans un projet réel, on modélise généralement :

1. `make files glob "src/**/*.c"`
2. un bake de mapping `files -> objets` (un nœud par fichier)
3. `link`/`archive` sur la collection d’objets

Objectif : parallélisme maximal + cache fin.

### Pattern B — “toolchain pinning”

- `tool <name>` doit pouvoir exprimer :
  - `exec` (binaire)
  - `expect_version` (contrainte)
  - `capsule` (policy)
- La version effective (probe) est figée dans `.mff` pour reproductibilité.

### Pattern C — “capsule hermétique”

- `env allow [...]`
- `fs allow_read [...]`
- `fs allow_write [...]`
- `net deny`
- `time stable true`

Le backend applique au mieux selon l’OS.

### Pattern D — “switch ergonomique”

- Flags de profil : `-debug`, `-release`, `-ci`
- Flags de target : `--linux-x64`, `--macos-arm64`, `--win-x64-msvc`, …
- Flags de plan : `-all`, `-local`

---

## Notes de compatibilité

- Un `.mff` est portable (structure normalisée) mais la reproduction des binaires dépend de la **disponibilité** des toolchains sur la machine.
- Les capacités sandbox varient selon OS : enforcement best-effort.
- Sur machines anciennes : privilégier `-j` bas, désactiver `-watch`, limiter la compression cache.

---

## Checklist “exemple avancé”

Un exemple avancé “complet” doit idéalement inclure :

- `store` (cache) + mode (`content` recommandé)
- `capsule` (policy) et association aux tools
- `profile` (debug/release/ci)
- `tool` déclaratifs (compile/link/archive/test/package)
- `bake` (ports typés) + `wire`
- `export` (cibles publiques)
- `plan` (scénarios)
- `switch` (mapping CLI)

---

## Références

- Spécification EBNF du buildfile (voir `muffin.ebnf`)
- Manpages : `docs/man/`
- API : `docs/api/`