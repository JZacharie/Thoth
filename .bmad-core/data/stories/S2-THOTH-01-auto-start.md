## Story: Démarrage Automatique au Boot Windows

**ID**: S2-THOTH-01
**Épic**: EPIC-THOTH-02
**Points**: 2
**Statut**: DONE

---

### User Story

**As a** Utilisateur
**I want** que Thoth se lance automatiquement au démarrage de Windows
**So that** je n'aie pas à le relancer manuellement après chaque redémarrage

---

### Acceptance Criteria

- [ ] Given Thoth est installé, when Windows démarre, alors Thoth est lancé automatiquement
- [ ] Given l'utilisateur veut désactiver le démarrage auto, when il coche une option dans le menu tray, alors Thoth est retiré du démarrage automatique
- [ ] Given le binaire a été déplacé/supprimé, when Windows tente de le lancer au démarrage, alors rien ne plante (entrée registry ignorée silencieusement)

---

### Technical Notes

**Fichier**: `src/auto_start.rs`

**Mécanisme**: Clé registry `HKCU\Software\Microsoft\Windows\CurrentVersion\Run`

```rust
pub fn enable_auto_start() -> anyhow::Result<()> {
    let key = r"Software\Microsoft\Windows\CurrentVersion\Run";
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let (run_key, _) = hkcu.create_subkey(key)?;
    run_key.set_value("Thoth", &std::env::current_exe()?.to_string_lossy().to_string())?;
    Ok(())
}

pub fn disable_auto_start() -> anyhow::Result<()> {
    let key = r"Software\Microsoft\Windows\CurrentVersion\Run";
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let run_key = hkcu.open_subkey_with_flags(key, KEY_SET_VALUE)?;
    run_key.delete_value("Thoth")?;
    Ok(())
}

pub fn is_auto_start_enabled() -> bool {
    // Vérifie si l'entrée existe et pointe vers le bon exécutable
}
```

**Dépendance**: `winapi` ou `windows-sys` pour l'accès au registry.

**Menu tray**: Ajouter une entrée "Démarrer avec Windows" avec une checkbox.

---

### Definition of Done

- [ ] Ajout/suppression de l'entrée registry fonctionnel
- [ ] Option dans le menu contextuel de la system tray
- [ ] Gestion du cas où le binaire a été déplacé
- [ ] `cargo clippy` et `cargo test` passent
