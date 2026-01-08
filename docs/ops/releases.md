# Releases

Cette page décrit un workflow **MAX** de releases pour Muffin : versioning, compatibilité `.mff`, publication multi-plateformes, signatures, changelog, et CI de release.

---

## Objectifs

- Publier des binaires Muffin fiables sur **Linux/macOS/Windows/BSD/Solaris**.
- Gérer la compatibilité du **binaire de compilation** `*.mff` (lecture/écriture).
- Standardiser : tags, artefacts, checksums, signatures, notes de release.
- Permettre des releases reproductibles (best-effort selon toolchains).

---

## Artefacts de release

### 1) Binaire CLI

- `muffin` (Unix)
- `muffin.exe` (Windows)

Optionnel (selon packaging) :

- `Muffin` (wrapper / alias)

### 2) Librairies

Si Muffin expose une lib (Rust) :

- `libmuffin.a` / `libmuffin.so` / `libmuffin.dylib` / `muffin.dll`

### 3) Fichiers de distribution

- `.tar.gz` (Unix)
- `.zip` (Windows)
- packages natifs (optionnel) : `.deb`, `.rpm`, `.pkg`, `.msi`

### 4) Métadonnées

- `SHA256SUMS`
- `SHA256SUMS.sig` (signature)
- `SBOM` (optionnel) : CycloneDX / SPDX

---

## Versioning

### Version du produit

Recommandation : SemVer `MAJOR.MINOR.PATCH`.

- **MAJOR** : rupture de compat CLI / format `.mff`
- **MINOR** : nouvelles features rétro-compatibles
- **PATCH** : correctifs

### Version du format `.mff`

Le `.mff` doit avoir une version propre :

- `mff_format_major`
- `mff_format_minor`

Règles recommandées :

- lecture : support `major` courant et éventuellement N-1
- écriture : écrire le `major` courant
- `minor` : forward-compat possible (fields optionnels)

---

## Compatibilité `.mff`

### Matrice (recommandée)

- Muffin `X` lit : `.mff` `major=X` et `major=X-1` (si policy)
- Muffin `X` écrit : `.mff` `major=X`

### Dépréciations

- annoncer dans release notes
- maintenir une période de transition

### Outils de migration

Commandes (concept) :

- `muffin mff upgrade <in.mff> --out <out.mff>`
- `muffin mff downgrade ...` (optionnel, rarement)

---

## Branching & tags

### Branches

- `main` : développement
- `release/*` (optionnel) : stabilisation

### Tags

- tag produit : `vMAJOR.MINOR.PATCH`
- tag format `.mff` (optionnel) : `mff-vMAJOR.MINOR`

---

## Changelog

### Format

Recommandation : *Keep a Changelog*.

Sections typiques :

- Added
- Changed
- Deprecated
- Removed
- Fixed
- Security

### Scope

- CLI : commandes/options
- Buildfile : syntaxe/validation
- `.mff` : format, compat
- Scheduler/cache/capsule

---

## Build de release (multi-plateformes)

### Targets (exemples)

- Linux x86_64 : `x86_64-unknown-linux-gnu`
- Linux arm64 : `aarch64-unknown-linux-gnu`
- macOS arm64 : `aarch64-apple-darwin`
- macOS x86_64 : `x86_64-apple-darwin` (si support)
- Windows x86_64 MSVC : `x86_64-pc-windows-msvc`
- FreeBSD x86_64 : `x86_64-unknown-freebsd`
- Solaris x86_64 : `x86_64-unknown-solaris`

### Packaging layout

Recommandation :

```text
muffin-<ver>-<target>/
  bin/
    muffin[.exe]
  share/
    docs/
    completions/
  LICENSE
  README
```

---

## Signatures & checksums

### Checksums

- générer `SHA256SUMS` pour tous les artefacts

### Signatures

Options :

- GPG (clé release)
- minisign
- sigstore (OIDC)

Règle : signer `SHA256SUMS`, pas chaque fichier.

---

## SBOM (optionnel)

- générer SBOM par target
- publier avec la release

---

## CI de release (pattern)

### Étapes

1. Vérifier tag + version
2. Construire binaires (matrice targets)
3. Exécuter tests (smoke)
4. Packager (`tar.gz`/`zip`)
5. Générer `SHA256SUMS`
6. Signer
7. Publier (GitHub Releases / registry interne)

### Smoke tests (recommandés)

- `muffin --version`
- `muffin doctor --tools`
- `build muffin` sur un exemple minimal
- `Muffin build` sur un exemple minimal

---

## Politique de support

Recommandation (exemple) :

- Supporter les 2 dernières versions MINOR
- Hotfixes sur la dernière PATCH

---

## Dépannage

- mismatch `.mff` major : utiliser `muffin mff upgrade`
- binaires non reproductibles : toolchains non identiques
- erreurs signature : clés/permissions CI

---

## Checklist release

- [ ] version bump (SemVer)
- [ ] changelog à jour
- [ ] tests OK
- [ ] matrice targets OK
- [ ] packages générés
- [ ] `SHA256SUMS` généré
- [ ] signatures OK
- [ ] notes de release
- [ ] compat `.mff` annoncée
