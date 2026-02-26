# Module Organization

Ce fichier donne une vue simple du code source Steel.

## Entree CLI

- `src/bin/steel.rs`: point d entree de la commande `steel`
- `src/commands.rs`: parsing des commandes + dispatch

## Build/Run

- `src/build_muf.rs`: phase de preparation/config
- `src/run_muf.rs`: execution des recettes

## Parsing

- `src/parser/lexer.rs`: decoupe du texte en tokens
- `src/parser/parser.rs`: construit les structures
- `src/read.rs` / `src/arscan.rs`: lecture/analyse support

## Validation/Resolution

- `src/validator.rs`: checks de coherence
- `src/variable.rs`, `src/expand.rs`, `src/implicit.rs`: resolution des valeurs

## Runtime/utilitaires

- `src/os.rs`: integration OS
- `src/job.rs`: execution de process
- `src/hash.rs`: empreintes
- `src/output.rs`: sorties

## Edition

- `src/bin/steecleditor.rs`: editeur terminal integre

## Mode mental

1. Lire `steelconf`
2. Parser/valider
3. Resoudre
4. Executer les recipes
