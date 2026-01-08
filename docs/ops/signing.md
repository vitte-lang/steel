# Signing

# Signing

Cette page décrit une stratégie **MAX** de signature et vérification pour Muffin : signatures de releases, checksums, provenance, et pratiques multi-plateformes.

---

## Objectifs

- Garantir l’intégrité des artefacts distribués (binaires, packages, `.mff`, SBOM).
- Permettre une vérification simple (offline possible).
- Standardiser la publication : `SHA256SUMS` + signature.
- Réduire les risques supply-chain (provenance, CI durcie).

---

## Ce qui doit être signé

### Artefacts de release

- archives : `.tar.gz`, `.zip`
- binaires : `muffin`, `muffin.exe`
- packages natifs (si présents) : `.deb`, `.rpm`, `.pkg`, `.msi`
- SBOM : `.sbom.*.json`

### Artefacts de build

Optionnel (selon politique) :

- `Muffinconfig.mff` (contrat de build)
- index/cache snapshots

---

## Modèle recommandé

### 1) Checksums

- générer un fichier `SHA256SUMS` contenant **tous** les artefacts.

Exemple (format classique) :

```text
<sha256>  muffin-1.2.3-x86_64-unknown-linux-gnu.tar.gz
<sha256>  muffin-1.2.3-x86_64-pc-windows-msvc.zip
<sha256>  muffin-1.2.3-x86_64-unknown-linux-gnu.sbom.cdx.json
```

### 2) Signature du manifest

Signer `SHA256SUMS` plutôt que chaque fichier.

- `SHA256SUMS.sig`

---

## Méthodes de signature

### A) GPG

- signature détachée : `SHA256SUMS.asc` ou `SHA256SUMS.sig`
- publier la clé publique (ou fingerprint)

### B) minisign

- très simple, adapté aux binaires

### C) Sigstore

- signatures basées OIDC
- bonne intégration CI

**Recommandation** : proposer 1 méthode principale (GPG ou Sigstore) + 1 alternative simple (minisign).

---

## Vérification (utilisateur)

### Étape 1 — vérifier signature

- vérifier `SHA256SUMS.sig` avec la clé publique.

### Étape 2 — vérifier checksums

- recalculer SHA256 local
- comparer avec `SHA256SUMS`

Exemples (concept) :

```text
# Linux/macOS/BSD/Solaris
sha256sum -c SHA256SUMS

# macOS (alternative)
shasum -a 256 -c SHA256SUMS

# Windows (PowerShell)
Get-FileHash .\muffin.exe -Algorithm SHA256
```

---

## Provenance (recommandé)

### Build provenance

- lier les artefacts à un commit (SHA)
- produire des attestations (si Sigstore)

### Matrice targets

- chaque target produit un set d’artefacts
- tous listés dans `SHA256SUMS`

---

## Gestion des clés

### Clés de release

- générer une clé dédiée “release”
- stocker en HSM ou secret manager
- rotation planifiée

### Accès CI

- limiter l’accès (job release uniquement)
- permissions minimales
- audit logs

---

## CI durcie (checklist)

- [ ] job de release isolé
- [ ] pin versions des actions/outils
- [ ] secrets scellés
- [ ] artefacts immuables
- [ ] génération `SHA256SUMS`
- [ ] signature `SHA256SUMS`
- [ ] publication

---

## Notes cross-platform

- Linux/BSD/Solaris : `sha256sum` souvent dispo
- macOS : `shasum -a 256`
- Windows : `Get-FileHash`

---

## Checklist

- [ ] `SHA256SUMS` contient tous les artefacts
- [ ] signature publiée (`.sig/.asc`)
- [ ] clé publique/fingerprint publié
- [ ] instructions de vérification
- [ ] CI release durcie