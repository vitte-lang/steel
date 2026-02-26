# Steel Manifest

Ce document explique Steel sans blabla.

## C est quoi Steel

Steel lit un fichier `steelconf` (comme un `Makefile` moderne) et lance les etapes de build.

Tu decris:
- les outils (`tool`)
- les recettes (`bake`)
- les executions (`run`)
- les sorties (`output`)

## Commandes de base

```bash
steel help
steel version
steel run --file steelconf --all
steel run --file steelconf --bake app
steel doctor
steel toolchain doctor
steel editor
```

## Workflow simple

1. Ecris/modifie `steelconf`.
2. Lance `steel run --file steelconf --all`.
3. Corrige les erreurs.
4. Relance.

## Flags utiles (`steel run`)

- `--file <path>`: chemin du fichier de config (defaut: `steelconf`)
- `--all`: execute toutes les recettes
- `--bake <name>`: execute une recette
- `--print`: dry-run (affiche les commandes sans executer)
- `--profile <name>`: choisit un profil
- `--log <path>`: ecrit un log
- `--log-mode <append|truncate>`: mode d ecriture du log

## Mini exemple steelconf

```text
!muf 4

[tool cc]
  .exec "cc"
..

[bake app]
  .make c_src cglob "src/**/*.c"
  [run cc]
    .takes c_src as "@args"
    .set "-O2" 1
    .emits exe as "-o"
  ..
  .output exe "target/out/app"
..
```

## Debug rapide

```bash
steel doctor
steel toolchain doctor
steel run --file steelconf --all --print
```
