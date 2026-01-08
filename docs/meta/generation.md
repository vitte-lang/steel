# Manpages

Cette section regroupe les pages de manuel de **Muffin** (style manpage). Elle documente le **contrat CLI**, la séparation **configure/build**, les artefacts (`*.muf/*.muff`, `*.mff`) et les commandes d’introspection.

**Rappels**

- `build muffin` : **configure** → parse/validate/resolve buildfiles → écrit **`Muffinconfig.mff`**
- `Muffin build` : **build** → lit `Muffinconfig.mff` → exécute le DAG via tools déclarés

---

## Index

- [muffin(1)](./muffin.1.md)
  - Commande principale : sous-commandes, options globales, fichiers

- [muffin-configure(1)](./muffin-configure.1.md)
  - Phase configuration : génération `.mff`, résolution, expansion globs, pinning toolchain

- [muffin-build(1)](./muffin-build.1.md)
  - Phase build : exécution DAG depuis `.mff`, cache, scheduler, logs

- [muffin-decompile(1)](./muffin-decompile.1.md)
  - Audit / décompilation : vue normalisée depuis `.mff` ou buildfile

- [muffin-graph(1)](./muffin-graph.1.md)
  - Export/inspection du DAG : DOT/JSON/TEXT

- [muffin-why(1)](./muffin-why.1.md)
  - Explication d’invalidation/rebuild : chaîne de dépendances

- [muffin-clean(1)](./muffin-clean.1.md)
  - Purge cache/artefacts (scope contrôlé)

- [muffin-doctor(1)](./muffin-doctor.1.md)
  - Diagnostic environnement/toolchains/capsule

---

## Artefacts et conventions

### Buildfiles

- `*.muf` / `*.muff` : buildfiles déclaratifs (configuration + règles)
- Entrées usuelles : `Muffinfile`, `build.muf`, `main.muff`, `master.muff`

### Binaire de compilation

- `*.mff` : **binaire de compilation** (contrat gelé)
- Canonique : `Muffinconfig.mff`

Contenu typique du `.mff` :

- DAG normalisé (bakes/ports/wires/exports/plans)
- liste exhaustive des fichiers (globs développés)
- paths normalisés
- toolchains effectives + versions
- empreintes d’invalidation (cache)
- trace (timings/logs) si activée

### Sorties (exemples cross-platform)

- objets : `*.o` (Unix), `*.obj` (Windows)
- statiques : `*.a` (Unix), `*.lib` (Windows)
- partagées : `*.so` (Linux/BSD/Solaris), `*.dylib` (macOS), `*.dll` (Windows)
- exécutables : sans extension (Unix), `*.exe` (Windows)

Recommandation : utiliser des **types logiques** dans Muffin (`bin.obj`, `lib.static`, `lib.shared`, `bin.exe`) et résoudre l’extension via `target`.

---

## Table de commandes (résumé)

| Intention | Commande | Sortie/effet |
|---|---|---|
| Configurer | `build muffin` | écrit `Muffinconfig.mff` |
| Configurer un plan | `build muffin <plan>` | `.mff` avec plan choisi |
| Construire | `Muffin build` | exécute le DAG depuis `.mff` |
| Auditer | `muffin decompile <path>` | vue normalisée |
| Expliquer rebuild | `muffin why <artifact>` | chaîne d’invalidation |
| Exporter DAG | `muffin graph ...` | dot/json/text |
| Purger | `muffin clean ...` | supprime cache/artefacts |
| Diagnostiquer | `muffin doctor ...` | état toolchains/capsule |

---

## Exemples rapides

```text
# Configure
build muffin
build muffin release
build muffin -all
build muffin -D target=x86_64-unknown-linux-gnu

# Build
Muffin build
Muffin build --plan release
Muffin build -j 16

# Introspection
muffin decompile Muffinconfig.mff
muffin why out/bin/app
muffin graph --format dot --out graph.dot

# Maintenance
muffin clean --scope cache
muffin doctor --tools
```

---

## Notes cross-platform (Linux / macOS / Windows / BSD / Solaris)

- Le `.mff` est **portable structurellement** (normalisation). Les binaires produits dépendent des toolchains disponibles.
- `capsule` : enforcement best-effort selon OS.
- Caches : privilégier `store mode content` pour des invalidations déterministes.

---

## Voir aussi

- Manuel : `docs/manual/`
- Exemples : `docs/examples/`
- API : `docs/api/`
