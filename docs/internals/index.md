---
title: "Internals"
slug: "/internals"
description: "Architecture interne de Muffin : modèle d’exécution, résolution, cache, jobs, et intégration toolchain."
toc: true
sidebar:
  order: 60
  label: "Internals"
---

# Internals

Cette section décrit **comment Muffin fonctionne en interne**.  
Elle s’adresse aux mainteneurs, contributeurs, et à toute personne souhaitant comprendre le moteur d’exécution au-delà de l’usage quotidien.

Si tu veux **utiliser** Muffin → voir **/how-to** et **/examples**.  
Si tu veux **étendre ou modifier** Muffin → tu es au bon endroit.

---

## Vue d’ensemble

Muffin est conçu comme un **moteur de build déterministe** orienté *graphes de jobs* :

1. Chargement et parsing des fichiers `.muf`
2. Résolution (targets, profiles, variables, imports)
3. Construction d’un graphe de jobs
4. Calcul des clés de cache
5. Exécution (ou récupération depuis le cache)
6. Émission des artefacts et diagnostics

---

## Pipeline interne

### 1) Parsing MUF

- Lexer / parser MUF
- Structure en blocs (`project`, `targets`, `bake`, `cmd`, etc.)
- Validation syntaxique minimale
- Tous les blocs doivent se terminer par `.end`

Résultat :
- AST MUF normalisé
- Aucune logique métier appliquée à ce stade

---

### 2) Résolution

La phase de résolution transforme l’AST en un **modèle exécutable**.

Elle inclut :

- Sélection de la `target` active
- Sélection du `profile`
- Résolution des `use` / imports
- Expansion des variables (`{vars.x}`, `{project.dirs.*}`)
- Évaluation des conditions (`when os == "linux"`)

Sortie :
- Configuration résolue, sans références symboliques restantes

---

### 3) Modèle de jobs

Chaque `bake` est transformé en **job(s)**.

Un job possède :

- Inputs (`takes`)
- Outputs (`emits`)
- Action (`do`)
- Dépendances (`depends`)
- Environnement (target, profile, env)

Les jobs sont reliés dans un **graphe acyclique (DAG)**.

---

## Cache et déterminisme

### Clé de cache

La clé de cache est calculée à partir de :

- Contenu des fichiers `takes`
- Définition du job (`do`, outils, flags)
- Target active
- Profile actif
- Variables d’environnement explicitement incluses

Objectif :
- **Même entrée → même sortie**
- Pas de rebuild inutile

---

### Cache local

- Stocké typiquement dans `Src/out/.cache`
- Index par clé de hash
- Contient :
  - artefacts
  - métadonnées
  - logs d’exécution

---

### Cache distant (optionnel)

- HTTP / objet / autre backend
- Même clé que le cache local
- Priorité :
  1. cache local
  2. cache distant
  3. exécution réelle

---

## Exécution des jobs

### Ordonnancement

- Topological sort du DAG
- Exécution parallèle possible si jobs indépendants
- Respect strict des dépendances

### Sandbox (optionnelle)

Un job peut être exécuté dans une sandbox contrôlée :

- Accès FS (read / write)
- Accès réseau
- Variables d’environnement
- Accès au temps

But :
- Reproductibilité
- Sécurité
- Builds CI fiables

---

## Toolchain

### Abstraction outil

Les outils (`clang`, `ar`, `ld`, `vittec`, etc.) sont décrits via :

- Nom logique
- Binaire réel
- Templates `.rsp.tmpl` (optionnel)

Les arguments sont générés **avant exécution**, jamais concaténés à la volée.

---

### Response files (`.rsp`)

Pourquoi :

- Limites de longueur de ligne (Windows)
- Traçabilité des flags
- Reproductibilité

Pipeline :

1. Expansion du template `.rsp.tmpl`
2. Écriture dans le cache
3. Invocation de l’outil avec `@file.rsp`

---

## Diagnostics

Chaque phase peut produire des diagnostics :

- Erreurs (bloquantes)
- Warnings
- Notes

Un diagnostic contient :

- Type
- Message
- Localisation (fichier / ligne / colonne si applicable)
- Contexte (target, profile, job)

Les diagnostics sont **structurés**, pas de simples strings.

---

## Modules (`mod.muf`)

Un module est une **unité packagée** :

- Identité (nom, version, namespace)
- Sources appartenant au module
- Artefacts produits
- Dépendances

Le moteur traite un module comme :

- une source de jobs
- une source d’artefacts
- une unité cacheable

---

## CLI et moteur

Le CLI (`build muffin …`) est **fin et déclaratif**.

Il se contente de :

- Charger le workspace
- Appliquer overrides (`--target`, `--profile`)
- Lancer le moteur
- Afficher diagnostics et progression

Toute la logique est dans le moteur, pas dans le CLI.

---

## Points clés de design

- **Déclaratif** : MUF décrit *quoi faire*, pas *comment*
- **Déterministe** : même entrée → même sortie
- **Composable** : targets, profiles, bakes réutilisables
- **Lisible** : priorité à la clarté plutôt qu’à la magie
- **Toolchain-agnostic** : clang/gcc/ld/lld interchangeables

---

## Voir aussi

- **/api** — structures internes et types
- **/reference** — syntaxe MUF complète
- **/how-to** — guides pratiques
- **/examples** — patterns concrets

---

Cette section documente le **contrat interne** de Muffin.  
Toute évolution majeure de l’architecture doit être reflétée ici.
