# Mirrors

# Mirrors

Cette page décrit une stratégie **MAX** de “mirrors” (miroirs) pour Muffin : réplication d’artefacts, caches distribués, registry interne, et fonctionnement cross-platform.

---

## Objectifs

- Accélérer les builds en CI et sur postes (réduire les téléchargements).
- Garantir la disponibilité (éviter SPOF : single point of failure).
- Stabiliser les dépendances (pinning + immutabilité).
- Permettre des environnements isolés (air-gapped / offline).

---

## Périmètre

Miroirs possibles :

- **Store Muffin** (cache content-addressed)
- **Toolchains** (compilateurs, runtimes)
- **Dépendances langages** (crates, nuget, npm, pip, etc.) via proxies
- **Artefacts de build** (binaires, packages)
- **Registry plugins/extensions** (si existant)

Hors-périmètre : secrets (gérés par un vault séparé).

---

## Terminologie

- **Mirror** : copie/replica d’un ensemble d’objets (artefacts) accessible via un protocole.
- **Origin** : source d’autorité (upstream).
- **Proxy cache** : miroir “lazy” qui ne télécharge qu’à la demande.
- **Seed** : pré-remplissage (warmup) du miroir.
- **CAS** : Content-Addressed Storage (hash → contenu).

---

## Types de mirrors

### 1) Mirror de store (CAS)

Le store Muffin en mode `content` est naturellement mirrorable.

- clé : hash du contenu
- objets immuables
- réplication simple : “copier les blobs + l’index”

**Recommandations** :

- stockage objet (S3 compatible) ou filesystem
- index séparé (SQLite/LMDB) + snapshots
- GC contrôlé (ne jamais supprimer un blob référencé)

### 2) Mirror de toolchains

Objectif : neutraliser les variations d’installation.

- compiler/runtimes packagés
- versions pinées
- checksums publiés

Pattern :

- origin : releases officielles
- mirror : artefacts internes (HTTP/S3)
- configure écrit dans `.mff` la version effective

### 3) Mirror de dépendances langages

Utiliser des solutions standard :

- crates : proxy registry
- nuget : feed proxy
- npm : registry proxy
- pip : index proxy

Muffin n’implémente pas forcément ces protocoles, mais peut orchestrer des tools configurés pour pointer vers les mirrors.

### 4) Mirror d’artefacts (build outputs)

Stockage des outputs (packages, installers, tarballs).

- immutabilité recommandée : tagger par hash/version
- métadonnées : target, profile, toolchain

---

## Design recommandé (MAX)

### A) Layout logique

- `store/` : blobs CAS + index
- `toolchains/` : archives + manifests
- `deps/` : proxies (ou caches tool-specific)
- `artifacts/` : outputs publiés
- `meta/` : politiques, ACL, logs

### B) Identité d’objet

Toujours attacher :

- hash (content)
- target
- profile
- toolchain id/version

### C) Politique d’immutabilité

- un objet publié ne change jamais
- si changement : nouvelle version/hash

### D) Vérification d’intégrité

- checksum à chaque fetch
- signature optionnelle (selon env)

---

## Protocoles

### Filesystem (LAN)

- NFS/SMB
- simple à déployer, mais attention concurrence/locking

### HTTP(S)

- cacheable
- compatible CI

### S3 compatible

- très bon fit pour CAS
- versioning possible

---

## Configuration côté Muffin

### Variables et overrides

Pattern :

- `-D mirror.store=<url>`
- `-D mirror.toolchains=<url>`
- `-D mirror.deps=<url>`

### Switch (mapping CLI)

Exemple (concept) :

```text
switch
  flag "--offline" set mirror.mode "offline"
  flag "--mirror-internal" set mirror.store "https://mirror.local/store"
.end
```

### Résolution

- en mode online : origin + fallback mirrors
- en mode offline : mirrors only (error si missing)

---

## CI : stratégie de mirrors

### 1) Warm caches

- pré-remplir le store (seed)
- pré-télécharger toolchains (toolchain cache)

### 2) Matrices targets

Pour éviter les collisions :

- namespace par target
- artefacts taggés par hash

### 3) Politique de purge

- LRU sur index
- caps taille/âge
- exceptions : releases, tags

---

## Poste développeur

- mirror local (filesystem) possible
- fallback sur mirror distant
- mode offline : builds reproductibles si mirror seedé

---

## Sécurité

- ACL en lecture seule pour la majorité
- écriture réservée (CI release)
- signatures/checksums
- audit logs

---

## Dépannage

### Symptômes

- builds lents : mirror cold / cache miss
- erreurs hash : corruption / mismatch toolchain
- 404 mirror : index périmé

### Actions

- exécuter `muffin doctor --tools`
- inspecter `muffin decompile Muffinconfig.mff` (toolchains pinées)
- vérifier export `muffin graph --format json`

---

## Checklist

- [ ] store en mode `content`
- [ ] index snapshotable
- [ ] toolchains packagées + checksums
- [ ] mirrors HTTP/S3 disponibles
- [ ] politique offline
- [ ] GC contrôlé
- [ ] ACL + audit