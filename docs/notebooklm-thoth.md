# Thoth — Présentation complète

> **Assistant de manipulation de texte instantanée par LLM pour Windows**
> Thoth est une application système écrite en Rust qui intercepte des raccourcis clavier globaux pour capturer du texte sélectionné, le traiter via un LLM (via la passerelle Pylos), et remplacer la sélection par le résultat — le tout en quelques centaines de millisecondes.

---

## 1. Vision & Problème résolu

### Problème
Les utilisateurs travaillent avec du texte dans de nombreuses applications (navigateur, email, IDE, Slack, Word) et doivent fréquemment :
- Traduire un passage dans une autre langue
- Reformuler un texte mal écrit
- Résumer un long paragraphe
- Corriger l'orthographe et la grammaire
- Répondre à un email
- Obtenir une explication

Sans Thoth, chaque action nécessite : copier le texte → aller dans un outil LLM → coller → attendre → copier le résultat → revenir à l'application → coller. Thoth réduit cela à une seule pression de touche.

### Solution
Thoth est un processus Windows silencieux (sans fenêtre) qui écoute 4 raccourcis clavier globaux. L'utilisateur sélectionne du texte, appuie sur le raccourci, et le texte est instantanément remplacé par le résultat du LLM.

---

## 2. Architecture

```
┌────────────────────────────────────────────────────────────────────┐
│                      Thoth (Rust)                                  │
│                                                                    │
│  INPUT LAYER                       PROCESSING LAYER                │
│  ┌──────────────┐                 ┌──────────────────┐            │
│  │ Windows      │── hotkey ──────▶│ Orchestrator     │            │
│  │ RegisterHotKey│                 │ (async main loop)│            │
│  └──────────────┘                 │                  │            │
│                                   │  ┌────────────┐  │            │
│  ┌──────────────┐                 │  │ Clipboard  │  │            │
│  │ GUI (eframe) │── instruction──▶│  │ Manager    │  │            │
│  │ Prompt/Config│                 │  │ (arboard)  │  │            │
│  │ Stats        │                 │  └────────────┘  │            │
│  └──────────────┘                 └──────┬───────────┘            │
│                                          │                        │
│  OUTPUT LAYER                            ▼                        │
│  ┌──────────────┐                 ┌──────────────────┐            │
│  │ Windows Toast│                 │ Pylos Client    │            │
│  │ Notifications│                 │ (reqwest, HTTPS)│            │
│  └──────────────┘                 │ Sensitive Filter│            │
│                                   │ Fallback Logic  │──▶ Pylos   │
│  ┌──────────────┐                 └──────────────────┘   Gateway  │
│  │ System Tray  │                                              │
│  │ (tray-icon)  │                                   ┌─────────┐ │
│  └──────────────┘                                   │  LLM    │ │
│                                                     │ Gemini  │ │
│  STORAGE LAYER                                      │ Gemma   │ │
│  ┌────────────────────────────┐                     └─────────┘ │
│  │ Windows Registry (DPAPI)  │                                   │
│  │  • HKCU\Software\Thoth    │                                   │
│  │    → Config (encrypted)   │                                   │
│  │    → History (plaintext)  │                                   │
│  └────────────────────────────┘                                   │
│  ┌────────────────────────────┐                                   │
│  │ File System                │                                   │
│  │  • thoth.log (redacted)    │                                   │
│  │  • metrics.json            │                                   │
│  └────────────────────────────┘                                   │
└────────────────────────────────────────────────────────────────────┘
```

### Flux de données (traduction par défaut)

1. L'utilisateur sélectionne du texte dans n'importe quelle application
2. Il appuie sur `Ctrl+Shift+Win+N`
3. Windows notifie Thoth via `WM_HOTKEY`
4. Thoth simule `Ctrl+C` pour copier le texte sélectionné
5. Thoth vérifie les données sensibles (API keys, JWT, etc.)
6. Si OK, envoie au LLM via Pylos en HTTPS avec authentification
7. Reçoit la réponse, l'écrit dans le presse-papier
8. Simule `Ctrl+V` pour remplacer la sélection par le résultat
9. Restaure le contenu original du presse-papier
10. Notifie l'utilisateur (toast Windows)

---

## 3. Stack technique

