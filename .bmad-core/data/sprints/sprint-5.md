# Sprint 3 — Thoth V3 (Polissage)

**Période**: 2026-07-12 → 2026-07-26
**Objectif**: Finitions et expérience utilisateur premium

---

## Sprint Goal

Offrir une expérience utilisateur complète et configurable : hotkey au choix,
mode silencieux, statistiques d'utilisation et documentation architecturale.

---

## Stories

| ID | Titre | Points | Statut | Épic |
|---|---|---|---|---|
| S3-THOTH-01 | Raccourci clavier configurable | 3 | TODO | EPIC-THOTH-03 |
| S3-THOTH-02 | Mode silencieux / toggle on/off | 1 | TODO | EPIC-THOTH-03 |
| S3-THOTH-03 | Métriques d'utilisation locales | 2 | TODO | EPIC-THOTH-03 |
| S3-THOTH-04 | ADR - Choix du framework hotkey | 1 | TODO | EPIC-THOTH-03 |

**Total**: 7 points

---

## Ordre de réalisation recommandé

```
S3-THOTH-04 (ADR hotkey) ────┐
S3-THOTH-02 (silent mode) ────┤─── en parallèle
    ↓                          │
S3-THOTH-01 (custom hotkey)  ←┘
    ↓
S3-THOTH-03 (local metrics)
```

## Définition de Prêt

- [x] User stories écrites
- [x] Notes techniques identifiant les fichiers impactés
- [x] Stories estimées

## Bloqueurs

- S3-THOTH-01 dépend de la capacité de `rdev` à désenregistrer/ré-enregistrer des hotkeys dynamiquement

## Notes

- Sprint léger (7 points) — possibilité d'ajouter des stories de dette technique
- S3-THOTH-04 est une story "paper" (documentation uniquement) — 1 point symbolique
- Prévoir une rétrospective de fin d'epic en fin de sprint
