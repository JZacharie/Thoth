# ADR-002: Architecture de Thoth

**Statut**: Proposé
**Date**: 2026-06-13
**Décideur**: Larry (Architect)

---

## Contexte

Thoth est une application système Windows légère qui intercepte un raccourci clavier,
communique avec la passerelle Pylos, et remplace le texte sélectionné. Elle doit être
fiable, rapide et invisible pour l'utilisateur.

## Décision

### Architecture modulaire

Thoth suit une architecture modulaire avec des responsabilités clairement séparées :

```
thoth/
├── src/
│   ├── main.rs              # Point d'entrée, initialisation
│   ├── config.rs            # Configuration (fichier + hot-reload)
│   ├── hotkey.rs            # Global keyboard hook (rdev)
│   ├── clipboard.rs         # Clipboard operations (arboard)
│   ├── pylos_client.rs      # Client HTTP Pylos (reqwest)
│   ├── orchestrator.rs      # Boucle principale, coordination
│   ├── notification.rs      # Notifications toast Windows
│   ├── tray.rs              # System tray icon & menu
│   ├── auto_start.rs        # Registry auto-start management
│   ├── metrics.rs           # Statistiques d'utilisation locales
│   └── error.rs             # Types d'erreur unifiés
```

### Choix des crates

| Besoin | Crate | Raison |
|---|---|---|
| Async runtime | `tokio` | Standard Rust, full-featured |
| Global hotkey | `rdev` | Multi-OS, simple API, testé |
| Clipboard | `arboard` | API propre, support Windows natif |
| HTTP client | `reqwest` | Standard Rust, async, JSON built-in |
| Notifications | `notify-rust` | Windows toast notifications |
| File watch | `notify` | Hot-reload de config |
| Serialization | `serde` + `toml` | Config en TOML |
| Logging | `tracing` | Structured logging, faible overhead |
| Error handling | `anyhow` | App-level errors, contexte riche |

### Gestion des erreurs

Toutes les erreurs sont propagées via `anyhow::Error` et loggées avec `tracing::error!`.
L'orchestrateur ne plante jamais — chaque erreur est absorbée et la boucle continue.

### Flux de données

```
Hotkey Event
    │
    ▼
Orchestrator
    │
    ├─► Clipboard.copy_selected_text()  ──►  Ctrl+C simulé + lecture presse-papier
    │
    ├─► PylosClient.translate(text)     ──►  POST /v1/chat/completions → Pylos
    │
    └─► Clipboard.paste_text(result)    ──►  Écriture presse-papier + Ctrl+V simulé
         │
         └─► Clipboard.restore()        ──►  Restauration du presse-papier original
```

### Cycle de vie du processus

```
Démarrage
    │
    ├─► Charger config (config.toml ou défauts)
    ├─► Initialiser system tray
    ├─► Enregistrer hotkey Win+N
    ├─► Lancer ConfigWatcher (hot-reload)
    │
    ▼
Boucle principale (orchestrator.run())
    │
    ├─► Attendre signal hotkey
    ├─► Copier texte → Pylos → Coller
    ├─► Logger résultat + métriques
    └─► Retour à l'attente
    │
    ▼
Signal d'arrêt (menu "Quitter")
    │
    ├─► Désenregistrer hotkey
    ├─► Libérer presse-papier
    ├─► Sauvegarder métriques
    └─► Exit
```

## Conséquences

**Positives**:
- Modules indépendants et testables unitairement
- Remplacement facile d'un composant (ex: `rdev` → `inputbot`)
- Hot-reload sans redémarrage
- Crash-proof : l'utilisateur ne perd jamais son presse-papier

**Négatives**:
- Dépendance à `rdev` spécifiquement Windows (non testable sur Linux)
- Pas de séparation hexagonale stricte (pas nécessaire pour une app de cette taille)
- La gestion du timing presse-papier est empirique (sleeps)

## Alternatives considérées

1. **Architecture hexagonale** (pylos-core / infrastructure / application) — trop lourde pour une app monocrate
2. **Un seul fichier main.rs** — maintenable jusqu'à ~500 lignes seulement
3. **Plugin system** pour les modèles — overkill pour un binaire standalone Windows

## Liens

- ADR-001: Bifrost migration strategy (Pylos)
- `.bmad-core/data/epics/epics-thoth-initial-development.md`
- `.bmad-core/data/sprints/sprint-3.md`
