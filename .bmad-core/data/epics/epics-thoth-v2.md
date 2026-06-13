# Epic: Améliorations V2 (Moyenne Priorité)

**ID**: EPIC-THOTH-02
**Statut**: Planifiée

> **Note**: S2-THOTH-05 (binary signing) is BLOCKED — requires a paid Authenticode code signing certificate. The remaining 4 stories in this epic are DONE.
**Priorité**: Moyenne

## Description

Améliorations qualité de vie et industrialisation de Thoth : démarrage automatique,
multilingue, résilience LLM, installation professionnelle et signature du binaire.

## Objectifs

1. **Auto-start** — Thoth se lance au démarrage de Windows
2. **Multilingue** — Support de 10 langues cibles configurables
3. **Résilience** — Fallback automatique entre modèles LLM
4. **Installation pro** — MSI pour installation/désinstallation propre
5. **Confiance** — Signature Authenticode du binaire

## Stories

| ID | Titre | Points | Statut |
|---|---|---|---|
| S2-THOTH-01 | Démarrage automatique au boot Windows | 2 | DONE |
| S2-THOTH-02 | Configuration langue cible | 2 | DONE |
| S2-THOTH-03 | Fallback modèle LLM | 3 | DONE |
| S2-THOTH-04 | Installateur MSI | 3 | DONE |
| S2-THOTH-05 | Signature du binaire Windows | 2 | BLOCKED |

**Total**: 12 points

## Dépendances

- EPIC-THOTH-01 (Sprint 1) terminé
- Certificat Authenticode (à acheter)
- WiX Toolset installé dans la CI

## Définition de Fini (Epic Done)

- [ ] Thoth se lance au démarrage de Windows
- [ ] L'utilisateur peut choisir sa langue cible
- [ ] Le fallback LLM fonctionne sans intervention
- [ ] L'installateur MSI est disponible sur la GitHub Release
- [ ] Le binaire est signé Authenticode
- [ ] `cargo test`, `cargo clippy`, `cargo fmt` passent
