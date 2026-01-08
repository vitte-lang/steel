# Muffin

![Muffin](https://img.shields.io/badge/Muffin-config-orange)

Muffin est la couche de configuration **déclarative** du build Vitte. Il **parse**, **valide** et **résout** un workspace (packages, profils, toolchains, targets), puis **génère un artefact de configuration stable** `Muffinconfig.mcfg` (Muffinconfig). Cet artefact est ensuite **consommé par Steel**, l’orchestrateur de build **DAG** (rules/targets), pour exécuter les étapes de compilation de manière déterministe.

## Points forts

- Configuration declarative, separee de l'execution.
- Resolution deterministe et sorties facilement outillables.
- Portabilite multi-OS/arch et profils explicites.
- Introspection via commandes `print` et export de graphes.

## Pipeline recommande

Le pipeline est volontairement scindé en deux phases : **Configuration** puis **Construction**.

1. **Configuration** — `build muffin`
   - Charge la config (workspace/packages/profils/targets/toolchains)
   - Valide la cohérence (contraintes, chemins, compatibilités)
   - Résout les valeurs (defaults, héritages, overrides)
   - **Émet** `Muffinconfig.mcfg` (artefact canonique)

2. **Construction** — `build steel`
   - Lit `Muffinconfig.mcfg`
   - Construit le graphe (DAG) des règles (inputs → outputs)
   - Exécute les règles (compile/link/generate/test/package) avec incrémental/cache

## Architecture

### Principe : « Freeze then Build »

- `build muffin` = **configure** : validation + résolution + **gel** de la configuration.
- `build steel` = **build** : orchestration des règles + production des artefacts.

Le fichier `Muffinconfig.mcfg` sert de **barrière contractuelle** entre :

- le monde **déclaratif** (profils, targets, options, résolution),
- et le monde **exécution** (DAG, règles, incrémental, cache, production).

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
   └── Muffinconfig.mcfg (configuration gelée et normalisée)

┌─────────────────────────────────────────────────────────────┐
│                    PHASE CONSTRUCTION                       │
│                      (build steel)                          │
└─────────────────────────────────────────────────────────────┘

Steel lit Muffinconfig.mcfg et :
  ├── Construit le DAG des règles
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
- Sérialisation de la configuration résolue en `Muffinconfig.mcfg`
- Export de graphes (texte, DOT, JSON)
- Génération d'artefacts pour Steel

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

**Steel** (Exécutif)
- *Comment y parvenir* : graphe de tâches, parallélisation, cache
- Isolation des détails d'exécution (compilateur, flags réels)
- Diagnostics de performance et débogage

**Muffinconfig.mcfg** (Contrat)
- Configuration **gelée** et **normalisée**
- Contient tout ce que Steel doit savoir
- Invalidation automatique du cache en cas de changement

### Rôle de Steel (Makefile moderne)

Steel joue le rôle d’un **Makefile contemporain** :

- Définition de **targets** et de **rules/steps** (transformations `inputs → outputs`)
- Construction d’un **graphe de dépendances** (DAG)
- Exécution **déterministe** (ordre topologique) avec **incrémental** et **cache**
- Diagnostics outillables : « pourquoi ça rebuild ? », « qui dépend de quoi ? »

### Contrat `Muffinconfig.mcfg`

`Muffinconfig.mcfg` contient une configuration **normalisée** et **explicite** (plus d’implicite côté Steel). Exemples de champs attendus :

- version de schéma (`mcfg 1`),
- host/target (OS/arch/triple),
- profil sélectionné,
- chemins (root/build/dist/cache),
- toolchain + fingerprint (invalidation cache),
- targets résolues et options associées,
- dépendances transitives résolues,
- variables d'environnement interpellées.

## Commandes

- `help` : aide generale ou par sous-commande.
- `fmt` : formate un fichier Muffin (si supporte).
- `check` : valide sans build, option `--emit` possible.
- `resolve` : génère `Muffinconfig.mcfg` (équivalent fonctionnel à `build muffin`).
- `print` : affiche une vue resolue (workspace, targets, etc.).
- `graph` : exporte le graphe (texte, DOT, etc.).
- `version` : version de Muffin.

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

## Format `Muffinconfig.mcfg` (apercu)

```text
mcfg 1

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
- `MUFFIN_EMIT` : chemin par défaut de `Muffinconfig.mcfg`.
- `MUFFIN_OFFLINE` : active le mode offline.
- `VITTE_BUILD` : chemin explicite vers le driver `build`.
- `VITTE_STEEL` : chemin explicite vers `steel`.

## Fichiers

- `muffin` ou `Muffinfile` : configuration principale.
- `Muffinconfig.mcfg` : configuration résolue (artefact canonique), consommée par Steel.
- `Steel/Steelfile` : point d’entrée des règles Steel (targets/rules).

## Documentation

- [ARCHITECTURE.md](ARCHITECTURE.md) — Détails internes, pipeline, modules
- [src/MODULE_ORGANIZATION.md](src/MODULE_ORGANIZATION.md) — Organisation des fichiers source
- Manpage: [doc/muffin.1](doc/muffin.1)

## Licence

Voir `COPYING`.
