# Spécifications d'Architecture et Sécurité : Capture d'Écran & MQTT Multi-OS

Ce document définit les paramètres requis, l'architecture logicielle multi-OS et les mesures de sécurité nécessaires pour l'implémentation de la fonctionnalité de capture d'écran, d'analyse par LLM Vision (Gemini 3.5 Flash) et de publication MQTT.

---

## 1. Paramètres Requis (Configuration)

Les paramètres suivants doivent être ajoutés dans le fichier de configuration `config.toml` (et éditables via l'interface GUI) :

### A. Paramètres de Stockage et MinIO S3
* `behavior.screenshot_history_limit` : Nombre maximal de captures d'écran à conserver en local (Défaut : `100`, pour éviter de saturer le disque).
* `behavior.question_answers_log_path` : Chemin du fichier de journalisation JSON (Défaut : `question_answers.log` dans le dossier de configuration).
* **MinIO S3 (Stockage Distant des Captures) :**
  * `s3.endpoint` : `https://minio-170-api.zacharie.org` (API MinIO).
  * `s3.bucket` : `thoth-screenshots` (Nom du bucket pour l'archivage).
  * `s3.access_key` : `joseph` (Identifiant d'accès MinIO).
  * `s3.secret_key` : Référencé dans le fichier local `.env` (variable `MINIO_SECRET_KEY`). **Ne doit jamais être commité dans Git.**

### B. Paramètres MQTT (Défaut)
* `mqtt.enabled` : Booléen (`true`/`false`) pour activer/désactiver l'envoi MQTT.
* `mqtt.broker` : `mqtt-emqx.p.zacharie.org` (Broker EMQX par défaut).
* `mqtt.port` : Port du broker (Défaut : `1883` / `8883` pour MQTTS).
* `mqtt.topic` : Topic de publication (Défaut : `thoth/answers`).
* `mqtt.client_id` : Identifiant client unique (Défaut : `thoth_[random_id]`).
* `mqtt.username` : `joseph` (Login MQTT par défaut).
* `mqtt.password` : Référencé dans le fichier local `.env` (variable `MQTT_PASSWORD`). **Ne doit jamais être commité dans Git.**

---

## 2. Architecture Multi-OS Compatible

Pour assurer le bon fonctionnement sur Windows, macOS et Linux, l'implémentation reposera sur les technologies multiplateformes suivantes :

```
┌─────────────────────────────────────────────────────────────────────────┐
│                          Thoth Core Application                         │
└─────────────────────────────────────────────────────────────────────────┘
        │                            │                            │
        ▼                            ▼                            ▼
┌──────────────────┐       ┌──────────────────┐       ┌──────────────────┐
│  Input & Hotkey  │       │ Window Capture   │       │ Secure Storage   │
│  (global-hotkey) │       │     (xcap)       │       │    (keyring)     │
└──────────────────┘       └──────────────────┘       └──────────────────┘
  - Windows: Win32           - Windows: GDI             - Windows: CredMgr
  - macOS: Carbon            - macOS: CoreGraphics      - macOS: Keychain
  - Linux: X11/Wayland       - Linux: X11 / PipeWire    - Linux: SecretServ
```

### A. Capture de la fenêtre active (`xcap`)
* **Windows :** Exploite les API GDI/User32 internes pour capturer la fenêtre au premier plan.
* **macOS :** Exploite l'API `CoreGraphics` (nécessite l'autorisation "Enregistrement d'écran" dans les préférences système).
* **Linux :** Supporte X11 (via `Xlib`). Sous Wayland, la capture dépend de portails système (comme PipeWire / Desktop Portal). Une invite de capture peut s'afficher selon les distributions.

### B. Simulation clavier (Copier/Coller)
* Abstraction via un module `platform_keys` :
  * Cibles Windows et Linux : Modificateur `Control` (`rdev::Key::ControlLeft`).
  * Cible macOS : Modificateur `Command` (`rdev::Key::MetaLeft`).

---

## 3. Solution Sécurisée

### A. Chiffrement des identifiants (MQTT & LLM & S3)
* **Pas de secret en clair :** Le mot de passe MQTT, la clé secrète MinIO et le token secret Pylos ne doivent pas être stockés en texte brut dans le fichier TOML.
* **Intégration de `keyring` :** Utiliser la caisse Rust `keyring` pour stocker ces secrets dans le gestionnaire sécurisé natif de l'OS :
  * Windows Credential Manager.
  * macOS Keychain Access.
  * Linux Secret Service (via D-Bus / `libsecret`).

### B. Transport Réseau Chiffré (MQTTS & HTTPS)
* **MQTTS obligatoire par défaut :** Chiffrement TLS. Les informations de questions/réponses capturées sur l'écran de l'utilisateur contiennent des données sensibles et ne doivent jamais transiter en texte clair sur le réseau local.
* **HTTPS forcé pour Pylos et MinIO :** Utilisation des endpoints HTTPS (`minio-170-api.zacharie.org` et `pylos-dev.p.zacharie.org`) avec vérification des certificats.

### C. Gestion du cycle de vie des captures d'écran
* **Sauvegarde distante & Nettoyage local :** Téléverser les captures vers le bucket MinIO S3 `thoth-screenshots`. Une fois l'envoi validé, la capture locale peut être immédiatement purgée ou conservée selon une rotation stricte (`screenshot_history_limit`) pour protéger la confidentialité locale de l'appareil.
* **Format des Logs :** Enregistrement local sous format JSON Lines (`.jsonl`) pour simplifier les analyses automatisées ultérieures.

---

## 4. Gestion des Secrets et GitLeaks

### A. Utilisation du fichier `.env`
* **Exclusion stricte :** Le fichier `.env` (contenant les variables d'environnement de secrets telles que `MINIO_SECRET_KEY` et `MQTT_PASSWORD`) doit être listé dans `.gitignore`.
* **Modèle de configuration :** Fournir un fichier exemple non sensible `example.env` (ou `.env.example`) décrivant les clés requises.

### B. Intégration de GitLeaks
* **Détection proactive :** Configurer un outil de détection de secrets **GitLeaks** pour analyser l'historique des commits et bloquer tout push contenant des secrets.
* **GitHub Actions :** Ajouter un job GitLeaks dans le workflow CI `.github/workflows/ci.yml` pour valider chaque PR et push sur la branche principale :
  ```yaml
  gitleaks-scan:
    name: gitleaks
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
        with:
          fetch-depth: 0
      - uses: gitleaks/gitleaks-action@v2
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
  ```
* **Pre-commit Hook :** Encourager les développeurs à exécuter `gitleaks protect --staged` localement avant chaque commit.
