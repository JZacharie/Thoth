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

---

## Plan d'action stratégique (Mois 1 - 7)

Pour mener à bien ce projet, le secret réside dans une approche **itérative et modulaire**. L'architecture doit séparer strictement l'intelligence (le moteur LLM et la logique métier) des spécificités graphiques de chaque OS.

Voici le plan d'action stratégique, découpé en 5 phases majeures, pour passer du concept à un outil de production performant.

---

### Phase 1 : Architecture & Cœur Mutualisé (Mois 1 - 2)

L'objectif est de bâtir les fondations techniques de l'application sans encore toucher à l'interface graphique.

*   **Choix de la pile technique :**
    *   **Core Logic :** Développement en **Rust**. C’est le choix idéal pour un outil système : empreinte mémoire minimale (< 50 Mo), sécurité des threads, et interopérabilité parfaite avec les APIs natives via C-bindings.
    *   **Moteur LLM & Routage :** Intégration d'un client API double (Local via un modèle léger comme *Llama-3-8B* via `llama.cpp` / *Ollama* pour la confidentialité, et distant via des APIs cloud pour les analyses complexes).
*   **Développement du Core Engine :**
    *   Mise en place de la gestion du cache des traductions/explications (PostgreSQL en local ou SQLite ultra-léger) pour éviter de requêter le LLM deux fois pour la même phrase.
    *   Création des algorithmes de traitement de texte (nettoyage des chaînes issues de l'OCR, détection des blocs de code).
    *   Création du système de binding (FFI) pour permettre aux futurs *frontends* de communiquer avec ce cœur en Rust.

---

### Phase 2 : R&D "OS Gateways" & Capture Contextuelle (Mois 3)

Cette phase valide la faisabilité technique de la capture passive sur chaque plateforme.

*   **Implémentation des briques de capture de texte/OCR :**
    *   **macOS :** Intégration du framework natif *Vision* pour l'OCR et des APIs d'accessibilité (`AXUIElement`).
    *   **Windows :** Implémentation de *Windows.Media.Ocr* et de l'arbre *UI Automation*.
    *   **Linux :** Développement du support pour *X11* (xdotool/Tesseract) et initialisation du pont *Wayland* via le portail *AT-SPI* ou les flux *PipeWire*.
*   **Création du gestionnaire de contexte :**
    *   Développement du module capable d'identifier l'application active (ex: si le processus parent est `Code.exe` ou `Cursor`, adapter le prompt du LLM en mode "Développement").

---

### Phase 3 : Développement des Interfaces Natives (Overlays) (Mois 4 - 5)

C'est ici que l'on crée l'expérience "Incrustation en premier plan" sans Electron, pour garantir une fluidité absolue.

*   **Création des 3 Frontends en parallèle :**
    *   **macOS UI :** Projet Swift / AppKit (`NSPanel` flottant, effet de flou natif vibré).
    *   **Windows UI :** Projet C# / WPF ou WinUI 3 (Styles de fenêtres étendus `WS_EX_TRANSPARENT`, effets *Mica*/*Acrylic*).
    *   **Linux UI :** Projet C++ / Qt6 ou GTK4 (Gestion des fenêtres de type dock/layer shell avec gestion de la transparence transparente au clic).
*   **Interfaçage :** Connecter ces interfaces graphiques aux fonctions de capture et au cœur Rust développé en Phase 1.

---

### Phase 4 : Ergonomie, UX Fine & Optimisations (Mois 6)

L'outil fonctionne, il faut maintenant le rendre agréable et non-intrusif.

*   **Comportement de l'Overlay :**
    *   Développement de l'algorithme de "Changement de focus" (la fenêtre s'estompe dès que la souris s'éloigne).
    *   Implémentation de l'ancrage magnétique (la bulle d'explication "suit" le déplacement de la fenêtre cible).
    *   Intégration du système de *Click-Through* (pouvoir cliquer à travers l'overlay si l'utilisateur interagit avec l'application du dessous).
*   **Optimisation des performances :**
    *   Chasse aux fuites mémoire et réduction du temps de latence de l'OCR (objectif : < 150ms entre le raccourci clavier et l'affichage des premières suggestions).

---

### Phase 5 : Phase Pilote & Déploiement (Mois 7)

*   **Beta Test Fermé :** Test de l'outil en conditions réelles (par exemple, sur des flux de travail quotidiens en développement logiciel, analyse de logs, ou lecture de documentations denses).
*   **Ajustement des Prompts du LLM :** Raffinement des instructions système (System Prompts) pour s'assurer que les explications fournies en incrustation soient courtes, percutantes et structurées sous forme de puces (Markdown léger).
*   **Packaging :** Création des installeurs natifs (`.dmg` notarisé pour Mac, `.msi` pour Windows, paquet `.deb` / Flatpak pour Linux).

---

### Résumé du planning de livraison

```
Mois 1 & 2 ────────────────► [Phase 1: Cœur Rust & Logique LLM]
Mois 3     ────────────────► [Phase 2: R&D Capture & OCR par OS]
Mois 4 & 5 ────────────────► [Phase 3: Développement des Interfaces Natives]
Mois 6     ────────────────► [Phase 4: UX Fine, Ancrage & Performance]
Mois 7     ────────────────► [Phase 5: Beta, Packaging & Release]
```

Ce plan d'action permet de valider les plus gros risques techniques (notamment la capture sous Wayland pour Linux et le non-vol de focus sur Windows/Mac) dès le premier tiers du projet, garantissant ainsi la viabilité de l'outil.

