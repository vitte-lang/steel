---
title: "Privacy Policy"
slug: "/legal/privacy"
description: "Politique de confidentialité du projet Muffin."
toc: true
---

# Privacy Policy

Cette page décrit la **politique de confidentialité** du projet **Muffin**.

Dernière mise à jour : 2026-01-08

---

## 1. Collecte de données

Le projet **Muffin** :

- ne collecte **aucune donnée personnelle**
- n’intègre aucun système de tracking
- n’embarque aucun service d’analytics
- n’effectue aucune télémétrie automatique

L’utilisation de Muffin (CLI, moteur de build, documentation) est **entièrement locale** par défaut.

---

## 2. Données locales

Muffin peut créer et manipuler des **données locales** sur la machine de l’utilisateur, notamment :

- fichiers de build (`.vo`, `.va`, `.exe`, etc.)
- caches locaux (`Src/out/.cache`)
- fichiers temporaires (ex: `.rsp`, artefacts intermédiaires)
- journaux d’exécution (logs)

Ces données :

- restent **exclusivement sur la machine de l’utilisateur**
- ne sont jamais transmises automatiquement à un tiers

---

## 3. Réseau et cache distant

Certaines fonctionnalités **optionnelles** peuvent impliquer des accès réseau, par exemple :

- cache distant (HTTP, objet, etc.)
- récupération de dépendances depuis un registry
- outils externes invoqués par l’utilisateur

Ces fonctionnalités :

- sont **désactivées par défaut**
- doivent être **explicitement configurées** par l’utilisateur
- dépendent de services tiers soumis à leurs propres politiques de confidentialité

Muffin ne contrôle pas et n’est pas responsable du comportement de ces services tiers.

---

## 4. Documentation et site web

Le site de documentation Muffin (le cas échéant) :

- ne déploie aucun cookie obligatoire
- n’utilise aucun tracker
- ne collecte aucune statistique utilisateur par défaut

Si le site est hébergé via une plateforme tierce (GitHub Pages, etc.), la politique de confidentialité de l’hébergeur s’applique.

---

## 5. Contributions

En contribuant au projet Muffin (code, documentation, issues) :

- les informations que vous fournissez volontairement (nom d’utilisateur, contenu des contributions) sont publiques
- ces données sont hébergées et gérées par la plateforme utilisée (ex: GitHub)

Muffin ne traite ni ne stocke directement ces données.

---

## 6. Sécurité

Muffin est conçu pour :

- limiter les accès réseau
- favoriser des builds déterministes et reproductibles
- permettre l’isolation via sandbox (si configurée)

Cependant, aucun logiciel ne peut garantir une sécurité absolue.  
L’utilisateur reste responsable de l’environnement dans lequel Muffin est exécuté.

---

## 7. Modifications de la politique

Cette politique de confidentialité peut évoluer :

- en cas d’ajout de fonctionnalités réseau
- en cas de changement d’hébergement
- pour des raisons légales ou techniques

Toute modification significative sera documentée dans cette page.

---

## 8. Contact

Pour toute question relative à la confidentialité :

- utiliser les issues du dépôt du projet
- contacter les mainteneurs via les canaux officiels

---

Cette politique vise à être **simple, transparente et minimale**.  
Muffin respecte la vie privée par conception.
