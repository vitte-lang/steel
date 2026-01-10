# MFF (Configuration resolue)

`Muffinconfig.mff` est l'artefact stable genere par `muffin build muffin`.

## Versioning

Le format est versionne via l'en-tete :

```text
mff 1
```

## Objectif

- Capture la configuration resolue (profil, target, chemins, tools).
- Sert de contrat deterministe pour l'execution.

## Exemple (indicatif)

```text
mff 1

host
  os "linux"
  arch "x86_64"
.end

profile "debug"
```

## Schema

Schema informel : structure stable, mais non publiee en JSON Schema a ce stade.
Voir la reference de configuration :

- `docs/reference/config/index.md`

## Compatibilite

- `mff 1` est stable.
- Toute rupture implique une nouvelle version d'en-tete.
