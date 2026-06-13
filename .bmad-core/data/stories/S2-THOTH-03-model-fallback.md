## Story: Fallback Modèle LLM

**ID**: S2-THOTH-03
**Épic**: EPIC-THOTH-02
**Points**: 3
**Statut**: DONE

---

### User Story

**As a** Utilisateur
**I want** que Thoth bascule automatiquement sur un modèle secondaire si le modèle principal échoue
**So that** la traduction fonctionne même quand gemma4 est indisponible

---

### Acceptance Criteria

- [ ] Given le modèle principal (`gemma4:12b`) est injoignable, when la requête échoue, alors Thoth tente automatiquement le modèle fallback (`gemini4:12b`)
- [ ] Given le fallback réussit, when la traduction est effectuée, alors le texte collé provient du fallback
- [ ] Given les deux modèles échouent, when les deux requêtes sont en erreur, alors Thoth notifie l'utilisateur
- [ ] Given le fallback est configuré avec le même endpoint Pylos, when le fallback est utilisé, alors la requête est envoyée au même endpoint avec un modèle différent

---

### Technical Notes

**Fichier**: `src/pylos_client.rs` et `src/config.rs`

**Configuration**:
```toml
[pylos]
endpoint = "http://localhost:3000"
model = "gemma4:12b"
fallback_model = "gemini4:12b"
timeout_secs = 10
```

**Logique de fallback**:
```rust
pub async fn translate(&self, text: &str) -> Result<String, TranslationError> {
    match self.translate_with_model(text, &self.config.model).await {
        Ok(result) => Ok(result),
        Err(e) => {
            tracing::warn!("Modèle principal échoué: {e}, tentative fallback");
            match self.translate_with_model(text, &self.config.fallback_model).await {
                Ok(result) => Ok(result),
                Err(e2) => {
                    tracing::error!("Fallback aussi échoué: {e2}");
                    Err(TranslationError::AllModelsFailed {
                        primary: Box::new(e),
                        fallback: Box::new(e2),
                    })
                }
            }
        }
    }
}
```

**Délai supplémentaire**: Le fallback peut inclure un petit délai (optionnel) pour laisser le modèle principal redémarrer.

---

### Definition of Done

- [ ] Fallback automatique sur modèle secondaire
- [ ] Notification utilisateur si les deux modèles échouent
- [ ] Configuration du fallback dans config.toml
- [ ] Tests avec mock simulant l'échec du modèle principal
- [ ] `cargo clippy` et `cargo test` passent
