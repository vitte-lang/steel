---
title: FAQ
slug: /faq/
description: "Questions fréquentes sur Muffin : buildfile (.muf), binaire de build (.mff), graph, cache, sandbox, targets et reproductibilité."
---

# FAQ

## Muffin, c’est quoi exactement ?

Muffin est un **orchestrateur de compilation** basé sur un **graphe d’actions typées**.

- Un fichier `Muffinfile` / `build.muf` décrit **quoi produire**, **comment**, et sous quelles **politiques** (sandbox/capsules, cache/store, plans).
- Muffin résout ce buildfile et produit un **binaire de build** `*.mff` qui fige la config + le graphe.
- Puis Muffin exécute ce `.mff` pour produire les artefacts (bins, libs, bundles, outputs).

---

## Quelle différence entre `.muf`, `.muff` et `.mff` ?

- `*.muf` : le **buildfile** (déclaratif). C’est la source de vérité de la configuration et du graphe.
- `*.muff` : convention de projet pour des buildfiles “par répertoire” (optionnel). Un projet peut en avoir plusieurs, puis les agréger dans un buildfile master.
- `*.mff` : le **binaire de build** généré par Muffin après validation/résolution. Il contient un snapshot stable : graph, tables (tools/capsules/stores), exports, plans, résolutions.

En pratique :

- tu édites des `.muf` / `.muff`
- Muffin génère un `.mff`
- Muffin exécute le `.mff`

---

## Pourquoi générer un `.mff` ? Pourquoi pas exécuter le `.muf` directement ?

Le `.mff` sert à :

- **stabiliser** une configuration résolue (mêmes entrées → même graph normalisé)
- **accélérer** (parsing/validation/résolution faits une fois)
- **auditer** (inspection/dump/verify)
- **reproduire** (un `.mff` se transporte mieux qu’un environnement implicite)

C’est l’unité “portable” : un build devient un artefact en soi.

---

## Les `.mff` sont des binaires de compilation du projet ?

Oui : un `.mff` représente la **compilation au sens orchestration**.

Il enregistre :

- l’arborescence et les entrées (globs/fichiers/valeurs)
- le graphe (bakes, ports, edges)
- les paramètres de toolchains/outils
- les politiques (capsules/sandbox, store/cache)
- les exports/plans

Ensuite, l’exécution du `.mff` produit les binaires du projet (ex: `.o/.obj`, `.a/.so/.dylib`, `.dll/.exe`, bundles, etc.).

---

## Comment se déroule le cycle “configure → build” ?

1) **configure** : Muffin lit `Muffinfile`/`build.muf`, valide et résout, puis génère `project.mff`.
2) **build** : Muffin exécute `project.mff` (scheduler + store + capsules), et produit les artefacts.

Commandes usuelles :

```bash
muffin configure --out .muffin/project.mff
muffin build --mff .muffin/project.mff --plan default
```

---

## `build muffin`, ça correspond à quoi ?

Dans beaucoup de projets, `build muffin` est un **wrapper** ergonomique.

Attendu :

- `build muffin` : exécute le plan par défaut
- `build muffin <plan>` : exécute un plan nommé
- flags : `-all`, `-debug`, `-release`, `-clean`, `-j`, `-watch`, `-why`, `-graph`, `-D KEY=VALUE`

L’implémentation réelle reste Muffin (CLI `muffin ...`).

---

## Muffin marche pour quels langages ?

Muffin est agnostique : il orchestre des **outils**.

Si tu peux décrire la chaîne de build en étapes (compiler, linker, packager, generator), Muffin peut piloter :

- C/C++ (obj + archives + linkage)
- Rust, C#, Java/Kotlin, Go, Swift, Zig, etc.
- projets mixtes (interop multi-toolchain)

La contrainte n’est pas “le langage”, mais la disponibilité des **outils** et la capacité à décrire les entrées/sorties.

---

## Erreurs frequentes (CLI)

### `error[U001]` commande inconnue

Fix:
- `muffin --help` pour la liste des commandes

### `error[C001]` config introuvable/invalide

Fix:
- verifier `--root` et `--file`

### `error[P001]` parsing MUF

Fix:
- verifier `!muf 4` et les blocs `[tag] ..`

### `error[X001]` tool execution failed

Fix:
- verifier `PATH` ou utiliser `--toolchain <dir>`

### `error[IO01]` erreur I/O

Fix:
- verifier permissions et chemins

## Voir aussi

- [Troubleshooting](/troubleshooting)

## Docs generees

- [Generated](/generated)

## Et pour les machines anciennes / récentes ?

Muffin vise l’uniformisation via :

- séparation **host/target**
- graph explicite (pas d’implicite “machine locale”)
- sandbox optionnelle (capsules)
- store content-addressed (reproductibilité)

