---
title: Formats
slug: /reference/formats/
description: Versioning MUF, MFF et run logs.
---

# Formats

**MUF v4.1**

## Versioning

Chaque format expose un en-tete versionne :

- MUF: `!muf 4`
- MFF: `mff 1`
- Run log: `format "muffin-runlog-1"`

## Lexer MUF (Hugo)

Pour activer la coloration MUF via Chroma :

```sh
cd docs/site/tools/hugo-muf
go run . -s ../../
```

## MUF (buildfiles)

Source declarative du build. Voir aussi la grammaire : `assets/grammar/muffin.ebnf`.

## MFF (config resolue)

Artefact stable genere par `muffin build muffin`.

## Run log

Log machine/humain produit par `muffin run`.

## Changelog

Historique des versions : `docs/reference/formats/changelog.md`.

## Voir aussi

- [Docs generees](/generated)
