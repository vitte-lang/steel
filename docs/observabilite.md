# Observabilite

Cette page decrit les commandes d'observabilite et le format des logs.

## muffin doctor

Diagnostic rapide de l'environnement (config + tools).

```sh
muffin doctor --root .
```

Format machine :

```sh
muffin doctor --root . --json
```

## muffin cache

Etat et nettoyage du cache local.

```sh
muffin cache status --root .
muffin cache clear --root .
```

Format machine :

```sh
muffin cache status --root . --json
```

## Logs de run

Le runner ecrit un log `.mff` par execution :

- Defaut : `target/muffin_run_<timestamp>.mff`
- Option : `--log <path>` et `--log-mode append|truncate`

Exemple :

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
