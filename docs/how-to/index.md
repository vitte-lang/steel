---
title: "How-to"
slug: "/how-to"
description: "Guides pratiques pour accomplir des tâches concrètes avec Muffin."
toc: true
sidebar:
  order: 20
  label: "How-to"
---

# How-to

Cette section regroupe des **guides orientés action**. Chaque page répond à une question précise : *comment faire X avec Muffin*.  
Si tu cherches la théorie ou la référence complète du langage MUF, consulte plutôt **/reference** ou **/api**.

---

## Démarrer

- **Créer un projet minimal**
  - Initialiser un dépôt avec `build.muf` et `mod.muf`
  - Compiler une première cible locale
- **Passer d’un projet minimal à un workspace**
  - Ajouter `discover`
  - Gérer plusieurs dossiers/programmes

---

## Builds et recettes

- **Créer un `build.muf` minimal**
- **Ajouter un `bake`**
  - `takes`, `emits`, `do`
  - Chaîner plusieurs `bake`
- **Générer des fichiers de configuration par dossier**
  - Pattern `.muff` / `.mcf`
- **Créer des commandes CLI**
  - `cmd build`, `cmd clean`, alias personnalisés

---

## Cibles (targets)

- **Utiliser la cible locale (host)**
- **Ajouter une cible Linux x86_64**
- **Ajouter une cible macOS aarch64**
- **Sélectionner une cible au build**
  - `--target`, `--profile`

---

## Profils et options

- **Créer des profils debug / release**
- **Partager des flags entre profils**
- **Activer/désactiver LTO**
- **Ajouter des defines conditionnelles**

---

## Toolchain

- **Configurer clang / ar / ld**
- **Utiliser des fichiers response (`.rsp`)**
- **Résoudre les problèmes de ligne de commande trop longue (Windows)**
- **Forcer un toolchain spécifique (CI / cross-compile)**

---

## Cache et performances

- **Activer le cache**
- **Comprendre les clés de cache**
- **Inclure des variables d’environnement dans le cache**
- **Réduire les cache misses**

---

## Organisation de projet

- **Structurer `Src/in` et `Src/out`**
- **Nommer les artefacts (`.va`, `.vo`, `.exe`)**
- **Séparer sources, objets et bibliothèques**
- **Nettoyer correctement (`clean`)**

---

## Modules et packaging

- **Créer un `mod.muf`**
- **Définir les sources d’un module**
- **Produire des artefacts packagés (zip)**
- **Préparer un module pour un registry**

---

## Dépannage

- **Le build ne produit rien**
- **Un outil n’est pas trouvé**
- **Erreur de parsing MUF**
- **Conflits entre target et profile**
- **Différences host vs CI**

---

## Voir aussi

- **/examples** — exemples prêts à l’emploi  
- **/reference** — référence du langage MUF  
- **/api** — API interne et modèle d’exécution  
- **/toolchain** — cibles, rsp, cross-compilation  

---

Ces guides sont conçus pour être **courts, concrets et copiables**.  
Si un how-to manque, ajoute-le ici : chaque nouvelle tâche mérite sa page dédiée.
