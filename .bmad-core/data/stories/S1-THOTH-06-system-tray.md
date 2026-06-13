## Story: System Tray & Background Service

**ID**: S1-THOTH-06
**Épic**: EPIC-THOTH-01
**Points**: 3
**Statut**: DONE

---

### User Story

**As a** Utilisateur
**I want** que Thoth reste discret en arrière-plan avec une icône dans la barre des tâches
**So that** je puisse savoir qu'il est actif et le quitter si nécessaire

---

### Acceptance Criteria

- [ ] Given Thoth est lancé, when l'application démarre, alors une icône apparaît dans la system tray
- [ ] Given l'icône est affichée, when l'utilisateur clique droit, alors un menu contextuel apparaît avec "Quitter"
- [ ] Given l'utilisateur choisit "Quitter", when le menu est cliqué, alors Thoth s'arrête proprement
- [ ] Given Thoth est en arrière-plan, when il n'y a pas d'erreur, alors aucune fenêtre n'est visible
- [ ] Given une erreur critique, when elle survient, alors une notification Windows peut être affichée (optionnel)

---

### Technical Notes

**Approche Rust**:
- Utiliser la crate `tray-item` ou `winapi` directement pour l'icône système
- Alternative: lancer Thoth sans fenêtre console (utiliser `#![windows_subsystem = "windows"]`)

**Crate recommandée**: `tray-icon` (wrapper Rust pour l'icône système Windows)

**main.rs**:
```rust
#![windows_subsystem = "windows"]

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let (shutdown_tx, shutdown_rx) = oneshot::channel();

    // Démarrer le system tray dans un thread séparé
    let tray_handle = tokio::task::spawn_blocking(|| {
        run_tray(shutdown_tx)
    });

    // Démarrer l'orchestrateur
    let mut orchestrator = Orchestrator::new()?;
    orchestrator.run(shutdown_rx).await?;

    Ok(())
}
```

**Configuration du build**:
```toml
# Cargo.toml
[package]
name = "thoth"

[[bin]]
name = "thoth"
path = "src/main.rs"

[profile.release]
opt-level = "z"     # Minimize size
lto = true
codegen-units = 1
```

---

### Definition of Done

- [ ] Icône dans la system tray au démarrage
- [ ] Menu contextuel avec "Quitter"
- [ ] Arrêt propre sur "Quitter"
- [ ] Aucune fenêtre visible en mode background
- [ ] `cargo clippy` et `cargo test` passent
