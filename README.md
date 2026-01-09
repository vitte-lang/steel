# Muffin

![Muffin](https://img.shields.io/badge/Muffin-config-orange)

Muffin est la couche de configuration **déclarative** du build Vitte. Il **parse**, **valide** et **résout** un workspace (packages, profils, toolchains, targets), puis **génère un artefact de configuration stable** `Muffinconfig.mff` (Muffinconfig). Cet artefact est ensuite **consommé par Vitte** pour appliquer les règles de construction et exécuter les étapes de compilation de manière déterministe.


## Points forts

- Configuration déclarative, séparée de l'exécution.
- Résolution déterministe et sorties facilement outillables.
- Portabilité multi-OS/arch et profils explicites.
- Introspection via commandes `print` et export de graphes.
- Mode dev via `build muffin -watch` et diagnostics `-why` / `-graph`.
- Overrides non-invasifs via `-D KEY=VALUE` (sans modifier le buildfile).

## Uniformisation totale (langages + machines)

Muffin vise une **uniformisation totale** du build : même modèle, mêmes commandes et mêmes sorties logiques, quel que soit le langage (Vitte, C/C++, C#, Rust, …) et quel que soit l’environnement (machines anciennes ou récentes, OS/arch hétérogènes).

- **Langage-agnostique** : l’intégration se fait via des **tools déclaratifs** (compile/link/archive/test/package), connectés dans un graphe typé.
- **Machine-agnostique** : l’exécution est pilotée par des **targets** (triples OS/arch) et des politiques stables (paths normalisés, cache, sandbox).
- **Contrat unique** : la configuration est gelée dans `Muffinconfig.mff` et consommée ensuite de manière déterministe.
- **Reproductibilité** : cache content-addressed + empreinte toolchain + policy capsule.
- **Observabilité** : diagnostics et introspection (`print`, `-why`, `-graph`) pour outiller CI, IDE et scripts.

L’objectif est de pouvoir orchestrer des projets **mono-langage** comme des projets **mixtes**, sur des environnements modernes comme sur des configurations plus anciennes, sans divergence de modèle ni de workflow.

## Pipeline recommandé

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

Chaque répertoire du projet peut contenir, à la racine du dossier, un fichier **`main.muff`** (et sa syntaxe associée). Ce fichier décrit à la fois :

- la **configuration locale** (paramètres effectifs, profil/target/toolchain, variables),
- et les **règles de construction** du dossier (inputs, outputs, liaisons, exécution).

Muffin fournit les binaires `muffin` / `Muffin` utilisés pour orchestrer ce flux.

#### Agrégation (fichier maître)

Par défaut, l’ensemble des fichiers `main.muff` présents dans les sous-répertoires peut être **intégré** dans un fichier maître **`master.muff`** à la racine du dépôt. `master.muff` sert de point d’ancrage du build global et permet de déclencher un build workspace tout en conservant une segmentation par dossier.

#### Configuration gelée et artefacts

Lors de la phase de configuration, `muffin` / `Muffin` peut **générer un fichier `.mff`** (configuration gelée) destiné à une compilation globale. Cette configuration est ensuite utilisée pour produire des artefacts binaires Vitte :

- **`.va`** : sortie de bibliothèque statique (si le dossier ou la target déclare une librairie),
- **`.vo`** : sortie de compilation standard (artefact de compilation),
- **Windows** : un exécutable **`.exe`** peut être produit en plus des artefacts `.vo`.

Les fichiers de compilation produisent typiquement des binaires **`.vo`**.

#### Commande de build

La construction d’un dossier (ou du workspace via `master.muff`) s’effectue en exécutant le plan principal, par exemple :

```text
Muffin build main.muff
```

Le build génère les sorties dans un répertoire d’artefacts **par dossier** (par convention `./.muffin/` ou `./.muff/` selon la configuration), tant qu’aucune erreur n’est détectée pendant la compilation.

#### Projets multi-outils

Le buildfile reste générique : Muffin est capable d’orchestrer des projets Vitte et des projets mixtes via des **tools** déclaratifs (par exemple pour C/C++/C#/Rust), tant que les toolchains et les étapes (compile/link/archive/package) sont décrites de manière explicite.

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

### Décompilation (audit de build)

Muffin expose une commande de **décompilation** orientée audit : elle permet de relire un artefact de configuration gelée **`.mff`** (ou un buildfile **`.muff`**) et de reconstituer une vue complète du projet : architecture, graphe, inputs/outputs, outils utilisés et paramètres.

- `decompile <project.mff>` : affiche l’architecture du build (DAG), la liste des fichiers, les ports, les toolchains et l’ensemble des valeurs résolues.
- `decompile <main.muff>` : affiche la configuration et les règles telles qu’elles seront gelées (vue normalisée), sans exécuter la construction.

Exemples :

```text
muffin decompile projet.mff
muffin decompile src/module/main.muff
muffin decompile projet.mff --format json
muffin decompile projet.mff --graph dot
```

Un fichier **`.mff`** enregistre la configuration **normalisée** et la **trace de compilation** (inputs, globs développés, règles, outils, arguments, empreintes). Il garantit une configuration uniforme sur toutes les machines.

Un buildfile **`.muff`** (ou un ensemble de `main.muff`) peut également être décompilé sur n’importe quelle machine pour retrouver :

- les listes d’inputs (fichiers, globs),
- les dépendances et l’ordre d’exécution,
- les bibliothèques/plugins référencés,
- les sorties attendues.

Limite : la reconstruction effective des binaires dépend de la **disponibilité** et de la **compatibilité** des toolchains (langages compilés, versions, targets). Muffin fournit la description complète ; la machine doit disposer des compilateurs/outils compatibles pour reproduire les artefacts.

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

Muffin est orienté blocs et se termine par `.end`. Commentaires `# ...`.

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
- `MUFFIN_PROFILE` : profil par défaut.
- `MUFFIN_EMIT` : chemin par défaut de `Muffinconfig.mff`.
- `MUFFIN_OFFLINE` : active le mode offline.
- `VITTE_BUILD` : chemin explicite vers le driver `build`.
- `PATH` : s’assurer que `muffin` et `vitte` sont accessibles (ou utiliser `./muffin` depuis la racine).

## Fichiers

- `muffin` ou `Muffinfile` : configuration principale.
- `*.vitte` : sources du projet.
- `Muffinconfig.mff` : configuration gelée et normalisée (artefact canonique) + trace outillable (graph, inputs, outils, empreintes), consommée par Vitte.
- `*.muf` : buildfiles (si le projet segmente la configuration par dossier/workspace).
- `*.muff` : configurations segmentées par répertoire (optionnel, générées ou maintenues selon le mode).
- `muffin/main.muff` : point d’ancrage de configuration/build (nom configurable, optionnel).
- Artefacts de build (selon targets/plateformes) : `*.vo` (Vitte compilation), `*.va` (Vitte librairie statique), `*.o` / `*.obj` (C/C++ objets), `*.a` / `*.lib` (archives statiques), `*.so` / `*.dylib` / `*.dll` (librairies partagées), `*.exe` (exécutables Windows).

## Documentation

- [ARCHITECTURE.md](ARCHITECTURE.md) — Détails internes, pipeline, modules
- [src/MODULE_ORGANIZATION.md](src/MODULE_ORGANIZATION.md) — Organisation des fichiers source
- Manpage: [doc/muffin.1](doc/muffin.1)

## Licence

Voir `COPYING`.
