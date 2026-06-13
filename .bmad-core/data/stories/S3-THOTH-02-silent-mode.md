## Story: Mode Silencieux / Toggle On/Off

**ID**: S3-THOTH-02
**Épic**: EPIC-THOTH-03
**Points**: 1
**Statut**: DONE

---

### User Story

**As a** Utilisateur
**I want** pouvoir désactiver temporairement Thoth depuis l'icône system tray
**So that** je puisse utiliser `Win+N` normalement sans déclencher la traduction

---

### Acceptance Criteria

- [ ] Given Thoth est actif, when l'utilisateur clique sur "Désactiver" dans le menu tray, alors le hotkey est désenregistré
- [ ] Given Thoth est désactivé, when l'utilisateur clique sur "Activer" dans le menu tray, alors le hotkey est ré-enregistré
- [ ] Given Thoth est désactivé, when l'icône tray est survolée, alors le tooltip affiche "Thoth - Désactivé"
- [ ] Given Thoth est activé, when l'icône tray est survolée, alors le tooltip affiche "Thoth - Actif"

---

### Technical Notes

**Fichiers**: `src/tray.rs`, `src/orchestrator.rs`

**État**:
```rust
#[derive(Clone, Copy, PartialEq)]
enum ThothState {
    Active,
    Disabled,
}
```

**Menu contextuel**:
```
Thoth - Actif          <-- checkmark
───────────────
Désactiver             <-- toggle
Langue cible ▸
───────────────
Démarrer avec Windows
───────────────
Quitter
```

**Icône**: Deux icônes différentes (activée/désactivée) dans les ressources du binaire, ou overlay.

---

### Definition of Done

- [ ] Toggle Activer/Désactiver dans le menu tray
- [ ] Désenregistrement/ré-enregistrement du hotkey
- [ ] Tooltip reflète l'état actuel
- [ ] `cargo clippy` et `cargo test` passent
