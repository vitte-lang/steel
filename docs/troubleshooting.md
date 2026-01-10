# Troubleshooting

Guide plus long pour diagnostiquer les problemes courants.

## 1) La commande `muffin` est introuvable

Symptome:
- `command not found` ou `muffin n'est pas reconnu`.

Diagnostic:
- `which muffin` (macOS/Linux) ou `where muffin` (Windows).

Fix:
- Ajouter le dossier contenant `muffin` au `PATH`.
- Ou utiliser le chemin complet vers `muffin`.

## 2) `error[U001]` commande inconnue

Cause:
- Sous-commande invalide ou faute de frappe.

Fix:
- `muffin --help` pour la liste des commandes.
- Verifier l'orthographe des flags.

## 3) `error[C001]` config introuvable ou invalide

Cause:
- `MuffinConfig.muf` absent ou mauvais `--root/--file`.

Fix:
- Verifier que le fichier existe.
- Utiliser `muffin doctor --root <dir>`.

## 4) `error[P001]` erreur de parsing MUF

Cause:
- Syntaxe MUF invalide.

Fix:
- Verifier le header `!muf 4`.
- Verifier les blocs `[block] ..` et les directives `.set/.make/...`.
- Reference: `assets/grammar/muffin.ebnf`.

## 5) `error[X001]` tool execution failed

Cause:
- `gcc`, `ar` ou autre outil introuvable ou en echec.

Fix:
- Verifier le `PATH` ou utiliser `--toolchain <dir>`.
- Consulter le log `.mff` (`--log`).

## 6) `error[IO01]` erreur d'E/S

Cause:
- Fichier/dossier inaccessible.

Fix:
- Verifier permissions et chemins.
- Tester avec un `--root` explicite.

## 7) Logs vides ou incomplets

Cause:
- Run en dry-run (`--print`) ou log vide.

Fix:
- Lancer sans `--print`.
- Utiliser `--log-mode truncate` pour un fichier propre.

## 8) Le cache ne s'invalide pas

Cause:
- Mtime inchangée ou glob mal defini.

Fix:
- Utiliser `--no-cache` pour forcer.
- Verifier la definition des `cglob`/`glob`.

## 9) Crash sans message (Windows)

Cause:
- Lancement par double clic sans console.

Fix:
- Lancer depuis un terminal: `muffin --help`.
- Consulter l'Observateur d'evenements (Windows).

## 10) Diagnostics rapides

Commandes utiles:

```sh
muffin doctor --root .
muffin cache status --root .
muffin run --root . --file MuffinConfig.muf --print
```

## 11) Cargo: "current package believes it's in a workspace when it's not"

Symptome:
- `current package believes it's in a workspace when it's not`

Fix:
- Ajouter le crate dans `workspace.members` du `Cargo.toml` racine.
- Ou l'exclure via `workspace.exclude`.
- Ou ajouter un `[workspace]` vide dans le crate.

## 12) Cargo: manifest manquant

Symptome:
- `manifest is missing either a [package] or a [workspace]`

Fix:
- Ajouter une section `[package]` complete.
- Ou declarer un workspace minimal via `[workspace]`.

## 13) Module Rust manquant

Symptome:
- `couldn't read .../src/mod.rs: No such file or directory`

Fix:
- Supprimer le `pub mod` correspondant si le module n'existe plus.
- Ou recreer le fichier `mod.rs` attendu.