Sur une machine plus ancienne, ça dépend surtout de :

- toolchain disponible
- compatibilité du target
- ressources (RAM/CPU/disque)

---

## Linux / Windows / macOS / BSD / Solaris : support ?

Objectif : couvrir tout l’écosystème via **targets** et **toolchains**.

Conventions :

- Linux/BSD/Solaris : objets `*.o`, libs `lib*.a` / `lib*.so`, exe sans extension
- Windows : objets `*.obj`, libs `*.lib`, exe `*.exe`, shared `*.dll`
- macOS : `*.o`, libs `lib*.a` / `lib*.dylib`

Le support effectif dépend du catalogue de targets et des backends toolchain/sandbox activés dans le projet.

---

## C’est quoi un “bake” ?

Un **bake** est un nœud du graphe :

- il déclare des **ports in/out** typés
- il construit des “ingredients” (`make glob/file/text/value`)
- il exécute des outils via `run tool ...` (avec mappings takes/emits)
- il produit des artefacts et peut forcer un chemin final via `output ... at ...`

Un bake est conçu pour être :

- déterministe
- cacheable
- parallélisable

---

## C’est quoi un “plan” ?

Un plan est un scénario d’exécution :

- `run exports` : construit ce qui est exporté
- `run bake.port` : lance une sortie précise

C’est le point d’entrée logique d’un `build`.

---

## “export”, ça sert à quoi ?

`export bake.out` marque une sortie comme :

- visible
- buildable via `-all` / `run exports`

Ça évite de “tout construire” par défaut et formalise ce qui est public.

---

## C’est quoi le store (cache) et comment l’invalidation marche ?

Le store conserve des sorties en fonction d’un **digest** :

- hash des inputs (fichiers triés + metadata selon mode)
- hash des args/outils
- hash de la config résolue

Modes :

- `content` : content-addressed (recommandé)
- `mtime` : invalidation timestamps
- `off` : désactivé

Debug :

```bash
muffin why bake.out
muffin graph --format dot --out build.dot
```

---

## Capsules (sandbox) : ça isole quoi ?

Une capsule est une policy d’exécution :

- env : allow/deny
- fs : allow_read / allow_write / deny / allow_write_exact
- net : allow/deny
- time : stable true/false

Objectif : réduire l’implicite et rendre les builds auditables.

---

## Muffin peut “décompiler” un `.mff` ?

Oui : tu peux inspector/dumper et reconstruire la vue “architecture” :

- graph
- inputs
- outils
- policies

Exemples :

```bash
muffin mff info project.mff
muffin mff dump project.mff --format json
muffin decompile project.mff --out out/architecture
```

À noter : la décompilation donne la configuration et le graphe, mais la reconstruction complète d’un build sur une autre machine dépend des toolchains disponibles.

---

## Peut-on reconstruire sur “n’importe quelle machine” ?

Le `.mff` est portable, mais la reconstruction complète dépend de :

- compatibilité des targets
- toolchains/outils existants
- bibliothèques système (CRT, SDK, sysroot)

Muffin vise à rendre ces contraintes explicites (tools + hints + capsules), pour éviter les surprises.

---

## Comment je force un profil / une target / une variable ?

Overrides typiques :

```bash
muffin configure --profile release --target x86_64-unknown-linux-gnu
muffin configure -D profile=release -D target=x86_64-unknown-linux-gnu
```

Puis build :

```bash
muffin build --mff .muffin/project.mff --plan release
```

---

## Comment je build tout ?

```bash
muffin build -all
```

Conventions : ça construit tous les `export` (ou tout le workspace selon policy du buildfile).

---

## Comment je nettoie ?

```bash
muffin clean --all
# ou
muffin clean --cache
muffin clean --artifacts
```

---

## Comment je debug un build qui “rebuild trop” ?

- `muffin why <artifact>` : explique la chaîne et l’invalidation
- `muffin graph --format dot` : dump du DAG
- `muffin build -vv` : logs détaillés

---

## Et le mode dev (watch) ?

```bash
muffin watch --plan default -j 8
```

Le watch reconfigure/rebuild de façon incrémentale sur événements FS (selon backend).

---

## Manifest vs Buildfile : pourquoi séparer ?

- **manifest** : identité, deps, publication, metadata (portable, “package view”).
- **buildfile** : exécution, graph, sandbox, outputs, scheduling (runtime view).

Ça permet :

- d’avoir un manifest stable même si le graph évolue
- de supporter plusieurs graphs (dev/ci/release) sans casser l’identité du projet

---

## Où sont les specs officielles ?

- CLI : `docs/site/generated/cli.md`
- Config schema (buildfile) : `docs/site/generated/config-schema.md`
- Manifest schema : `docs/site/generated/manifest-schema.md`
- Target schema : `docs/site/generated/target-schema.md`
