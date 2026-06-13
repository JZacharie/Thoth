# Epic: Polissage & Finitions (Faible Priorité)

**ID**: EPIC-THOTH-03
**Statut**: DONE
**Priorité**: Faible

## Description

Dernières finitions pour une expérience utilisateur premium : hotkey configurable,
mode silencieux, statistiques locales, et documentation architecturale.

## Objectifs

1. **Hotkey configurable** — Pas seulement Win+N, n'importe quelle combinaison
2. **Mode silencieux** — Toggle on/off depuis la system tray
3. **Statistiques** — Compteurs d'utilisation locaux
4. **ADR** — Documentation du choix du framework hotkey

## Stories

| ID | Titre | Points | Statut |
|---|---|---|---|
| S3-THOTH-01 | Raccourci clavier configurable | 3 | DONE |
| S3-THOTH-02 | Mode silencieux / toggle on/off | 1 | DONE |
| S3-THOTH-03 | Métriques d'utilisation locales | 2 | DONE |
| S3-THOTH-04 | ADR - Choix du framework hotkey | 1 | DONE |

**Total**: 7 points

## Dépendances

- EPIC-THOTH-02 (Sprint 2) terminé

## Définition de Fini (Epic Done)

- [ ] Le hotkey est entièrement configurable
- [ ] Le mode silencieux est accessible depuis la tray
- [ ] Les statistiques sont visibles depuis le menu tray
- [ ] ADR-003 est approuvé
- [ ] `cargo test`, `cargo clippy`, `cargo fmt` passent
