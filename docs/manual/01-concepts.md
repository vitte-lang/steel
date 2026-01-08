# Concepts

Cette section formalise les concepts fondamentaux de Muffin : **buildfiles**, **binaire de compilation `.mff`**, **graphe (DAG)**, **bakes/ports/wires**, **tools**, **capsules**, **store/cache**, **targets** et **introspection**.

---

## 1) Buildfiles (`*.muf` / `*.muff`)

Un buildfile décrit **ce qu’on veut construire** et **comment**.

- Déclare des entités top-level : `store`, `capsule`, `var`, `profile`, `tool`, `bake`, `wire`, `export`, `plan`, `switch`.
- Est **déclaratif** : pas de logique impérative opaque, uniquement des déclarations et des connexions.
- Peut être **segmenté** :
  - `main.muff` par répertoire,
  - `master.muff` à la racine pour agréger.

Le buildfile peut servir à piloter des projets mono-langage ou multi-langages : la construction est une orchestration de **tools**.

---

## 2) Binaire de compilation (`*.mff`)

Un fichier `*.mff` est le **fruit de la phase configure**.

C’est un **binaire de compilation** :

- Vue normalisée du build (DAG complet)
- Expansion des globs (`src/**/*.c` → liste exhaustive)
- Chemins normalisés (stables cross-platform)
- Valeurs résolues (profiles, targets, variables)
- Outils/toolchains effectifs + paramètres
- Empreintes d’invalidation (cache) + trace outillable

Propriétés :

- **Stable** : indépendant du format source du buildfile
- **Portable** : inspectable sur toute machine
- **Outillable** : base pour `decompile`, `why`, `graph`

Limite : reproduire les binaires dépend de la disponibilité des toolchains.

---

## 3) Graphe de build (DAG)

Muffin modélise le build comme un **graphe acyclique orienté**.

- Les nœuds sont des **bakes**.
- Les arêtes sont des dépendances explicites via **wires**.
- L’exécution est un **ordre topologique**.

Avantages :

- Parallélisation sûre (`-j`)
- Incrémental déterministe
- Cache fin (au niveau nœud/artefact)

---

## 4) Bake

Un `bake` est une **unité de calcul** (nœud du DAG).

Il expose :

- des **ports d’entrée** (`in`) typés
- des **ports de sortie** (`out`) typés
- une ou plusieurs actions : `make` (inputs) et `run tool` (exécution)

### Ports (`in` / `out`)

Les ports sont des contrats typés.

- `in name: type`
- `out name: type`

Exemples de types logiques :

- `src.glob` : liste de fichiers
- `bin.obj` : objet compilé
- `lib.static` : archive statique
- `bin.exe` : exécutable

Le type est un **type logique** : l’extension réelle dépend du target.

---

## 5) Wire

`wire` connecte un port `out` vers un port `in`.

- `wire a.out -> b.in`

C’est la dépendance explicite :

- ordonne le build
- transporte la valeur/artefact
- permet d’expliquer un rebuild (`why`)

---

## 6) Export

`export` marque une sortie comme **buildable** (cible publique).

- `export app.exe`

Les exports sont généralement ce que `-all` (ou un plan “all”) vise.

---

## 7) Plan

Un `plan` décrit un **scénario d’exécution**.

- `plan default` : plan par défaut
- `plan release` : plan dédié packaging/optimisation
- `plan ci` : plan orienté CI/test

Le plan référence :

- `run exports` (tout ce qui est exportable)
- `run <bake.port>` (cible précise)

---

## 8) Switch

`switch` mappe des flags CLI à des actions de configuration.

- sélection de profil : `-debug`, `-release`
- sélection de target : `--linux-x64`, `--win-x64-msvc`
- sélection de plan : `-all`, `-local`

Cela fournit une ergonomie stable et portable.

---

## 9) Tool

Un `tool` représente un exécutable déclaré.

- `exec` : chemin/nom
- `expect_version` : contrainte (optionnelle)
- `sandbox` : exécution sous policy
- `capsule` : policy attachée

Les **tools** permettent l’uniformisation multi-langages : compilateurs, linkers, générateurs, runners, packagers.

---

## 10) run tool

`run tool` décrit l’appel d’un tool avec mapping ports → flags.

- `takes <port> as "--flag"`
- `emits <port> as "--out"`
- `set "--key" <value>`

L’objectif est de figer :

- l’API d’invocation
- les arguments effectifs
- l’invalidation (hash des inputs + args + toolchain)

---

## 11) Capsule

Une `capsule` est une **policy d’exécution**.

Champs :

- `env` : allow/deny de variables
- `fs` : allow_read/allow_write/deny
- `net` : allow/deny
- `time` : `stable true/false`

Notes :

- Le modèle est portable.
- L’enforcement dépend des capacités OS (best-effort possible).

---

## 12) Store (cache)

Un `store` décrit un cache d’artefacts.

- `mode content` : content-addressed (recommandé)
- `mode mtime` : compat (moins déterministe)
- `mode off` : désactivé

Le cache est alimenté par les empreintes du `.mff`.

---

## 13) Targets

Un **target** décrit pour qui on construit.

- Triple OS/ARCH/ABI (ex: `x86_64-unknown-linux-gnu`)

Impact :

- extensions (`.o` vs `.obj`, `.a` vs `.lib`, `.exe`, `.so/.dylib/.dll`)
- flags toolchain
- compatibilité d’exécution

---

## 14) Profiles

Un **profile** groupe des réglages cohérents.

- `debug` : checks, symboles, opt faible
- `release` : opt élevée, stripping possible
- `ci` : réglages orientés intégration

Les profiles doivent être résolus en valeurs effectives dans `.mff`.

---

## 15) Incrémental

Le build est incrémental : Muffin rejoue le minimum.

Causes typiques d’invalidation :

- input modifié
- glob expansion différente
- arguments changés
- toolchain/version changée
- capsule/policy changée
- target/profile changés

---

## 16) Introspection

Muffin expose des commandes d’analyse :

- `decompile` : reconstitue DAG + config
- `why` : explique un rebuild
- `graph` : export DOT/JSON
- `doctor` : diagnostic environnement

Le `.mff` est la source de vérité pour ces vues.

---

## 17) Glossaire (rapide)

- **Buildfile** : `*.muf/*.muff` (déclaratif)
- **`.mff`** : binaire de compilation (contrat gelé)
- **Bake** : nœud du DAG
- **Port** : entrée/sortie typée
- **Wire** : liaison `out → in`
- **Tool** : exécutable déclaré
- **Capsule** : policy sandbox
- **Store** : cache
- **Export** : cible publique
- **Plan** : scénario d’exécution
