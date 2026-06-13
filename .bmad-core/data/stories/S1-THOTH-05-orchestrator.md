## Story: Orchestrateur Principal

**ID**: S1-THOTH-05
**Épic**: EPIC-THOTH-01
**Points**: 5
**Statut**: DONE

---

### User Story

**As a** Utilisateur
**I want** que l'ensemble du processus soit fluide et rapide (copie → Pylos → collage)
**So that** je puisse traduire/corriger du texte en une fraction de seconde

---

### Acceptance Criteria

- [ ] Given le hotkey est déclenché, when du texte est sélectionné, alors le cycle complet s'exécute en moins de 3 secondes
- [ ] Given Pylos répond avec le texte traduit, when le collage est effectué, alors le texte original est remplacé sans artefact
- [ ] Given le texte est très long (>1000 mots), when la requête est envoyée, alors Thoth gère correctement les longs textes
- [ ] Given une erreur réseau, when la requête échoue, alors Thoth logge l'erreur et ne colle pas de texte erroné
- [ ] Given un conflit de presse-papier, when l'opération échoue, alors Thoth restaure l'état initial
- [ ] **SECURITE** : Given le hotkey est déclenché deux fois rapidement, when le second déclenchement arrive moins de 500ms après le premier, alors il est ignoré (debounce)
- [ ] **SECURITE** : Given le texte sélectionné contient une clé API (`sk-...`), when le filtre heuristique détecte le pattern, alors Thoth ignore et notifie l'utilisateur sans envoyer à Pylos

---

### Technical Notes

**Fichier**: `src/orchestrator.rs`

**Flux**:
```rust
pub struct Orchestrator {
    hotkey_rx: mpsc::Receiver<()>,
    clipboard: ClipboardManager,
    pylos: PylosClient,
    config: Config,
    last_trigger: Instant,      // Debounce
}

impl Orchestrator {
    pub async fn run(&mut self) {
        loop {
            // 1. Attendre le signal du hotkey
            self.hotkey_rx.recv().await;

            // 2. Debounce : ignorer si déclenché il y a < 500ms
            let now = Instant::now();
            if now.duration_since(self.last_trigger) < Duration::from_millis(500) {
                continue;
            }
            self.last_trigger = now;

            // 3. Petite pause pour laisser le système se stabiliser
            sleep(Duration::from_millis(100)).await;

            // 3. Simuler Ctrl+C et lire le presse-papier
            let original_text = self.clipboard.copy_selected_text()?;

            // 4. Skip si pas de texte
            if original_text.is_empty() { continue; }

            // 5. Filtre heuristique : ignorer si le texte ressemble à un secret
            if contains_sensitive_data(&original_text) {
                tracing::warn!("texte ignoré: contient des données sensibles");
                self.clipboard.restore()?;
                continue;
            }

            // 6. Envoyer à Pylos
            let translated = self.pylos.translate(&original_text).await?;

            // 6. Coller le résultat
            self.clipboard.paste_text(&translated)?;
        }
    }
}
```

**Gestion d'erreurs**:
- Utiliser `anyhow::Error` pour la propagation
- Logger toutes les erreurs avec `tracing::error!`
- Ne jamais planter sur une erreur — continuer la boucle

**Performance**:
- Le délai entre Ctrl+C et la lecture du presse-papier doit être suffisant mais minimal
- Utiliser des `Instant` pour mesurer le temps de cycle et le logger en debug

---

### Definition of Done

- [ ] Cycle complet hotkey → copie → requête → collage fonctionnel
- [ ] Debounce 500ms implémenté et testé
- [ ] Filtre heuristique de données sensibles (clés API, tokens, cartes bancaires)
- [ ] Gestion des erreurs sans crash
- [ ] Skip si texte vide
- [ ] Cycle < 3s mesuré
- [ ] Tests d'intégration avec mock Pylos
- [ ] `cargo clippy` et `cargo test` passent
