# Config schema (generated)

# Config schema (generated)

Ce document décrit le **schéma de configuration** de Muffin (buildfile `Muffinfile` / `build.muf`) et la représentation logique du graphe (binaire `*.mff`).

L’objectif est d’avoir un format **stable**, lisible, validable et sérialisable :

- `*.muf` : source déclarative (buildfile).
- `*.mff` : binaire de build résolu (configuration + graph + invariants).

---

## 1) Glossaire

- **workspace** : racine logique du projet.
- **artifact** : sortie typée produite par un `bake`.
- **bake** : nœud d’exécution (unité de calcul) dans le DAG.
- **port** : entrée/sortie typée (`in` / `out`) d’un bake.
- **wire** : connexion explicite `out -> in`.
- **plan** : scénario d’exécution (sélection de ce qui est lancé).
- **export** : port de sortie déclarée “buildable”.
- **tool** : exécutable déclaré, résolu, versionné, sandboxable.
- **capsule** : policy d’exécution hermétique (env/fs/net/time).
- **store** : cache d’artefacts (content-addressed / mtime / off).

---

## 2) Fichiers et conventions

### 2.1 Buildfile

Nom :

- `Muffinfile` (recommandé)
- `build.muf` (équivalent)

Encodage : UTF-8.

### 2.2 Binaire de build

Nom :

- `*.mff`

Rôle :

- snapshot **résolu** et reproductible de la configuration et du graph.

---

## 3) Format buildfile `*.muf`

### 3.1 En-tête

**Obligatoire** et placé en première ligne logique.

Syntaxe :

- `muffin bake <int>`

Champs :

- `bake_version: int` : version du dialecte. Exemple: `2`.

Règle :

- Toute version inconnue → erreur de validation.

---

## 4) Modèle logique (AST)

Le buildfile se valide en un modèle logique composé :

- `Header`
- `Stmt[]` (instructions top-level)

### 4.1 Identifiants

- `ident` : `[A-Za-z_][A-Za-z0-9_]*`
- Les identifiants sont **case-sensitive**.

### 4.2 Valeurs

Type `Value` (union) :

- `String` : `"..."` (support `\n`, `\r`, `\t`, `\"`, `\\`).
- `Int` : entier décimal.
- `Bool` : `true|false`.
- `List<Value>` : `[ ... ]`.
- `Ident` : identifiant simple (référence courte autorisée selon contexte).

Règles :

- Les conversions implicites sont **interdites** (ex: int → text) sauf policy explicite.
- Les listes sont hétérogènes possible au niveau syntaxe, mais peuvent être restreintes par contexte.

---

## 5) Déclarations top-level (Stmt)

Les instructions top-level sont :

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

Ordre :

- libre au niveau syntaxe
- les résolutions `ref` exigent que la cible existe après la phase de collecte (deux passes).

---

## 6) `set` (global setters)

### 6.1 But

Affecter des paramètres globaux (ex: plan par défaut, profil, target, modes).

### 6.2 Syntaxe

- `set <ident> <value>`

### 6.3 Champs

- `key: ident`
- `value: Value`

### 6.4 Clés standard (convention)

- `plan` : plan par défaut
- `profile` : profil actif
- `target` : target active
- `cache` : `content|mtime|off`
- `sandbox` : `on|off|strict`

Règle :

- Les clés non reconnues → warning (ou error en `--strict`).

---

## 7) `store` (cache)

### 7.1 Syntaxe

```text
store <name>
  path "..."
  mode content|mtime|off
.end
```

### 7.2 Champs

- `name: ident`
- `path: string` (obligatoire)
- `mode: store_mode` (défaut: `content`)

### 7.3 `store_mode`

- `content` : cache content-addressed (hash inputs + args + toolchain)
- `mtime` : invalidation basée sur timestamps
- `off` : désactive le store

### 7.4 Règles

- `path` doit être un chemin workspace (absolu ou relatif)
- un `store` peut être référencé implicitement par défaut, ou explicitement par future extension.

---

## 8) `capsule` (sandbox policy)

### 8.1 Syntaxe

```text
capsule <name>
  env allow ["HOME", "PATH"]
  fs allow_read ["./src", "./include"]
  fs allow_write ["./out"]
  net deny
  time stable true
.end
```

### 8.2 Champs

- `name: ident`

`capsule_item` :

- `env allow|deny list_string`
- `fs allow_read|allow_write|deny list_string`
- `fs allow_write_exact list_string`
- `net allow|deny`
- `time stable bool`

### 8.3 Détails

