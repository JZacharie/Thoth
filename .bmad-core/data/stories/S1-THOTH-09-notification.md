## Story: Feedback Utilisateur (Notification Toast)

**ID**: S1-THOTH-09
**Épic**: EPIC-THOTH-01
**Points**: 2
**Statut**: DONE

---

### User Story

**As a** Utilisateur
**I want** voir une notification Windows discrète après chaque traduction
**So that** je sache si la traduction a réussi ou échoué sans deviner

---

### Acceptance Criteria

- [ ] Given la traduction réussit, when le collage est effectué, alors une notification toast "✓ Traduit" s'affiche brièvement
- [ ] Given Pylos est injoignable, when la requête échoue, alors une notification "✗ Pylos introuvable" s'affiche
- [ ] Given le texte sélectionné est vide, when le hotkey est pressé, alors aucune notification n'est affichée
- [ ] Given le presse-papier est vide après Ctrl+C, when la copie échoue, alors une notification "✗ Impossible de copier le texte" s'affiche
- [ ] La notification disparaît automatiquement après 2 secondes

---

### Technical Notes

**Fichier**: `src/notification.rs`

**Approche**: Utiliser `winapi` ou la crate `notify-rust` pour les notifications toast Windows.

```rust
pub fn notify_success() {
    show_toast("Thoth", "✓ Texte traduit avec succès");
}

pub fn notify_error(context: &str) {
    show_toast("Thoth", &format!("✗ {}", context));
}
```

**Dépendance**: `notify-rust = "4"` (crate Rust pour notifications de bureau, compatible Windows toast)

**Types d'erreurs**:
- `pylos_unreachable` — Pylos ne répond pas
- `clipboard_error` — Échec de copie/coller
- `timeout` — La requête a dépassé le timeout
- `empty_selection` — Aucun texte sélectionné

---

### Definition of Done

- [ ] Notifications toast pour succès et erreurs
- [ ] Pas de notification pour les skips silencieux (texte vide)
- [ ] Disparition automatique après 2s
- [ ] `cargo clippy` et `cargo test` passent
