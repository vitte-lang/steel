---
title: Observabilite
slug: /observabilite/
description: Doctor, cache et logs pour diagnostiquer Muffin.
---

# Observabilite

## muffin doctor

```sh
muffin doctor --root .
muffin doctor --root . --json
```

## muffin cache

```sh
muffin cache status --root .
muffin cache clear --root .
muffin cache status --root . --json
```

## Logs de run

Par defaut :

- `target/muffin_run_<timestamp>.mff`

Options :

- `--log <path>`
- `--log-mode append|truncate`

## Voir aussi

- [Formats](/reference/formats)
- [Docs generees](/generated)
