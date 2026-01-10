# MUF (Buildfiles)

Le format MUF est la source declarative (buildfiles) utilisee par Muffin.

## Versioning

Le format est versionne via l'en-tete :

```text
!muf 4
```

La version courante est v4.1.

## Structure

- Fichier texte UTF-8.
- Blocs : `[tag]` ... `..`
- Directives : `.set`, `.make`, `.output`, `.takes`, `.emits`, etc.
- Commentaires : `;; ...`

## Exemple minimal

```text
!muf 4

[workspace]
  .set name "app"
  .set profile "debug"
..

[tool gcc]
  .exec "gcc"
..

[bake app]
  .make c_src cglob "src/**/*.c"
  [run gcc]
    .takes c_src as "@args"
    .emits exe as "-o"
  ..
  .output exe "target/out/app"
..
```

## Schema

Schema informel base sur la grammaire :

- `assets/grammar/muffin.ebnf`

## Compatibilite

- En cas de changement majeur, increment de version d'en-tete.
- La compat ascendante n'est pas garantie entre versions majeures.
