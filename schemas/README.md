# Muffin machine-readable schemas

This directory hosts JSON schemas for stable, machine-readable outputs
intended for IDE/CI tooling.

Currently covered:
- `muffin.graph.json/1` -> `schemas/muffin.graph.json.schema.json`
- `muffin.decompile.report` -> `schemas/muffin.decompile.report.schema.json`
- `muffin.fingerprints.json/1` -> `schemas/muffin.fingerprints.json.schema.json`

Notes:
- The MUF language syntax is specified in `assets/grammar/muffin.ebnf`.
- The `.mff` text format is specified in `doc/manifest.md` (section "Format Muffinconfig.mff").
