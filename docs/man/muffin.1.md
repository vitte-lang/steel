# muffin(1)

## NAME

**muffin** — interface de commande Muffin : configuration, build, introspection et maintenance (multi-langages)

## SYNOPSIS

```text
muffin <command> [options]

# Phase configuration (wrapper)
build muffin [plan] [flags]

# Phase build (exécution depuis `.mff`)
Muffin build [options]
```

## DESCRIPTION

`muffin` fournit la CLI de Muffin : analyse des buildfiles (`*.muf` / `*.muff`), génération du **binaire de compilation** `.mff`, exécution du build, introspection (audit/graph/why) et maintenance (clean/doctor).

Muffin sépare strictement :

- **configure** : parse/validate/resolve → **écrit** `Muffinconfig.mff`.
- **build** : lit `Muffinconfig.mff` → exécute le DAG via des **tools déclarés**.

Le fichier `.mff` est un **binaire de compilation** normalisé : graphe (DAG), fichiers (globs développés), targets/profils, tools/toolchains et empreintes d’invalidation. Il garantit une configuration uniforme et outillable pour des projets mono-langage ou mixtes (C/C++/C#/Rust/Vitte, etc.).

## COMMANDS

### configure

- `muffin configure [options]`
  - Génère un `.mff` à partir d’un buildfile.
  - Voir : [muffin-configure(1)](./muffin-configure.1.md)

### build

- `Muffin build [options]`
  - Exécute le build depuis un `.mff`.
  - Voir : [muffin-build(1)](./muffin-build.1.md)

### decompile

- `muffin decompile <path> [options]`
  - Audit : reconstitue architecture/config depuis `.mff` ou `.muff`.
  - Voir : [muffin-decompile(1)](./muffin-decompile.1.md)

### why

- `muffin why <artifact> [options]`
  - Explique une invalidation/rebuild (chaîne de dépendances).
  - Voir : [muffin-why(1)](./muffin-why.1.md)

### graph

- `muffin graph [options]`
  - Export/inspection du DAG.
  - Voir : [muffin-graph(1)](./muffin-graph.1.md)

### clean

- `muffin clean [options]`
  - Purge cache/artefacts.
  - Voir : [muffin-clean(1)](./muffin-clean.1.md)

### doctor

- `muffin doctor [options]`
  - Diagnostic environnement/toolchains/capsule.
  - Voir : [muffin-doctor(1)](./muffin-doctor.1.md)

## GLOBAL OPTIONS

- `-v`, `--verbose`
  - Verbosité accrue.

- `-q`, `--quiet`
  - Sorties minimales.

- `--format <fmt>`
  - Format de sortie (selon commande) : `text` (défaut), `json`.

- `--log <path>`
  - Écrit un log machine-friendly (si supporté).

## FILES

- `*.muf` / `*.muff` : buildfiles (configuration + règles)
- `Muffinconfig.mff` : binaire de compilation canonique (contrat de build)
- `./.muffin/` (ou `./.muff/`) : répertoire d’artefacts/cache (selon config)

## EXIT STATUS

- `0` : succès
- `1` : erreur d’exécution
- `2` : erreur de configuration/usage

## EXAMPLES

```text
# Configure (génère Muffinconfig.mff)
build muffin

# Configure un plan
build muffin release

# Override target
build muffin -D target=x86_64-unknown-linux-gnu

# Build depuis Muffinconfig.mff
Muffin build

# Décompiler / auditer
muffin decompile Muffinconfig.mff

# Export DOT du graphe
muffin graph --mff Muffinconfig.mff --format dot --out graph.dot

# Pourquoi un rebuild ?
muffin why out/bin/app

# Nettoyer cache/artefacts
muffin clean --scope cache

# Diagnostiquer l’environnement
muffin doctor --tools
```

## SEE ALSO

- [muffin-build(1)](./muffin-build.1.md)
- [muffin-configure(1)](./muffin-configure.1.md)
- [muffin-decompile(1)](./muffin-decompile.1.md)
- [muffin-graph(1)](./muffin-graph.1.md)
- [muffin-why(1)](./muffin-why.1.md)
- [muffin-clean(1)](./muffin-clean.1.md)
- [muffin-doctor(1)](./muffin-doctor.1.md)
