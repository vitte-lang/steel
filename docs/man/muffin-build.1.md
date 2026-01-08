# muffin-build(1)

# muffin-build(1)

## NAME

**muffin-build** — exécute la construction à partir d’un binaire de compilation `.mff` (DAG + tools)

## SYNOPSIS

```text
Muffin build [options]
Muffin build --plan <name> [options]
Muffin build --mff <path> [options]
```

## DESCRIPTION

`Muffin build` exécute la phase **construction** à partir d’un binaire `.mff` (par défaut `./Muffinconfig.mff`).

Le fichier `.mff` est un **binaire de compilation** : il fige un graphe normalisé (DAG), la liste exhaustive des fichiers (globs développés), les outils/toolchains, les paramètres effectifs, et les empreintes d’invalidation. La phase build lit ce contrat et rejoue uniquement les étapes nécessaires (incrémental/cache).

Muffin est multi-langages : la construction est orchestrée via des **tools déclarés** (compilateurs, linkers, générateurs, runners de tests, packagers). Les artefacts produits dépendent des targets (ex: `*.o`/`*.obj`, `*.a`/`*.lib`, `*.so`/`*.dylib`/`*.dll`, `*.exe`).

## OPTIONS

### Sélection du contrat

- `--mff <path>`
  - Chemin du binaire `.mff` à consommer (défaut : `./Muffinconfig.mff`).

- `--plan <name>`
  - Exécute un plan spécifique défini dans le contrat `.mff` (défaut : plan `default` si présent, sinon le premier plan).

### Exécution

- `-j <n>`
  - Parallélisme (scheduler). `-j 1` force l’exécution séquentielle.

- `--jobs <n>`
  - Alias de `-j`.

- `--dry-run`
  - N’exécute pas les tools : affiche l’ordre et la liste des étapes qui seraient jouées.

- `--keep-going`
  - Continue l’exécution malgré des erreurs (le résultat final reste en erreur si au moins une étape échoue).

- `--fail-fast`
  - Stoppe au premier échec (défaut).

- `--timeout <ms>`
  - Timeout par étape (si supporté par le runner/OS).

### Cache

- `--cache <mode>`
  - Override du mode cache : `content`, `mtime`, `off`.
  - `content` (recommandé) utilise une empreinte déterministe.

- `--no-cache`
  - Alias : `--cache off`.

- `--rebuild`
  - Force la reconstruction (ignore le cache) pour les étapes concernées.

### Ciblage

- `--target <triple>`
  - Force un target (override runtime). À utiliser si le `.mff` contient plusieurs targets ou si l’impl le supporte.

- `--host <triple>`
  - Force un host (rare, usage intégration).

### Sorties / logs

- `-v`, `--verbose`
  - Verbosité accrue.

- `-q`, `--quiet`
  - Sorties minimales.

- `--log <path>`
  - Écrit un log machine-friendly.

- `--format <fmt>`
  - Format d’affichage (selon impl) : `text` (défaut), `json`.

### Debug d’invalidation

- `--why <artifact>`
  - Explique pourquoi un artefact doit être rejoué (chaîne inputs → étapes → outputs).

- `--graph [--format dot|json]`
  - Exporte le DAG et quitte (ou en plus, selon impl).

## EXIT STATUS

- `0` : build réussi
- `1` : erreur de build (au moins une étape échouée)
- `2` : erreur de contrat/configuration (mff invalide, plan absent, etc.)

## EXAMPLES

```text
# Build depuis le contrat par défaut
Muffin build

# Build en parallèle
Muffin build -j 16

# Build d’un plan nommé
Muffin build --plan release

# Consommer un `.mff` spécifique
Muffin build --mff ./out/Muffinconfig.mff

# Dry-run
Muffin build --dry-run

# Désactiver cache
Muffin build --no-cache

# Comprendre une invalidation
Muffin build --why out/bin/app

# Export DAG
Muffin build --graph --format dot
```

## NOTES

- La portabilité du `.mff` est structurelle (normalisation). La reproduction des binaires dépend de la disponibilité et compatibilité des toolchains (compilateurs/outils) sur la machine.
- Les sandbox `capsule` peuvent être best-effort selon OS.

## SEE ALSO

- [muffin(1)](./muffin.1.md)
- [muffin-configure(1)](./muffin-configure.1.md)
- [muffin-decompile(1)](./muffin-decompile.1.md)
- [muffin-why(1)](./muffin-why.1.md)
- [muffin-graph(1)](./muffin-graph.1.md)
- [muffin-clean(1)](./muffin-clean.1.md)
- [muffin-doctor(1)](./muffin-doctor.1.md)