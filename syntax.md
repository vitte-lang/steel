# Syntaxe MUF v4.1

Ce document resume la grammaire MUF v4.1 ("Bracket + Dot Ops", sans `.end`).

## Structure generale

Un fichier MUF est compose de :

- un shebang optionnel (ligne commencant par `#!`),
- un BOM optionnel (`\uFEFF`),
- un en-tete obligatoire,
- puis une suite d'elements (lignes vides, commentaires, blocs).

Forme globale :

```ebnf
MufFile = OptShebang , OptBOM , Header , { SpacingItem } ;
```

## En-tete

L'en-tete doit etre sur une ligne propre :

```
!muf <version>
```

- `<version>` est un entier (ex: `4`).

## Blocs

Un bloc est delimite par un en-tete de bloc et un marqueur de fermeture `..`.

- Ouverture : `[TAG nom?]`
- Fermeture : `..` (seul sur la ligne, espaces autorises)

Les blocs peuvent etre imbriques.

Exemple :

```muf
[project demo]
  .set name "demo"
  [build]
    .run "make"
  ..
..
```

## Directives

Une directive commence par un point et peut prendre des arguments :

```
.op arg1 arg2 ...
```

- `op` est un identifiant (`Name`).
- Les arguments sont des `Atom` (voir plus bas).

## Commentaires

Un commentaire commence par `;;` et va jusqu'a la fin de la ligne :

```muf
;; ceci est un commentaire
```

## Lexemes

### Name

- Commence par une lettre (`A..Z` ou `a..z`) ou `_`.
- Se poursuit par lettres, chiffres ou `_`.

Exemples : `name`, `_tag`, `build1`

### String

Chaine entre guillemets doubles :

- Echapements : `\"`, `\\`, `\n`, `\r`, `\t`, `\0`, `\xNN`, `\uNNNN`.
- Pas de retour a la ligne dans une chaine.

### Number

- Entier : `+42`, `-7`, `0`
- Flottant : `3.14`, `-0.5`, `1.2e3`

### Ref

Reference de la forme :

```
~name/name/.../name
```

## Espace et lignes

- `WS` : espace ou tabulation.
- `NL` : fin de ligne (`\n` ou `\r\n`).
- Les lignes vides et les lignes de commentaire sont autorisees entre les elements.

## Rappel (EBNF minimal)

```ebnf
Block     = WS0 , BlockHead , WS0 , NL , { BlockItem } , WS0 , BlockClose , WS0 , NL ;
BlockHead = "[" , WS0 , Tag , [ WS1 , Name ] , WS0 , "]" ;
BlockClose = ".." ;
Directive = WS0 , "." , Op , { WS1 , Atom } , WS0 , NL ;
Atom = Ref | String | Number | Name ;
```
