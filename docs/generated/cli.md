# CLI (generated)

# CLI (generated)

Cette page documente l’interface en ligne de commande (CLI) de **Muffin**.

Muffin opère en deux phases principales :

- **configure** : lecture du buildfile (`Muffinfile` / `build.muf`), validation, résolution, génération d’un **binaire de build** `*.mff`.
- **build** : exécution du graphe depuis un `.mff`, scheduling, sandbox (capsules), cache (store), production des artefacts.

> Notation utilisée :
> - `muf` = buildfile
> - `mff` = binaire de build (graph + config résolue)
> - `plan` = scénario d’exécution

---

## 0) Raccourcis et conventions

### Binaire

- `muffin` : commande principale.
- `build muffin` : alias ergonomique (si le projet fournit un wrapper `build`).

### Fichiers

- Buildfile : `Muffinfile` (recommandé) ou `build.muf`.
- Binaire de build : `*.mff` (ex: `Muffinconfig.mff`).

### Cibles (targets)

Muffin utilise un identifiant target (exemples) :

- `x86_64-unknown-linux-gnu`
- `aarch64-unknown-linux-gnu`
- `x86_64-apple-darwin`
- `aarch64-apple-darwin`
- `x86_64-pc-windows-msvc`
- `x86_64-unknown-freebsd`
- `x86_64-unknown-openbsd`
- `x86_64-unknown-netbsd`
- `x86_64-unknown-dragonfly`
- `x86_64-unknown-solaris`

---

## 1) Aide et diagnostic global

### Afficher l’aide

```bash
muffin help
muffin -h
muffin --help
```

### Version

```bash
muffin --version
muffin version
```

### Vérifier l’installation / environnement

```bash
muffin doctor
muffin doctor --json
```

Sorties typiques :

- host détecté
- targets supportées
- outils trouvés + versions
- backend sandbox disponible (capsules)
- état du store (permissions / locks)

---

## 2) Phase configuration (buildfile → .mff)

### 2.1 Configurer (générer le `.mff`)

```bash
muffin configure
```

Options usuelles :

```bash
muffin configure \
  --file Muffinfile \
  --out .muffin/Muffinconfig.mff
```

### 2.2 Validation stricte

```bash
muffin configure --strict
muffin configure --warn-as-error
```

### 2.3 Choisir un plan par défaut

```bash
muffin configure --set plan=default
muffin configure --set plan=release
```

### 2.4 Cible et profils

```bash
muffin configure --target x86_64-unknown-linux-gnu
muffin configure --profile debug
muffin configure --profile release
```

### 2.5 Overrides de variables (sans toucher le buildfile)

```bash
muffin configure -D KEY=VALUE
muffin configure -D target=x86_64-unknown-linux-gnu
muffin configure -D profile=release
```

### 2.6 Produire uniquement le graph / introspection

```bash
muffin configure --emit mff
muffin configure --emit graph
muffin configure --emit schema
```

---

## 3) Phase build (exécuter le `.mff`)

### 3.1 Build par défaut

```bash
muffin build
```

- utilise le `.mff` par défaut (si généré) ou déclenche une configuration implicite selon policy.

### 3.2 Build avec un `.mff` explicite

```bash
muffin build --mff .muffin/Muffinconfig.mff
```

### 3.3 Exécuter un plan

```bash
muffin build --plan default
muffin build --plan release
muffin build --plan ci
```

### 3.4 Construire tout ce qui est exporté

```bash
muffin build -all
muffin build --all
```

### 3.5 Cibler une exécution précise

```bash
muffin build --run exports
muffin build --run bake_name.out_port
```

---

## 4) Cache / Store

### 4.1 Nettoyage

```bash
muffin clean
muffin clean --cache
muffin clean --artifacts
muffin clean --all
```

### 4.2 Mode cache

```bash
muffin build --cache content
muffin build --cache mtime
muffin build --cache off
```

### 4.3 Statut du store

```bash
muffin store status
muffin store gc
muffin store gc --dry-run
muffin store verify
```

---

## 5) Parallélisme et scheduling

### 5.1 Parallélisme

```bash
muffin build -j 16
muffin build --jobs 16
```

### 5.2 Affinité / limites (optionnel)

```bash
muffin build --max-load 8
muffin build --nice 10
```

### 5.3 Mode silencieux / verbeux

```bash
muffin build -q
muffin build -v
muffin build -vv
```

---

## 6) Watch (développement)

### 6.1 Rebuild incrémental

```bash
muffin watch
muffin watch --plan default
muffin watch -j 8
```

### 6.2 Filtrer ce qui déclenche

```bash
muffin watch --include "src/**" --exclude "**/*.tmp"
```

---

## 7) Graph et compréhension d’un rebuild

