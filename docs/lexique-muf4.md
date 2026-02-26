# Lexique MUF4

Ce lexique explique les mots cles les plus utiles de `steelconf`.

## Structure

- `!muf 4`: en tete du fichier
- `[bloc nom] ... ..`: bloc
- `.directive ...`: action/parametre dans un bloc

## Blocs importants

- `tool <name>`: declare un outil (`cc`, `rustc`, etc.)
- `bake <name>`: recette de build
- `run <tool>`: etape qui appelle l outil dans un `bake`
- `profile <name>`: profil (debug/release)

## Directives utiles

- `.exec "cmd"`: commande de l outil
- `.make <id> <kind> <pattern>`: liste de fichiers source
- `.takes <id> as "flag"`: branche une entree sur un flag
- `.emits <port> as "flag"`: branche une sortie sur un flag
- `.set "flag" value`: ajoute un flag/valeur
- `.output <port> "path"`: sortie finale

## Exemple minimal

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
