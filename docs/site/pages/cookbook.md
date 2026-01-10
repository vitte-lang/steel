---
title: Cookbook
slug: /cookbook/
description: Recettes MUF reelles pour C, lib+app et multi-tool.
---

# Cookbook

Recettes pratiques basées sur les exemples du repo.

![Cookbook run](/assets/img/cookbook.svg)

## C app (single bake)

Exemple :

- config: `examples/gcc/MuffinConfig.muf`
- source: `examples/gcc/app/main.c`

```sh
muffin run --root examples/gcc --file MuffinConfig.muf --bake app
```

Dry-run :

```sh
muffin run --root examples/gcc --file MuffinConfig.muf --bake app --print
```

Outputs :

- `examples/gcc/target/out/app`
- `examples/gcc/target/muffin_run_*.mff`

## Library + app (deps)

Exemple multi-bakes :

- config: `examples/gcc/MuffinConfig_multi.muf`
- lib: `examples/gcc/lib/lib.c`
- app: `examples/gcc/app/main.c`

```sh
muffin run --root examples/gcc --file MuffinConfig_multi.muf --bake app
```

Build tout :

```sh
muffin run --root examples/gcc --file MuffinConfig_multi.muf --all
```

Outputs :

- `examples/gcc/target/out/libmylib.a`
- `examples/gcc/target/out/app`

## Multi-tool (gcc + ar)

Centraliser les chemins via variables :

```muf
[workspace]
  .set inc_dir "lib"
  .set lib_dir "target/out"
  .set lib_name "mylib"
..

[run gcc]
  .include "${inc_dir}"
  .libdir "${lib_dir}"
  .lib "${lib_name}"
..
```

## Voir aussi

- [Formats](/reference/formats)
- [Observabilite](/observabilite)
