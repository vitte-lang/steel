# MUF

# MUF

Référence (index) du format **MUF** : buildfiles `*.muf` / `*.muff` utilisés par Muffin.

Les buildfiles décrivent un **graph** de build (bakes/ports/wires/tools/capsules/stores/plans) qui sera compilé en **binaire de compilation** `*.mff` via la phase **configure**.

---

## Rappels

- `build muffin` / `muffin configure` : compile `*.muf/*.muff` → `Muffinconfig.mff`
- `Muffin build` / `muffin build` : exécute le DAG depuis `Muffinconfig.mff`

---

## Index

- [Syntaxe et header](./syntax.md)
- [Statements (store/capsule/var/profile/tool/bake/wire/export/plan/switch/set)](./statements.md)
- [Types et valeurs](./types-and-values.md)
- [Bakes, ports, wiring](./bakes.md)
- [Tools](./tools.md)
- [Capsules (sandbox)](./capsules.md)
- [Store (cache)](./store.md)
- [Plans et switch (CLI mapping)](./plans-and-switch.md)
- [Best practices](./best-practices.md)

---

## Conventions

- Les blocs top-level sont terminés par `.end`.
- Les commentaires sont `# ...`.
- Les chemins sont relatifs quand possible.
- Les globs doivent produire une liste triée stable.

---

## Exemple minimal

```muf
muffin bake 2

# Entrées
bake app_src
  out files: src.glob
  make files glob "src/**/*.c"
.end

# Tool
tool cc
  exec "cc"
  sandbox true
  capsule build
.end

# Capsule
capsule build
  env allow ["PATH"]
  fs allow_read  ["./src"]
  fs allow_write ["./out"]
  net deny
  time stable true
.end

# Compile
bake app_obj
  in  files: src.glob
  out obj: bin.obj
  run tool cc
    takes files as "--input"
    emits obj as "--output"
  .end
.end

export app_obj.obj

plan default
  run exports
.end
```

---

## Notes

- La référence normative de la grammaire MUF est l’EBNF du repo (si présent), versionnée par `muffin bake <int>`.
- Les pages de cette section détaillent chaque statement et les règles de validation.

---

## Voir aussi

- CLI : `docs/reference/cli/index.md`
- Formats : `docs/reference/formats/index.md`
- Manpages : `docs/man/index.md`