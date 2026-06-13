## Story: Raccourci Clavier Configurable

**ID**: S3-THOTH-01
**Épic**: EPIC-THOTH-03
**Points**: 3
**Statut**: DONE

---

### User Story

**As a** Utilisateur
**I want** pouvoir changer le raccourci clavier global (pas seulement Win+N)
**So that** je puisse utiliser un raccourci qui ne conflit pas avec mes autres applications

---

### Acceptance Criteria

- [ ] Given la config contient `hotkey = "Ctrl+Shift+T"`, when l'utilisateur presse Ctrl+Shift+T, alors Thoth se déclenche
- [ ] Given le hotkey est modifié dans le fichier de config, when le hot-reload détecte le changement, alors le nouveau hotkey est enregistré sans redémarrage
- [ ] Given un hotkey invalide est configuré, when Thoth essaie de l'enregistrer, alors il logge l'erreur et conserve l'ancien hotkey
- [ ] Given `Win + N` est le défaut, when aucune config n'est fournie, alors Win+N est utilisé

---

### Technical Notes

**Fichiers**: `src/config.rs` (champ `hotkey`), `src/hotkey.rs`

**Format du hotkey**: Chaîne comme `"Win+N"`, `"Ctrl+Shift+T"`, `"Alt+Space"`.

**Parsing**:
```rust
#[derive(Debug, Clone, PartialEq)]
pub struct Hotkey {
    pub modifiers: Vec<Modifier>,   // Win, Ctrl, Alt, Shift
    pub key: Key,                    // N, T, Space, F1..F12
}

impl FromStr for Hotkey {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split('+').collect();
        // Dernier élément = la touche, avant = modificateurs
        // ...
    }
}
```

**Ré-enregistrement**: Le hotkey hook doit pouvoir être arrêté et redémarré avec le nouveau binding sans redémarrer le processus.

```rust
pub struct HotkeyManager {
    current: Option<Hotkey>,
}

impl HotkeyManager {
    pub fn rebind(&mut self, new_hotkey: Hotkey) -> anyhow::Result<()> {
        self.unregister()?;
        self.register(new_hotkey)?;
        self.current = Some(new_hotkey);
        Ok(())
    }
}
```

**Combinaisons valides**:
- `Win + {letter, number, F1-F12}`
- `Ctrl + {letter, number, F1-F12}`
- `Alt + {letter, number, F1-F12}`
- `Ctrl + Shift + {letter, number, F1-F12}`
- `Alt + Shift + {letter, number, F1-F12}`

---

### Definition of Done

- [ ] Parsing de la chaîne de hotkey
- [ ] Ré-enregistrement dynamique via hot-reload config
- [ ] Validation des combinaisons valides
- [ ] Tests unitaires du parsing
- [ ] `cargo clippy` et `cargo test` passent
