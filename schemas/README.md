# Flan machine-readable schemas

This directory hosts JSON schemas for stable, machine-readable outputs
intended for IDE/CI tooling.

Currently covered:
- `flan.graph.json/1` -> `schemas/flan.graph.json.schema.json`
- `flan.decompile.report` -> `schemas/flan.decompile.report.schema.json`
- `flan.fingerprints.json/1` -> `schemas/flan.fingerprints.json.schema.json`

Notes:
- The MUF language syntax is specified in `assets/grammar/flan.ebnf`.
- The `.mff` text format is specified in `doc/manifest.md` (section "Format Flanconfig.mff").
