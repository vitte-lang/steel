# Supply chain

# Supply chain

Cette page décrit une stratégie **MAX** de sécurisation supply-chain pour Muffin : provenance, durcissement CI, dépendances, mirrors, signatures, SBOM, et politiques d’exécution.

---

## Objectifs

- Réduire les risques de compromission (CI, dépendances, releases).
- Rendre les builds auditables (provenance, traçabilité, SBOM).
- Standardiser : signatures, checksums, pinning toolchains.
- Supporter environnements offline/air-gapped via mirrors.

---

## Menaces couvertes (modèle)

- dépendance compromise (crate, npm, nuget, etc.)
- action CI compromise (supply-chain des workflows)
- toolchain compromise (compiler/linker)
- artefact altéré entre build et publication
- exfiltration de secrets en CI
- pollution cache (cache poisoning)

---

## Principes

### 1) Immutabilité

- artefacts publiés immuables (hash + version)
- store content-addressed (`mode content`)

### 2) Pinning

- pin versions des actions CI
- pin toolchains (version + checksum)
- lockfiles des dépendances (Cargo.lock, etc.)

### 3) Vérification systématique

- checksums sur download
- signatures sur release
- scans vulnérabilités

---

## CI hardening

### A) Permissions minimales

- limiter `GITHUB_TOKEN` (read-only par défaut)
- job release isolé

### B) Pinning actions

- utiliser SHA de commit des actions
- éviter `@v1` non pin

### C) Isolation

- runners dédiés pour release
- secrets accessibles uniquement sur release job

### D) Logs et audit

- conserver logs
- attacher provenance aux artefacts

---

## Dépendances

### Rust

- `Cargo.lock` versionné
- `cargo audit` / `cargo deny`

### Autres écosystèmes

- proxys/mirrors standard
- pin versions

---

## Toolchains

- packager et mirror les toolchains
- checksums + signatures
- écrire les versions effectives dans `.mff`

---

## Cache & store

### Prévenir cache poisoning

- store content-addressed
- séparer cache CI / cache dev
- vérifier hash au restore

### GC

- ne jamais supprimer un blob référencé
- snapshots index

---

## Mirrors

- mirrors CAS (store)
- mirrors toolchains
- mode offline

Voir : `docs/ops/mirrors.md`.

---

## Signatures

- `SHA256SUMS` + signature
- option : Sigstore attestations

Voir : `docs/ops/signing.md`.

---

## SBOM

- CycloneDX + SPDX
- publier par target

Voir : `docs/ops/sbom.md`.

---

## Provenance

### Attestations

- lier commit SHA → artefact
- attacher metadata (target/profile/toolchain)

### Reproductibilité

- best-effort selon toolchains

---

## Policies d’exécution (capsule)

- capsule hermétique par défaut
- net deny en build
- allowlists FS/ENV

---

## Checklist “MAX supply-chain”

- [ ] artefacts immuables (hash + version)
- [ ] store `content`
- [ ] actions CI pinées par SHA
- [ ] job release isolé
- [ ] toolchains pinées + mirror
- [ ] `SHA256SUMS` signé
- [ ] SBOM publié
- [ ] scans vulnérabilités (audit/deny)
- [ ] caches séparés + hash verify
- [ ] capsule hermétique (best-effort)