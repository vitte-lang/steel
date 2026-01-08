# Reference

Point d’entrée des **références** Muffin (contrats, formats, CLI, configuration).

Cette section est normative : elle décrit les interfaces stables et les conventions attendues.

---

## Modèle

Muffin sépare strictement :

- **Configure** : `build muffin` / `muffin configure` → génère le **binaire de compilation** `Muffinconfig.mff`
- **Build** : `Muffin build` / `muffin build` → exécute le DAG depuis `Muffinconfig.mff`

---

## Index

### CLI

- [CLI (index)](./cli/index.md)
- [muffin configure](./cli/configure.md)

### Configuration

- [Config (index)](./config/index.md)

### Formats

- [Formats (index)](./formats/index.md)

### Buildfiles (MUF)

- [MUF (index)](./muf/index.md)

### Targets

- [Targets (index)](./targets/index.md)

---

## Conventions globales

- chemins : préférer les chemins relatifs ; accepter `/` et `\\` en entrée ; normalisation interne stable.
- encodage : UTF-8 recommandé (JSON/NDJSON en UTF-8).
- déterminisme : tri stable des globs et sérialisation stable du `.mff`.
- types logiques : préférer `bin.obj`, `lib.static`, `lib.shared`, `bin.exe` et résoudre les extensions via `target`.

---

## Voir aussi

- Manpages : `docs/man/index.md`
- Manuel : `docs/manual/`
- Ops : `docs/ops/index.md`
- Méta : `docs/meta/index.md`
