# Docs workflow

# Docs workflow

Ce document décrit le workflow de documentation du projet **Muffin** : conventions, structure, génération, style, et règles de contribution.

---

## Objectifs

- Documentation **portable** et **multi-langages** (Muffin n’est pas lié à un langage).
- Séparation claire : **spécification buildfile** (`.muf/.muff`), **binaire de compilation** (`.mff`), **CLI**, **API Rust**, **exemples**.
- Contenu **outillable** : indexables, manpages cohérentes, liens internes stables.
- Éviter les doublons : chaque concept a un emplacement “source de vérité”.

---

## Arborescence de la doc

Chemins recommandés :

```text
docs/
  api/                 # API interne (Rust): modules, schémas, IO .mff
    index.md
  examples/            # Exemples buildfiles
    muf/
      minimal/
      advanced/
  man/                 # Manpages (style unix) en Markdown
    index.md
    muffin.1.md
    muffin-configure.1.md
    muffin-build.1.md
    muffin-decompile.1.md
    muffin-graph.1.md
    muffin-why.1.md
    muffin-clean.1.md
    muffin-doctor.1.md
  manual/              # Manuel narratif (chapitres)
    00-introduction.md
    01-concepts.md
    99-appendix.md
  meta/                # Méta-docs (workflow, style, contribution)
    docs-workflow.md
```

---

## Sources de vérité

- **EBNF / grammaire buildfile** : `muffin.ebnf` (ou `docs/spec/` si introduit)
- **CLI** : `docs/man/*.md` (manpages)
- **Manuel narratif** : `docs/manual/*.md`
- **API** : `docs/api/index.md` + doc modules
- **Exemples** : `docs/examples/`

Règle : si une information est “normative” (comportement attendu), elle doit être écrite d’abord dans la manpage ou la spec.

---

## Conventions d’écriture

### Ton et style

- Français technique, concis, orienté usage.
- Définir un terme à sa première occurrence.
- Préférer les listes à puces à de longs paragraphes.
- Pas de promesses non vérifiables : si un comportement est “selon impl”, le marquer.

### Noms et termes

- **Buildfile** : `*.muf` / `*.muff`
- **Binaire de compilation** : `*.mff` (ex: `Muffinconfig.mff`)
- **Configure** : `build muffin` (génère `.mff`)
- **Build** : `Muffin build` (exécute depuis `.mff`)
- **Bake** : nœud du DAG
- **Port** : `in/out`
- **Wire** : liaison `out → in`
- **Tool** : exécutable déclaré
- **Capsule** : policy sandbox
- **Store** : cache d’artefacts

### Blocs de code

- Commandes : bloc `text`.
- Buildfiles : bloc `muf`.
- Code Rust : bloc `rust`.
- Éviter les blocs trop longs : si nécessaire, référencer un fichier d’exemple.

---

## Cross-platform (Linux / macOS / Windows / BSD / Solaris)

### Exemples de targets

- `x86_64-unknown-linux-gnu`
- `aarch64-unknown-linux-gnu`
- `x86_64-apple-darwin`
- `aarch64-apple-darwin`
- `x86_64-pc-windows-msvc`
- `x86_64-unknown-freebsd`
- `x86_64-unknown-solaris`

### Artefacts (référence)

- objets : `*.o` / `*.obj`
- statiques : `*.a` / `*.lib`
- partagées : `*.so` / `*.dylib` / `*.dll`
- exécutables : `*.exe` (Windows) / sans extension (Unix)

Dans la doc, privilégier les **types logiques** (`bin.obj`, `lib.static`, `bin.exe`) plutôt que les extensions.

---

## Règles de mise à jour

### Quand modifier quoi ?

- Nouvelle commande CLI :
  1) ajouter/mettre à jour la manpage correspondante dans `docs/man/`
  2) mettre à jour `docs/man/index.md`
  3) ajouter un exemple minimal si nécessaire dans `docs/examples/`

- Nouvelle fonctionnalité de buildfile :
  1) mettre à jour `muffin.ebnf`
  2) ajouter un exemple `.muf`/`.muff`
  3) documenter le concept dans `docs/manual/01-concepts.md`

- Nouveau format/changement `.mff` :
  1) documenter le versioning/compat dans `docs/api/index.md` et `docs/manual/`
  2) mettre à jour les manpages `configure/build/decompile`

### Règles anti-doublons

- Les manpages décrivent le **contrat CLI**.
- Le manuel décrit le **modèle mental**.
- Les exemples montrent la **syntaxe**.
- L’API décrit le **code**.

---

## Génération et validation

### Lint (recommandations)

- Liens internes : pas d’URL brutes dans les manpages, préférer liens relatifs.
- Titres : un seul H1 par fichier.
- Sections : `NAME`, `SYNOPSIS`, `DESCRIPTION`, `OPTIONS`, `EXAMPLES`, `SEE ALSO` pour les manpages.

### Validation manpages

Checklist par fichier `docs/man/*.md` :

- `NAME` et `SYNOPSIS` présents
- options documentées (même si “selon impl”)
- exemples d’usage minimal
- `SEE ALSO` cohérent

### Cohérence terminologique

- `.mff` = **binaire de compilation** (pas “config texte”)
- `build muffin` = **configure**
- `Muffin build` = **build**

---

## Template manpage

```md
# <cmd>(1)

## NAME

**<cmd>** — <résumé>

## SYNOPSIS

```text
<cmd> [options]
```

## DESCRIPTION

...

## OPTIONS

...

## EXIT STATUS

...

## EXAMPLES

...

## SEE ALSO

...
```

---

## Template chapitre (manuel)

```md
# Titre

Résumé.

---

## Section

Contenu.
```

---

## Contribution

- Les changements docs doivent être atomiques (une PR = un sujet).
- Les exemples `.muf` doivent être exécutables dans l’esprit (pas de placeholders non expliqués).
- Toute nouvelle commande doit avoir au minimum : manpage + entrée index + exemple minimal.

---

## Historique

- Les docs doivent rester compatibles avec les versions de `.mff`.
- En cas de changement majeur, ajouter une note dans `docs/api/index.md` et ajuster les manpages.