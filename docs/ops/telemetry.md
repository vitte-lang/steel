# Telemetry

# Telemetry

Cette page décrit une stratégie **MAX** de télémétrie pour Muffin : métriques, traces, logs, événements, privacy, configuration, et export cross-platform.

---

## Objectifs

- Comprendre les performances : temps par bake, cache hit/miss, I/O.
- Diagnostiquer : échecs tools, invalidations, blocages scheduler.
- Améliorer la fiabilité : détection d’anomalies, régressions.
- Préserver la confidentialité : télémétrie **opt-in** ou contrôlée.
- Rester portable : Linux/macOS/Windows/BSD/Solaris.

---

## Principes

### 1) Observabilité par couches

- **Logs** : narratif, humain.
- **Métriques** : agrégations (compteurs, histogrammes).
- **Traces** : chronologie (spans) du run.
- **Événements** : occurrences structurées (ex: `cache_miss`).

### 2) Sorties machine-friendly

- exporter en **JSON** (stable)
- exporter en **NDJSON** (stream)
- exporter en **OpenTelemetry** (si activé)

### 3) Confidentialité par design

- pas de données personnelles par défaut
- anonymisation/suppression de chemins si demandé
- opt-in explicite pour envoi réseau

---

## Niveaux de télémétrie

Proposition de niveaux (flags) :

- `--telemetry off` : rien (par défaut si politique stricte)
- `--telemetry local` : écrit fichiers locaux (logs/metrics/traces)
- `--telemetry otlp` : export OTLP vers collector

Niveau détaillé :

- `--telemetry-level minimal|normal|verbose`

---

## Modèle d’événements

Chaque événement est structuré :

- `ts` : timestamp
- `run_id` : identifiant du run
- `severity` : info/warn/error
- `kind` : `metric|event|span|log`
- `name` : nom stable (ex: `bake.start`)
- `attrs` : map (clé/valeurs)

Exemple NDJSON (concept) :

```json
{"ts":"2026-01-08T09:12:33.120Z","run_id":"r-01H...","kind":"event","name":"bake.start","attrs":{"bake":"compile_c","target":"x86_64-unknown-linux-gnu"}}
```

---

## Logs

### Contenu recommandé

- phases : configure/build
- target/profile
- tool invocations (résumé)
- décisions cache (hit/miss + raison)
- erreurs : code + message + hints

### Formats

- texte (console)
- JSON/NDJSON

### Rotation

- `./.muffin/logs/<run_id>.log`
- conserver N runs ou N jours

---

## Métriques

### Métriques clés

#### Build

- `muffin.run.duration_ms` (histogram)
- `muffin.run.exit_code` (counter)

#### Scheduler

- `muffin.scheduler.ready_queue_depth` (gauge)
- `muffin.scheduler.jobs_running` (gauge)
- `muffin.scheduler.stalls_total` (counter)

#### Bake

- `muffin.bake.duration_ms` (histogram)
- `muffin.bake.failures_total` (counter)
- `muffin.bake.replayed_total` (counter)

#### Cache / store

- `muffin.cache.hit_total` (counter)
- `muffin.cache.miss_total` (counter)
- `muffin.cache.store_bytes_read` (counter)
- `muffin.cache.store_bytes_written` (counter)

#### I/O

- `muffin.io.files_read_total` (counter)
- `muffin.io.files_written_total` (counter)
- `muffin.io.bytes_read_total` (counter)
- `muffin.io.bytes_written_total` (counter)

#### Tools

- `muffin.tool.duration_ms` (histogram)
- `muffin.tool.exit_code_total{code}` (counter)

### Labels (tags)

Recommandations :

- `target`, `profile`, `plan`
- `bake`
- `tool`
- `cache_mode`

Attention : éviter les labels à cardinalité explosive (ex: paths complets).

---

## Traces (spans)

### Hiérarchie recommandée

- `run` (racine)
  - `configure`
    - `parse`
    - `validate`
    - `resolve`
    - `glob_expand`
    - `write_mff`
  - `build`
    - `read_mff`
    - `schedule`
    - `bake/<name>`
      - `cache_lookup`
      - `tool/<name>`
      - `write_outputs`

### Export

- local : JSON traces
- remote : OTLP (OpenTelemetry)

---

## Confidentialité (privacy)

### Données sensibles

Éviter par défaut :

- chemins absolus
- noms d’utilisateurs
- variables d’environnement
- contenu de fichiers

### Redaction

Options :

- `--redact-paths` : remplace `/home/user/...` par `/<redacted>/...`
- `--hash-paths` : hash stable (permet corrélation sans fuite)

### Consentement

- export réseau désactivé par défaut
- activer explicitement `--telemetry otlp` + `--otlp-endpoint`

---

## Configuration

### Flags (proposition)

- `--telemetry off|local|otlp`
- `--telemetry-level minimal|normal|verbose`
- `--telemetry-out <dir>`
- `--telemetry-format json|ndjson`
- `--otlp-endpoint <url>`
- `--otlp-headers <k=v,...>`
- `--redact-paths` / `--hash-paths`

### Variables d’environnement

- `MUFFIN_TELEMETRY=off|local|otlp`
- `MUFFIN_OTLP_ENDPOINT=...`

---

## Stockage local

Recommandation :

```text
.muffin/
  logs/
  metrics/
  traces/
  runs/
```

- un dossier par run : `runs/<run_id>/`
- fichiers :
  - `events.ndjson`
  - `metrics.json`
  - `traces.json`
  - `summary.json` (rollup)

---

## Intégration CI

### Objectifs

- attacher un résumé aux artefacts CI
- comparer les métriques (régression)

### Pattern

- `--telemetry local --telemetry-out ./ci_telemetry`
- uploader `ci_telemetry/` en artefact

---

## Cross-platform

- timestamps : ISO 8601 UTC
- chemins : normaliser en interne, redacter en sortie
- clock : préférer monotonic pour durées

---

## Dépannage

- trop de logs : réduire `--telemetry-level`
- perf impact : désactiver traces détaillées
- cardinalité élevée : supprimer labels variables

---

## Checklist

- [ ] télémétrie opt-in (ou contrôlée)
- [ ] export JSON/NDJSON stable
- [ ] labels maîtrisés
- [ ] redaction paths
- [ ] rollup summary
- [ ] intégration CI (artefacts)
- [ ] compat cross-platform