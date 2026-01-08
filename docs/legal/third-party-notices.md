---
title: "Third-Party Notices"
slug: "/legal/third-party-notices"
description: "Avis et licences des logiciels et outils tiers utilisés avec Muffin."
toc: true
---

# Third-Party Notices

Cette page liste les **logiciels, outils et technologies tiers** susceptibles d’être utilisés avec **Muffin**, ainsi que leurs licences respectives.

Muffin **n’embarque pas** ces outils par défaut ; ils sont généralement **installés séparément** par l’utilisateur ou fournis par l’environnement système.

---

## Outils de compilation et de build

### LLVM / Clang

- **Projet** : LLVM / Clang  
- **Usage** : compilation C/C++, assemblage, édition de liens (via `clang`, `ld.lld`)  
- **Licence** : Apache License 2.0 avec exceptions LLVM  
- **Site** : https://llvm.org/

---

### GNU Binutils (`ar`, `ranlib`, `ld`)

- **Projet** : GNU Binutils  
- **Usage** : création d’archives statiques, index de symboles, édition de liens  
- **Licence** : GNU General Public License v3 (ou ultérieure), avec exceptions selon les composants  
- **Site** : https://www.gnu.org/software/binutils/

---

### GCC (optionnel)

- **Projet** : GNU Compiler Collection  
- **Usage** : compilation alternative à Clang  
- **Licence** : GNU General Public License v3 (ou ultérieure), avec Runtime Library Exception  
- **Site** : https://gcc.gnu.org/

---

### Apple Xcode / Command Line Tools (macOS)

- **Projet** : Apple Xcode & Command Line Tools  
- **Usage** : toolchain macOS (clang, ld, SDKs)  
- **Licence** : Apple Software License Agreement  
- **Site** : https://developer.apple.com/xcode/

---

## Outils système et utilitaires

### `sha256sum`

- **Projet** : GNU Coreutils  
- **Usage** : génération de checksums  
- **Licence** : GNU General Public License v3 (ou ultérieure)  
- **Site** : https://www.gnu.org/software/coreutils/

---

### `zip`

- **Projet** : Info-ZIP / système  
- **Usage** : empaquetage d’artefacts  
- **Licence** : Info-ZIP License (ou licence système équivalente)  
- **Site** : https://infozip.sourceforge.net/

---

## Plateformes et services

### Git / GitHub

- **Projet** : Git  
- **Usage** : gestion de versions, contributions  
- **Licence** : GNU General Public License v2  
- **Site** : https://git-scm.com/

- **Service** : GitHub  
- **Usage** : hébergement du dépôt, issues, CI, documentation  
- **Conditions** : GitHub Terms of Service  
- **Site** : https://github.com/

---

## Documentation et formats

### Markdown

- **Format** : Markdown  
- **Usage** : documentation Muffin  
- **Licence** : dépend de l’implémentation (CommonMark, GitHub Flavored Markdown, etc.)  
- **Site** : https://commonmark.org/

---

### JSON / SPDX

- **Format** : JSON  
- **Usage** : métadonnées, diagnostics, SBOM  
- **Licence** : standard ouvert

- **Projet** : SPDX  
- **Usage** : Software Bill of Materials (SBOM)  
- **Licence** : Creative Commons Attribution License 3.0  
- **Site** : https://spdx.dev/

---

## Notes importantes

- Muffin **ne redistribue pas** ces outils, sauf mention explicite contraire.
- Chaque outil reste soumis à **sa propre licence**.
- Il incombe à l’utilisateur de vérifier la **compatibilité des licences** dans son contexte (commercial, open-source, CI, etc.).
- Cette liste peut évoluer en fonction des fonctionnalités et de l’environnement d’exécution.

---

## Mises à jour

Cette page est mise à jour lorsque :

- un nouvel outil tiers est recommandé ou documenté
- une dépendance devient optionnelle ou obligatoire
- une information de licence change

---

Pour toute question concernant les licences tierces, consulter les fichiers de licence officiels des projets concernés ou ouvrir une issue sur le dépôt Muffin.
