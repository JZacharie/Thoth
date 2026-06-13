## Story: Configuration Hot-Reload

**ID**: S1-THOTH-11
**Épic**: EPIC-THOTH-01
**Points**: 3
**Statut**: DONE

---

### User Story

**As a** Utilisateur
**I want** modifier l'endpoint Pylos ou le modèle sans redémarrer Thoth
**So that** je puisse basculer entre différents backends LLM à la volée

---

### Acceptance Criteria

- [ ] Given un fichier `%APPDATA%/thoth/config.toml`, when Thoth démarre, alors il lit la configuration
- [ ] Given le fichier de config est modifié, when un changement est détecté, alors Thoth recharge la config sans redémarrer
- [ ] Given la config est rechargée, when le hotkey est pressé, alors les nouveaux paramètres sont utilisés
- [ ] Given le fichier de config contient une erreur de syntaxe, when le hot-reload tente de charger, alors Thoth conserve l'ancienne config et logge l'erreur
- [ ] Given la config contient `pylos_endpoint`, `model`, `timeout_secs`, `system_prompt`, `language`, when le chargement réussit, alors tous les champs sont appliqués

---

### Technical Notes

**Fichiers**: `src/config.rs`

**Format de configuration** (TOML) :
```toml
[pylos]
endpoint = "http://localhost:3000"
model = "gemma4:12b"
timeout_secs = 10

[behavior]
system_prompt = """
Tu es un traducteur et correcteur de texte ultra-précis...
"""
target_language = "fr"
restore_clipboard = true
show_notifications = true
```

**Hot-reload** :
```rust
pub struct Config {
    pub pylos: PylosConfig,
    pub behavior: BehaviorConfig,
}

pub struct ConfigWatcher {
    path: PathBuf,
    current: Arc<RwLock<Config>>,
}

impl ConfigWatcher {
    /// Surveille le fichier de config via `notify` crate
    pub async fn watch(&self) {
        let mut watcher = RecommendedWatcher::new(...)?;
        watcher.watch(&self.path, RecursiveMode::NonRecursive)?;
        loop {
            match watcher.rx.recv().await {
                Some(Ok(event)) if event.kind.is_modify() => {
                    match self.reload() {
                        Ok(cfg) => *self.current.write() = cfg,
                        Err(e) => tracing::error!("Config invalide: {e}"),
                    }
                }
            }
        }
    }
}
```

**Dépendance**: `notify = "7"` pour la surveillance de fichiers, `toml = "0.8"` pour le parsing.

**Default**: Si aucun fichier de config n'existe, Thoth utilise les valeurs par défaut (localhost:3000, gemma4:12b, 10s).

---

### Definition of Done

- [ ] Configuration chargée depuis `%APPDATA%/thoth/config.toml` au démarrage
- [ ] Hot-reload via `notify` crate avec fallback sur ancienne config si erreur
- [ ] Tous les paramètres opérationnels sont externalisés
- [ ] `cargo clippy` et `cargo test` passent