### 7.1 Exporter le graph

```bash
muffin graph
muffin graph --format text
muffin graph --format dot
muffin graph --format json
muffin graph --out build.dot
```

### 7.2 Expliquer une invalidation

```bash
muffin why <artifact>
muffin why bake_name.out_port
muffin why --json bake_name.out_port
```

### 7.3 Tracer une dépendance

```bash
muffin trace bake_a.out -> bake_b.in
muffin trace --from bake_a.out
muffin trace --to bake_b.in
```

---

## 8) Capsules (sandbox)

### 8.1 Activer / désactiver

```bash
muffin build --sandbox on
muffin build --sandbox off
```

### 8.2 Mode strict

```bash
muffin build --sandbox strict
```

### 8.3 Inspection

```bash
muffin capsule list
muffin capsule show <name>
```

---

## 9) Tools (exécutables déclarés)

### 9.1 Lister

```bash
muffin tool list
muffin tool show <name>
```

### 9.2 Vérifier versions attendues

```bash
muffin tool check
muffin tool check --json
```

### 9.3 Forcer la résolution d’un tool

```bash
muffin tool resolve <name>
muffin tool resolve <name> --path /custom/bin/tool
```

---

## 10) Plans / Exports

### 10.1 Lister les plans

```bash
muffin plan list
muffin plan show <plan>
```

### 10.2 Lister les exports

```bash
muffin export list
muffin export show
```

### 10.3 Exécuter un plan

```bash
muffin run <plan>
muffin run exports
muffin run bake_name.out_port
```

---

## 11) Formats de sortie (text/json/ndjson)

### 11.1 Logs

```bash
muffin build --format text
muffin build --format json
muffin build --format ndjson
```

### 11.2 Emission d’un rapport

```bash
muffin build --report out/report.json
muffin build --report-format json
```

---

## 12) Décompilation / introspection `.mff`

Le `.mff` représente l’état résolu (graph + config) et sert de point d’entrée reproductible.

### 12.1 Inspecter un `.mff`

```bash
muffin mff info project.mff
muffin mff dump project.mff --format json
muffin mff dump project.mff --format text
```

### 12.2 Exporter l’architecture (graph + fichiers)

```bash
muffin decompile project.mff --out out/architecture
muffin decompile project.mff --out out/architecture --format json
```

### 12.3 Vérifier l’intégrité

```bash
muffin mff verify project.mff
muffin mff verify project.mff --strict
```

---

## 13) Commandes orientées plateformes (Linux / macOS / Windows / BSD / Solaris)

### 13.1 Détection host/target

```bash
muffin host
muffin target list
muffin target set x86_64-unknown-linux-gnu
```

### 13.2 Toolchains système (hooks)

```bash
muffin toolchain detect
muffin toolchain show
muffin toolchain pin --compiler /path/to/cc
```

### 13.3 Helpers d’inspection (wrap)

```bash
muffin inspect file path/to/bin
muffin inspect symbols path/to/bin
muffin inspect deps path/to/bin
```

---

## 14) Exemples de workflows

### 14.1 Workflow standard

```bash
muffin configure --out .muffin/Muffinconfig.mff
muffin build --mff .muffin/Muffinconfig.mff --plan default
```

### 14.2 Tout construire

```bash
muffin configure
muffin build -all
```

### 14.3 Release multi-target

```bash
muffin configure --target x86_64-unknown-linux-gnu --profile release --out .muffin/linux.mff
muffin configure --target x86_64-apple-darwin --profile release --out .muffin/macos.mff
muffin configure --target x86_64-pc-windows-msvc --profile release --out .muffin/windows.mff

muffin build --mff .muffin/linux.mff   -all
muffin build --mff .muffin/macos.mff   -all
muffin build --mff .muffin/windows.mff -all
```

### 14.4 Comprendre un rebuild

```bash
muffin build --plan default
muffin why bake_app.exe
muffin graph --format dot --out build.dot
```

---

## 15) Codes d’erreur (convention)

- `0` : succès
- `1` : erreur générique
- `2` : erreur de configuration/buildfile
- `3` : tool manquant / version incompatible
- `4` : violation capsule/sandbox
- `5` : deadlock / lock store
- `6` : cycle détecté dans le graph

---

## 16) Commande wrapper `build muffin`

Si le projet fournit un wrapper `build`, ces formes sont attendues :

```bash
build muffin
build muffin <plan>
build muffin -all
build muffin -debug
build muffin -release
build muffin -clean
build muffin -j 16
build muffin -watch
build muffin -why <artifact>
build muffin -graph
build muffin -D KEY=VALUE
```

> `build muffin -debug` / `-release` sont des raccourcis vers `--profile debug|release`.