# Linting

# Linting

Ce document décrit une stratégie **MAX** de linting pour le projet Muffin : code (Rust), buildfiles (`.muf/.muff`), binaire de compilation (`.mff`), documentation (`docs/`), et intégration CI.

---

## Objectifs

- Standardiser la qualité (style, warnings, erreurs) sur tout le repo.
- Empêcher les régressions via CI (gates).
- Produire des diagnostics exploitables (format texte + JSON).
- Assurer la portabilité (Linux/macOS/Windows/BSD/Solaris).

---

## Périmètre

### Code

- Rust (`src/`, `crates/` si présents)
- Tests (`tests/`, `fuzz/`, `bench/` si présents)
- Scripts (`scripts/`, `tools/` si présents)

### Formats Muffin

- Buildfiles : `*.muf` / `*.muff`
- Binaire de compilation : `*.mff`

### Documentation

- `docs/**/*.md`

---

## Lint Rust (recommandé)

### 1) rustfmt

Objectif : format déterministe.

```text
cargo fmt --all -- --check
```

### 2) clippy

Objectif : détecter erreurs/patterns dangereux.

```text
cargo clippy --all-targets --all-features -- -D warnings
```

### 3) compilation “warnings as errors”

```text
RUSTFLAGS="-D warnings" cargo check --all-targets --all-features
```

### 4) tests

```text
cargo test --all
```

### 5) audit dépendances (optionnel)

```text
cargo deny check
cargo audit
```

---

## Lint buildfiles (`.muf/.muff`)

### Objectifs

- parsing valide (lexer/parser)
- validation schéma (store/capsule/tool/bake/wire/export/plan/switch)
- cohérence types ports (`in/out`)
- références résolues (`wire` vers ports existants)
- conventions (noms, out_dir, targets)

### Commande (pattern)

Proposer un mode strict :

```text
muffin configure --input <file.muf> --out /dev/null --strict
```

Ou un linter dédié (si impl) :

```text
muffin lint <file.muf> --format json
```

### Règles recommandées

- `store` présent si cache activé
- `capsule` référencée par les `tool` sandboxables
- `export` ne référence que des `out` ports
- `plan default` présent
- `switch` : flags non ambigus
- globs : chemins relatifs stables

---

## Lint `.mff`

### Objectifs

- header/version valide
- schéma compatible (forward/back)
- checks d’intégrité (tables, offsets, index)
- déterminisme (ordre stable)

### Commandes (pattern)

```text
muffin mff check Muffinconfig.mff
muffin decompile Muffinconfig.mff --format json > /dev/null
```

---

## Lint documentation

### 1) markdownlint (Node)

```text
npx markdownlint-cli2 "docs/**/*.md" "README.md"
```

Recommandation : config `markdownlint-cli2.jsonc`.

### 2) liens

Voir : `docs/meta/link-checking.md`.

### 3) style docs (conventions)

- Un seul H1 par fichier
- manpages : sections `NAME/SYNOPSIS/DESCRIPTION/OPTIONS/EXAMPLES/SEE ALSO`
- liens internes en relatif
- pas d’URL brutes dans les manpages (préférer liens relatifs)

---

## Lint cross-platform

### Normalisation chemins

- toujours accepter `/` et `\\` en entrée
- normaliser en format interne stable
- tests sur :
  - Windows drive letters (`C:`)
  - UNC (`\\server\\share`)
  - symlinks (si support)

### Globs

- comportement défini (case sensitivity selon FS)
- tri stable des résultats

### Encodages

- UTF-8 recommandé
- diagnostics : offsets bytes + positions ligne/col

---

## CI (gates)

### Profil CI minimal

Étapes recommandées :

1. `cargo fmt --check`
2. `cargo clippy -D warnings`
3. `cargo test`
4. lint docs (markdownlint)
5. link checking (lychee)
6. lint buildfiles exemples (`docs/examples/**/*.muf`)

### Exemple GitHub Actions (pattern)

```yaml
- name: Format
  run: cargo fmt --all -- --check

- name: Clippy
  run: cargo clippy --all-targets --all-features -- -D warnings

- name: Tests
  run: cargo test --all

- name: Docs lint
  run: npx markdownlint-cli2 "docs/**/*.md" "README.md"

- name: Link check
  uses: lycheeverse/lychee-action@v1
  with:
    args: >-
      --no-progress
      --timeout 20
      --max-concurrency 16
      --accept 200,206,429
      --exclude-mail
      "docs/**/*.md" "README.md"

- name: Lint examples
  run: |
    muffin configure --input docs/examples/muf/minimal/mod.muf --out /dev/null --strict
    muffin configure --input docs/examples/muf/advanced/targets/local.muf --out /dev/null --strict
```

---

## Pré-commit (recommandé)

Pattern local :

```text
cargo fmt --all
cargo clippy --all-targets --all-features
npx markdownlint-cli2 "docs/**/*.md" "README.md"
lychee "docs/**/*.md" "README.md"
```

---

## Checklist avant merge

- [ ] `cargo fmt --check` OK
- [ ] `cargo clippy -D warnings` OK
- [ ] `cargo test` OK
- [ ] docs : markdownlint OK
- [ ] docs : link checking OK
- [ ] exemples `.muf/.muff` valides (`--strict`)
- [ ] pas de régression cross-platform (paths/globs)

---

## Notes

- La CLI exacte peut varier : ce document propose une stratégie et des patterns.
- Si un linter dédié buildfiles est ajouté, le brancher en gate CI.