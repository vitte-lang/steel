# Steel

![Steel](https://img.shields.io/badge/Steel-config-orange)
[![Stars](https://img.shields.io/github/stars/vitte-lang/steel?style=flat-square)](https://github.com/vitte-lang/steel/stargazers)
[![Forks](https://img.shields.io/github/forks/vitte-lang/steel?style=flat-square)](https://github.com/vitte-lang/steel/network/members)
[![Issues](https://img.shields.io/github/issues/vitte-lang/steel?style=flat-square)](https://github.com/vitte-lang/steel/issues)
[![Last Commit](https://img.shields.io/github/last-commit/vitte-lang/steel?style=flat-square)](https://github.com/vitte-lang/steel/commits/main)
[![License](https://img.shields.io/github/license/vitte-lang/steel?style=flat-square)](https://github.com/vitte-lang/steel/blob/main/COPYING)

Steel repose sur **un seul fichier de verite**: `steelconf`.

Si tu connais `make`, pense a `steelconf` comme un `Makefile` moderne:

- plus structure (blocs explicites `tool`, `bake`, `run`)
- plus lisible pour les gros projets
- meilleur controle des profils et des outils

Important: dans le workflow normal, tu travailles avec `steelconf`.

## Demarrage rapide

Depuis la racine du projet:

```bash
steel run --file steelconf --all
steel doctor
```

- `steel run --file steelconf --all`: lance toutes les recettes declarees dans `steelconf`.
- `steel doctor`: verifie ton environnement.

Tu peux aussi verifier la config en amont avec:

```bash
steel build steelconf
```

## Makefile vs steelconf

### Exemple `Makefile`

```makefile
CC = cc
CFLAGS = -O2

app: src/main.c
	$(CC) $(CFLAGS) src/main.c -o target/out/app
```

### Equivalent `steelconf`

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

## Pourquoi `steelconf` est mieux qu un gros Makefile

- Structure claire: chaque bloc a un role precis.
- Evolutif: plus simple a maintenir quand le projet grossit.
- Outillage: `steel editor` comprend la syntaxe `steelconf`.
- Portabilite: tu decris l intention; Steel orchestre l execution.

## Commandes utiles

- `steel help`
- `steel version`
- `steel run --file steelconf --all`
- `steel run --file steelconf --bake <nom>`
- `steel run --file steelconf --all --print` (dry-run)
- `steel toolchain doctor`
- `steel editor`
- `steel editor-setup`

## Fichiers importants

- `steelconf`: fichier principal (equivalent au `Makefile` dans l esprit).
- `steel.log` / `*.mff`: logs d execution selon options.

## Documentation utile

- `doc/manifest.md`
- `doc/steel.1`
- `docs/lexique-muf4.md`
- `src/MODULE_ORGANIZATION.md`

## Licence

Voir `COPYING`.
