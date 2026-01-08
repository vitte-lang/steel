---
title: "API"
slug: "/api"
description: "Référence API interne de Muffin : resolver, generator, schémas, sérialisation, diagnostics."
sidebar:
  order: 40
  label: "API"
toc: true
---

# API

Cette section documente l’API interne/exposée de **Muffin** (librairie et modules), utilisée par le CLI, les générateurs, et les intégrations (CI, IDE, tooling). L’objectif est de fournir une **référence stable**, orientée “contrats” : types, invariants, erreurs, et flux d’exécution.

> Pour les usages “utilisateur final”, privilégier **/reference** et **/manual**.  
> Ici, on suppose une compréhension du pipeline Muffin : parsing → validation → resolve → graph → jobs → exécution → artefacts.

---

## Vue d’ensemble

### Couches principales

- **Schema** : description normative des formats (mcfg/muff/muf/targets), validation et compat.
- **Serializer** : lecture/écriture des formats et normalisation (canonical form).
- **Diagnostics** : collecte d’erreurs, warnings, notes, spans, suggestions, codes.
- **Resolver** : construction du modèle projet (config per-dir, graph, dépendances, targets).
- **Generator** : production des sorties (rsp, commandes, artefacts, arborescences out).
- **Hashing / Cache** : empreintes, clés de cache, invalidation, store.
- **VMS** : modélisation “progname / jobs / dir / functions” pour la résolution multi-répertoires.

---

## Index des modules

- [`resolver`](./resolver.md) — chargement projet, résolution graph, targets, profils.
- [`generator`](./generator.md) — génération des commandes/outils, rsp, outputs.
- [`schema`](./schema.md) — schémas, validation, compatibilité.
- [`serializer`](./serializer.md) — parsing/émission, canonicalisation.
- [`diagnostics`](./diagnostics.md) — erreurs/warnings, spans, rendering.
- [`hashing`](./hashing.md) — hashing stable, clés, fingerprints.
- (optionnel) [`vms`](../spec/vms/index.md) — la spec VMS (référence normative).

---

## Contrats communs

### Identités et chemins

Muffin manipule plusieurs formes de “chemin” :

- **Path (host)** : chemin OS réel (`C:\...` / `/Users/...`)
- **ProjectPath (logical)** : chemin relatif au project root (`Src/in/...`)
- **VPath (virtual)** : chemin normalisé pour hashing/cache (`src/in/...` canonique)

Recommandations :

- Canonicaliser tôt (séparateurs, `..`, symlinks selon policy).
- Conserver les chemins *originaux* pour diagnostics (UX).
- Utiliser les chemins *canon* pour hashing/cache.

### Erreurs

Toutes les APIs retournent soit un `Result<T, MuffinError>` (ou équivalent), soit un couple `(T, Diagnostics)` si on accepte de continuer avec warnings.

Catégories usuelles :

- `ConfigNotFound`
- `ParseError(format, span, message)`
- `ValidationFailed(rule, details)`
- `ResolveFailed(reason)`
- `CompilationFailed(tool, exit_code, stdout, stderr)`
- `IoError(path, source)`
- `Unsupported(feature, target)`
- `InvariantViolation(context, hint)` *(dev-only, hard fail)*

### Diagnostics (format)

Un diagnostic structuré contient typiquement :

- `code` : identifiant stable (`MUFFIN0001`)
- `severity` : error|warning|note|help
- `message` : texte principal
- `primary_span` : (file, range)
- `secondary_spans` : n spans de contexte
- `hint` / `fixit` : suggestion (optionnelle)
- `cause_chain` : causes imbriquées (optionnel)
- `tags` : `["resolver", "schema", "muf"]`

---

## Cycle de vie “API” (pipeline)

### 1) Load

- découvrir le **project root**
- lire `.muff/` / `.muffin/` (selon conventions)
- charger les configs per-dir (`*.mcfg` ou équivalent)
- charger les manifests (`mod.muf`, `build.muf`, targets)

### 2) Parse + Validate

- parse (serializer)
- validate (schema)
- produire diagnostics (sans side-effects)

### 3) Resolve

- sélectionner target + profile
- résoudre variables/implicites
- construire graph (units, deps, outputs)
- préparer jobs (VMS / execution plan)

### 4) Generate

- émettre RSP et lignes de commande
- calculer clés de cache / fingerprints
- produire artefacts (obj, libs, exe, vo/va)

### 5) Execute (CLI layer)

- exécuter jobs
- collecter sorties
- finaliser (dSYM, strip, codesign, etc.)
- écrire états (cache/store)

---

## Compatibilité & stabilité

### “Stable API” vs “Internal”

- **Stable** : schémas de formats, CLI JSON output, structures de diagnostics, conventions de layout.
- **Internal** : types intermédiaires de résolution et graph (susceptibles d’évoluer rapidement).

Politique suggérée :

- versionner les schémas (`schema_version`)
- tagger les breaking changes via `RFC` + `CHANGELOG`
- fournir des “migrations” (ou normalisations) au niveau serializer

---

## Conventions de versionnement

- **SemVer** pour la lib (si publiée)
- Version **indépendante** pour schémas `muf/mcfg` si nécessaire
- Dans les fichiers :
  - `schema: muffin.mcfg/v1`
  - `schema: muffin.muf/v2`

---

## Sécurité & sandboxing

Certaines APIs déclenchent des actions à risque :

- exécution de toolchain
- accès FS (store/capsule)
- réseau (fetch deps)

Contrat :

- aucune exécution implicite dans `parse/validate/resolve`
- exécution uniquement via couche “runner”/CLI, avec policy explicite (capsule)

---

## Exemples d’intégration (haut niveau)

### Intégration IDE / LSP

- exposer :
  - parse + validate en continu
  - diagnostics structurés
  - navigation vers spans
- éviter :
  - resolve complet si lourd (mode “lazy”)

### CI

- `muffin doctor` (préflight)
- `muffin build -all` (build)
- `muffin graph --json` (audit deps)
- export SBOM (optionnel)

---

## Glossaire API

- **Unit** : entité compilable (crate/module/package/directory build unit)
- **Job** : action exécutable (compile, archive, link, gen)
- **Artifact** : sortie matérielle (obj/lib/exe/vo/va)
- **Fingerprint** : hash stable des inputs + config
- **Store** : cache de contenu (CAS) + index
- **Capsule** : policy d’exécution (fs/env/net/time)

---

## Voir aussi

- Manuel : [/manual](../manual/00-introduction.md)
- Référence CLI : [/reference/cli](../reference/cli/index.md)
- Spec formats : [/spec](../spec/index.md)
- RFC : [/rfc](../rfc/index.md)
