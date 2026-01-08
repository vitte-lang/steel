# API — Muffin

Cette section documente l’API interne (Rust) exposée par **Muffin** : modèles de données, schémas, lecture/écriture de `.mff`, parsing/validation des buildfiles `.muf/.muff`, exécution (graph + tools), sandbox (`capsule`) et introspection (`decompile`, `why`, `graph`).

> Objectif : une API stable, portable et outillable, utilisable par la CLI, CI, IDE, plugins, et intégrations multi-langages.

---

## Modules

- **diag** : diagnostics structurés (codes, messages, rendering)
- **span** : positions, fichiers, spans
- **path** : normalisation cross-platform, globs, canon
- **hash** : empreintes (fingerprint) et clés de cache
- **capsule** : policy sandbox (env/fs/net/time) + backends OS
- **store** : cache CAS (content-addressed) + index + GC
- **tool** : description et exécution d’outils (probe, runner)
- **graph** : DAG (bakes/ports/wires/plans) + topo + exports
- **mcfg** : parsing/AST/HIR, lowering, typecheck, resolve, validator
- **mff** : schéma binaire `.mff`, reader/writer, index, trace

---

## Contrats clés

### Buildfile (`*.muf` / `*.muff`)

- Format déclaratif qui décrit **configuration + règles**.
- Produit, après résolution, un **binaire de compilation** `.mff`.

### Binaire `.mff`

`Muffinconfig.mff` est l’artefact canonique :

- Graphe normalisé (DAG) : nœuds, ports, wires, exports, plans
- Inputs développés (globs résolus), chemins normalisés
- Outils déclarés et paramètres effectifs (toolchain)
- Empreintes d’invalidation (cache) et trace de construction

---

## Exécution

L’exécution est découplée en deux phases :

1. **configure** : parse/validate/resolve → écrit `.mff`
2. **build** : lit `.mff` → exécute le DAG via tools déclarés

---

## Introspection

- `decompile` : reconstitue l’architecture et la config depuis `.mff` / `.muff`
- `why` : explique une invalidation/rebuild (chaîne de dépendances)
- `graph` : export du DAG (texte/JSON/DOT)

---

## Stabilité

- Les structures sérialisées `.mff` doivent rester compatibles :
  - versioning explicite
  - champs extensibles
  - formats déterministes

---

## Index des pages API

- `diag` — diagnostics
- `span` — sources et positions
- `path` — chemins et globs
- `capsule` — sandbox et policies
- `store` — cache CAS
- `tool` — exécution d’outils
- `graph` — DAG et scheduling
- `mcfg` — buildfiles et résolution
- `mff` — binaire de compilation

---

## Exemples rapides

### Lire un `.mff`

```rust
use muffin::mff::Reader;

fn main() -> anyhow::Result<()> {
    let mff = Reader::open("Muffinconfig.mff")?.read_all()?;
    println!("plans: {}", mff.plans.len());
    Ok(())
}
```

### Décompiler (API)

```rust
use muffin::mcfg::decompile::Decompile;

fn main() {
    let out = Decompile::new().run_path("Muffinconfig.mff");
    println!("{out}");
}
```

### Export DOT

```rust
use muffin::graph::dot::DotExport;

fn main() {
    let dot = DotExport::from_mff_path("Muffinconfig.mff");
    println!("{dot}");
}
```
