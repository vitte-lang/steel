# Versioning

# Versioning

Cette page définit une stratégie **complète** de versioning pour Muffin : version du produit, version du buildfile (`*.muf/*.muff`), version du binaire de compilation (`*.mff`), compatibilité, migration, et politiques de support.

---

## 1) Objectifs

- Donner un cadre stable pour :
  - l’évolution de la CLI,
  - l’évolution de la syntaxe buildfile,
  - l’évolution du format `.mff`.
- Permettre une compatibilité maîtrisée (forward/back)
- Réduire la dette de migration
- Documenter les ruptures et les dépréciations

---

## 2) Les 3 couches de versioning

Muffin évolue à trois niveaux distincts :

1. **Version produit** (binaire `muffin`) : `MAJOR.MINOR.PATCH`
2. **Version buildfile** (`muf_version`) : version de la grammaire/constructs
3. **Version `.mff`** (`mff_format`) : version du binaire de compilation

Ces versions ne sont pas identiques : un binaire Muffin peut supporter plusieurs versions buildfile et `.mff`.

---

## 3) Version produit (SemVer)

### 3.1. Schéma

- `MAJOR` : rupture (CLI, formats, comportements contractuels)
- `MINOR` : ajout rétro-compatible
- `PATCH` : correctifs

### 3.2. Règles

- La CLI doit rester stable au sein d’un `MAJOR`.
- Une option/commande supprimée implique un bump `MAJOR` (ou une période de dépréciation avant).
- Les “bugfix” ne doivent pas modifier un contrat observable sans note explicite.

### 3.3. Identifiants de build

Recommandations :

- inclure commit SHA dans `muffin --version` (optionnel)
- exposer un `build_id` dans les exports JSON

---

## 4) Version buildfile (`*.muf/*.muff`)

### 4.1. “Header version”

Le buildfile commence par un header explicite :

```text
muffin bake <int>
```

Ce `<int>` est la **version du langage buildfile** (ou du sous-ensemble “Bakefile”).

### 4.2. Compatibilité buildfile

Règles recommandées :

- Un Muffin `X` doit lire :
  - la version buildfile courante,
  - et idéalement N-1 (si la surface est stable).

- L’écriture/génération de buildfiles par Muffin doit utiliser la version courante.

### 4.3. Dépréciations

- Toute nouvelle construction doit être introduite d’abord comme “optionnelle”.
- Une construction dépréciée doit :
  - émettre un warning (code stable)
  - avoir une date/version de suppression annoncée

### 4.4. Migration buildfile

Commandes (concept) :

```text
muffin muf upgrade build.muf --to 3 --out build.v3.muf
muffin muf format build.muf --in-place
muffin muf lint build.muf --strict
```

Le mode upgrade doit produire un diff minimal et préserver l’intention.

---

## 5) Version `.mff` (binaire de compilation)

### 5.1. Schéma

Le format `.mff` doit inclure :

- `mff_major`
- `mff_minor`
- `mff_patch` (optionnel)

Recommandation : `major/minor` suffisent ; `patch` utile si corrections de sérialisation.

### 5.2. Compatibilité `.mff`

Règles recommandées :

- **Lecture** : supporter `major = current` et éventuellement `major = current-1`
- **Écriture** : écrire `major = current`
- **Minor** :
  - ajouter des champs optionnels
  - ignorer les champs inconnus
  - conserver un ordre stable pour déterminisme

### 5.3. Backward vs forward

- backward-compat : Muffin récent lit `.mff` ancien
- forward-compat : Muffin ancien lit `.mff` récent (limité)

Objectif réaliste : backward-compat forte, forward-compat partielle.

---

## 6) Règles de rupture (breaking changes)

### 6.1. Définition

Une rupture est tout changement qui :

- casse un script CI existant
- casse la lecture d’un buildfile valide (version supportée)
- casse la lecture d’un `.mff` (major supporté)
- modifie l’output final sans modification d’inputs (non déterminisme)

### 6.2. Process

- annoncer dans changelog
- fournir un chemin de migration
- garder une période de dépréciation (si possible)

---

## 7) Politiques de support

### 7.1. Support produit

Exemple de policy :

- Supporter les 2 dernières versions **MINOR** du même `MAJOR`.
- Hotfixes uniquement sur la dernière `PATCH`.

### 7.2. Support `.mff`

- lire `major current` + `major current-1`
- signaler clairement la date de fin de support `major current-1`

### 7.3. Support buildfile

- lire buildfile `bake current` + `bake current-1` (si possible)

---

## 8) Migration `.mff`

### 8.1. Upgrade

Commande (concept) :

```text
muffin mff upgrade Muffinconfig.mff --out Muffinconfig.new.mff
```

### 8.2. Downgrade

- rarement recommandé
- uniquement si format strictement compatible

Commande (concept) :

```text
muffin mff downgrade Muffinconfig.mff --to-major 1 --out Muffinconfig.v1.mff
```

### 8.3. Validation

```text
muffin mff check Muffinconfig.mff
muffin decompile Muffinconfig.mff --format json > /dev/null
```

---

## 9) Versioning des schémas internes

Si Muffin sérialise des tables internes (AST/HIR/IR), recommander :

- versionner les schémas “wire format” séparément
- garder des champs optionnels
- éviter de changer l’ordre/déterminisme

---

## 10) Impacts cross-platform

- Les chemins doivent être normalisés dans `.mff`.
- Les différences d’extensions sont gérées via `target`.
- Les policies `capsule` sont best-effort selon OS.

Un bump de version ne doit pas dépendre d’une plateforme unique.

---

## 11) Changelog et communication

- Chaque release doit préciser :
  - version produit
  - version `.mff` supportée (read/write)
  - version buildfile supportée

Exemple de bloc release note :

```text
Compatibility
- Buildfile: bake 2 (read), bake 3 (read/write)
- MFF: major 1 (read), major 2 (read/write)
```

---

## 12) Checklist

- [ ] SemVer produit respecté
- [ ] buildfile header version présent
- [ ] `.mff` versionné (major/minor)
- [ ] matrice compat documentée (read/write)
- [ ] outils migration (`muf upgrade`, `mff upgrade`) 
- [ ] warnings de dépréciation (codes stables)
- [ ] changelog “Compatibility” par release