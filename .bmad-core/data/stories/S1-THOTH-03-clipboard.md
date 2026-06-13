## Story: Clipboard Management (Copier/Coller)

**ID**: S1-THOTH-03
**Épic**: EPIC-THOTH-01
**Points**: 3
**Statut**: DONE

---

### User Story

**As a** Utilisateur
**I want** que Thoth copie automatiquement le texte sélectionné et colle le résultat
**So that** je n'aie pas à faire Ctrl+C/Ctrl+V manuellement

---

### Acceptance Criteria

- [ ] Given le hotkey est déclenché, when l'utilisateur a du texte sélectionné, alors Thoth copie ce texte via `Ctrl+C` simulé
- [ ] Given le texte est copié, when Thoth le lit depuis le presse-papier, alors le texte original est récupéré
- [ ] Given la réponse Pylos est reçue, when Thoth écrit dans le presse-papier, alors le nouveau texte remplace l'ancien
- [ ] Given le nouveau texte est dans le presse-papier, when Thoth simule `Ctrl+V`, alors le texte collé remplace la sélection originale
- [ ] Given l'opération terminée, when Thoth restaure le presse-papier original, alors le contenu précédent est préservé (optionnel — nice to have)

---

### Technical Notes

**Fichier**: `src/clipboard.rs`

**Fonctions**:
```rust
use arboard::Clipboard;

/// Simule Ctrl+C et récupère le texte du presse-papier
pub fn copy_selected_text() -> anyhow::Result<String>

/// Écrit le texte dans le presse-papier et simule Ctrl+V
pub fn paste_text(text: &str) -> anyhow::Result<()>

/// Simule une frappe clavier (Ctrl+C, Ctrl+V)
fn simulate_key_combo(key: &[VirtualKey]) -> anyhow::Result<()>
```

**Dépendances**:
- `arboard = "3"` — accès au presse-papier Windows
- `rdev` — simulation de touches `Ctrl+C` / `Ctrl+V`

**Simulation de touches**:
```rust
rdev::simulate(&rdev::EventType::KeyPress(Key::ControlLeft)).ok();
rdev::simulate(&rdev::EventType::KeyPress(Key::KeyC)).ok();
rdev::simulate(&rdev::EventType::KeyRelease(Key::KeyC)).ok();
rdev::simulate(&rdev::EventType::KeyRelease(Key::ControlLeft)).ok();
```

**Timing**: Un petit délai (`tokio::time::sleep`) est nécessaire entre la simulation des touches et la lecture du presse-papier pour laisser Windows se synchroniser.

---

### Definition of Done

- [ ] Copie automatique du texte sélectionné
- [ ] Injection du texte traduit dans le presse-papier
- [ ] Simulation Ctrl+V pour coller le résultat
- [ ] Gestion des timeouts (si le presse-papier est verrouillé)
- [ ] `cargo clippy` et `cargo test` passent
