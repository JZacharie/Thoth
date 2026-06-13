## Story: Tests d'Acceptance E2E Simulés

**ID**: S1-THOTH-12
**Épic**: EPIC-THOTH-01
**Points**: 3
**Statut**: DONE

---

### User Story

**As a** Developer
**I want** des tests automatisés qui simulent le cycle complet hotkey → copie → Pylos → collage
**So that** je puisse valider le comportement sans matériel Windows réel ni appui clavier physique

---

### Acceptance Criteria

- [ ] Given un mock Pylos, when `Win + N` est simulé, alors le cycle complet s'exécute dans un test
- [ ] Given le mock retourne un texte traduit, when le test vérifie le presse-papier, alors le texte collé est le texte mocké
- [ ] Given le mock est injoignable, when la requête échoue, alors le test vérifie que le presse-papier original est restauré
- [ ] Given le texte sélectionné est vide, when le hotkey est simulé, alors rien n'est envoyé à Pylos
- [ ] Tous les tests s'exécutent dans `cargo test --workspace` sans accès réseau

---

### Technical Notes

**Fichier**: `tests/e2e.rs`

**Approche** : Tester l'orchestrateur avec des mocks pour chaque dépendance externe.

```rust
// Mock Hotkey: un canal au lieu du vrai hook clavier
struct MockHotkeySender {
    tx: mpsc::Sender<()>,
}

impl MockHotkeySender {
    fn press(&self) {
        self.tx.try_send(()).ok();
    }
}

// Mock Clipboard: un String partagé au lieu du vrai presse-papier
struct MockClipboard {
    content: Arc<Mutex<String>>,
    previous: Option<String>,
}

// Mock Pylos: un serveur HTTP local (axum ou wiremock)
struct MockPylos {
    server: MockServer,
}
```

**Crates de test**:
- `wiremock = "0.6"` — mock HTTP server pour Pylos
- `tokio::test` — runtime async pour les tests

**Scénarios de test**:
| Test | Description |
|---|---|
| `e2e_happy_path` | Cycle complet → traduction réussie |
| `e2e_pylos_down` | Pylos refuse → notification erreur, presse-papier restauré |
| `e2e_empty_selection` | Texte vide → skip silencieux |
| `e2e_long_text` | Texte > 10k caractères → gestion correcte |
| `e2e_clipboard_restore` | Vérifie que le presse-papier original est restauré |
| `e2e_config_reload` | Hot-reload de config → nouveau modèle utilisé |

---

### Definition of Done

- [ ] Tests E2E avec mock hotkey, mock clipboard, mock Pylos
- [ ] Scénarios : succès, erreur réseau, texte vide, texte long
- [ ] 100% des scénarios s'exécutent sans environnement Windows réel
- [ ] `cargo test --workspace` inclut ces tests
- [ ] `cargo clippy` passe
