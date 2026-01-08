# SBOM

# SBOM

Cette page décrit une stratégie **MAX** de génération et publication de SBOM (Software Bill of Materials) pour Muffin : formats, outils, intégration CI, signatures, et consommation en audit.

---

## Objectifs

- Publier une SBOM par release (et idéalement par artefact/target).
- Permettre l’audit : dépendances, licences, vulnérabilités.
- Couvrir : code Muffin, dépendances Rust, toolchains packagées (si distribuées), artefacts.
- Rester compatible multi-plateformes (Linux/macOS/Windows/BSD/Solaris).

---

## Portée

### 1) SBOM du produit Muffin

Inclut :

- binaire `muffin` / `muffin.exe`
- crates Rust (directes/transitives)
- licences

### 2) SBOM des toolchains (optionnel)

Si Muffin distribue des toolchains :

- compiler/runtimes packagés
- hashes/checksums
- licences

### 3) SBOM des artefacts construits (optionnel)

Pour des pipelines packaging :

- packages `.tar.gz` / `.zip` / `.deb` / `.rpm` / `.pkg` / `.msi`
- dépendances runtime si connues

---

## Formats recommandés

### CycloneDX

- JSON ou XML
- bon écosystème (scanners, dashboards)

### SPDX

- SPDX JSON/YAML
- standard très répandu en compliance

**Recommandation** : produire **CycloneDX JSON** + **SPDX JSON** en release.

---

## Identifiants et traçabilité

Chaque SBOM doit inclure :

- nom produit : `muffin`
- version : `vMAJOR.MINOR.PATCH`
- target : `x86_64-unknown-linux-gnu`, etc.
- hash binaire (SHA256)
- build id (commit SHA)
- timestamp

---

## Outils (Rust)

### Cargo SBOM

- `cargo sbom` (selon disponibilité)

### CycloneDX pour Rust

- `cargo cyclonedx`

### SPDX

- conversion/outil dédié (selon stack)

### Scanners vulnérabilités

- `cargo audit`
- `cargo deny`

---

## Pipeline CI (pattern)

### Étapes recommandées

1. Build release (par target)
2. Générer SBOM Rust deps
3. Attacher hash du binaire dans la SBOM
4. Scanner vulnérabilités (audit)
5. Publier SBOM avec release

### Exemples (concept)

CycloneDX (Rust) :

```text
cargo install cargo-cyclonedx
cargo cyclonedx --format json
```

SPDX (concept) :

```text
# selon outil choisi
spdx-sbom-generator -o sbom.spdx.json
```

---

## Publication

Recommandation : publier à côté des artefacts :

- `muffin-<ver>-<target>.sbom.cdx.json`
- `muffin-<ver>-<target>.sbom.spdx.json`

Et lier dans les release notes.

---

## Signatures

- générer `SHA256SUMS`
- signer `SHA256SUMS`
- inclure les SBOM dans `SHA256SUMS`

Optionnel : signer aussi chaque SBOM.

---

## Consommation (audit)

### Cas d’usage

- compliance licences
- audit vulnérabilités
- traçabilité supply-chain

### Commandes typiques

- importer SBOM dans un outil de scan
- vérifier hashes
- comparer versions

---

## Checklist

- [ ] SBOM CycloneDX JSON
- [ ] SBOM SPDX JSON
- [ ] hash binaire inclus
- [ ] commit SHA inclus
- [ ] `SHA256SUMS` inclut SBOM
- [ ] signatures OK
- [ ] scan vulnérabilités (audit/deny)