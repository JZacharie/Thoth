# Privacy Policy / Politique de confidentialité

**Last updated / Dernière mise à jour : June 2026**

## English

### Overview

Thoth is a desktop system-tray application that captures selected text via global hotkeys and sends it to an LLM gateway for translation, reformulation, or custom processing. It also supports screenshot-based analysis via Gemini Vision.

### Data Collected

**Text selection** – When you press the configured hotkey, Thoth simulates `Ctrl+C` to copy the currently selected text from any application. This text is sent to the configured LLM endpoint (Pylos gateway or any OpenAI-compatible API) for processing.

**Screenshots** – When using the Vision hotkey, Thoth captures a screenshot of the active window. The image is uploaded to a configurable S3/MinIO bucket and sent to Gemini Vision for analysis. Results may be published to a configurable MQTT broker.

**Configuration** – API endpoints, model names, credentials, language preferences, and hotkey bindings are stored locally:
- **Windows**: Encrypted via DPAPI (`CryptProtectData`) in `HKCU\Software\Thoth`
- **macOS**: Keychain via the `keyring` crate
- **Linux**: Plain TOML file in the XDG config directory

**Usage metrics** – Anonymous statistics (translation count, error count, response latency, model used) are stored locally in a JSON file. These never leave your machine unless you explicitly configure otherwise.

### Data Sharing

Thoth sends data to the following configurable third-party services:

| Service | Purpose | Data sent |
|---------|---------|-----------|
| **Pylos gateway** (default: `pylos-dev.p.zacharie.org`) | LLM text processing | Selected text, language preference |
| **Gemini Vision** (via Pylos) | Image analysis | Screenshot image |
| **S3/MinIO** (default: `minio-170-api.zacharie.org`) | Screenshot storage | Screenshot image |
| **MQTT/EMQX** (default: `mqtt-emqx.p.zacharie.org`) | Result publication | Analysis text |

All endpoints, credentials, and service URLs are fully configurable in the settings GUI or config file. You can point Thoth to your own self-hosted infrastructure.

### Data Security

- API secrets are stored encrypted at rest (DPAPI on Windows, Keychain on macOS)
- All network communication uses TLS by default
- The application binary supports Authenticode signature verification on Windows release builds

### Your Control

- **`--insecure` flag**: Bypasses the signature check for development/self-signed builds
- Configuration editor: accessible via `thoth --config`
- All remote endpoints are freely configurable
- Local-only operation is possible when pointing to a local LLM

### Third-Party Links

Thoth does not embed third-party analytics, tracking, or telemetry. No data is sent to any service without explicit user action (hotkey press).

---

## Français

### Vue d'ensemble

Thoth est une application de bureau en arrière-plan qui capture du texte sélectionné via des raccourcis clavier et l'envoie à une passerelle LLM pour traduction, reformulation ou traitement personnalisé. Elle supporte également l'analyse d'images via Gemini Vision.

### Données collectées

**Sélection de texte** – Lorsque vous appuyez sur le raccourci configuré, Thoth simule un `Ctrl+C` pour copier le texte sélectionné dans n'importe quelle application. Ce texte est envoyé au point de terminaison LLM configuré (passerelle Pylos ou toute API compatible OpenAI) pour traitement.

**Captures d'écran** – Lors de l'utilisation du raccourci Vision, Thoth capture une capture d'écran de la fenêtre active. L'image est téléchargée vers un bucket S3/MinIO configurable et envoyée à Gemini Vision pour analyse. Les résultats peuvent être publiés vers un broker MQTT configurable.

**Configuration** – Points de terminaison API, noms de modèles, identifiants, préférences linguistiques et raccourcis sont stockés localement :
- **Windows** : Chiffré via DPAPI (`CryptProtectData`) dans `HKCU\Software\Thoth`
- **macOS** : Trousseau via la crate `keyring`
- **Linux** : Fichier TOML dans le répertoire de config XDG

**Métriques d'utilisation** – Des statistiques anonymes (nombre de traductions, nombre d'erreurs, latence, modèle utilisé) sont stockées localement dans un fichier JSON. Elles ne quittent jamais votre machine sauf configuration explicite.

### Partage des données

Thoth envoie des données aux services tiers configurables suivants :

| Service | Objectif | Données envoyées |
|---------|----------|------------------|
| **Passerelle Pylos** (défaut : `pylos-dev.p.zacharie.org`) | Traitement LLM | Texte sélectionné, langue cible |
| **Gemini Vision** (via Pylos) | Analyse d'image | Capture d'écran |
| **S3/MinIO** (défaut : `minio-170-api.zacharie.org`) | Stockage des captures | Capture d'écran |
| **MQTT/EMQX** (défaut : `mqtt-emqx.p.zacharie.org`) | Publication des résultats | Texte d'analyse |

Tous les points de terminaison, identifiants et URL de service sont entièrement configurables dans l'interface des paramètres ou le fichier de configuration. Vous pouvez pointer Thoth vers votre propre infrastructure auto-hébergée.

### Sécurité des données

- Les secrets API sont chiffrés au repos (DPAPI sur Windows, Trousseau sur macOS)
- Toutes les communications réseau utilisent TLS par défaut
- Le binaire supporte la vérification de signature Authenticode sur les versions release Windows

### Votre contrôle

- **Option `--insecure`** : Contourne la vérification de signature pour les builds de développement/auto-signés
- Éditeur de configuration : accessible via `thoth --config`
- Tous les points de terminaison distants sont librement configurables
- Un fonctionnement purement local est possible en pointant vers un LLM local

### Liens tiers

Thoth n'intègre pas d'analytique, de pistage ou de télémétrie tiers. Aucune donnée n'est envoyée à un service sans action explicite de l'utilisateur (pression d'un raccourci clavier).
