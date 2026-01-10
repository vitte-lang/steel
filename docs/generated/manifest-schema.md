# Manifest schema (generated)

# Manifest schema (generated)

Ce document décrit le **schéma du manifest** Muffin (métadonnées projet + dépendances + publication). Il est distinct du buildfile `Muffinfile` / `build.muf` (qui décrit le graphe de build).

- **manifest** : identité du projet, dépendances, options de distribution.
- **buildfile** : exécution (bakes/tools/capsules/wires/plans).

Le manifest vise un format :

- stable (versionné)
- portable (multi-OS/arch)
- validable (schema)
- sérialisable (TOML/JSON/YAML possible selon impl)

---

## 1) Fichiers et conventions

### 1.1 Noms usuels

Conventions proposées (compatibles multi-projets) :

- `manifest.muf` : manifest “standard” d’un projet
- `workspace.muf` : manifest d’un workspace (multi-modules)
- `mod.muf` : manifest de module (lib/plugin)

Règle :

- Le loader cherche par priorité : `manifest.muf` → `mod.muf` → `workspace.muf` (ou selon policy).

### 1.2 Version du schéma

Le manifest commence par un header :

```text
muffin manifest 1
```

- `manifest_version: int` : version du dialecte manifest.

Toute version inconnue → erreur.

---

## 2) Modèle logique

Le manifest compile vers un modèle logique (AST) :

- `Header { manifest_version }`
- `Document { sections... }`

Le document est organisé en **sections** nommées. Chaque section est un bloc top-level.

---

## 3) Types communs

### 3.1 `ident`

- `[A-Za-z_][A-Za-z0-9_]*` (case-sensitive)

### 3.2 `string`

- UTF-8

### 3.3 `semver`

Contrainte recommandée : SemVer (`MAJOR.MINOR.PATCH`), avec opérateurs de compatibilité (impl-dependent) :

- `"1.2.3"`
- `"^1.2"`
- `"~1.2"`
- `">=1.0,<2.0"`

### 3.4 `target_triple`

- `arch-vendor-os-abi` (convention) :
  - `x86_64-unknown-linux-gnu`
  - `aarch64-apple-darwin`
  - `x86_64-pc-windows-msvc`
  - `x86_64-unknown-freebsd`
  - `x86_64-unknown-solaris`

### 3.5 `path`

- relatif workspace (recommandé) ou absolu.

### 3.6 `list<T>`

- liste homogène par contexte.

---

## 4) Sections top-level

Sections standard (convention) :

- `[project]`
- `[workspace]`
- `[dependencies]`
- `[dev-dependencies]`
- `[build-dependencies]`
- `[features]`
- `[targets]`
- `[artifacts]`
- `[profiles]`
- `[tools]`
- `[publish]`
- `[registries]`
- `[patch]`
- `[scripts]`

Chaque section peut être absente. Les sections inconnues → warning (ou error en strict).

---

## 5) Section `[project]`

### 5.1 But

Décrire l’identité d’un projet (package) : nom, version, type (lib/bin/plugin), métadonnées.

### 5.2 Champs

- `name: string` (obligatoire)
- `version: semver` (obligatoire)
- `kind: string` (défaut: `"app"`) valeurs usuelles :
  - `"app"` (exécutable)
  - `"lib"` (lib)
  - `"plugin"`
  - `"tool"`
  - `"workspace-root"`
- `description: string`
- `license: string` (SPDX recommandé)
- `authors: list<string>`
- `homepage: string`
- `repository: string`
- `documentation: string`
- `readme: string` (path)
- `keywords: list<string>`
- `categories: list<string>`
- `edition: string` (ex: `"2026"`)
- `language: string` (optionnel, ex: `"c"`, `"cpp"`, `"rust"`, `"vitte"`, `"mixed"`)

### 5.3 Règles

- `name` doit être stable (sert au cache, registry, signatures).
- `version` doit être strictement SemVer si publication.

---

## 6) Section `[workspace]`

### 6.1 But

Décrire un workspace multi-modules (monorepo).

### 6.2 Champs

- `members: list<path>` (obligatoire si section présente)
- `exclude: list<path>`
- `default-member: path`
- `resolver: string` (ex: `"v2"`)

### 6.3 Règles

- `members` doivent pointer vers des répertoires contenant un manifest.
- `default-member` doit être inclus dans `members`.

---

## 7) Dépendances

### 7.1 Sections

- `[dependencies]` : runtime
- `[dev-dependencies]` : tests/outils
- `[build-dependencies]` : build tooling

### 7.2 Forme

Chaque dépendance se déclare par clé :

- `dep_name = <dep_spec>`

### 7.3 `dep_spec` (union)

#### a) Registry

- `version: semver`
- `registry: string` (optionnel, défaut: `"default"`)

#### b) Git

- `git: string`
- `rev: string` | `tag: string` | `branch: string`
- `subdir: string` (optionnel)

#### c) Path

- `path: path`

#### d) URL / archive

- `url: string`
- `sha256: string` (recommandé)

### 7.4 Options communes

- `optional: bool` (défaut: false)
- `features: list<string>`
- `default-features: bool` (défaut: true)
- `target: target_triple` (optionnel)
- `platform: string` (alias lisible, ex `"linux"`, `"windows"`, `"macos"`, `"bsd"`, `"solaris"`)

### 7.5 Règles

- `path` et `git` sont mutuellement exclusifs.
- `url` exige un hash (policy `--strict`).

---

## 8) Section `[features]`

### 8.1 But

Gating de dépendances/options (build reproducible).

### 8.2 Forme