#### ENV
- `allow` : whitelist
- `deny` : blacklist

#### FS
- `allow_read` : read-only mounts
- `allow_write` : write scope
- `deny` : deny explicit
- `allow_write_exact` : write uniquement sur chemins exacts (pas de glob implicite)

#### NET
- `allow` / `deny`

#### TIME
- `stable true` : stabilise temps/horloges selon backend (reproductibilité)

### 8.4 Règles

- Toute policy manquante prend la valeur **par défaut** du runtime (souvent “deny”).
- Les listes sont des `string_lit`.

---

## 9) `var` (variables typées)

### 9.1 Syntaxe

```text
var <name> : <type_ref> = <value>
```

### 9.2 Champs

- `name: ident`
- `type: TypeRef`
- `value: Value`

### 9.3 `type_ref`

- `prim_type` : `text|int|bool|bytes`
- `artifact_type` : `a.b` ou `a.b.c` (chemin typé)

Exemples :

- `src.glob`
- `ir.module`
- `bin.exe`
- `lib.static.va`

### 9.4 Règles

- `value` doit être compatible avec `type_ref`.
- Les `artifact_type` utilisés en `var` servent souvent de “handles” ou d’alias.

---

## 10) `profile` (profils)

### 10.1 Syntaxe

```text
profile <name>
  set opt "3"
  set debug true
.end
```

### 10.2 Champs

- `name: ident`
- `items: ProfileItem[]`

`ProfileItem` :

- `set <key> <value>`

### 10.3 Clés usuelles (convention)

- `opt` : niveau d’optimisation
- `debug` : symboles / checks
- `lto` : link-time
- `strip` : stripping

Règle :

- Clés inconnues → warning (ou error en strict).

---

## 11) `tool` (exécutables déclarés)

### 11.1 Syntaxe

```text
tool <name>
  exec "vittec"
  expect_version "^1.0"
  sandbox true
  capsule default
.end
```

### 11.2 Champs

- `name: ident`
- `exec: string` (obligatoire)
- `expect_version: string` (optionnel)
- `sandbox: bool` (défaut: `true`)
- `capsule: ident` (optionnel)

### 11.3 Règles

- `exec` peut être un binaire résolu via PATH ou une route explicite.
- `expect_version` est un contrat (regex/semver selon impl).
- `capsule` référence une capsule déclarée.

---

## 12) `bake` (nœud de build)

### 12.1 Syntaxe

```text
bake <name>
  in  src  : src.glob
  out exe  : bin.exe
  make src glob "src/**/*.vit"
  run tool vittec
    takes src as "--src"
    emits exe as "--out"
    set "--emit" "exe"
  .end
  cache content
  output exe at "./out/app"
.end
```

### 12.2 Champs

- `name: ident`
- `ports_in: PortIn[]`
- `ports_out: PortOut[]`
- `make: MakeStmt[]`
- `runs: RunBlock[]`
- `cache: cache_mode` (défaut: `content`)
- `output: OutputStmt[]` (optionnel)

### 12.3 Ports

`PortIn` :

- `in <name> : <type_ref>`

`PortOut` :

- `out <name> : <type_ref>`

Règles :

- Les noms de ports sont uniques dans leur scope.

### 12.4 `make` (ingredients builder)

Syntaxe :

- `make <ident> <make_kind> "..." `

`make_kind` :

- `glob` : pattern de fichiers
- `file` : fichier unique
- `text` : contenu inline
- `value` : valeur brute (string encodée)

Règles :

- Le `<ident>` désigne un output “local” dans le bake (souvent câblé via `wire` ou `run`).

### 12.5 `run tool` (exécution d’outil)

Syntaxe :

```text
run tool <tool_name>
  takes <in_port> as "--flag"
  emits <out_port> as "--out"
  set "--k" <value>
.end
```

Champs :

- `tool: ident`
- `takes: Takes[]`
- `emits: Emits[]`
- `set: RunSet[]`

Règles :

- `tool` doit exister.
- `takes` référence un port `in` ou un ident local `make` selon policy.
- `emits` référence un port `out`.

### 12.6 `cache` (mode)

- `cache content|mtime|off`

### 12.7 `output` (chemin final)

- `output <out_port> at "./path"`

Règle :

- Le chemin est workspace-rooted (ou relatif).

---

## 13) `wire` (wiring)

### 13.1 Syntaxe

- `wire <ref> -> <ref>`

### 13.2 `ref`

- `ident` (var globale)
- `bake.port` (port d’un bake)

### 13.3 Règles

