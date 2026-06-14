# Thoth — Document de connaissance du projet

## Présentation générale

**Thoth** est une application système légère écrite en **Rust** qui permet la manipulation de texte et l'analyse visuelle en temps réel grâce à des grands modèles de langage (LLM). Son nom est inspiré du dieu égyptien Thoth, divinité du savoir, de l'écriture et de la sagesse.

L'application s'exécute en arrière-plan sous forme d'un processus silencieux. Elle intercepte des **raccourcis clavier globaux**, capture le texte sélectionné ou l'image de la fenêtre active, traite les données via une passerelle LLM (Pylos) ou publie des résultats sur un broker MQTT, puis remplace le texte d'origine ou logue les réponses de manière transparente pour l'utilisateur.

### Cas d'usage principaux

*   **Traduction instantanée** : Sélectionner un texte dans n'importe quel logiciel, appuyer sur `Ctrl+Shift+Win+N` (configurable), et le texte est automatiquement traduit dans la langue cible configurée.
*   **Traduction vers l'anglais** : Raccourci dédié `Ctrl+Shift+Win+,` pour forcer la traduction vers l'anglais.
*   **Reformulation / clarification** : Réécrire un texte mal formulé, trop verbeux ou peu clair via `Ctrl+Shift+Win+R`.
*   **Instruction personnalisée** : Ouvrir un panneau GUI avec `Ctrl+Shift+Win+:`, saisir n'importe quelle instruction ("résume ce texte", "réponds à cet email"), et le résultat remplace la sélection.
*   **Analyse visuelle et publication MQTT** : Capturer la fenêtre active avec `Ctrl+Shift+Win+P`, analyser les questions de l'écran avec un modèle multimodal, archiver l'image sur MinIO S3, et pousser les réponses validées sur le broker EMQX MQTT.

---

## Langage et technologies

### Rust
Thoth est entièrement écrit en **Rust** (édition 2024), avec une version minimale supportée (MSRV) de **1.88.0**. Le choix de Rust garantit une faible empreinte mémoire, une sécurité mémoire stricte sans ramasse-miettes, et des binaires autonomes.

### Dépendances principales

