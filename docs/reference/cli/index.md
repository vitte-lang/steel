# CLI

# CLI

Référence CLI (index) de **Muffin**.

Muffin est un orchestrateur de build **multi-langages** et **multi-plateformes**. La CLI est structurée en deux phases :

- **Configure** : construit un contrat exécutable — le **binaire de compilation** `Muffinconfig.mff`
- **Build** : exécute le DAG depuis `Muffinconfig.mff`

---

## 1) Entrées et artefacts

### Entrées

- Buildfiles : `*.muf` / `*.muff`
  - noms usuels : `Muffinfile`, `build.muf`, `main.muff`, `master.muff`

### Sorties

- Binaire de compilation : `*.mff`
  - canonique : `Muffinconfig.mff`

### Sorties de build (multi-plateformes)

Muffin manipule des **types logiques** (ex: `bin.obj`, `lib.static`, `bin.exe`) et résout les extensions selon `target`.

- objets : `.o` (Unix), `.obj` (Windows)
- statiques : `.a` (Unix), `.lib` (Windows)
- partagées : `.so` (Linux/BSD/Solaris), `.dylib` (macOS), `.dll` (Windows)
- exécutables : sans extension (Unix), `.exe` (Windows)

---

## 2) Commandes (vue d’ensemble)

### 2.1. Syntaxe canonique

La CLI canonique est :

```text
muffin <command> [options]
```

Commandes principales :

- `configure` : génère `.mff` depuis un buildfile
- `build` : exécute le DAG depuis `.mff`
- `decompile` : audit/décompilation (buildfile ou `.mff`)
- `graph` : export/inspection du DAG
- `why` : explication d’invalidation/rebuild
- `clean` : purge cache/artefacts
- `doctor` : diagnostic environnement/toolchains
- `mff` : outils bas niveau sur `.mff` (check/upgrade)
- `muf` : outils bas niveau sur buildfiles (lint/format/upgrade)

### 2.2. Entrypoints historiques (compat)

Deux entrypoints peuvent exister pour refléter les deux phases :

- `build muffin` → équivalent `muffin configure ...`
- `Muffin build` → équivalent `muffin build ...`

Recommandation : documenter/automatiser ces wrappers, mais garder la sémantique canonique via `muffin <command>`.

---

## 3) Global options (toutes commandes)

Options génériques (pattern) :

```text
--help
--version
--quiet
--verbose
--color auto|always|never
--format text|json|ndjson
--telemetry off|local|otlp
--telemetry-level minimal|normal|verbose
--telemetry-out <dir>
--redact-paths | --hash-paths
--locale <bcp47>
```

Notes :

- `--format` pilote les sorties machine-friendly.
- `--locale` ne doit pas impacter les champs structurés JSON.

---

## 4) `muffin configure`

### Rôle

- parse buildfiles
- validation schéma et références
- expansion des globs
- résolution `profile` / `target` / overrides
- pinning toolchains
- sérialisation du contrat → `.mff`

### Synopsis

```text
muffin configure [options]
```

### Options (pattern)

```text
--input <path>            # buildfile d’entrée (sinon auto-discovery)
--out <path>              # chemin du .mff (défaut: Muffinconfig.mff)
--plan <name>             # plan par défaut à embarquer
--all                      # construit la liste des exports (ou workspace)
--strict                   # warnings → erreurs (lint)
-D KEY=VALUE               # override variables
--profile <name>
--target <triple>
--no-cache                 # configure sans store (selon impl)
--dry-run                  # affiche la résolution sans écrire (selon impl)
```

### Exemples

```text
muffin configure
muffin configure --input build.muf --out Muffinconfig.mff
muffin configure --plan release -D target=x86_64-unknown-linux-gnu
muffin configure --strict -D profile=debug
```

---

## 5) `muffin build`

### Rôle

- lit `.mff`
- reconstruit DAG
- scheduler (parallélisme sûr)
- cache lookup/store
- exécute `run tool` sous `capsule`

### Synopsis

```text
muffin build [options]
```

