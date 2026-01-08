# RFC

# RFC

Index des RFC de Muffin.

Les RFC définissent les **contrats** (syntax, formats, CLI, ops) et documentent les décisions d’architecture.

---

## Workflow

### Statuts

- `draft` : en discussion
- `proposed` : prêt à review
- `accepted` : adopté
- `rejected` : refusé
- `superseded` : remplacé

### Cycle

1. créer `docs/rfc/NNNN-<slug>.md`
2. proposer en PR
3. review + itérations
4. marquer `accepted` et déplacer dans `docs/rfc/accepted/`
5. si remplacé : `superseded` + lien vers la nouvelle RFC

---

## Index

### Accepted

- [RFC 0001 — MUF surface syntax](./accepted/0001-muf-surface-syntax.md)

### Drafts

- (à venir)

---

## Conventions

- format : `NNNN-<slug>.md`
- NNNN : numéro à 4 chiffres (zéro-pad)
- titre : `RFC NNNN — <Title>`
- sections minimales : Summary, Motivation, Goals, Non-goals, Design, Compatibility, Migration, Status

---

## Voir aussi

- Référence MUF : `docs/reference/muf/index.md`
- CLI : `docs/reference/cli/index.md`
- Formats : `docs/reference/formats/index.md`
- Versioning : `docs/ops/versioning.md`