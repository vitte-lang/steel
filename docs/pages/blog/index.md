# Blog

---
title: Blog
slug: /blog/
description: Notes de conception, annonces et deep-dives sur Muffin (build graph, .muf, .mff, CLI, targets).
---

# Blog

Ici : des notes d’architecture, des annonces, et des retours d’implémentation sur **Muffin**.

- **Buildfile** : `Muffinfile` / `build.muf`
- **Binaire de build** : `*.mff` (snapshot résolu : graph + config)
- **Runtimes** : scheduling, store, capsules, targets

---

## À lire en priorité

- **CLI** : commandes, phases `configure` / `build`, graph, cache, sandbox.
  - Voir : `docs/site/generated/cli.md`
- **Config schema** : grammaire, règles de validation, représentation logique du `.mff`.
  - Voir : `docs/site/generated/config-schema.md`
- **Manifest schema** : identité projet, deps, publication, workspace.
  - Voir : `docs/site/generated/manifest-schema.md`
- **Target schema** : host/target, normalisation, formats, toolchains.
  - Voir : `docs/site/generated/target-schema.md`

---

## Derniers articles

> Index statique (à compléter avec des posts dans `docs/site/pages/blog/`).
> Convention recommandée : `YYYY-MM-DD-titre.md`.

- _À venir_ : “`.mff` comme unité reproductible : snapshot, intégrité, décompilation”
- _À venir_ : “Capsules : politiques env/fs/net/time et modèles de threat”
- _À venir_ : “Store : content-addressed, invalidations, GC et verifications”
- _À venir_ : “Targets : mapping multi-OS/arch + toolchains + sysroots”

---

## Tags

Proposition de taxonomie (pour organiser les posts) :

- `design` : décisions de design (format, sémantique, invariants)
- `cli` : interface et ergonomie
- `schema` : grammaires, validateurs, sérialisation
- `graph` : DAG, wiring, scheduler
- `cache` : store, invalidation, perf
- `sandbox` : capsules, isolation
- `targets` : host/target, cross, toolchains
- `interop` : intégration C/C++/C#/Rust/etc.
- `release` : CI, packaging, publication

---

## Écrire un post

### Structure minimale

```md
---
title: "Titre"
date: 2026-01-08
description: "Résumé"
tags: [design, graph]
---

# Titre

Contenu.
```

### Contenu attendu

- Un **problème** clair (UX / build / reproductibilité / perf)
- Une **décision** (ou une proposition) avec invariants
- Un **exemple** concret (commandes + extrait `.muf`)
- Les **trade-offs** (ce qu’on gagne / ce qu’on refuse)

---

## Roadmap éditoriale

- Décrire la séparation **manifest vs buildfile**
- Spécifier le cycle complet **configure → .mff → build**
- Formaliser les invariants :
  - determinism (tri stable, hashing stable)
  - DAG sans cycles
  - compat types ports
  - sandbox policy explicite
- Documenter la décompilation `.mff` (audit / debug / reproductibilité)

---

## Navigation

- **Docs** : `/docs/`
- **CLI** : `/docs/generated/cli/` (selon routing du site)
- **Schemas** : `/docs/generated/`