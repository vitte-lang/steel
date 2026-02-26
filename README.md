# Steel

![Steel](https://img.shields.io/badge/Steel-config-orange)

Steel est la couche de configuration **declarative** du build. En bref : il lit le `steelconf`, valide le workspace (packages, profils, toolchains, targets) et produit un **artefact stable** (`steel.log` / `steelconfig.mff`). Vitte s appuie ensuite dessus pour executer le build de facon deterministe.

## Site downloads policy

- La page Downloads pointe vers un **lien unique GitHub Releases** pour toutes les plateformes.
- La page Downloads expose aussi le lien **VS Code extension** (Marketplace).
- Si vous mettez a jour le copy site, gardez cette convention pour eviter les regressions editoriales.

### Local QA

Depuis `docs/angular`:

```text
npm ci
npm run build:verify
```

Depuis la racine du repo:

```text
git diff --quiet -- docs/site
```

Avant d'ouvrir une PR qui touche les docs/outillage, lance aussi:

```text
./scripts/verify-editorconfig.sh
```

Cela valide les sections et l'indentation attendue pour `steelconf` et `*.muf` dans le flux QA local.

### CI quick jobs

- `site-urls-quick`: verifie vite les URLs Releases/Marketplace.
- `site-quick`: lance build site + verification de synchronisation `docs/site`.
- `doc-editorial-quick`: lance les tests editoriaux README.

### When to run qa-site-local.sh

- Avant une PR qui modifie `docs/angular` ou `docs/site`.
- Avant une release docs pour verifier la chaine QA locale complete.



## Configuration declarative (exemple)

Un `steelconf` décrit explicitement le **workspace**, les **profils**, les **targets** et la **toolchain**. Pas de regles ad-hoc : la structure reste lisible, composable et stable.


## CLI (raccourci)

Commandes (details dans `doc/manifest.md`). Pense ceci comme un petit pense-bete : le coeur reste `steel run steelconf`.
- `steel run` : lance un build (alias de `build`)
- `steel build` : build unique (alias de `run`)
- `steel doctor` : diagnostic environnement
- `steel editor` : ouvrir Mitsou editor
- `steel editor-setup` : installe la syntaxe steelconf (vim/nano/emacs/etc)
- `steel help` : aide
- `steel version` : version

Exemples:
```text
steel run steelconf
steel build steelconf
steel toolchain doctor
```

## Steel Editor (steecleditor)

Steel fournit un editeur terminal integre pour `steelconf` : **steecleditor**. C est un editeur simple et rapide, pense pour iterer sur la config sans quitter le terminal.
L'installation `editor-setup` configure la coloration syntaxique steelconf pour Vim, Nano et Emacs.

Lancer:
```text
steel editor
steel editor path/to/steelconf
steel editor *.c
steel editor *.cpp
steel editor *.py
```

Fonctions principales:
- Autocompletion steelconf (blocs/directives + snippets, type VSCode).
- Indentation intelligente, auto-close des crochets.
- Recherche, aller a une ligne, remplacement.
- Tabs multi-fichiers, recent files, session restore.
- Diff rapide vs disque et mini-map.
- Syntax highlight (steelconf + C/C++/Python/Java/etc).
- Autosave optionnel, mode lecture seule, themes.

Raccourcis utiles:
- `Ctrl+S` save, `Ctrl+O` open, `Ctrl+Q` quit
- `Ctrl+F` search, `F3` next match, `Ctrl+L` go to line
- `Ctrl+P` find file, `F2` recent files
- `Ctrl+R` steel run, `Ctrl+Shift+E` jump run error
- `Ctrl+Shift+G` glob preview, `Ctrl+Shift+I` insert snippet
- `Ctrl+Shift+L` completion debug (language + completion sources)
- `Ctrl+Shift+Alt+L` debug verbose (extension + scanners + filters)

Config (optionnel):
```text
~/.config/steel/steecleditor.conf
```


## Voir aussi

- `doc/manifest.md#liste-rapide-commandes`
- `doc/manifest.md#flags-frequents`
- `doc/manifest.md#cli-complete`


### Flux de traitement

```
┌─────────────────────────────────────────────────────────────┐
│                    PHASE CONFIGURATION                      │
│                      (build steel)                         │
└─────────────────────────────────────────────────────────────┘

1. CHARGEMENT
   ├── steelconf (workspace + packages + profils)
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
   └── steel.log (configuration gelée et normalisée)

┌─────────────────────────────────────────────────────────────┐
│                    PHASE CONSTRUCTION                       │
│                      (build vitte)                          │
└─────────────────────────────────────────────────────────────┘

Vitte lit steel.log et :
  ├── Construit le DAG des étapes
  ├── Résout les dépendances de fichiers
  ├── Exécute l'ordre topologique
  └── Gère l'incrémental et le cache
```

### Composants principaux

#### Parser (`arscan.rs`, `read.rs`)
- Analyse lexicale et syntaxique des fichiers Steel
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
- Sérialisation de la configuration 
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

**Steel** (Déclaratif)
- *Ce qu'on veut construire*, *comment* (profils), *pour qui* (targets)
- Séparation claire entre configuration et exécution
- Validation et cohérence

**Vitte** (Exécutif)
- *Comment y parvenir* : graphe de tâches, parallélisation, cache
- Isolation des détails d'exécution (compilateur, flags réels)
- Diagnostics de performance et débogage

**steel.log** (Contrat)
- Configuration **gelée** et **normalisée**
- Invalidation automatique du cache en cas de changement


## Variables d'environnement

- `MUFFIN_FILE` : chemin du fichier Steel.
- `MUFFIN_PROFILE` : profil par défaut.
- `MUFFIN_EMIT` : chemin par défaut de `steel.log`.
- `MUFFIN_OFFLINE` : active le mode offline.
- `VITTE_BUILD` : chemin explicite vers le driver `build`.
- `PATH` : s’assurer que `steel` et `vitte` sont accessibles (ou utiliser `./steel` depuis la racine).

## Fichiers

- `steel` ou `steelconf` : configuration principale.
- `steel.log` : configuration gelée et normalisée (artefact canonique) + trace outillable (graph, inputs, outils, empreintes), consommée par Vitte.

## Documentation

- [docs/quickstart.md](docs/quickstart.md) — Quickstart MUF v4.1
- [docs/cookbook.md](docs/cookbook.md) — Recettes (C, lib+app, multi-tool)
- [docs/migration.md](docs/migration.md) — Migration vers MUF v4.1
- [docs/faq-erreurs.md](docs/faq-erreurs.md) — FAQ erreurs (codes + fixes)
- [docs/observabilite.md](docs/observabilite.md) — Observabilite (doctor, cache, logs)
- [docs/troubleshooting.md](docs/troubleshooting.md) — Troubleshooting (guide long)
- [docs/reference/formats/index.md](docs/reference/formats/index.md) — Formats + versioning
- [ARCHITECTURE.md](ARCHITECTURE.md) — Détails internes, pipeline, modules
- [src/MODULE_ORGANIZATION.md](src/MODULE_ORGANIZATION.md) — Organisation des fichiers source
- Manpage: [doc/steel.1](doc/steel.1)

## Licence

Voir `COPYING`.

# Steel

![Steel](https://img.shields.io/badge/Steel-config-orange)

Steel est le **compilateur de configuration** du build Vitte.

- **Entrée** : un seul fichier à la racine du dépôt : **`steelconf`**.
- **Sortie** : un **binaire universel** de configuration **`steelconf.mub`** (Universal Binary Config), identique d’une machine à l’autre.
- **Consommateur** : **Vitte** lit `steelconf.mub` pour exécuter le build (DAG, cache, incrémental) de manière déterministe.

L’objectif : **un modèle unique**, **un workflow unique**, **un artefact unique**.


2) **Construire (build)**



- lit la config résolue depuis `target/`
- exécute le DAG (compile/link/test/package)
- gère incrémental + cache

---

## Layout généré dans `target/`

Par défaut, Steel crée (ou met à jour) une arborescence standard :

```text
target/
  steel/
    config.mub
    graph.json
    fingerprints.json
  build/
    <triple>/
      <profile>/
        obj/
        gen/
  out/
    <triple>/
      <profile>/
        bin/
        lib/
  cache/
    cas/
    meta/
```


---

## Commandes

### Configuration


Flags usuels :

### Introspection




---

## Fichiers

- **`steelconf`** : source de configuration (unique).
- **`target/`** : **racine canonique** de toutes les sorties (config résolue + build + cache + outputs).

---

## Licence

Voir `COPYING`.
# Steel

![Steel](https://img.shields.io/badge/Steel-config-orange)

## Licence

Voir `COPYING`.