- La source doit être un `out` (ou un `var` exportant un artifact).
- La destination doit être un `in`.
- Les types doivent être compatibles.

---

## 14) `export` (ports exportables)

### 14.1 Syntaxe

- `export <ref>`

### 14.2 Règle

- `<ref>` doit référencer un **port out** (`bake.port`).

---

## 15) `plan` (scénarios)

### 15.1 Syntaxe

```text
plan <name>
  run exports
  run bake_a.out
.end
```

### 15.2 Champs

- `name: ident`
- `items: PlanItem[]`

`PlanItem` :

- `run exports`
- `run <ref>`

### 15.3 Règles

- Un plan vide est valide mais ne fait rien.
- `run <ref>` doit référencer un `out` exportable ou un port existant selon policy.

---

## 16) `switch` (mapping CLI)

### 16.1 Syntaxe

```text
switch
  flag "-debug" set profile "debug"
  flag "-release" set profile "release"
  flag "-all" run exports
.end
```

### 16.2 Champs

- `items: SwitchItem[]`

`SwitchItem` :

- `flag "<flag>" <switch_action>`

`switch_action` :

- `set <ident> <value>`
- `set plan "<name>"`
- `run exports|<ref>`

### 16.3 Règles

- Les flags doivent être uniques.
- Les actions `set` sont évaluées avant `run`.

---

## 17) Validation (règles transverses)

### 17.1 Unicité

- `store.name` unique
- `capsule.name` unique
- `tool.name` unique
- `profile.name` unique
- `bake.name` unique
- `plan.name` unique

### 17.2 Références

- Toute référence `ident` / `bake.port` doit se résoudre.
- Types compatibles sur `wire`.

### 17.3 Cycles

- Les `wire` définissent un DAG : un cycle est une erreur.

### 17.4 Déterminisme

- Le graph final doit être déterministe (ordre d’énumération stable).
- Les entrées `glob` doivent être triées selon policy (lexicographique) avant hashing.

---

## 18) Représentation logique du `.mff`

Le `.mff` encapsule :

- header (version + metadata)
- workspace (root)
- tables (stores/capsules/tools/profiles)
- graph (bakes + ports + edges)
- exports
- plans
- switch mapping
- digest (integrity)

### 18.1 Header

Champs (convention) :

- `format: "mff"`
- `format_version: int`
- `created_at: int` (unix seconds) ou time stable policy
- `host: string` (platform id)
- `default_plan: string`
- `default_profile: string`

### 18.2 Tables

- `stores: Store[]`
- `capsules: Capsule[]`
- `tools: Tool[]`
- `profiles: Profile[]`
- `vars: Var[]`

### 18.3 Graph

- `bakes: Bake[]`

Bake (résolu) :

- `name`
- `in_ports: Port[]`
- `out_ports: Port[]`
- `steps: Step[]` (make/run/output/caching)
- `cache_mode`

Edges :

- `edges: Edge[]` où `Edge = { from: Ref, to: Ref, type: TypeRef }`

### 18.4 Exports / Plans

- `exports: Ref[]`
- `plans: { name, runs[] }[]`

### 18.5 Integrity

- `digest_algo: "blake3" | "sha256" | ...`
- `digest: bytes` (hash sur contenu normalisé)

---

## 19) Compatibilité multi-plateformes (convention)

Le schéma vise l’uniformisation :

- host/target séparés
- toolchains résolues via `tool` + policies
- artefacts typés et ports explicites
- sandbox optionnelle mais intégrable

Targets (exemples) :

- Linux : `x86_64-unknown-linux-gnu`, `aarch64-unknown-linux-gnu`
- macOS : `x86_64-apple-darwin`, `aarch64-apple-darwin`
- Windows : `x86_64-pc-windows-msvc`
- BSD : `x86_64-unknown-freebsd`, `x86_64-unknown-openbsd`, `x86_64-unknown-netbsd`
- Solaris : `x86_64-unknown-solaris`

---

## 20) Exemple minimal

```text
muffin bake 2

store default
  path "./.muffin/store"
  mode content
.end

profile debug
  set opt 0
  set debug true
.end

tool cc
  exec "cc"
  sandbox true
.end

bake app
  out exe: bin.exe
  make src glob "src/**/*.c"
  run tool cc
    takes src as "--src"
    emits exe as "--out"
  .end
.end

export app.exe

plan default
  run exports
.end
```

---

## 21) Compatibilité de schéma

- Les ajouts sont **append-only** si possible.
- Toute breaking change → incrément de `bake_version`.
- Un buildfile `bake_version` inconnu → refus.