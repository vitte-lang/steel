# Introduction

# Introduction

Muffin est un orchestrateur de build **multi-langages** et **multi-plateformes**. Il fournit un modèle de construction unifié basé sur un **graphe de tâches typé** (DAG), et sépare strictement la phase **configuration** de la phase **construction**.

Le cœur du système est un artefact canonique : **`Muffinconfig.mff`**.

- `*.muf` / `*.muff` : buildfiles déclaratifs (configuration + règles)
- `*.mff` : **binaire de compilation** gelé et normalisé (contrat de build)

`Muffinconfig.mff` capture l’architecture du build (DAG), la liste exhaustive des inputs (globs développés), les targets/profils, les tools/toolchains et les empreintes d’invalidation. Il garantit une configuration uniforme, portable et outillable.

---

## Modèle (haut niveau)

### Deux phases

1. **Configure** — `build muffin`
   - Charge les buildfiles (`*.muf` / `*.muff`)
   - Valide la cohérence (schémas, types, règles)
   - Résout les valeurs (defaults, héritage, overrides)
   - Développe les globs et normalise les chemins
   - Émet `Muffinconfig.mff`

2. **Build** — `Muffin build`
   - Lit `Muffinconfig.mff`
   - Construit l’ordre d’exécution (topologie du DAG)
   - Exécute les étapes via les **tools déclarés**
   - Gère l’incrémental et le cache

### Pourquoi un binaire `.mff` ?

- **Stabilité** : configuration normalisée, indépendante du format d’entrée
- **Portabilité** : l’artefact est lisible/inspectable sur toute machine
- **Observabilité** : introspection complète (graph, why, decompile)
- **Détermination** : empreintes d’invalidation figées (inputs + outils + args)

---

## Concepts clés

### Bake graph

- **`bake`** : nœud du DAG (unité de calcul)
- **`in` / `out`** : ports typés (contrats d’entrées/sorties)
- **`wire`** : connexion explicite `out → in`
- **`export`** : sortie buildable (cible publique)
- **`plan`** : scénario d’exécution (pipeline haut niveau)

### Tools

Un **tool** est un exécutable déclaré de manière stable :

- chemin (`exec`)
- contrainte de version (`expect_version`, si activée)
- exécution sandboxable (`sandbox` + `capsule`)

Muffin n’est pas lié à un langage : chaque langage ou étape (compile, link, archive, generate, test, package) est modélisé comme un ou plusieurs tools.

### Capsule (sandbox policy)

Une **capsule** décrit une politique d’exécution hermétique :

- `env` : variables autorisées/interdites
- `fs` : lecture/écriture autorisées, deny-lists
- `net` : autoriser/refuser le réseau
- `time` : stabilité temporelle

L’enforcement dépend des capacités du système, mais le **modèle** reste identique.

### Store (cache)

Un **store** est un cache d’artefacts :

- `content` : content-addressed (recommandé)
- `mtime` : basé sur dates (mode compat)
- `off` : désactivé

---

## Artefacts de build (exemples)

Selon targets/plateformes et toolchains :

- Objets : `*.o` / `*.obj`
- Archives statiques : `*.a` / `*.lib`
- Librairies partagées : `*.so` / `*.dylib` / `*.dll`
- Exécutables : `*.exe` (Windows) ou sans extension (Unix)
- Artefacts spécifiques possibles (optionnels) : `*.vo` / `*.va`

---

## Workflow recommandé

### 1) Placer un buildfile

À la racine du workspace :

- `Muffinfile` (recommandé) ou `build.muf`

Ou bien segmentation par répertoire :

- `main.muff` dans chaque dossier
- `master.muff` à la racine (agrégation)

### 2) Configurer

```text
build muffin
```

Produit : `./Muffinconfig.mff`

### 3) Construire

```text
Muffin build
```

---

## Introspection et outillage

Muffin expose des commandes orientées analyse :

- **décompiler** : reconstituer l’architecture depuis `.mff` ou `.muff`
- **why** : expliquer une invalidation/rebuild
- **graph** : exporter le DAG
- **doctor** : diagnostiquer toolchains et capacités d’exécution

Exemples :

```text
muffin decompile Muffinconfig.mff
muffin why out/bin/app
muffin graph --format dot --out graph.dot
muffin doctor --tools
```

---

## Compatibilité (machines anciennes / récentes)

- Sur machines anciennes : privilégier `-j` faible, désactiver `-watch`, réduire la pression I/O
- Sur machines récentes : parallélisme élevé, cache content-addressed, watch via events si disponible

Le contrat `.mff` est portable, mais la reproduction des binaires dépend de la présence d’outils compatibles (compilateurs, linkers, runtimes).

---

## Glossaire

- **Buildfile** : fichier `*.muf/*.muff` (déclaratif)
- **`.mff`** : binaire de compilation (contrat gelé)
- **DAG** : graphe acyclique orienté (ordre d’exécution)
- **Bake** : nœud de calcul
- **Port** : entrée/sortie typée
- **Wire** : liaison `out → in`
- **Tool** : exécutable déclaré
- **Capsule** : policy sandbox
- **Store** : cache d’artefacts

---

## Suite

- Manpages : `docs/man/`
- Exemples : `docs/examples/`
- API : `docs/api/`