### Langage : Rust
- **Performances natives** — zéro GC, compilation AOT, binaire < 10 Mo
- **Sûreté mémoire** — le borrow checker élimine les fuites mémoire et les data races
- **Rust 1.88+** — edition 2024, latest language features

### Bibliothèques principales

| Domaine | Crate | Version | Usage |
|---|---|---|---|
| Async runtime | `tokio` | 1.x | Boucle événementielle, tasks concurrentes |
| GUI native | `eframe` / `egui` | 0.29 | Fenêtres overlay prompt, config, stats |
| HTTP client | `reqwest` | 0.13 | Communication HTTPS avec Pylos |
| Clipboard | `arboard` | 3.x | Lecture/écriture du presse-papier |
| Key simulation | `rdev` | 0.5 | Simulation Ctrl+C / Ctrl+V |
| Windows API | `windows-sys` | 0.61 | Hotkeys, DPAPI, signature verification |
| Win Registry | `winreg` | 0.56 | Stockage config, historique |
| Tray icon | `tray-icon` | 0.24 | Icône barre d'état Windows |
| Notifications | `notify-rust` | 4.x | Toasts Windows |
| Logging | `tracing` | 0.1 | Log structuré, multiples backends |
| Serialization | `serde` / `serde_json` / `toml` | 1.x | Config, métriques, requêtes |
| Regex | `regex` | 1.x | Détection données sensibles |

### Déploiement
- Binaire unique `thoth.exe` (standalone, lié statiquement)
- Installateur MSI via WiX (`installer/Thoth.wxs`)
- CI/CD GitHub Actions : build, sign (Authenticode), release
- Consommation mémoire : ~15-30 Mo en veille

---

## 4. Les 4 hotkeys

| Hotkey | Action | Description | Prompt système |
|---|---|---|---|
| `Ctrl+Shift+Win+N` | Traduction (langue cible) | Traduit et corrige le texte | `"Traduis en {langue}. Corrige l'orthographe/grammaire."` |
| `Ctrl+Shift+Win+,` | Traduction (anglais) | Traduit et corrige vers l'anglais | `"Traduis en anglais."` |
| `Ctrl+Shift+Win+:` | Instruction personnalisée | Ouvre GUI → entrez consigne → Entrée | Historique sauvegardé dans le registre |
| `Ctrl+Shift+Win+R` | Reformulation | Reformule pour clarté et style | `"Reformule, clarifie, restructure..."` |

---

## 5. Sécurité — Approche "Defense in Depth"

### 5.1 Configuration au repos
- **Chiffrement DPAPI** (`CryptProtectData`) — lié au compte utilisateur Windows
- Stockage dans le registre (`HKCU\Software\Thoth\Config`) en `REG_BINARY`
- Migration automatique depuis l'ancien fichier plat `config.toml` (supprimé après migration)
- L'historique des prompts est stocké dans `HKCU\Software\Thoth\History`

### 5.2 Transport
- **HTTPS imposé** — tout endpoint non-localhost est automatiquement converti de `http://` vers `https://`
- Validation TLS par défaut (certificats standards)
- Flag `--insecure` pour développement local (localhost ou certificats auto-signés)
- Double en-tête d'authentification : `X-Thoth-Secret` + `Authorization: Bearer`

### 5.3 Intégrité du binaire
- **WinVerifyTrust** au démarrage (release builds uniquement)
- Vérifie la signature Authenticode avant d'exécuter la boucle principale
- Blocage immédiat (exit code 1) si signature invalide

### 5.4 Détection de données sensibles
Avant tout envoi au LLM, le texte est scanné avec des regex pour détecter :

| Type | Pattern |
|---|---|
| OpenAI API Keys | `sk-[a-zA-Z0-9]{20,}` |
| OpenAI Project Keys | `pk-[a-zA-Z0-9]{20,}` |
| AWS Access Key ID | `(?i)AKIA[A-Z0-9]{16}` |
| GitHub Tokens | `gh[pousr]_[a-zA-Z0-9]{36,255}` |
| JSON Web Tokens | `eyJ[a-zA-Z0-9_-]+\.[...]+\.[...]+` |
| Private Keys | `-----BEGIN.*PRIVATE KEY-----` |
| Credit Cards | `\b\d{4}[\s-]?\d{4}[\s-]?\d{4}[\s-]?\d{4}\b` |
| Slack Tokens | `xox[bp]-[a-zA-Z0-9-]{10,}` |
| Slack Webhooks | `(?i)slack` |
| MongoDB URIs | `(?i)mongodb://` |
| PostgreSQL URIs | `(?i)postgres(ql)?://` |
| MySQL URIs | `(?i)mysql://` |