### Options (pattern)

```text
--mff <path>               # défaut: Muffinconfig.mff
--plan <name>
--exports                   # exécuter uniquement les exports
-j <n>                      # parallélisme
--keep-going                # ne stoppe pas au premier échec (selon impl)
--no-cache                  # désactive cache/store
--dry-run                   # planning sans exécution
--watch                     # mode dev (FS events)
```

### Exemples

```text
muffin build
muffin build --plan release -j 16
muffin build --mff ./sub/.muff/Muffinconfig.mff
muffin build --dry-run --format json
```

---

## 6) `muffin decompile`

### Rôle

- rendre lisible un buildfile ou un `.mff`
- exporter une vue normalisée : graph, tools, targets, inputs, hashes

### Synopsis

```text
muffin decompile <path> [options]
```

### Options (pattern)

```text
--format text|json
--out <path>
--summary                  # vue condensée
--full                     # dump complet (selon impl)
```

### Exemples

```text
muffin decompile Muffinconfig.mff
muffin decompile build.muf --format json
```

---

## 7) `muffin why`

### Rôle

- expliquer pourquoi un artefact est (re)construit
- afficher la chaîne de dépendances + cause d’invalidation

### Synopsis

```text
muffin why <artifact> [options]
```

### Options (pattern)

```text
--format text|json
--depth <n>
--show-hash                # afficher empreintes inputs/args/toolchain
```

---

## 8) `muffin graph`

### Rôle

- export du DAG

### Synopsis

```text
muffin graph [options]
```

### Options (pattern)

```text
--mff <path>
--format text|json|dot
--out <path>
--plan <name>
--exports-only
```

### Exemples

```text
muffin graph --format dot --out graph.dot
muffin graph --format json --out graph.json
```

---

## 9) `muffin clean`

### Rôle

- purge contrôlée des caches/artefacts

### Synopsis

```text
muffin clean [options]
```

### Options (pattern)

```text
--scope cache|logs|out|all
--confirm                  # garde-fou non interactif
--dry-run
```

---

## 10) `muffin doctor`

### Rôle

- diagnostic toolchains
- vérification environment
- checks capsule/store

### Synopsis

```text
muffin doctor [options]
```

### Options (pattern)

```text
--tools
--capsule
--store
--format text|json
```

---

## 11) Commandes bas niveau

### 11.1. `muffin mff`

Objectif : manipuler/valider le format `.mff`.

```text
muffin mff check <file.mff>
muffin mff upgrade <file.mff> --out <new.mff>
```

### 11.2. `muffin muf`

Objectif : linter/formater/migrer les buildfiles.

```text
muffin muf lint <file.muf> --strict
muffin muf format <file.muf> [--in-place]
muffin muf upgrade <file.muf> --to <bake_version> --out <new.muf>
```

---

## 12) Variables d’environnement

Patterns recommandés :

```text
MUFFIN_LANG=fr-FR
MUFFIN_TELEMETRY=off|local|otlp
MUFFIN_OTLP_ENDPOINT=<url>
MUFFIN_HOME=<dir>
MUFFIN_STORE=<dir|url>
```

Notes :

- Les variables d’environnement sont des overrides ; l’équivalent CLI doit exister.

---

## 13) Exit status (convention)

- `0` : succès
- `1` : échec générique
- `2` : erreur d’usage (arguments)
- `3` : erreur parse/validation buildfile
- `4` : erreur `.mff` (format/compat)
- `5` : échec tool exécuté (command exit)

(selon impl)

---

## 14) Notes OS (Linux / macOS / Windows / BSD / Solaris)

- chemins : accepter `/` et `\\` en entrée ; normalisation interne stable.
- consoles Windows : encodage best-effort (préférer JSON en CI).
- watch : support variable (FS events).
- sandbox capsule : enforcement best-effort selon OS.

---

## 15) Voir aussi

- Manpages : `docs/man/index.md`
- Manuel : `docs/manual/`
- Méta : `docs/meta/`
- Ops : `docs/ops/index.md`