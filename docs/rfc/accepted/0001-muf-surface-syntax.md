# RFC 0001 (accepted)

# RFC 0001 — MUF surface syntax (accepted)

## Summary

Cette RFC définit la **syntaxe de surface** des buildfiles Muffin (`*.muf` / `*.muff`) et leurs règles de validation de base.

Un buildfile MUF décrit un **graph de build typé** (bakes/ports/wires/tools/capsules/stores/plans) qui sera compilé en un **binaire de compilation** `*.mff` pendant la phase **configure**.

- **Configure** : `build muffin` / `muffin configure` → génère `Muffinconfig.mff`
- **Build** : `Muffin build` / `muffin build` → exécute le DAG depuis `Muffinconfig.mff`

MUF est conçu pour être :

- **multi-langages** (C/C++/C#/Rust/etc.)
- **multi-plateformes** (Linux/macOS/Windows/BSD/Solaris)
- **déterministe** (tri stable, sérialisation stable)
- **auditable** (décompilation `.mff`, exports JSON)

---

## Motivation

Les systèmes de build existants ont des modèles de dépendances souvent implicites, une faible typage des artefacts, et des comportements qui varient selon l’environnement.

MUF vise un modèle :

- graph explicite (DAG)
- entrées/sorties typées
- actions déclaratives (tools + sandbox)
- cache content-addressed (store)
- compilation du buildfile en contrat binaire (`.mff`) pour stabiliser la résolution

---

## Goals

- Définir une syntaxe **orientée lignes**, simple à parser.
- Définir des constructs top-level terminés par `.end`.
- Définir une base normative : keywords, blocs, valeurs, listes.
- Définir les règles minimales de validation (références, ports, exports, plans).
- Assurer une compatibilité cross-platform (paths, encodage, newlines).

---

## Non-goals

- Définir le format binaire `.mff` en détail (c’est une RFC séparée).
- Définir une policy complète de sandbox pour chaque OS (best-effort selon OS).
- Définir un langage de scripts embedded.

---

## File extensions and discovery

- `*.muf` : buildfile principal (recommandé)
- `*.muff` : variante/overlay (autorisé)

Discovery (recommandation) :

- `Muffinfile`
- `build.muf`
- `main.muff`
- `master.muff`

---

## Encoding and newlines

- Encodage recommandé : **UTF-8**.
- Newlines acceptées : `\n` et `\r\n`.
- Commentaires : `#` jusqu’à la fin de ligne.

---

## Versioning: header

Chaque buildfile commence par un header versionné :

```text
muffin bake <int>
```

- `<int>` = version de la syntaxe/contrat MUF.
- Toute évolution non rétro-compatible doit incrémenter cette valeur.

---

## Lexical elements

### Identifiers

- `ident` : `[A-Za-z_][A-Za-z0-9_]*`

### Strings

- `"..."` avec escapes : `\"`, `\\`, `\n`, `\r`, `\t`

### Integers

- décimal : `0` ou `[1-9][0-9]*`

### Booleans

- `true` | `false`

### Lists

- `[...]` avec séparateur `,` (espaces tolérés)

---

## Top-level statements

Un buildfile contient une suite de statements top-level.

Keywords top-level :

- `store`
- `capsule`
- `var`
- `profile`
- `tool`
- `bake`
- `wire`
- `export`
- `plan`
- `switch`
- `set`

Tous les blocs sont terminés par `.end`.

---

## `set` (global)

Setter global optionnel.

Syntaxe :

```text
set <ident> <value>
```

Usage typique :

- définir un profil par défaut
- définir une variable de haut niveau

---

## `var` (typed variables)

Déclare une variable typée.

Syntaxe :

```text
var <name> : <type> = <value>
```

Types primitifs :

- `text` | `int` | `bool` | `bytes`

Types d’artefacts (noms qualifiés) :

- `<ident>.<ident>(.<ident>)*`

Exemples :

```text
var target : text = "x86_64-unknown-linux-gnu"
var debug  : bool = false
var srcs   : src.glob = "src/**/*.c"
```

---

## `store` (cache)

Déclare un store (cache) utilisable par le scheduler.

Syntaxe :

```text
store <name>
  path "..."
  mode content|mtime|off
.end
```

Règles :

- `path` est requis si `mode != off`.
- `mode content` est recommandé pour l’immutabilité.

---

## `capsule` (sandbox policy)

Déclare une capsule de sandbox.

Syntaxe :

```text
capsule <name>
  env allow ["..."] | env deny ["..."]
  fs allow_read  ["..."]
  fs allow_write ["..."]
  fs allow_write_exact ["..."]
  fs deny ["..."]
  net allow|deny
  time stable true|false
.end
```

Notes :

- l’enforcement est best-effort selon OS.
- `time stable true` implique une horloge stable et/ou une source monotonic pour durées (selon impl).

---

## `profile`

Un profil regroupe des setters appliqués lors de la résolution.

Syntaxe :

```text
profile <name>
  set <key> <value>
  set <key> <value>
.end
```

Exemples :

```text
profile debug
  set opt "0"
  set debug true
.end

profile release
  set opt "3"
  set strip true
.end
```

---

## `tool`

Déclare un exécutable utilisé par les bakes.

Syntaxe :

```text
tool <name>
  exec "..."
  expect_version "..."      # optionnel
  sandbox true|false
  capsule <capsule_name>     # optionnel
.end
```

Règles :

- `exec` requis.
- si `sandbox true`, `capsule` est fortement recommandé.

---

## `bake`

Un bake est un **nœud** du DAG.

Syntaxe :

```text
bake <name>
  in  <port> : <type>
  out <port> : <type>

  make <port> glob  "..."    # ingredient builder
  make <port> file  "..."
  make <port> text  "..."
  make <port> value "..."

  run tool <tool_name>
    takes <in_port> as "--flag"
    emits <out_port> as "--flag"
    set "--flag" <value>
  .end

  cache content|mtime|off
  output <out_port> at "./path"
.end
```

### Ports

- `in` et `out` sont typés (`type_ref`).
- Un port `make` est un **out** construit par un ingredient builder.

### `make`

`make` produit un output de type `src.glob`/etc. à partir d’un literal.

- `glob` : pattern de fichiers
- `file` : fichier unique
- `text` : string
- `value` : literal/ident (selon impl)

### `run tool`

- `takes` relie un **port** à un flag
- `emits` relie un **port out** à un flag
- `set` ajoute un flag/value constant

### `cache`

- `content` : hash sur inputs+args+toolchain (recommandé)
- `mtime` : invalidation sur timestamps
- `off` : pas de cache

### `output`

Force un chemin de sortie final (workspace root).

---

## `wire`

Connexion explicite `out → in`.

Syntaxe :

```text
wire <ref> -> <ref>
```

Références :

- `x` : variable globale
- `bake.port` : port d’un bake

Exemple :

```text
wire app_src.files -> app_obj.files
```

---

## `export`

Marque un port `out` comme exportable/buildable.

Syntaxe :

```text
export <ref>
```

Règle : la référence doit pointer vers un **port out**.

---

## `plan`

Définit un scénario d’exécution.

Syntaxe :

```text
plan <name>
  run exports
  run bake.port
.end
```

Règles :

- `plan default` est recommandé.
- `run exports` exécute tous les `export`.

---

## `switch`

Mappe des flags CLI vers des actions.

Syntaxe :

```text
switch
  flag "--debug"  set profile "debug"
  flag "--release" set profile "release"
  flag "--ci"     set plan "ci"
  flag "--all"    run exports
.end
```

Actions :

- `set <ident> <value>`
- `set plan "..."`
- `run exports | ref`

---

## Values

`value` accepte :

- `string_lit`
- `int_lit`
- `bool_lit`
- `list`
- `ident` (référence simple, selon impl)

List :

```text
[ value, value, ... ]
```

---

## Determinism requirements

Pour garantir des builds stables :

- tri stable des résultats de globs
- sérialisation `.mff` stable (ordre)
- hash content-addressed stable (`store mode content`)
- normalisation des chemins avant hashing

---

## Validation rules (minimum)

### Naming

- ident non vide
- unicité des noms top-level (tools/capsules/profiles/bakes)

### References

- `wire` : source doit être `out` ; destination doit être `in`
- `export` : cible doit être `out`
- `plan` : `run ref` doit référencer `exports` ou une ref existante

### Ports

- ports `in` et `out` d’un même bake : noms uniques
- `takes` doit référencer un port `in` existant
- `emits` doit référencer un port `out` existant

### Types

- `wire` exige la compatibilité des types (`out_type` assignable à `in_type`)

### Tool usage

- `run tool X` exige `tool X` défini
- si `tool.sandbox true` et `capsule` absent : warning (ou error en `--strict`)

### Plans

- `plan default` recommandé
- si absent : résolution par défaut définie par l’implémentation (warning recommandé)

---

## Error model

Diagnostics recommandés :

- IDs stables `MFxxxx`
- rendu texte localisé
- rendu JSON stable

Exemples d’erreurs :

- header manquant / version inconnue
- `export` pointe vers un `in`
- `wire` vers port inexistant
- `run tool` vers tool inconnu
- type mismatch (`src.glob` → `bin.obj`)

---

## Example (complete)

```muf
muffin bake 2

store local
  path "./.muffin/store"
  mode content
.end

capsule build
  env allow ["PATH"]
  fs allow_read  ["./src"]
  fs allow_write ["./out", "./.muffin"]
  net deny
  time stable true
.end

tool cc
  exec "cc"
  sandbox true
  capsule build
.end

bake app_src
  out files: src.glob
  make files glob "src/**/*.c"
.end

bake app_obj
  in  files: src.glob
  out obj: bin.obj
  run tool cc
    takes files as "--input"
    emits obj as "--output"
  .end
.end

wire app_src.files -> app_obj.files
export app_obj.obj

plan default
  run exports
.end
```

---

## Backward compatibility

- La version est portée par `muffin bake <int>`.
- Un binaire Muffin peut supporter plusieurs versions MUF (au minimum current et N-1, selon policy).
- Toute suppression de keyword ou changement de sémantique doit être traité comme breaking.

---

## Migration

Commandes (concept) :

```text
muffin muf lint build.muf --strict
muffin muf format build.muf --in-place
muffin muf upgrade build.muf --to 3 --out build.v3.muf
```

---

## Alternatives considered

- DSL basé sur JSON/TOML/YAML : rejeté (manque de blocs, faible ergonomie, ambiguïtés multi-lignes)
- DSL basé sur scripts : rejeté (non déterminisme, sandbox plus complexe)

---

## Implementation notes

- Parser tolérant aux espaces, mais structuré sur des keywords.
- Pas d’indentation significative (l’indentation est décorative).
- Le front-end MUF doit produire une IR stable (AST → graph) avant sérialisation `.mff`.

---

## Status

Accepted.

---

## References

- `docs/reference/muf/index.md`
- `docs/reference/cli/configure.md`
- `docs/ops/versioning.md`