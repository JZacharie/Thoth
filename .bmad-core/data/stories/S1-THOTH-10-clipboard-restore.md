## Story: Préservation du Presse-papier Original

**ID**: S1-THOTH-10
**Épic**: EPIC-THOTH-01
**Points**: 1
**Statut**: DONE

---

### User Story

**As a** Utilisateur
**I want** que le contenu original de mon presse-papier soit restauré après la traduction
**So that** je ne perde pas ce que j'avais copié avant

---

### Acceptance Criteria

- [ ] Given un contenu X dans le presse-papier, when la traduction est déclenchée, alors Thoth sauvegarde X avant de copier le texte sélectionné
- [ ] Given la traduction réussit, when le collage est effectué, alors le presse-papier est restauré avec X
- [ ] Given la traduction échoue, when l'erreur est gérée, alors le presse-papier est restauré avec X
- [ ] Given le presse-papier est vide initialement, when l'opération termine, alors le presse-papier est vidé

---

### Technical Notes

**Fichier**: `src/clipboard.rs`

**Logique**:
```rust
pub struct ClipboardManager {
    inner: Clipboard,
}

impl ClipboardManager {
    pub fn capture_and_copy(&mut self) -> anyhow::Result<String> {
        // 1. Sauvegarder le contenu actuel du presse-papier
        let previous = self.inner.get_text().ok();

        // 2. Simuler Ctrl+C
        simulate_copy();

        // 3. Attendre que le presse-papier soit mis à jour
        sleep(Duration::from_millis(100));

        // 4. Lire le nouveau contenu
        let selected = self.inner.get_text()?;

        // 5. Stocker l'ancien contenu pour restauration
        self.previous = previous;

        Ok(selected)
    }

    pub fn paste_and_restore(&mut self, text: &str) -> anyhow::Result<()> {
        // 1. Écrire le texte traduit dans le presse-papier
        self.inner.set_text(text)?;

        // 2. Simuler Ctrl+V
        simulate_paste();

        // 3. Restaurer le contenu original
        sleep(Duration::from_millis(50));
        if let Some(prev) = self.previous.take() {
            self.inner.set_text(prev)?;
        } else {
            self.inner.clear()?;
        }

        Ok(())
    }
}
```

**Timing**: Les délais entre les opérations presse-papier doivent être testés sous Windows pour trouver le sweet spot.

---

### Definition of Done

- [ ] Sauvegarde du presse-papier avant copie
- [ ] Restauration après collage (succès ou échec)
- [ ] Gestion du cas "presse-papier vide initialement"
- [ ] `cargo clippy` et `cargo test` passent
