# Sprint 1 — Thoth

**Période**: 2026-06-14 → 2026-06-28
**Objectif**: Développement initial de l'application Thoth

---

## Sprint Goal

Livrer la première version fonctionnelle de Thoth : un assistant Windows capable de traduire/corriger le texte sélectionné via `Win + N` en utilisant la passerelle Pylos.

---

## Stories

| ID | Titre | Points | Statut | Épic |
|---|---|---|---|---|
| S1-THOTH-01 | Scaffolding du projet Rust | 2 | DONE | EPIC-THOTH-01 |
| S1-THOTH-02 | Global hotkey Win+N | 5 | DONE | EPIC-THOTH-01 |
| S1-THOTH-03 | Clipboard management (copier/coller) | 3 | DONE | EPIC-THOTH-01 |
| S1-THOTH-04 | Client HTTP Pylos | 3 | DONE | EPIC-THOTH-01 |
| S1-THOTH-05 | Orchestrateur principal | 5 | DONE | EPIC-THOTH-01 |
| S1-THOTH-06 | System tray & background service | 3 | DONE | EPIC-THOTH-01 |
| S1-THOTH-07 | Packaging Windows | 2 | DONE | EPIC-THOTH-01 |
| S1-THOTH-08 | CI/CD pipeline GitHub Actions | 3 | DONE | EPIC-THOTH-01 |
| S1-THOTH-09 | Feedback utilisateur (notification toast) | 2 | DONE | EPIC-THOTH-01 |
| S1-THOTH-10 | Préservation presse-papier original | 1 | DONE | EPIC-THOTH-01 |
| S1-THOTH-11 | Configuration hot-reload | 3 | DONE | EPIC-THOTH-01 |
| S1-THOTH-12 | Tests d'acceptance E2E simulés | 3 | DONE | EPIC-THOTH-01 |

**Total**: 35 points

---

## Ordre de réalisation recommandé

```
S1-THOTH-01 (scaffolding)
    ↓
S1-THOTH-02 (hotkey) ──────┐
S1-THOTH-03 (clipboard) ────┤
S1-THOTH-04 (pylos client) ─┤
    ↓                        │
S1-THOTH-05 (orchestrator) ←┘
    ↓
S1-THOTH-06 (system tray)
    ↓
S1-THOTH-07 (packaging)
    ↓
S1-THOTH-08 (CI/CD pipeline)
    ↓
S1-THOTH-09 (notification) ────┐
S1-THOTH-10 (clipboard restore)─┤
S1-THOTH-11 (hot-reload config)─┤─── en parallèle
    ↓                           │
S1-THOTH-12 (E2E tests) ←───────┘
```

Les stories 02, 03, 04, 09, 10 et 11 peuvent être développées en parallèle.

---

## Définition de Prêt (Definition of Ready)

- [x] User stories écrites avec critères d'acceptation
- [x] Notes techniques identifiant les fichiers/modules impactés
- [x] Stories estimées en points
- [ ] Aucune dépendance externe bloquante

## Définition de Fini (Definition of Done)

- [ ] Tous les critères d'acceptation sont remplis
- [ ] Tests écrits (`cargo test --workspace` passe)
- [ ] `cargo clippy -- -D warnings` passe
- [ ] `cargo fmt --all` appliqué
- [ ] Code revu (PR)
- [ ] Aucune régression

---

## Bloqueurs

- `rdev` doit fonctionner sous Windows pour l'interception du hotkey — à valider
- Pylos doit être accessible sur `localhost:3000` — documenter le prérequis

## Notes

- Développer et tester sous Windows (le projet ne fonctionne pas sous Linux/macOS)
- Utiliser `tracing` pour le logging, avec sortie fichier vers `%APPDATA%/thoth/logs/`
- Version initiale : configuration hardcodée, sera externalisée dans S1-THOTH-11
- **Perf budget** : binaire < 5MB, latence cycle < 2s, mémoire < 50MB
- **QA checklist** (validation manuelle) : tester avec Chrome, Notepad, VS Code, Teams, Word
- **Rétrospective** prévue en fin de sprint (story à 0 points)
- Ajouter `tracing-error` pour capturer les backtraces structurées
- Hook panic qui écrit un crash report dans `%APPDATA%/thoth/crash-reports/`
