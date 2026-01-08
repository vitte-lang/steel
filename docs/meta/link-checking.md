# Link checking

# Link checking

Ce document décrit une stratégie **MAX** de vérification des liens (internes et externes) pour la documentation Muffin.

---

## Objectifs

- Garantir que la documentation est **navigable** (liens internes corrects).
- Détecter les liens externes cassés ou redirigés.
- Empêcher les régressions via CI (pré-commit + pipeline).
- Uniformiser les conventions de liens (relative paths) dans `docs/`.

---

## Périmètre

À vérifier :

- `docs/man/*.md` (manpages)
- `docs/manual/*.md` (manuel)
- `docs/api/*.md`
- `docs/examples/**/*.md`
- `README.md` (si présent)

Exclusions recommandées :

- fichiers générés (si un dossier `docs/_generated/` existe)
- vendored docs (si importées)

---

## Conventions de liens

### Liens internes

- Utiliser **des chemins relatifs**.
- Préférer des liens directs vers les fichiers `.md`.

Exemples :

- `docs/man/index.md` → `./muffin-build.1.md`
- `docs/manual/00-introduction.md` → `../man/muffin.1.md`

### Liens externes

- Éviter de dépendre d’URL instables.
- Éviter les “raw URLs” dans les manpages ; préférer une référence stable (si disponible).

---

## Vérification locale (recommandée)

### Option A — lychee

`lychee` est un checker rapide et scriptable.

Commande (exemple) :

```text
lychee \
  --no-progress \
  --timeout 20 \
  --max-concurrency 16 \
  --accept 200,206,429 \
  --exclude-mail \
  "docs/**/*.md" "README.md"
```

### Option B — markdown-link-check

Alternative Node.

```text
npx markdown-link-check -q -c .github/markdown-link-check.json docs/**/*.md README.md
```

---

## Configuration (exemples)

### `.lychee.toml` (exemple)

```toml
# Exemple minimal
# Placer à la racine du repo

timeout = 20
max_concurrency = 16
accept = [200, 206, 429]
exclude_mail = true

# Exclusions
exclude_path = [
  "docs/_generated",
]
```

### `markdown-link-check.json` (exemple)

```json
{
  "timeout": "20s",
  "retryOn429": true,
  "aliveStatusCodes": [200, 206, 429],
  "ignorePatterns": [
    {"pattern": "^#"},
    {"pattern": "^mailto:"},
    {"pattern": "^file:"}
  ]
}
```

---

## CI (pattern)

### GitHub Actions (exemple)

Étapes typiques :

1. checkout
2. installer l’outil
3. exécuter le check

Exemple “lychee” :

```yaml
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
```

Exemple “markdown-link-check” :

```yaml
- name: Link check
  run: npx markdown-link-check -q -c .github/markdown-link-check.json docs/**/*.md README.md
```

---

## Règles d’échec / tolérances

- Les liens internes cassés doivent être **bloquants**.
- Les liens externes peuvent être :
  - bloquants si dans `docs/man/` (contrat CLI)
  - warnings ailleurs (selon politique)

Gestion des faux positifs :

- ignorer des domaines connus instables
- accepter des codes `429` (rate limit)
- ajouter retry/backoff en CI

---

## Checklist avant merge

- [ ] Tous les liens internes `docs/` résolus
- [ ] Aucune référence à un fichier renommé
- [ ] Les pages index (`docs/man/index.md`, `docs/meta/index.md`) pointent vers des chemins valides
- [ ] Les exemples mentionnés existent et sont cohérents

---

## Notes

- Les liens relatifs doivent rester valides si le repo est cloné sur Linux/macOS/Windows/BSD/Solaris.
- Pour des docs hors GitHub (sites statiques), vérifier aussi le rendu (anchors générés).