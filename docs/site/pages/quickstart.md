---
title: Quickstart
slug: /quickstart/
description: Demarrer rapidement avec Muffin et MUF v4.1.
---

# Quickstart

Chemin minimal pour un projet C simple avec MUF v4.1.

![Quickstart run](/assets/img/quickstart.svg)

**MUF v4.1**

## Install

```sh
cargo install --path . --force
```

## Build config (MUF v4.1)

Creer `MuffinConfig.muf` a la racine :

```muf
!muf 4

[workspace]
  .set name "app"
  .set target_dir "target"
  .set profile "debug"
..

[profile debug]
  .set opt 0
  .set debug 1
  .set ndebug 0
..

[tool gcc]
  .exec "gcc"
..

[bake app]
  .make c_src cglob "**/*.c"
  [run gcc]
    .takes c_src as "@args"
    .set "-std=c17" 1
    .set "-O${opt}" 1
    .set "-g" "${debug}"
    .set "-DNDEBUG" "${ndebug}"
    .set "-Wall" 1
    .set "-Wextra" 1
    .emits exe as "-o"
  ..
  .output exe "target/out/app"
..

[export]
  .ref app
..
```

## Run

```sh
muffin run --root . --file MuffinConfig.muf
```

Dry-run :

```sh
muffin run --root . --file MuffinConfig.muf --print
```

## Logs

Par defaut, un log est ecrit dans `target/muffin_run_<timestamp>.mff`.

```sh
muffin run --root . --file MuffinConfig.muf --log target/muffin_run.mff --log-mode truncate
```

## Common commands

```sh
muffin build muffin --file MuffinConfig.muf
muffin run --root . --file MuffinConfig.muf
muffin print --file MuffinConfig.muf
```

## Resultats attendus

- binaire: `target/out/app`
- log: `target/muffin_run_*.mff`

```sh
ls -l target/out/app
ls -t target/muffin_run_*.mff | head -n 1
```

## Prochaine etape

- [Cookbook](/cookbook)
- [Troubleshooting](/troubleshooting)
