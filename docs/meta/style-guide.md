# Docs style guide

# Docs style guide

Guide de style **MAX** pour la documentation Muffin. Objectif : des docs cohérentes, lisibles, navigables, et exploitables (humain + machine).

---

## 1) Principes

- **Clarté** : une idée par paragraphe.
- **Précision** : éviter le flou (“peut-être”, “souvent”) sans qualifier.
- **Traçabilité** : si le comportement dépend de l’implémentation, écrire **"selon impl"**.
- **Stabilité** : les commandes, flags et noms d’artefacts sont stables.
- **Portabilité** : écrire en pensant Linux/macOS/Windows/BSD/Solaris.

---

## 2) Conventions terminologiques (source de vérité)

Toujours utiliser ces termes :

- **Buildfile** : `*.muf` / `*.muff`
- **Binaire de compilation** : `*.mff` (ex: `Muffinconfig.mff`)
- **Configure** : `build muffin` (génère `.mff`)
- **Build** : `Muffin build` (exécute depuis `.mff`)
- **Bake** : nœud du DAG
- **Port** : `in` / `out`
- **Wire** : liaison `out → in`
- **Tool** : exécutable déclaré
- **Capsule** : policy sandbox
- **Store** : cache d’artefacts
- **Export** : cible publique
- **Plan** : scénario d’exécution

Interdits (ou à éviter) :

- “fichier de config texte” pour `.mff`
- mélanger “configure” et “build”
- donner des extensions réelles au lieu des types logiques (sauf section dédiée)

---

## 3) Structure d’un document

### 3.1. Un seul H1

- Chaque fichier `*.md` contient exactement **un** titre `#`.

### 3.2. Sections

- Utiliser `##` puis `###`.
- Éviter la profondeur > 3.

### 3.3. Style manpages

Manpages (`docs/man/*.md`) : sections recommandées

- `NAME`
- `SYNOPSIS`
- `DESCRIPTION`
- `OPTIONS`
- `EXIT STATUS`
- `EXAMPLES`
- `SEE ALSO`

### 3.4. Style manuel

Chapitres `docs/manual/*.md` :

- résumé court
- sections orientées modèle mental
- liens vers manpages/exemples

---

## 4) Règles d’écriture

### 4.1. Voix et ton

- Français technique, direct.
- Pas de marketing.
- Pas de promesses non testables.

### 4.2. Paragraphes

- 2 à 6 lignes max.
- Préférer listes à puces.

### 4.3. Mots “à manier”

- si un comportement est optionnel : écrire “optionnel (selon impl)”
- si un comportement est best-effort : écrire “best-effort (selon OS)”

---

## 5) Code blocks

### 5.1. Langage du bloc

- commandes : `text`
- buildfiles : `muf`
- Rust : `rust`
- JSON : `json`

### 5.2. Longueur

- Éviter > 80 lignes.
- Si nécessaire : déplacer dans `docs/examples/` et lier.

### 5.3. Commandes

- Toujours donner un exemple minimal.
- Puis un exemple avancé.

---

## 6) Liens

### 6.1. Liens internes

- Chemins relatifs.
- Lier vers un fichier (pas vers un dossier) si possible.

### 6.2. Liens externes

- Éviter les URLs brutes dans les manpages.
- Préférer une page d’index/référence.

### 6.3. Stabilité des anchors

- Les titres doivent être uniques par fichier.
- Éviter la ponctuation exotique.

---

## 7) Exemples

### 7.1. Exemples minimaux

- doivent tenir dans un fichier
- pas de placeholders non expliqués
- doivent illustrer un concept précis

### 7.2. Exemples avancés

- peuvent être multi-fichiers
- doivent inclure un index `README.md`
- doivent documenter : inputs/tools/outputs/plan

---

## 8) Cross-platform

### 8.1. Chemins

- Montrer des chemins relatifs (`./out/bin/app`).
- Quand un exemple Windows est nécessaire, utiliser `C:/...` (plus lisible que `C:\\...`).

### 8.2. Extensions

- Préférer les types logiques (`bin.obj`) ; indiquer les extensions seulement dans l’annexe.

### 8.3. Shell

- Éviter les commandes bash-only dans les docs de base.
- Si nécessaire, fournir équivalents : PowerShell + POSIX.

---

## 9) Diagnostics

- Toujours inclure : fichier, ligne/col, message.
- Si dispo : ID (`MFxxxx`).
- Si le diagnostic est structuré : montrer l’exemple JSON.

---

## 10) Checklists

### 10.1. Checklist nouvelle page

- [ ] un H1
- [ ] liens relatifs valides
- [ ] exemples exécutables mentalement
- [ ] terminologie conforme

### 10.2. Checklist nouvelle commande

- [ ] manpage créée/maj
- [ ] entrée dans `docs/man/index.md`
- [ ] exemple minimal ajouté
- [ ] options documentées

---

## 11) Template rapide

```md
# Titre

Résumé.

---

## Section

Contenu.
```