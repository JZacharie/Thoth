## Story: Scaffolding du projet Rust

**ID**: S1-THOTH-01
**Épic**: EPIC-THOTH-01
**Points**: 2
**Statut**: DONE

---

### User Story

**As a** Developer
**I want** initialiser le projet Rust avec les dépendances et la structure
**So that** l'équipe peut développer sur une base solide

---

### Acceptance Criteria

- [ ] Given le projet, when `cargo build` est exécuté, alors le binaire compile sans erreur
- [ ] Given le projet, when `cargo test` est exécuté, alors les tests passent
- [ ] Given le projet, when `cargo clippy -- -D warnings` est exécuté, alors aucun warning
- [ ] Given le projet, when `cargo fmt --all` est exécuté, alors le code est formaté

---

### Technical Notes

**Structure du projet**:
```
thoth/
├── Cargo.toml
├── src/
│   ├── main.rs
│   ├── hotkey.rs        — Global keyboard hook
│   ├── clipboard.rs     — Clipboard operations
│   ├── pylos_client.rs  — HTTP client for Pylos
│   ├── orchestrator.rs  — Main orchestration logic
│   └── config.rs        — Configuration
```

**Dépendances initiales** (`Cargo.toml`):
```toml
[dependencies]
tokio = { version = "1", features = ["full"] }
rdev = "0.5"
arboard = "3"
reqwest = { version = "0.12", features = ["json"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
tracing = "0.1"
tracing-subscriber = "0.3"
anyhow = "1"
```

**OS target**: Windows 10/11 uniquement

---

### Implementation Notes

- Utiliser `cargo init --name thoth`
- Configurer `tracing-subscriber` avec sortie fichier pour le débogage
- Ajouter `.cargo/config.toml` avec `rustflags = ["-D warnings"]`

---

### Definition of Done

- [x] `cargo build` réussit
- [x] `cargo test` passe
- [x] `cargo clippy` passe
- [x] `cargo fmt` appliqué
