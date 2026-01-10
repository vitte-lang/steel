---
title: Troubleshooting
slug: /troubleshooting/
description: Diagnostic complet pour erreurs Muffin et Rust/Cargo.
---

# Troubleshooting

## 1) `muffin` introuvable

Diagnostic:
- `which muffin` (macOS/Linux) ou `where muffin` (Windows)

Fix:
- ajouter le dossier contenant `muffin` au `PATH`

## 2) `error[U001]` commande inconnue

Fix:
- `muffin --help`

## 3) `error[C001]` config introuvable/invalide

Fix:
- verifier `--root` et `--file`
- `muffin doctor --root <dir>`

## 4) `error[P001]` parsing MUF

Fix:
- verifier `!muf 4` et les blocs `[tag] ..`

## 5) `error[X001]` tool execution failed

Fix:
- verifier le `PATH` ou `--toolchain <dir>`

## 6) `error[IO01]` erreur I/O

Fix:
- verifier permissions et chemins

## 7) Logs vides

Fix:
- ne pas utiliser `--print`
- `--log-mode truncate`

## 8) Cache non invalide

Fix:
- `--no-cache`
- verifier les globs

## 9) Windows crash sans message

Fix:
- lancer depuis un terminal
- consulter l'Observateur d'evenements

## 10) Cargo: workspace/manifests

Problemes frequents:
- `current package believes it's in a workspace when it's not`
- `manifest is missing either a [package] or a [workspace]`

Fix:
- ajouter `workspace.members` ou `workspace.exclude`
- ajouter un `[workspace]` vide si besoin

## 11) Modules Rust manquants

Fix:
- supprimer le `pub mod` ou recreer `mod.rs`

## 12) Diagnostics rapides

```sh
muffin doctor --root .
muffin cache status --root .
muffin run --root . --file MuffinConfig.muf --print
```
