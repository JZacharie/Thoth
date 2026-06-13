## Story: Global Hotkey Win+N

**ID**: S1-THOTH-02
**Épic**: EPIC-THOTH-01
**Points**: 5
**Statut**: DONE

---

### User Story

**As a** Utilisateur
**I want** appuyer sur `Win + N` depuis n'importe quelle application Windows
**So that** le texte sélectionné soit automatiquement traité

---

### Acceptance Criteria

- [ ] Given l'application est en arrière-plan, when l'utilisateur appuie sur `Win + N`, alors le hotkey est intercepté
- [ ] Given une autre application est au premier plan, when `Win + N` est pressé, alors l'événement est capturé (pas de conflit)
- [ ] Given le hotkey est pressé, when aucun texte n'est sélectionné, alors l'action est ignorée silencieusement
- [ ] Given le hotkey est pressé, when du texte est sélectionné, alors le callback de traitement est déclenché

---

### Technical Notes

**Fichier**: `src/hotkey.rs`

**Approche**: Utiliser `rdev` pour l'écoute globale du clavier.

```rust
pub fn listen_hotkey(tx: mpsc::Sender<()>) -> Result<()> {
    // Filtrer les événements pour détecter Win + N
    // Note: rdev utilise Key::MetaLeft/Right pour la touche Win
    //       Key::KeyN pour la touche N
}
```

**Dépendances**:
- `rdev = "0.5"` (ou `inputbot` si `rdev` pose problème sous Windows)

**Points d'attention**:
- `rdev` nécessite des privilèges sous certains OS, mais pas sous Windows avec le hook clavier
- Le canal `mpsc` de Tokio est utilisé pour notifier l'orchestrateur
- Gérer le key repeat (ignorer si la touche est maintenue)

---

### Definition of Done

- [ ] Le hotkey `Win + N` est correctement intercepté
- [ ] Aucune interférence avec le comportement natif de Windows
- [ ] Tests unitaires du module hotkey
- [ ] `cargo clippy` et `cargo test` passent
