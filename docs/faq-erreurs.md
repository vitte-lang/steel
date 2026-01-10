# FAQ Erreurs

Erreurs frequentes et solutions rapides.

## `error[U001]` commande inconnue

Cause: commande ou sous-commande invalide.

Solution:
- `muffin --help` pour la liste des commandes.
- Verifier l'orthographe et la casse.

## `error[C001]` config introuvable/invalide

Cause: fichier MUF absent ou mal reference.

Solution:
- Verifier le chemin: `--root` et `--file`.
- Confirmer que `MuffinConfig.muf` existe.

## `error[P001]` erreur de parsing MUF

Cause: syntaxe MUF invalide.

Solution:
- Verifier le header `!muf 4` et les blocs `[x] ..`.
- Regarder `assets/grammar/muffin.ebnf`.
- Tester sans execution: `muffin run --print`.

## `error[X001]` echec d'execution tool

Cause: une commande (gcc/ar/etc) a echoue.

Solution:
- Verifier que la toolchain est dans le `PATH`.
- Ou utiliser `--toolchain <dir>` pour pointer vers les binaires.
- Inspecter le log `--log` ou `target/muffin_run_*.mff`.

## `error[IO01]` erreur E/S

Cause: fichier ou dossier inaccessible.

Solution:
- Verifier permissions et chemins.
- Essayer un `--root` explicite.

## Windows: `command not found`

Cause: le dossier de `muffin.exe` n'est pas dans le `PATH`.

Solution:
- Lancer via le chemin complet: `.\muffin.exe --help`.
- Ajouter le dossier au `PATH` systeme/utilisateur.

## Rust/Cargo: "current package believes it's in a workspace when it's not"

Cause: un `Cargo.toml` est detecte dans un workspace sans etre membre.

Solution:
- Ajouter le chemin dans `workspace.members` du `Cargo.toml` racine.
- Ou l'exclure via `workspace.exclude`.
- Ou ajouter un `[workspace]` vide dans le crate isole.

## Rust/Cargo: "manifest is missing either a [package] or a [workspace]"

Cause: `Cargo.toml` incomplet.

Solution:
- Ajouter une section `[package]` valide.
- Ou marquer un workspace minimal: `[workspace]`.

## Rust/Cargo: "failed to read ... Cargo.toml (No such file or directory)"

Cause: chemin de dependance incorrect.

Solution:
- Verifier le `path = "../..."` dans `Cargo.toml`.
- Corriger l'arborescence ou le `workspace.members`.

## Log ecrase a chaque run

Solution:
- Utiliser `--log <path>` et `--log-mode append`.
- Ou laisser le log date par defaut: `target/muffin_run_<timestamp>.mff`.
