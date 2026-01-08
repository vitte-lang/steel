# Config

Référence “configuration” (index) de Muffin.

Cette section couvre :

- fichiers d’entrée (`*.muf/*.muff`) et conventions
- génération et contenu du **binaire de compilation** `*.mff`
- variables, overrides (`-D KEY=VALUE`) et sources de configuration
- profils (`profile`), targets, toolchains
- caches (`store`) et sandbox (`capsule`)

---

## Index

- [Buildfiles](./buildfiles.md)
- [MFF](./mff.md)
- [Variables & overrides](./variables.md)
- [Profiles](./profiles.md)
- [Targets](./targets.md)
- [Toolchains](./toolchains.md)
- [Store](./store.md)
- [Capsule](./capsule.md)

---

## Rappels

- `build muffin` : **configure** → génère `Muffinconfig.mff`
- `Muffin build` : **build** → exécute le DAG depuis `Muffinconfig.mff`

---

## Conventions

- Les exemples utilisent des chemins relatifs.
- Les types logiques (`bin.obj`, `lib.static`, `bin.exe`) sont préférés aux extensions réelles.
- Les sorties machine-friendly utilisent `--format json|ndjson`.

---

## Voir aussi

- CLI : `docs/reference/cli/index.md`
- Ops : `docs/ops/index.md`
- Manpages : `docs/man/index.md`
