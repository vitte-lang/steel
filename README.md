# Muffin

![Muffin](https://img.shields.io/badge/Muffin-config-orange)

Muffin est la couche de configuration **déclarative** du build Vitte. Il **parse**, **valide** et **résout** un workspace (packages, profils, toolchains, targets), puis **génère un artefact de configuration stable** `Muffinconfig.mff` (Muffinconfig). Cet artefact est ensuite **consommé par Vitte** pour appliquer les règles de construction et exécuter les étapes de compilation de manière déterministe.

## Points forts

- Configuration declarative, separee de l'execution.
- Resolution deterministe et sorties facilement outillables.
- Portabilite multi-OS/arch et profils explicites.
- Introspection via commandes `print` et export de graphes.
- Mode dev via `build muffin -watch` et diagnostics `-why` / `-graph`.
- Overrides non-invasifs via `-D KEY=VALUE` (sans modifier le buildfile).

## Pipeline recommande

Le pipeline est volontairement scindé en deux phases : **Configuration** puis **Construction**.

1. **Configuration** — `build muffin`
   - Charge la config (workspace/packages/profils/targets/toolchains)
   - Valide la cohérence (contraintes, chemins, compatibilités)
   - Résout les valeurs (defaults, héritages, overrides)
   - **Émet** `Muffinconfig.mff` (artefact canonique)

2. **Construction** — `build vitte`
   - Lit `Muffinconfig.mff`
   - Prépare le graphe des étapes (inputs → outputs)
   - Exécute les étapes (compile/link/generate/test/package) avec incrémental/cache

## Architecture

### Principe : « Freeze then Build »

- `build muffin` = **configure** : validation + résolution + **gel** de la configuration.
- `build vitte` = **build** : orchestration des étapes + production des artefacts.

Le fichier `Muffinconfig.mff` sert de **barrière contractuelle** entre :

- le monde **déclaratif** (profils, targets, options, résolution),
- et le monde **exécution** (DAG, règles, incrémental, cache, production).

### Détection de reconstruction (incrémental)

Au cœur du pipeline, **Muffin** calcule automatiquement **ce qui doit être reconstruit**.

- À partir du buildfile, Muffin **résout** toutes les entrées (globs/fichiers/valeurs) et construit une vue normalisée du workspace.
- Il produit un artefact `Muffinconfig.mff` qui contient notamment :
  - la liste **exhaustive** des inputs (fichiers, globs développés, dépendances transitives),
  - les paramètres de build (profil/target/toolchain),
  - les empreintes nécessaires à l’invalidation (hash des inputs/args/toolchain/policy).
- **Vitte** consomme `Muffinconfig.mff` et détermine, de manière déterministe, la liste minimale des étapes à rejouer.


Pendant la construction, les sources `*.vitte` sont transformées en artefacts (objets, librairies, exécutables) selon les targets et les règles résolues.

### Segmentation par répertoire (fichiers `*.muff`)

Muffin peut, en option, **segmenter** le workspace en unités par répertoire et **générer un fichier de configuration `*.muff` par dossier** afin de figer localement :

- la liste exhaustive des sources et dépendances associées au dossier,
- les paramètres effectifs (profil/target/toolchain),
- les sorties attendues (librairie, artefact de compilation, exécutable).

Exemple d’unité (dossier) :

- Entrée (point d’ancrage du dossier) : `src/in/<folder>/_.vitte`
- Sources associées (exemples) :
  - `src/program/lib.vitte`
  - `src/program/error.vitte`
  - `src/program/read.vitte`
  - `src/program/output.vitte`
- Sorties (selon target) :
  - `src/out/lib/<compilation_folder>.va` : bibliothèque statique
  - `src/out/bin/<compilation_folder>.vo` : artefact de compilation
  - Windows uniquement : `src/out/bin/<compilation_folder>.exe`

Le répertoire `/muffin` peut contenir un fichier d’ancrage `main.muff` (nom configurable) servant de point d’entrée de configuration/build.
Par défaut, les liaisons entre unités (`wire`) peuvent être automatisées au niveau workspace (via le buildfile), puis gelées dans `Muffinconfig.mff`.

Les builds et artefacts intermédiaires sont isolés dans un répertoire interne (recommandé) : `./.muffin/` (ou `./.muff/` selon la configuration).

### Flux de traitement

```
┌─────────────────────────────────────────────────────────────┐
│                    PHASE CONFIGURATION                      │
│                      (build muffin)                         │
└─────────────────────────────────────────────────────────────┘

1. CHARGEMENT
   ├── Muffinfile (workspace + packages + profils)
   ├── Toolchains (compilateurs, linkers, outils)
   └── Targets (spécifications de build)

2. VALIDATION
   ├── Syntaxe des fichiers
   ├── Références (packages, targets, profils existants)
   ├── Compatibilités (OS/arch, versions, dépendances)
   └── Contraintes (chemins, permissions, etc.)

3. RÉSOLUTION
   ├── Application des profils et héritage
   ├── Résolution des overrides
   ├── Interpolation des variables
   ├── Calcul des empreintes toolchain (fingerprints)
   └── Résolution des dépendances transitives

4. ÉMISSION
   └── Muffinconfig.mff (configuration gelée et normalisée)

┌─────────────────────────────────────────────────────────────┐
│                    PHASE CONSTRUCTION                       │
│                      (build vitte)                          │
└─────────────────────────────────────────────────────────────┘

Vitte lit Muffinconfig.mff et :
  ├── Construit le DAG des étapes
  ├── Résout les dépendances de fichiers
  ├── Exécute l'ordre topologique
  └── Gère l'incrémental et le cache
```

### Composants principaux

#### Parser (`arscan.rs`, `read.rs`)
- Analyse lexicale et syntaxique des fichiers Muffin
- Constructs de blocs (workspace, package, profile, target, etc.)
- Gestion des commentaires et du formatage

#### Validation (`dependancies.rs`, `config.rs`)
- Vérification de la cohérence globale
- Résolution des références (packages → targets, targets → toolchains)
- Validation des contraintes et compatibilités

#### Résolution (`variable.rs`, `expand.rs`, `implicit.rs`)
- Héritage de profil et application des options
- Interpolation des variables d'environnement et macros
- Résolution des valeurs par défaut (implicites)

#### Génération (`interface.rs`, `output.rs`)
- Sérialisation de la configuration résolue en `Muffinconfig.mff`
- Export de graphes (texte, DOT, JSON)
- Génération d'artefacts pour Vitte

### Modèle de données

**Workspace** (conteneur global)
- nom, racine, profils, targets, packages

**Package** (unité compilable)
- nom, version, kind (bin/lib/test/doc), dépendances
- répertoires source, chemins d'inclusion, etc.

**Profile** (ensemble d'options)
- optimisation, debug, features, variables spécifiques
- inheritance (profils dérivés)

**Target** (objectif de build)
- nom, kind (binary/library/test), sources associées
- règles implicites (dépend du kind et du language)

**Toolchain** (ensemble d'outils)
- compilateur, linker, archiveur
- flags par défaut, environnement, version

### Responsabilités de chaque couche

**Muffin** (Déclaratif)
- *Ce qu'on veut construire*, *comment* (profils), *pour qui* (targets)
- Séparation claire entre configuration et exécution
- Validation et cohérence

**Vitte** (Exécutif)
- *Comment y parvenir* : graphe de tâches, parallélisation, cache
- Isolation des détails d'exécution (compilateur, flags réels)
- Diagnostics de performance et débogage

**Muffinconfig.mff** (Contrat)
- Configuration **gelée** et **normalisée**
- Contient tout ce que Vitte doit savoir pour construire
- Invalidation automatique du cache en cas de changement

### Rôle de Vitte (orchestration de build)

Vitte orchestre la phase **construction** à partir de la configuration gelée :

- Définition des **targets** et des **étapes** (transformations `inputs → outputs`)
- Construction d’un **graphe de dépendances** (DAG)
- Exécution **déterministe** (ordre topologique) avec **incrémental** et **cache**
- Diagnostics outillables : « pourquoi ça rebuild ? », « qui dépend de quoi ? »

### Contrat `Muffinconfig.mff`

`Muffinconfig.mff` contient une configuration **normalisée** et **explicite** (plus d’implicite côté build). Exemples de champs attendus :

- version de schéma (`mcfg 1`),
- host/target (OS/arch/triple),
- profil sélectionné,
- chemins (root/build/dist/cache),
- toolchain + fingerprint (invalidation cache),
- targets résolues et options associées,
- dépendances transitives résolues,
- variables d'environnement interpellées.

## Commandes

### Build

```text
build muffin [<plan>] [flags] [-- <args>]
```

- `build muffin` : exécute le **plan par défaut** (ex: `default`).
- `build muffin <plan>` : exécute un **plan nommé** (ex: `release`, `ci`, `package`).

#### Flags de build

- `-all` : construit tous les outputs **exportables** (ou tout le workspace selon policy).
- `-debug` : force le profil `debug`.
- `-release` : force le profil `release`.
- `-clean` : purge cache + artefacts (garde-fou conseillé : `--yes`).
- `-j <n>` : parallélisme du scheduler (ex: `-j 16`).
- `-watch` : mode dev (rebuild incrémental sur événements FS).
- `-why <artifact|ref>` : explique la chaîne de dépendances + la cause d’invalidation.
- `-graph[=text|dot|json]` : dump du DAG (format selon implémentation).
- `-D KEY=VALUE` : overrides de variables (répétable).

Exemples :

```text
build muffin
build muffin release
build muffin -all
build muffin -debug
build muffin -release -j 16
build muffin -D profile=release -D target=x86_64-apple-darwin
build muffin -why app.exe
build muffin -graph=dot
build muffin -watch
```

### Validation / émission

- `check` : valide la configuration sans exécuter la construction.
  - options usuelles : `--profile <name>`, `--target <name>`, `--emit <path>`
- `resolve` : résout et **génère** `Muffinconfig.mff` (équivalent fonctionnel à `build muffin` côté configuration).
  - options usuelles : `--emit <path>`, `--profile <name>`, `--target <name>`

Exemples :

```text
check
check --profile debug
resolve --emit ./Muffinconfig.mff
```

### Introspection

- `print <scope>` : affiche une vue résolue (format texte/JSON selon implémentation).
  - scopes typiques : `workspace`, `packages`, `targets`, `profiles`, `toolchains`, `vars`, `plans`, `exports`
- `graph [--format <text|dot|json>]` : export du graphe (sans build).
- `why <artifact|ref>` : diagnostic « pourquoi ça rebuild ? » (alias possible de `build muffin -why`).

Exemples :

```text
print workspace
print targets
graph --format dot
why vittec_driver::vittec
```

### Maintenance

- `fmt [<file>]` : formate un fichier Muffin.
- `clean` : purge cache + artefacts (alias possible de `build muffin -clean`).
- `cache <cmd>` : gestion du store/cache.
  - `cache stats` : statistiques.
  - `cache gc` : nettoyage.
- `doctor` : diagnostic environnement (outils, droits, chemins, sandbox).

Exemples :

```text
fmt
clean
cache stats
cache gc
doctor
```

### Divers

- `help [<cmd>]` : aide globale ou par sous-commande.
- `version` : version de Muffin.

### Options globales (toutes commandes)

- `-v / --verbose` : sortie détaillée.
- `-q / --quiet` : sortie réduite.

### Exemples multi-OS (Linux / macOS / Windows / BSD / Solaris)

> Objectif : exécuter les commandes `build muffin ...` de façon identique sur toutes les plateformes. Les étapes ci-dessous installent uniquement les utilitaires de base et supposent que `muffin` et `vitte` sont disponibles dans le projet (ou dans le `PATH`).

#### Linux (Debian/Ubuntu)

```bash
sudo apt update
sudo apt install -y git ca-certificates curl

# build
build muffin
build muffin -all
build muffin -debug
```

#### Linux (Fedora/RHEL)

```bash
sudo dnf install -y git ca-certificates curl

build muffin
build muffin -release -j 16
```

#### Linux (Arch)

```bash
sudo pacman -S --needed git ca-certificates curl

build muffin -all
```

#### macOS (Homebrew)

```bash
brew install git

build muffin
build muffin -watch
```

#### Windows (PowerShell)

```powershell
# Prérequis (au choix)
# winget install --id Git.Git -e
# winget install --id OpenJS.NodeJS.LTS -e   # si ton projet en a besoin

# Exécution
build muffin
build muffin -all
build muffin -why app.exe
```

Notes Windows :
- Si `build` n’est pas une commande globale, exécuter depuis la racine du projet via `./muffin` (si présent) ou ajouter le binaire au `PATH`.
- En CI Windows, privilégier `--color never` et `--json` pour l’outillage.

#### FreeBSD

```sh
sudo pkg install -y git ca_root_nss

build muffin
build muffin -graph=dot
```

#### OpenBSD

```sh
doas pkg_add git

build muffin
```

#### NetBSD

```sh
# pkgin (si configuré)
# sudo pkgin install git ca-certificates

build muffin
```

#### Solaris / illumos (OpenIndiana)

```sh
pfexec pkg install developer/versioning/git

build muffin
build muffin -release
```

Conseils cross-OS :
- Utiliser des chemins relatifs dans les buildfiles (`./dist`, `./.muffin/cache`) pour limiter les divergences.
- Si un fichier `muffin` est un script, vérifier le bit exécutable (`chmod +x muffin`) et utiliser des fins de lignes LF.
- Pour l’outillage, préférer `muffin print ... --json`, `muffin graph --format ...`, `muffin why ...`.

## Format du fichier Muffin (apercu)

Muffin est oriente blocs et se termine par `.end`. Commentaires `# ...`.

```text
muf 2

workspace
  name "vitte"
  root "."
.end

package hello
  kind "bin"
  version "0.1.0"
  src_dir "src"
.end

profile debug
  opt "0"
  debug true
.end

default
  target "hello"
.end
```

## Format `Muffinconfig.mff` (apercu)

```text
mff 1

host
  os "linux"
  arch "x86_64"
.end

profile "debug"

target
  name "syntax_smoke"
  kind "test"
.end

paths
  root "/path/to/repo"
  dist "dist"
.end
```

## Variables d'environnement

- `MUFFIN_FILE` : chemin du fichier Muffin.
- `MUFFIN_PROFILE` : profil par defaut.
- `MUFFIN_EMIT` : chemin par défaut de `Muffinconfig.mff`.
- `MUFFIN_OFFLINE` : active le mode offline.
- `VITTE_BUILD` : chemin explicite vers le driver `build`.
- `PATH` : s’assurer que `muffin` et `vitte` sont accessibles (ou utiliser `./muffin` depuis la racine).

## Fichiers

- `muffin` ou `Muffinfile` : configuration principale.
- `*.vitte` : sources du projet.
- `Muffinconfig.mff` : configuration résolue (artefact canonique), consommée par Vitte.
- `*.muf` : buildfiles (si le projet segmente la configuration par dossier/workspace).
- `*.muff` : configurations segmentées par répertoire (optionnel, générées ou maintenues selon le mode).
- `muffin/main.muff` : point d’ancrage de configuration/build (nom configurable, optionnel).
- `*.va` / `*.vo` : artefacts de build (librairie statique / artefact de compilation), selon targets et plateformes.

## Documentation

- [ARCHITECTURE.md](ARCHITECTURE.md) — Détails internes, pipeline, modules
- [src/MODULE_ORGANIZATION.md](src/MODULE_ORGANIZATION.md) — Organisation des fichiers source
- Manpage: [doc/muffin.1](doc/muffin.1)

## Licence

Voir `COPYING`.
