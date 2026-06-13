# Sprint 2 — Thoth V2

**Période**: 2026-06-28 → 2026-07-12
**Objectif**: Améliorations qualité de vie et industrialisation

---

## Sprint Goal

Rendre Thoth prêt pour une utilisation quotidienne professionnelle : démarrage automatique,
support multilingue, résilience LLM, installateur MSI et signature du binaire.

---

## Stories

| ID | Titre | Points | Statut | Épic |
|---|---|---|---|---|
| S2-THOTH-01 | Démarrage automatique au boot Windows | 2 | TODO | EPIC-THOTH-02 |
| S2-THOTH-02 | Configuration langue cible | 2 | TODO | EPIC-THOTH-02 |
| S2-THOTH-03 | Fallback modèle LLM | 3 | TODO | EPIC-THOTH-02 |
| S2-THOTH-04 | Installateur MSI | 3 | TODO | EPIC-THOTH-02 |
| S2-THOTH-05 | Signature du binaire Windows | 2 | TODO | EPIC-THOTH-02 |

**Total**: 12 points

---

## Ordre de réalisation recommandé

```
S2-THOTH-01 (auto-start) ────────────────┐
S2-THOTH-02 (target language) ───────────┤
S2-THOTH-03 (model fallback) ────────────┤─── en parallèle
S2-THOTH-05 (binary signing) ────────────┘
    ↓
S2-THOTH-04 (MSI installer)
```

## Définition de Prêt

- [x] User stories écrites
- [x] Notes techniques identifiant les fichiers impactés
- [x] Stories estimées

## Bloqueurs

- Certificat Authenticode à acheter (~200-300€/an chez DigiCert ou Sectigo)
- WiX Toolset v4 à installer dans la CI

## Notes

- S2-THOTH-05 nécessite un certificat payant — à commander en début de sprint
- Le fallback (S2-THOTH-03) utilise le même endpoint Pylos, juste un modèle différent
- L'installateur MSI empaquette le binaire signé (S2-THOTH-05 doit précéder S2-THOTH-04)