### 5.5 Logs
- **Aucun texte utilisateur ou LLM** n'est écrit dans `thoth.log`
- Seules la longueur et l'empreinte (hash) du contenu sont enregistrées
- Exemple : `"orchestrator: translating text to fr (len: 142, hash: 0xa3f7b2c1)"`

### 5.6 Processus
- **Zéro processus externe** — tout est fait in-process
- GUI native eframe/egui (Win32), pas de PowerShell ni HTA
- Simulation clavier via `rdev` (API Windows SendInput)

---

## 6. Interface Utilisateur

### 6.1 Barre d'état
Icône dans la notification area (systray) avec menu :
- Statut actif/désactivé
- Activer/Désactiver
- Démarrer avec Windows (check)
- Configuration → ouvre GUI config
- Statistiques → ouvre GUI stats
- Journaux → ouvre le log dans PowerShell
- Quitter

### 6.2 GUI de prompt (instruction personnalisée)
Fenêtre overlay native (toujours au premier plan) avec :
- Champ de saisie pour l'instruction
- Historique des 20 dernières consignes
- Navigation au clavier (flèches haut/bas)
- Sélection par clic sur un item de l'historique
- Bouton Lancer / Annuler
- Spinner de chargement pendant l'exécution

### 6.3 GUI de configuration
Éditeur complet avec champs pour :
- Endpoint Pylos
- Modèle principal + modèle de secours
- Timeout
- Clé secrète
- Langue cible
- Restauration presse-papier
- Notifications
- Anti-rebond (debounce)
- Raccourci clavier
- Bouton Enregistrer (redémarre automatiquement)

### 6.4 GUI de statistiques
Tableau de bord :
- Traductions réussies
- Erreurs rencontrées
- Volume de texte traité (octets)
- Latence moyenne (ms)
- Utilisation par modèle (nombre de fois)
- Bouton Réinitialiser

---

## 7. Cas d'usage

### Traduction rapide (scénario principal)
1. Vous lisez un email en anglais dans Outlook
2. Vous sélectionnez le texte
3. `Ctrl+Shift+Win+N` → le texte est remplacé par sa version française

### Reformulation professionnelle
1. Vous avez écrit un paragraphe brouillon dans Slack
2. Vous le sélectionnez
3. `Ctrl+Shift+Win+R` → le texte est réécrit de façon claire et structurée

### Instruction personnalisée
1. Vous sélectionnez un long article dans le navigateur
2. `Ctrl+Shift+Win+:` → la fenêtre overlay s'ouvre
3. Vous tapez "résume en 3 phrases" et appuyez sur Entrée
4. Le résumé remplace l'article sélectionné

### Correction orthographique
1. Vous sélectionnez un texte avec des fautes
2. `Ctrl+Shift+Win+N` → le texte est corrigé et traduit si besoin

---

## 8. Configuration et personnalisation

### Flags CLI
```
thoth.exe              → Service d'arrière-plan
thoth.exe --prompt     → GUI instruction uniquement
thoth.exe --config     → GUI configuration
thoth.exe --stats      → GUI statistiques
thoth.exe --insecure   → Désactive vérification TLS
```

### Variables d'environnement
```
RUST_LOG=debug         → Logs détaillés
RUST_LOG=thoth=trace   → Logs très détaillés
RUST_LOG=off           → Pas de logs
```

---

## 9. Métriques et monitoring

