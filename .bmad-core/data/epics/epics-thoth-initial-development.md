# Epic: Développement Initial de Thoth

**ID**: EPIC-THOTH-01
**Statut**: DONE
**Priorité**: Haute

## Description

Développer l'application Thoth, un assistant de traduction et correction instantané pour Windows.
L'applicatif écoute un raccourci clavier global (`Win + N`), copie le texte sélectionné,
l'envoie à la passerelle locale Pylos via l'API OpenAI-compatible, et remplace le texte
par la réponse traduite/corrigée.

## Objectifs

1. **Scaffolding** — Projet Rust, structure, dépendances
2. **Global Hotkey** — Intercepter `Win + N` sous Windows
3. **Clipboard** — Copier/coller automatique avec arboard
4. **Client Pylos** — Requête POST /v1/chat/completions avec reqwest
5. **Prompt système** — Appliquer le prompt strict de traduction/correction
6. **Orchestration** — Enchaînement complet copie → requête → collage
7. **Background service** — Fonctionner en arrière-plan (system tray)
8. **Packaging** — Build release Windows, installateur
9. **Notifications** — Feedback utilisateur via toasts Windows
10. **Préservation clipboard** — Restaurer le presse-papier original
11. **Hot-reload** — Configuration dynamique sans redémarrage
12. **Tests E2E** — Simulation du cycle complet en CI

## Stories

| ID | Titre | Points | Statut |
|---|---|---|---|
| S1-THOTH-01 | Scaffolding du projet Rust | 2 | DONE |
| S1-THOTH-02 | Global hotkey Win+N | 5 | DONE |
| S1-THOTH-03 | Clipboard management (copier/coller) | 3 | DONE |
| S1-THOTH-04 | Client HTTP Pylos | 3 | DONE |
| S1-THOTH-05 | Orchestrateur principal | 5 | DONE |
| S1-THOTH-06 | System tray & background service | 3 | DONE |
| S1-THOTH-07 | Packaging Windows | 2 | DONE |
| S1-THOTH-08 | CI/CD pipeline GitHub Actions | 3 | DONE |
| S1-THOTH-09 | Feedback utilisateur (notification toast) | 2 | DONE |
| S1-THOTH-10 | Préservation presse-papier original | 1 | DONE |
| S1-THOTH-11 | Configuration hot-reload | 3 | DONE |
| S1-THOTH-12 | Tests d'acceptance E2E simulés | 3 | DONE |

## Dépendances

- Pylos gateway en cours d'exécution (port 3000)
- Modèle `gemma4:12b` ou `gemini4:12b` disponible

## Définition de Fini (Epic Done)

- [ ] L'application s'installe et s'exécute sur Windows 10/11
- [ ] `Win + N` déclenche le traitement dans n'importe quel champ de saisie
- [ ] Le texte sélectionné est remplacé par sa traduction/correction
- [ ] Aucun texte superflux (formules de politesse, markdown) n'est collé
- [ ] `cargo test`, `cargo clippy`, `cargo fmt` passent
