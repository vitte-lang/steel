# Localization

Ce document décrit une stratégie **MAX** d’internationalisation/localisation (i18n/l10n) pour Muffin : CLI, diagnostics, manpages, manuel, et messages machine-friendly.

---

## Objectifs

- CLI utilisable dans plusieurs langues sans casser les scripts.
- Diagnostics lisibles par humains **et** par machines.
- Documentation localisable sans dupliquer la source de vérité.
- Portabilité (Linux/macOS/Windows/BSD/Solaris) et compat encodage.

---

## Périmètre

- CLI : commandes, help, erreurs, warnings
- Diagnostics : parser/validator/runner
- Logs : sorties humaines + sorties JSON
- Documentation : `docs/` (manpages + manual)

Hors-périmètre (si non impl) : UI graphique.

---

## Principes

### 1) Séparer les identifiants et les phrases

Chaque message doit avoir :

- un **code stable** (ID) : `MFxxxx`
- une **phrase localisée** (texte)
- des **paramètres structurés** (placeholders)

Exemple (concept) :

- ID : `MF1001`
- clé : `parser.unexpected_token`
- params : `{ expected: ["ident"], got: "RBRACE", span: ... }`

### 2) La sortie machine-friendly est indépendante de la langue

- Les scripts/CI doivent consommer des champs structurés, pas des phrases.
- Les phrases localisées sont un “rendering” du diagnostic.

### 3) UTF-8 partout

- sources `.muf/.muff` : UTF-8 recommandé
- docs : UTF-8
- Windows : gérer console/code page (best-effort)

---

## Niveaux de sortie

### Niveau A — Texte (humain)

- localisé
- orienté action
- inclut contexte (span, fichier, ligne/col)

### Niveau B — JSON (machine)

- stable
- non dépendant de la langue
- inclut : ID, catégorie, sévérité, span, params

Exemple (concept) :

```json
{
  "id": "MF1001",
  "severity": "error",
  "category": "parser",
  "message_key": "parser.unexpected_token",
  "locale": "fr-FR",
  "rendered": "Token inattendu : attendu ident, reçu '}'",
  "params": {
    "expected": ["ident"],
    "got": "RBRACE"
  },
  "span": {
    "file": "build.muf",
    "line": 12,
    "col": 8,
    "len": 1
  }
}
```

---

## Choix de langue (locale)

### 1) Règles de sélection

Priorité recommandée :

1. `--lang <tag>` / `--locale <tag>` (flag explicite)
2. variable d’environnement `MUFFIN_LANG` / `LANG` / `LC_ALL`
3. fallback : `en-US`

### 2) Tag BCP 47

- utiliser des tags type `fr-FR`, `en-US`, `de-DE`.

---

## CLI : stabilité et localisation

### Help / usage

- Le texte de help peut être localisé.
- Les **noms de commandes et flags** restent en ASCII et stables.

Exemples :

- `muffin build`, `muffin decompile`, `muffin doctor` restent identiques.
- `--format json`, `--strict`, `-j` restent identiques.

### Messages d’erreur

- Le message localisé est rendu depuis un diagnostic structuré.
- Les IDs (`MFxxxx`) sont affichés.

---

## Diagnostics : design recommandé

### Structure

- `id` : `MFxxxx`
- `severity` : `error|warning|note|help`
- `category` : `lexer|parser|schema|resolve|runner|io|mff`
- `span` : file + range + line/col
- `params` : map de valeurs
- `hints` : suggestions localisables

### Rendering

- un renderer `text` (localisé)
- un renderer `json` (stable)

---

## Documentation

### Stratégie

- garder une seule source de vérité (FR ou EN) pour le **contrat**.
- générer/traduire ensuite si nécessaire.

Recommandation :

- Manpages : source `en-US` (contrat CLI stable) + traduction FR (optionnelle)
- Manuel : FR principal + sections techniques invariantes

### Conventions de fichiers (proposition)

- `docs/en/` et `docs/fr/`
- ou suffixes : `muffin.1.en.md`, `muffin.1.fr.md`

Le choix dépend du générateur doc.

---

## Plurals, genres, formats

- utiliser ICU MessageFormat si besoin (pluriels)
- sinon placeholders simples (MVP)

Formats :

- nombres : culture-invariant en JSON
- temps/durations : ISO 8601 en JSON

---

## Tests

### Golden tests

- golden JSON (stable)
- golden texte par locale (tolérance moindre)

### Cas à couvrir

- tokens/unicode
- chemins Windows (backslashes)
- encodage console Windows
- messages multi-lignes (snippets)

---

## Checklist implémentation

- [ ] `--locale/--lang` + `MUFFIN_LANG`
- [ ] IDs `MFxxxx` uniques
- [ ] renderer JSON stable
- [ ] renderer texte localisé
- [ ] tables de traduction
- [ ] tests golden
- [ ] docs : stratégie de duplication minimale

---

## Notes

- La localisation doit éviter de casser les scripts : privilégier JSON.
- Les messages restent actionnables via IDs et categories, même sans traduction.
