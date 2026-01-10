# Run log (.mff)

Les executions `muffin run` produisent un log `.mff` lisible par machine et humain.

## Versioning

Le format est versionne via un header `format` :

```text
[log meta]
format "muffin-runlog-1"
..
```

## Structure

- `[log meta]` : meta globale (format, outil, version, timestamp).
- `[bake log "<name>"]` : groupe les runs par bake.
- `[run log]` : commande + sortie + statut.
- `[run summary]` : fin d'execution.

## Exemple (extrait)

```text
[log meta]
format "muffin-runlog-1"
tool "muffin"
version "muffin 0.1.0"
ts_iso "2026-01-10T04:16:37Z"
..

[bake log "app"]
[run log]
ts 1768018597
ts_iso "2026-01-10T04:16:37Z"
duration_ms 72
cmd "gcc -std=c17 -O0 -g -Wall -Wextra app/main.c -o target/out/app"
status 0
ok true
..
runs 1
duration_ms 72
..
[run summary]
ts_iso "2026-01-10T04:16:37Z"
..
```

## Ecriture

Options utiles :

- `--log <path>`
- `--log-mode append|truncate`

## Compatibilite

`muffin-runlog-1` est stable. Toute rupture implique un nouvel identifiant.
