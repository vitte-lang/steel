# FORMATS

# Formats

Référence des **formats** manipulés par Muffin : buildfiles (`*.muf/*.muff`), binaire de compilation (`*.mff`), exports (JSON/NDJSON/DOT), et formats “ops” (checksums, signatures, SBOM).

---

## Portée

Muffin manipule plusieurs familles de formats :

- **Entrées** : buildfiles déclaratifs (`*.muf`, `*.muff`)
- **Contrat** : binaire de compilation (`*.mff`, ex: `Muffinconfig.mff`)
- **Exports** : graph/diagnostics/télémétrie en `text/json/ndjson/dot`
- **Ops** : `SHA256SUMS`, signatures (`.sig/.asc`), SBOM (`CycloneDX/SPDX`)

---

## Index

### Entrées

- [Buildfiles (`.muf/.muff`)](./muf.md)

### Contrat

- [MFF (`.mff`)](./mff.md)

### Exports

- [JSON / NDJSON](./json.md)
- [DOT (graph)](./dot.md)

### Ops

- [Checksums (SHA256SUMS)](./checksums.md)
- [Signatures](./signatures.md)
- [SBOM](./sbom.md)

---

## Conventions communes

### 1) Encodage

- Texte : UTF-8 recommandé.
- JSON/NDJSON : UTF-8 obligatoire.

### 2) Chemins

- Entrée : accepter `/` et `\\`.
- Interne : normalisation stable.
- Sortie (si exposée) : possibilité de redaction/hashing (voir télémétrie).

### 3) Déterminisme

Recommandations :

- tri stable des globs
- sérialisation stable (ordre) pour `.mff`
- hashes stables (content-addressed) lorsque `store mode content`

### 4) Types logiques

Préférer des types logiques (`bin.obj`, `lib.static`, `lib.shared`, `bin.exe`) et résoudre l’extension via `target`.

---

## Matrice cross-platform (extensions usuelles)

- objets : `.o` (Unix), `.obj` (Windows)
- statiques : `.a` (Unix), `.lib` (Windows)
- partagées : `.so` (Linux/BSD/Solaris), `.dylib` (macOS), `.dll` (Windows)
- exécutables : sans extension (Unix), `.exe` (Windows)

---

## Où se trouvent ces formats dans le repo

- CLI (référence) : `docs/reference/cli/`
- Config (référence) : `docs/reference/config/`
- Ops (process) : `docs/ops/`
- Manpages (contrat CLI) : `docs/man/`

---

## Voir aussi

- `muffin configure` : génère `Muffinconfig.mff`
- `muffin build` : exécute depuis `.mff`
- `muffin decompile` : vue normalisée (buildfile ou `.mff`)
- `muffin graph` : export DOT/JSON
- `muffin why` : explication d’invalidation