Thoth enregistre des métriques dans `metrics.json` (dans le même dossier que l'exécutable) :
- `total_translations` — nombre de traductions réussies
- `total_errors` — nombre d'erreurs
- `total_bytes_processed` — volume total de texte traité
- `total_latency_ms` — latence cumulée
- `model_usage` — dictionnaire modèle → nombre d'utilisations

Accès via : GUI Statistiques (`--stats`) ou menu tray.

---

## 10. Développement et CI/CD

### Build
```bash
cargo build              # Debug
cargo build --release    # Release (optimisé, strippé, LTO)
```

### Tests
```bash
cargo test               # 35 unit + 7 integration
cargo clippy              # Linting
cargo fmt --all           # Formatage
```

### Pipeline GitHub Actions

| Job | Durée ≈ | Outils |
|---|---|---|
| `lint` | ~30s | actionlint, typos |
| `check` | ~3min | cargo fmt, clippy, test, deny |
| `msrv` | ~3min | Rust 1.88.0 |
| `build` | ~5min | cargo build --release |
| `msi` | ~2min | WiX (candle + light) |
| `sign` | ~1min | Azure Key Vault + Windows SDK SignTool |
| `release` | ~1min | GitHub Releases API |

### Certification de signature
- Certificate Authority : DigiCert (ou Azure Code Signing)
- Niveau : Individual / Organization Validation
- Algorithme : SHA-256 / RSA-4096
- Timestamp : RFC 3161
- Période : renouvellement annuel

---

## 11. Privacité et RGPD

- **Aucune donnée personnelle collectée** — Thoth ne fait pas de télémétrie
- **Aucun service externe** — tout passe par votre propre instance Pylos
- **Les logs sont caviardés** — aucun texte utilisateur n'est persistant
- **Le texte est effacé après usage** — une fois le résultat collé, le presse-papier est restauré
- **Configuration chiffrée** — DPAPI lie les secrets à votre session Windows

---

## 12. Limitations connues

- **Windows, macOS, Linux** — support complet Windows/macOS, partiel Linux (pas de tray ni hotkeys globaux)
- **Screenshot & MQTT** — nécessite MinIO S3 et broker EMQX configurés (optionnel)
- **Dépendance à Pylos** — nécessite une instance Pylos (ou compatible OpenAI API) accessible
- **Simulation clavier** — `Ctrl+C`/`Ctrl+V` simulé peut échouer avec certaines applications (UWP, jeux, terminaux spécifiques)
- **Délai de copie** — 100ms d'attente entre `Ctrl+C` et la lecture du presse-papier (paramétré)
- **Non-détection des images** — Thoth ne traite que le texte, pas les images dans le presse-papier

---

## 13. Glossaire

| Terme | Définition |
|---|---|
| **Pylos** | Passerelle LLM sécurisée — reçoit les requêtes de Thoth, les relaye au LLM, retourne la réponse |
| **DPAPI** | Data Protection API — API Windows pour le chiffrement lié à l'utilisateur |
| **WinVerifyTrust** | API Windows de validation de signature Authenticode |
| **eframe/egui** | Framework GUI immédiat en Rust, natif Win32 |
| **RegisterHotKey** | API Windows pour l'enregistrement de raccourcis globaux |
| **Authenticode** | Standard Microsoft de signature de code |
| **WiX** | Windows Installer XML — outil de création d'installateurs MSI |
| **arboard** | Crate Rust de gestion du presse-papier multiplateforme |
| **rdev** | Crate Rust de simulation d'entrées clavier/souris |
| **LTO** | Link-Time Optimization — optimisation du binaire à l'édition de liens |
| **LTO** | Link-Time Optimization — optimisation du binaire à l'édition de liens |

---

## 14. Annexes

### A. Dépendances complètes
```
├── tokio (async runtime)
├── arboard (clipboard)
├── reqwest (HTTP/HTTPS client)
├── serde + serde_json + toml (serialization)
├── tracing + tracing-subscriber + tracing-appender (logging)
├── anyhow + thiserror (error handling)
├── regex (sensitive data patterns)
├── time (temporal)
├── sys-locale (system language)
├── image + eframe/egui (GUI + icons)
├── rdev / notify-rust / tray-icon / winreg / windows-sys (Windows-specific)
```

### B. Taille du binaire
```
Debug   : ~80 Mo (unoptimized, full debug symbols)
Release : ~7-9 Mo (LTO, stripped, panic=abort)
MSI     : ~3-5 Mo (compressed)
```

### C. Processus de release
1. Push de tag `v*` → GitHub Actions déclenche `sign` + `release`
2. Code signing via SignTool + Azure Key Vault
3. Création de GitHub Release avec `thoth.exe` signé + `Thoth.msi`
4. Notification aux utilisateurs (optionnel : auto-update à implémenter)