- `feature_name = [ "dep:xxx", "feature:yyy", "cfg:..." ]`

Conventions :

- `default = [ ... ]`

### 8.3 Règles

- Une feature peut activer :
  - une dépendance optionnelle (`dep:NAME`)
  - une feature d’une dépendance (`NAME/feat` ou `feature:...` selon impl)

---

## 9) Section `[targets]`

### 9.1 But

Décrire les outputs attendus (produits) du projet pour un ou plusieurs targets.

### 9.2 Champs

Structure recommandée :

- `[targets.<triple>]` ou `[targets.<alias>]`

Sous-champs (exemples) :

- `enabled: bool`
- `default: bool`
- `toolchain: string` (nom logique)
- `link: string` (ex: `"static"`, `"shared"`, `"exe"`)
- `defines: list<string>`
- `cflags: list<string>`
- `ldflags: list<string>`

### 9.3 Règles

- Les options `cflags/ldflags` sont des hints (le buildfile reste source de vérité de l’exécution).

---

## 10) Section `[artifacts]`

### 10.1 But

Nommer et typer des artefacts “exportables” côté manifest (surcouche metadata), alignés avec les `export` du buildfile.

### 10.2 Champs

- `default: list<string>` (ex: `["bin:app", "lib:core"]`)

Déclarations :

- `[[artifacts.bin]]`
- `[[artifacts.lib]]`
- `[[artifacts.plugin]]`

Champs communs :

- `name: string` (obligatoire)
- `path: path` (optionnel, output attendu)
- `target: target_triple` (optionnel)
- `public: bool` (défaut: true)
- `strip: bool` (optionnel)

---

## 11) Section `[profiles]`

### 11.1 But

Déclarer des profils de build au niveau manifest (valeurs), consommés par le buildfile.

### 11.2 Champs

- `[profiles.debug]` / `[profiles.release]` / `[profiles.ci]` etc.

Sous-champs recommandés :

- `opt: int` (0..3)
- `debug: bool`
- `lto: bool`
- `strip: bool`
- `sanitize: list<string>` (ex: `["asan","ubsan"]`)
- `panic: string` (ex: `"abort"`)

Règle :

- Les profils du manifest peuvent être “mappés” vers ceux du buildfile via `set profile` / `switch`.

---

## 12) Section `[tools]`

### 12.1 But

Déclarer des outils requis (tooling) : compilateurs, linkers, générateurs.

### 12.2 Champs

- `[tools.<name>]`

Sous-champs :

- `exec: string` (obligatoire)
- `version: string` (contrainte)
- `install: string` (hint: command line)
- `capsule: string` (référence policy)

Note : le buildfile conserve la sémantique d’exécution stricte.

---

## 13) Section `[publish]`

### 13.1 But

Configuration publication (registry), signatures, politiques.

### 13.2 Champs

- `enabled: bool` (défaut: false)
- `registry: string` (défaut: `"default"`)
- `publisher: string` (id)
- `visibility: string` (ex: `"public"`, `"private"`, `"unlisted"`)
- `include: list<path>`
- `exclude: list<path>`
- `sign: bool` (défaut: true)
- `signature-algo: string` (ex: `"ed25519"`)
- `checksum: string` (ex: `"sha256"`, `"blake3"`)

### 13.3 Règles

- `sign=true` exige une clé dispo (keystore/runtime).
- `include/exclude` s’appliquent au bundle publié.

---

## 14) Section `[registries]`

### 14.1 But

Définir des registries (source de dépendances), avec policies.

### 14.2 Champs

- `[registries.<name>]`

Sous-champs :

- `url: string`
- `token-env: string` (nom de variable d’env)
- `tls: bool` (défaut: true)
- `pin: string` (clé publique / fingerprint, optionnel)

---

## 15) Section `[patch]`

### 15.1 But

Surcharger une dépendance (hotfix) sans modifier le graphe amont.

Forme :

- `[patch.<dep_name>]` → `dep_spec`

Règles :

- `patch` ne change pas l’API attendue ; uniquement la source.

---

## 16) Section `[scripts]`

### 16.1 But

Définir des commandes de haut niveau (dev UX), mappables vers des `plan`.

### 16.2 Champs

- `build: string` (ex: `"muffin build"`)
- `test: string`
- `lint: string`
- `fmt: string`
- `ci: string`

Règle :

- les scripts sont des alias ; l’exécution réelle passe par le runtime (capsule/store selon policy).

---

## 17) Exemples

### 17.1 Projet minimal

```text
muffin manifest 1

[project]
name = "hello"
version = "0.1.0"
kind = "app"
license = "MIT"

[dependencies]
# empty

[profiles.debug]
opt = 0
debug = true

[profiles.release]
opt = 3
debug = false
strip = true
```

### 17.2 Workspace

```text
muffin manifest 1

[workspace]
members = ["crates/core", "crates/app"]
exclude = ["third_party/"]

[project]
name = "workspace-root"
version = "0.0.0"
kind = "workspace-root"
```

### 17.3 Dépendances mixtes

```text
muffin manifest 1

[project]
name = "demo"
version = "1.2.0"

[dependencies]
serde = { version = "^1.0", registry = "default" }
my_lib = { path = "../my_lib" }
upstream = { git = "https://example.com/up.git", rev = "abcdef" }
archive = { url = "https://example.com/a.tar.gz", sha256 = "..." }

[registries.default]
url = "https://registry.example.com"
token-env = "MUFFIN_TOKEN"
```

---

## 18) Compatibilité et évolution

- Ajouts : **append-only** si possible.
- Breaking change : incrément `manifest_version`.
- Les champs inconnus : warning (ou error en strict).

```