| Crate | Rôle | Target |
|---|---|---|
| `tokio` | Runtime asynchrone (moteur principal d'événements) | Multiplateforme |
| `reqwest` | Client HTTP/HTTPS pour appeler l'API LLM | Multiplateforme |
| `serde` / `serde_json` | Sérialisation/désérialisation JSON | Multiplateforme |
| `eframe` / `egui` | Interface graphique native (prompt, config, stats) | Multiplateforme |
| `arboard` | Gestion du presse-papier (lecture et écriture) | Multiplateforme |
| `rdev` | Simulation de frappes clavier (copier-coller adaptatif) | Multiplateforme |
| `xcap` | Capture d'écran et découpage par fenêtre active | Multiplateforme |
| `rumqttc` | Client de publication MQTT | Multiplateforme |
| `rust-s3` | Client de stockage compatible AWS S3 / MinIO | Multiplateforme |
| `keyring` | Stockage sécurisé des identifiants (Keychain, Credential Manager) | Multiplateforme |
| `directories` | Chemins d'accès standards pour les fichiers de configuration | Multiplateforme |
| `rfd` | Boîtes de dialogue système portables pour les rapports d'erreur | Multiplateforme |
| `tray-icon` | Icône et menu dans la barre d'état système | Multiplateforme |
| `notify-rust` | Notifications toast natives (Windows, macOS, Linux) | Multiplateforme |
| `winreg` / `windows-sys` | Liaisons spécifiques Windows | Windows uniquement |

---

## Architecture du système (Multi-OS & Roadmap)

```
┌──────────────────────────────────────────────────────────────────────────┐
│  Thoth Application (Windows, macOS, Linux)                               │
│                                                                          │
│  INPUT / TRIGGER                   PROCESSING / CORE                     │
│  ┌────────────────┐               ┌──────────────────┐                   │
│  │ global-hotkey  │── hotkey ────▶│ Orchestrator     │                   │
│  │ (Win/Carbon/X1)│               │ (async main loop)│                   │
│  └────────────────┘               │                  │                   │
│                                   │  ┌────────────┐  │                   │
│  ┌────────────────┐               │  │ Clipboard  │  │                   │
│  │ GUI (eframe)   │── prompt ────▶│  │ Manager    │  │                   │
│  │ Stats/Config   │               │  └────────────┘  │                   │
│  └────────────────┘               └──────┬───────────┘                   │
│                                          │                               │
│  OUTPUT LAYER                            ▼                               │
│  ┌────────────────┐               ┌──────────────────┐                   │
│  │ System Tray    │               │ Pylos Client     │──▶ LLM            │
│  │ (tray-icon)    │               │ (Gemini 3.5)     │   (Inférence)     │
│  └────────────────┘               └──────┬───────────┘                   │
│                                          │                               │
│  STORAGE / INTEGRATION                   ▼                               │
│  ┌────────────────┐               ┌──────────────────┐                   │
│  │ Secure Storage │               │ S3 (MinIO)       │──▶ Captures       │
│  │ (keyring)      │               │ MQTT (EMQX)      │──▶ Broker MQTT    │
│  └────────────────┘               └──────────────────┘                   │
└──────────────────────────────────────────────────────────────────────────┘
```

### Flux de traitement multi-OS :
1. **L'utilisateur appuie sur un raccourci clavier** (ex: `Ctrl+Shift+Win+P` pour la capture/analyse).
2. **`global-hotkey` intercepte l'événement** selon le système d'exploitation sous-jacent (Win32, Carbon sur macOS, ou X11 sur Linux).
3. **L'orchestrateur déclenche l'action :**
   * **Flux Texte :** Copie via `arboard`, vérification par le filtre de sécurité de données sensibles, puis appel Pylos.
   * **Flux Image (xcap) :** Capture de la fenêtre active au format PNG.
4. **Archivage S3 (MinIO) :** L'image PNG est téléversée sur le serveur MinIO de destination (`https://minio-170-api.zacharie.org`, bucket `thoth-screenshots`) avec les clés d'accès configurées.
5. **Appel Multimodal (Vision) :** Pylos transmet l'image encodée en Base64 à **Gemini 3.5 Flash**. Le prompt système est optimisé pour n'obtenir que le préfixe de réponse correct (ex. "A" ou "3") ou la réponse la plus concise possible.
6. **Logique de Repli (Fallback) :** Si la détection d'image échoue ou ne trouve aucune question, Thoth simule `Ctrl+A`/`Cmd+A` puis `Ctrl+C`/`Cmd+C` pour copier l'ensemble du texte de la fenêtre et procède à une analyse textuelle.
7. **Journalisation et Notification :** Le couple question/réponse est consigné dans `question_answers.log` (format JSON Lines) et publié sur le broker EMQX MQTT (`mqtt-emqx.p.zacharie.org`) sur le topic `thoth/answers`.

---

## Sécurité et Gestion des Secrets

Thoth applique le principe de défense en profondeur pour protéger la confidentialité et l'intégrité du système de l'utilisateur :

*   **Chiffrement des Secrets au repos (`keyring`) :** Le mot de passe MQTT, la clé secrète MinIO et le token secret Pylos ne sont plus stockés en clair. Ils sont conservés dans le gestionnaire d'identifiants natif de l'OS (Keychain macOS, Secret Service Linux, Windows Credential Manager).
*   **Protection du dépôt (GitLeaks) :** Aucun mot de passe n'est stocké dans le dépôt Git. Les secrets de développement locaux sont stockés dans un fichier `.env` listé dans `.gitignore` (un gabarit `.env.example` est disponible). GitLeaks est configuré en tant que pre-commit hook et dans la CI GitHub Actions pour détecter et bloquer toute fuite de secret.
*   **Caviardage complet des logs :** Les logs généraux (`thoth.log`) ne contiennent aucun texte utilisateur ni information confidentielle, mais uniquement des empreintes de hachage et des métadonnées anonymisées.
*   **Données Sensibles (Regex Filter) :** Filtre de sécurité pré-envoi interceptant 11 expressions régulières sensibles (clés d'API OpenAI/AWS/GitHub, JWT, clés privées, numéros de cartes bancaires, etc.).
*   **Communications Chiffrées :** HTTPS obligatoire pour Pylos/MinIO et MQTTS (MQTT sur TLS avec port `8883`) obligatoire pour EMQX.

---

## Configuration et Raccourcis Clavier

### Emplacements de Configuration
Les configurations locales (`config.toml`, logs et captures temporaires) sont stockées dans les dossiers standards de l'OS via la caisse `directories` :
*   **Windows :** `%APPDATA%\thoth\`
*   **macOS :** `~/Library/Application Support/Thoth/`
*   **Linux :** `~/.config/thoth/`

### Table des Raccourcis Clavier

| Raccourci par défaut | Action | Description | Destination / Modèles |
|---|---|---|---|
| `Ctrl+Shift+Win+N` | Traduction | Traduit le texte sélectionné dans la langue cible | Pylos (Modèle principal) |
| `Ctrl+Shift+Win+,` | Traduction EN | Force la traduction vers l'anglais | Pylos |
| `Ctrl+Shift+Win+R` | Reformulation | Optimise le style et clarifie le texte sélectionné | Pylos |
| `Ctrl+Shift+Win+:` | Instruction GUI | Ouvre l'overlay egui pour saisir une consigne libre | Pylos (Historique persistant) |
| `Ctrl+Shift+Win+P` | Capture & MQTT | Capture la fenêtre active, résout les questions | MinIO S3 / Gemini 3.5 / EMQX MQTT |

---

## Intégration CI/CD Multi-OS

La compilation et la publication de Thoth sont automatisées via GitHub Actions (`ci.yml`) pour les trois plateformes majeures :
*   **Windows :** Génération du binaire autonome et de l'installateur compressé MSI (WiX 4) avec signature Authenticode.
*   **macOS :** Compilation pour architectures x86_64 et Apple Silicon (aarch64) avec signature de code d'application Apple.
*   **Linux :** Génération d'exécutables et de paquets de distribution standard (`.deb` / `tar.gz`).
*   **Contrôle qualité :** Lancement systématique de `cargo test`, `clippy`, `cargo-deny`, `typos` et du scan de fuite de secrets **GitLeaks**.
