# Manpages

# Manpages

Cette documentation regroupe les pages de manuel de **Muffin**, en style *manpage* : objectifs, syntaxe, commandes, flags, formats d’artefacts et conventions cross-platform.

> Muffin est un orchestrateur de build multi-langages.
> Le binaire **`Muffinconfig.mff`** est l’artefact canonique gelé et normalisé (graph, inputs, outils, empreintes), consommé par **Muffin Build**.

---

## Index

- [muffin(1)](./muffin.1.md) — commande principale, configuration et exécution
- [muffin-build(1)](./muffin-build.1.md) — construction à partir de `Muffinconfig.mff`
- [muffin-configure(1)](./muffin-configure.1.md) — phase de configuration (génération `.mff`)
- [muffin-decompile(1)](./muffin-decompile.1.md) — audit / décompilation (`.mff`, `.muff`)
- [muffin-graph(1)](./muffin-graph.1.md) — export et inspection du DAG
- [muffin-why(1)](./muffin-why.1.md) — explication de rebuild (chaîne d’invalidation)
- [muffin-clean(1)](./muffin-clean.1.md) — purge des caches/artefacts
- [muffin-doctor(1)](./muffin-doctor.1.md) — diagnostic d’environnement et toolchains

---

## Conventions

### Artefacts

- `*.muff` / `*.muf` : buildfiles (configuration + règles)
- `*.mff` : **binaire de compilation** gelé et normalisé (contrat de build)
- Sorties (selon targets/plateformes) :
  - `*.o` / `*.obj` (objets)
  - `*.a` / `*.lib` (archives statiques)
  - `*.so` / `*.dylib` / `*.dll` (librairies partagées)
  - `*.exe` (exécutables Windows)
  - `*.vo` / `*.va` (artefacts Vitte, si utilisés)

### Targets

Muffin utilise des triples `OS/ARCH/ABI` (ex: `x86_64-unknown-linux-gnu`, `aarch64-apple-darwin`, `x86_64-pc-windows-msvc`) pour rendre la construction explicite, portable et reproductible.

### Sorties et chemins

- Les chemins sont normalisés (format POSIX interne) pour assurer la stabilité cross-platform.
- Les globs sont développés pendant la phase `configure` et figés dans `.mff`.

---

## Guide rapide

- Configuration : `build muffin`
- Construction : `Muffin build`
- Tout construire : `build muffin -all` (ou plan dédié)
- Inspecter : `muffin decompile <projet.mff>`
- Comprendre un rebuild : `muffin -why <artifact>`
- Exporter le graphe : `muffin -graph --format dot`