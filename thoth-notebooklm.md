# Thoth — Document de connaissance du projet

## Présentation générale

**Thoth** est une application système légère pour Windows, écrite en **Rust**, qui permet la manipulation de texte en temps réel grâce à des grands modèles de langage (LLM). Son nom est inspiré du dieu égyptien Thoth, divinité du savoir, de l'écriture et de la sagesse.

L'application s'exécute en arrière-plan sous forme d'un processus système Windows (sans fenêtre principale). Elle intercepte des **raccourcis clavier globaux**, capture le texte sélectionné dans n'importe quelle application ouverte, l'envoie à un LLM via une API HTTP, puis remplace le texte d'origine par la réponse du modèle — tout cela de manière transparente pour l'utilisateur, sans changer d'application.

### Cas d'usage principaux

- **Traduction instantanée** : sélectionner un texte en anglais dans un email, un document ou un navigateur, appuyer sur `Ctrl+Shift+Win+N`, et le texte est automatiquement traduit en français (ou toute autre langue configurée).
- **Traduction vers l'anglais** : raccourci dédié `Ctrl+Shift+Win+,` pour toujours traduire vers l'anglais, quelle que soit la langue configurée.
- **Reformulation / clarification** : réécrire un texte mal formulé, trop verbeux ou peu clair via `Ctrl+Shift+Win+R`.
- **Instruction personnalisée** : ouvrir un panneau GUI avec `Ctrl+Shift+Win+:`, saisir n'importe quelle instruction ("résume ce texte", "réécris en style formel", "réponds à cet email"), et le résultat remplace la sélection.

---

## Langage et technologies

### Rust

Thoth est entièrement écrit en **Rust** (édition 2024), avec une version minimale supportée de **1.88.0**. Le choix de Rust est justifié par :
- La faible empreinte mémoire (application légère en arrière-plan)
- La sécurité mémoire sans ramasse-miettes
- L'accès direct aux API Win32 via FFI
- La compilation en un seul binaire autonome (`.exe`) sans dépendances d'exécution

### Dépendances principales

| Crate | Rôle |
|---|---|
| `tokio` | Runtime asynchrone (moteur principal d'événements) |
| `reqwest` | Client HTTP/HTTPS pour appeler l'API LLM |
| `serde` / `serde_json` | Sérialisation/désérialisation JSON |
| `eframe` / `egui` | Interface graphique native (prompt, config, stats) |
| `arboard` | Gestion du presse-papier (lecture et écriture) |
| `rdev` | Simulation de frappes clavier (Ctrl+C, Ctrl+V) |
| `tray-icon` | Icône et menu dans la barre d'état système |
| `notify-rust` | Notifications toast Windows |
| `winreg` | Lecture/écriture dans le registre Windows |
| `windows-sys` | Liaison aux API Win32 (hotkeys, DPAPI, fenêtres) |
| `tracing` / `tracing-subscriber` | Journalisation structurée |
| `tracing-appender` | Écriture des logs dans un fichier (`thoth.log`) |
| `regex` | Détection de données sensibles dans le texte |
| `anyhow` / `thiserror` | Gestion d'erreurs ergonomique |
| `wiremock` | Mocking HTTP pour les tests d'intégration |

---

## Architecture du système

### Vue d'ensemble

```
┌──────────────────────────────────────────────────────────────────┐
│  Thoth (Processus Windows — Rust)                                │
│                                                                   │
│  ┌────────────┐   ┌──────────────┐   ┌───────────────────┐       │
│  │  Hotkey    │──▶│ Orchestrateur│──▶│ Client Pylos      │──▶     │
│  │  Listener  │   │ (boucle      │   │ (reqwest, HTTPS)  │  POST  │
│  │ (Register  │   │  principale) │   └───────────────────┘        │
│  │  HotKey)   │   │              │           │                    │
│  └────────────┘   │  ┌─────────┐ │           ▼                    │
│                   │  │Presse-  │ │    ┌──────────────┐            │
│  ┌────────────┐   │  │papier   │ │    │ Passerelle   │──▶▶ LLM   │
│  │  System    │   │  │(arboard)│ │    │ Pylos/Ollama │            │
│  │  Tray      │   │  └─────────┘ │    └──────────────┘            │
│  └────────────┘   └──────────────┘                                │
│                                                                   │
│  ┌────────────┐   ┌──────────────┐   ┌──────────────┐             │
│  │  GUI       │   │  Métriques   │   │ Notifications│             │
│  │  eframe    │   │  (JSON)      │   │(notify-rust) │             │
│  └────────────┘   └──────────────┘   └──────────────┘             │
│                                                                   │
│                   ┌──────────────────────────────────────┐        │
│                   │  Windows DPAPI (CryptProtectData)     │        │
│                   │  → HKCU\Software\Thoth\Config        │        │
│                   │  → HKCU\Software\Thoth\History       │        │
│                   └──────────────────────────────────────┘        │
└──────────────────────────────────────────────────────────────────┘
```

### Flux de traitement d'un raccourci clavier

Voici le flux complet, de l'appui sur le raccourci jusqu'au remplacement du texte :

1. **L'utilisateur appuie sur `Ctrl+Shift+Win+N`** dans n'importe quelle application (navigateur, éditeur, email, etc.)
2. **Windows déclenche un événement `WM_HOTKEY`** car Thoth a enregistré ce raccourci global via `RegisterHotKey` (API Win32).
3. **L'orchestrateur reçoit l'événement** via un canal Tokio `mpsc` et l'identifie comme `HotkeyAction::TranslateDefault`.
4. **Capture du texte** : Thoth sauvegarde le presse-papier actuel, simule `Ctrl+C` via `rdev`, et lit le contenu copié depuis le presse-papier.
5. **Vérification de sécurité** : le texte est analysé par le filtre de données sensibles. Si une clé API, un JWT, une clé privée, un numéro de carte bancaire ou une URI de base de données est détecté, la requête est bloquée et une notification d'avertissement est affichée.
6. **Appel LLM** : le `PylosClient` envoie une requête POST à `/v1/chat/completions` (format OpenAI) avec le texte et le prompt système approprié, via HTTPS, avec les en-têtes d'authentification.
7. **Réponse** : si le modèle principal échoue, un modèle de secours est automatiquement essayé.
8. **Collage** : le résultat est écrit dans le presse-papier, `SetForegroundWindow` remet le focus sur la fenêtre d'origine, puis `Ctrl+V` est simulé pour coller.
9. **Restauration** : le contenu original du presse-papier est restauré.
10. **Métriques** : le succès est enregistré (volume traité, latence, modèle utilisé) et une notification toast de succès est affichée.

---

## Modules du code source

### `src/main.rs` — Point d'entrée

Le point d'entrée initialise :
- Le **runtime Tokio** (asynchrone)
- La **journalisation** vers `thoth.log` avec filtre configurable via `RUST_LOG`
- Le **gestionnaire de panique** : en cas de crash, une boîte de dialogue native Windows propose d'ouvrir le fichier log
- La **vérification de signature Authenticode** au démarrage (builds de release uniquement) via `WinVerifyTrust` — bloque l'exécution si le binaire n'est pas signé correctement
- L'analyse des **arguments CLI** : `--config`, `--prompt`, `--stats`, `--insecure`
- Le **test de connexion** et de traduction au démarrage (traduit "Hello world" pour valider la configuration)

### `src/orchestrator.rs` — Boucle principale

L'orchestrateur est le cœur de l'application. Il tourne en boucle asynchrone, attend les événements de raccourcis clavier, puis coordonne :
- La capture de texte via le `ClipboardManager`
- La détection de données sensibles via `is_sensitive()`
- L'appel au `PylosClient` selon le type d'action
- Le collage du résultat
- L'enregistrement des métriques

Pour l'action `ExecuteInstruction` (raccourci `:`) , l'orchestrateur spawn un nouveau processus `thoth.exe --prompt` au lieu de traiter directement — cela permet au panneau GUI de s'ouvrir de façon indépendante.

### `src/pylos_client.rs` — Client LLM

Le `PylosClient` gère toute la communication avec l'API LLM. Ses responsabilités :

**Construction des prompts** : chaque action a son prompt système dédié.
- **Traduction** : *"Tu es un traducteur et correcteur de texte ultra-précis. Ta tâche est de traduire, corriger l'orthographe/grammaire et rendre le texte fourni clair et concis. Traduis le texte suivant en [langue]. Tu dois UNIQUEMENT retourner le texte corrigé et traduit. Ne commence JAMAIS ta réponse par des formules de politesse..."*
- **Instruction personnalisée** : *"Tu es un assistant personnel intelligent et ultra-précis. Analyse le texte fourni, identifie la consigne ou l'action demandée et exécute-la directement sur le reste du texte. Génère UNIQUEMENT la réponse ou le résultat final attendu..."*
- **Reformulation** : *"Tu es un rédacteur et communicant d'exception spécialisé dans la clarification textuelle. Ton objectif est de reformuler, clarifier les idées et restructurer le texte fourni pour le rendre extrêmement fluide, compréhensible et percutant, tout en préservant fidèlement son sens d'origine. Améliore le style, élimine les redondances et structure les arguments logiquement..."*

**Authentification** : chaque requête HTTP porte deux en-têtes :
- `X-Thoth-Secret: <secret>` — en-tête propriétaire de la passerelle Pylos
- `Authorization: Bearer <secret>` — en-tête standard OpenAI pour compatibilité maximale

**Logique de fallback** : si le modèle principal retourne une erreur HTTP, le client réessaie automatiquement avec `fallback_model`. Si les deux échouent, une erreur est propagée.

**Sanitisation des endpoints** : les suffixes `/v1/` et les doubles barres obliques sont supprimés automatiquement. Les endpoints non-localhost sont automatiquement convertis en HTTPS (sauf avec `--insecure`).

**Détection de données sensibles** (fonction `contains_sensitive_data`) — expressions régulières compilées à l'exécution :
- Clés OpenAI : `sk-[a-zA-Z0-9]{20,}` et `pk-[a-zA-Z0-9]{20,}`
- JWT : `eyJ...eyJ...` (structure à trois parties encodée Base64)
- Clés privées : `-----BEGIN * PRIVATE KEY-----`
- Numéros de carte bancaire : 4 blocs de 4 chiffres séparés par espace ou tiret
- Clés AWS : `AKIA[A-Z0-9]{16}`
- Tokens GitHub : `gh[pousr]_[a-zA-Z0-9]{36,255}`
- Tokens Slack : `xox[bp]-[a-zA-Z0-9-]{10,}`
- URIs de bases de données : `mongodb://`, `postgres://`, `mysql://`

### `src/config.rs` — Configuration chiffrée

La configuration est une structure TOML avec deux sections :

**Section `[pylos]`** :
- `endpoint` : URL de la passerelle LLM (défaut: `https://pylos-dev.p.zacharie.org`)
- `model` : nom du modèle principal (défaut: `gemini4:e2b`)
- `fallback_model` : modèle de secours optionnel (défaut: `gemma4:12b`)
- `timeout_secs` : timeout HTTP en secondes (défaut: 30)
- `secret` : clé d'API, UUID auto-généré au premier démarrage

**Section `[behavior]`** :
- `target_language` : code ISO 639-1 de la langue cible (détecté depuis la locale système)
- `restore_clipboard` : restaurer le presse-papier après chaque opération (défaut: true)
- `show_notifications` : afficher les notifications toast (défaut: true)
- `debounce_ms` : délai anti-rebond pour les raccourcis (défaut: 500ms)
- `hotkey` : raccourci principal configurable (défaut: `Ctrl+Shift+Win+N`)

**Stockage** : la config est sérialisée en TOML, **chiffrée via Windows DPAPI** (`CryptProtectData`), et stockée en `REG_BINARY` dans `HKCU\Software\Thoth\Config`. Aucun fichier en clair n'existe sur le disque. L'historique des instructions est stocké dans `HKCU\Software\Thoth\History`.

### `src/hotkey.rs` — Gestion des raccourcis clavier

Le module hotkey gère l'enregistrement des raccourcis globaux via `RegisterHotKey` (Win32). Il supporte :
- **Modificateurs** : `Win`, `Ctrl`, `Alt`, `Shift` (combinables)
- **Touches** : lettres A-Z, chiffres 0-9, touches spéciales (`Space`, `F1`-`F24`, `Comma`, `Semicolon`, `Colon`)

Le parser de pattern (`HotkeyPattern::parse`) prend une chaîne comme `"Ctrl+Shift+Win+N"` et la décompose en modificateurs et touche principale. Quatre raccourcis sont enregistrés simultanément :
1. Raccourci principal (configurable) → `TranslateDefault`
2. Raccourci principal + `,` → `TranslateEnglish`
3. Raccourci principal + `:` → `ExecuteInstruction`
4. Raccourci principal + `R` → `Reformulate`

### `src/gui.rs` — Interface graphique

L'interface est implémentée avec **eframe/egui** (rendu natif OpenGL via WGL). Elle comprend trois panneaux distincts, chacun pouvant être ouvert indépendamment :

**Panneau de prompt** (`--prompt`) :
- Champ de saisie multi-ligne pour entrer l'instruction
- Historique des instructions précédentes navigable (flèches haut/bas, clic)
- Le texte sélectionné est automatiquement récupéré depuis le presse-papier
- Appuyer sur `Entrée` ou cliquer "Exécuter" déclenche l'appel LLM
- Le résultat remplace la sélection dans l'application d'origine

**Éditeur de configuration** (`--config`) :
- Champs pour endpoint, modèle, modèle fallback, timeout, secret
- Sélecteur de langue cible (menu déroulant)
- Champ pour le raccourci principal
- Cases à cocher pour restauration clipboard et notifications
- Bouton de sauvegarde qui chiffre et stocke dans le registre

**Tableau de bord statistiques** (`--stats`) :
- Nombre total de traductions réussies
- Nombre d'erreurs
- Volume total traité (en caractères ou bytes)
- Latence moyenne
- Répartition d'usage par modèle

### `src/tray.rs` — Icône système

L'icône dans la barre d'état système (zone de notification Windows) expose un menu contextuel avec :
- Accès à la configuration
- Accès aux statistiques
- Activation/désactivation du démarrage automatique
- Réinitialisation des statistiques
- Quitter

### `src/metrics.rs` — Métriques d'utilisation

Les métriques sont sérialisées en **JSON** et persistées dans un fichier local. Elles enregistrent :
- Compteur de traductions réussies
- Compteur d'erreurs
- Volume total de texte traité
- Latence cumulée (pour calculer la moyenne)
- Dictionnaire d'usage par nom de modèle

### `src/clipboard.rs` — Gestion du presse-papier

Le `ClipboardManager` encapsule `arboard` pour lire et écrire dans le presse-papier Windows. Il gère également :
- La sauvegarde du contenu original avant chaque opération
- La restauration après l'opération (si activée)
- La simulation de `Ctrl+C` via `rdev` pour forcer la copie du texte sélectionné
- La simulation de `Ctrl+V` pour coller le résultat

### `src/notification.rs` — Notifications

Trois types de notifications toast via `notify-rust` :
- `notify_success()` : confirmation discrète après une traduction/reformulation réussie
- `notify_error(message)` : alerte en cas d'échec (connexion, paste, clipboard...)
- `notify_warning(message)` : avertissement en cas de données sensibles détectées

### `src/auto_start.rs` — Démarrage automatique

Gère l'entrée dans `HKCU\Software\Microsoft\Windows\CurrentVersion\Run` pour démarrer Thoth automatiquement avec Windows.

---

## Sécurité

### Principe de défense en profondeur

Thoth applique plusieurs couches de sécurité indépendantes :

| Couche | Mécanisme | Détail |
|---|---|---|
| **Configuration au repos** | DPAPI (`CryptProtectData`) | Chiffrement lié à l'identité Windows de l'utilisateur — impossible à déchiffrer sur un autre compte |
| **Transport** | HTTPS enforced | Tout endpoint non-localhost est automatiquement promu en `https://`; la vérification TLS est activée par défaut |
| **Authentification API** | Double en-tête | `X-Thoth-Secret` + `Authorization: Bearer` sur chaque requête |
| **Données sensibles** | Filtrage regex pré-envoi | 11 patterns couvrant clés API, JWT, clés privées, cartes bancaires, tokens Slack, URIs DB |
| **Logs** | Caviardage complet | Jamais de texte utilisateur dans les logs — seulement `(len: N, hash: 0x...)` |
| **Intégrité binaire** | WinVerifyTrust | Vérifie la signature Authenticode au démarrage (release uniquement) |
| **Saisie utilisateur** | Parser strict | Validation du pattern de hotkey avant tout appel Win32 |
| **Interface** | Native uniquement | Pas de PowerShell, pas de HTA, pas de WebView — uniquement eframe/egui natif |

### Gestion des secrets

Le `secret` de configuration est un UUID généré aléatoirement au premier démarrage. Il est stocké chiffré dans le registre. Il n'est jamais loggé. Il est transmis en HTTPS via les deux en-têtes d'authentification.

---

## Intégration avec la passerelle Pylos / Ollama

### API utilisée

Thoth consomme l'API de complétion de chat au format **OpenAI** :
```
POST /v1/chat/completions
Content-Type: application/json
X-Thoth-Secret: <secret>
Authorization: Bearer <secret>

{
  "model": "gemini4:e2b",
  "messages": [
    {"role": "system", "content": "<prompt système>"},
    {"role": "user",   "content": "<texte à traiter>"}
  ]
}
```

La réponse attendue suit le format standard OpenAI :
```json
{
  "choices": [
    {"message": {"content": "<résultat LLM>"}}
  ]
}
```

### Test de connexion au démarrage

Au démarrage, Thoth effectue deux vérifications :
1. **Test de connectivité** : GET `/v1/models` — vérifie que l'endpoint est joignable
2. **Test de traduction** : traduit la chaîne "Hello world" — valide que le modèle est opérationnel

En cas d'échec, un avertissement est loggé mais l'application démarre quand même (pour permettre la configuration).

### Compatibilité

Thoth est compatible avec tout service exposant l'API `/v1/chat/completions` au format OpenAI :
- **Pylos** (passerelle interne à Zacharie.org)
- **Ollama** (`http://localhost:11434`)
- **LM Studio**, **Jan**, ou tout autre serveur local OpenAI-compatible
- Les APIs cloud directes (OpenAI, Anthropic via wrapper OpenAI, etc.)

---

## Pipeline CI/CD

Le pipeline GitHub Actions (`ci.yml`) est organisé en jobs séquentiels :

| Job | Outil | Objectif |
|---|---|---|
| `lint` | actionlint + typos | Vérification de la syntaxe des workflows et des fautes de frappe |
| `check` | fmt + clippy + nextest + cargo-deny | Qualité de code, tests, licences et vulnérabilités |
| `msrv` | Rust 1.88.0 | Vérifie la compatibilité avec la version minimale supportée |
| `build` | cargo build --release | Compilation du binaire de production |
| `msi` | WiX 4 | Construction de l'installeur MSI Windows (push sur main) |
| `msi-release` | WiX 4 | MSI signé pour les releases taggées (`v*`) |
| `release` | softprops/action-gh-release | Publication GitHub Release avec binaire + MSI + SHA256 |

Les caches Cargo (registry + artefacts de compilation) sont partagés entre jobs via `sccache` et `Swatinem/rust-cache`.

---

## Structure du projet

```
Thoth/
├── src/
│   ├── main.rs          # Point d'entrée, CLI, panic handler, signature check
│   ├── lib.rs           # Ré-exports publics, flag global --insecure
│   ├── config.rs        # Structures de config, DPAPI, registre Windows
│   ├── orchestrator.rs  # Boucle principale d'événements
│   ├── clipboard.rs     # Presse-papier + simulation Ctrl+C/V
│   ├── pylos_client.rs  # Client HTTP, prompts LLM, filtre sensible, fallback
│   ├── hotkey.rs        # Enregistrement hotkey Win32, parsing pattern
│   ├── gui.rs           # GUI eframe/egui (prompt, config, stats)
│   ├── dialog.rs        # Point d'entrée legacy pour le prompt GUI
│   ├── tray.rs          # Icône et menu barre d'état système
│   ├── notification.rs  # Notifications toast Windows
│   ├── metrics.rs       # Statistiques d'utilisation (JSON)
│   └── auto_start.rs    # Démarrage automatique (registre Windows)
├── tests/
│   └── integration.rs   # Tests d'intégration avec wiremock (mock HTTP)
├── installer/
│   └── Thoth.wxs        # Définition WiX 4 de l'installeur MSI
├── resources/           # Icône et ressources Windows
├── .github/
│   └── workflows/
│       └── ci.yml       # Pipeline CI/CD complet
├── Cargo.toml           # Manifeste Rust (MSRV: 1.88)
├── deny.toml            # Règles cargo-deny (licences, advisories)
├── .typos.toml          # Dictionnaire typos-cli
└── local-ci.ps1         # Script CI local (Windows PowerShell)
```

---

## Tests

### Tests unitaires

Les tests unitaires sont co-localisés dans chaque module source (`#[cfg(test)]`) :
- **`pylos_client.rs`** : 8 tests couvrant la sanitisation d'endpoint, les noms de langues, la détection de clé API (OpenAI `sk-`/`pk-`), JWT, clé SSH privée, carte bancaire (avec et sans tirets/espaces), texte normal non détecté, texte vide, et la fonction publique `is_sensitive`
- Tests de la logique de parsing des hotkeys dans `hotkey.rs`
- Tests de validation des langues supportées dans la configuration

### Tests d'intégration

Le fichier `tests/integration.rs` utilise **wiremock** pour simuler la passerelle Pylos. Les tests vérifient :
- La traduction réussie : le mock retourne une réponse JSON valide
- L'en-tête `X-Thoth-Secret` est bien présent dans chaque requête
- Le fallback automatique sur le modèle de secours en cas d'erreur du modèle principal
- La reformulation réussie
- Le rejet des instructions sur données sensibles

### Script CI local

`local-ci.ps1` reproduit le pipeline complet localement en 9 étapes :
1. `cargo fmt --check`
2. `cargo clippy -- -D warnings`
3. `cargo test`
4. `cargo build`
5. `cargo deny check`
6. `cargo audit`
7. `cargo outdated`
8. `typos`
9. `cargo udeps`

---

## Configuration et démarrage rapide

### Installation depuis les sources

```bash
git clone https://github.com/JZacharie/Thoth.git
cd Thoth
cargo build --release
./target/release/thoth.exe
```

### Configuration initiale

Thoth génère automatiquement une configuration sécurisée au premier démarrage. Pour la modifier :

```bash
# Ouvrir l'éditeur de configuration
./target/release/thoth.exe --config
```

Ou via le menu clic-droit sur l'icône dans la barre d'état système.

### Arguments de ligne de commande

| Argument | Effet |
|---|---|
| *(aucun)* | Démarre le service en arrière-plan avec l'écouteur de raccourcis |
| `--prompt` | Ouvre le panneau de saisie d'instruction (GUI overlay always-on-top) |
| `--config` | Ouvre l'éditeur de configuration GUI |
| `--stats` | Ouvre le tableau de bord des statistiques |
| `--insecure` | Désactive la vérification TLS et l'upgrade HTTPS (développement local uniquement) |

### Journalisation

```bash
# Niveau debug pour tous les modules
RUST_LOG=debug  ./target/release/thoth.exe

# Niveau trace pour le module hotkey uniquement
RUST_LOG=thoth::hotkey=trace  ./target/release/thoth.exe
```

Les logs sont écrits dans `thoth.log` à côté de l'exécutable. Niveau par défaut : `info`. **Aucun texte utilisateur n'est jamais loggé** — uniquement des longueurs et empreintes de hachage.

---

## Raccourcis clavier par défaut

| Action | Raccourci | Description |
|---|---|---|
| Traduire (langue configurée) | `Ctrl+Shift+Win+N` | Traduit le texte sélectionné vers la langue cible |
| Traduire en anglais | `Ctrl+Shift+Win+,` | Force la traduction vers l'anglais |
| Instruction personnalisée | `Ctrl+Shift+Win+:` | Ouvre le panneau GUI pour saisir une instruction |
| Reformuler | `Ctrl+Shift+Win+R` | Reformule le texte pour plus de clarté et de fluidité |

Tous ces raccourcis partagent la même base (`Ctrl+Shift+Win` par défaut), configurable via `behavior.hotkey`.

---

## Langues supportées

| Code | Langue |
|---|---|
| `fr` | Français |
| `en` | Anglais |
| `es` | Espagnol |
| `de` | Allemand |
| `it` | Italien |
| `pt` | Portugais |
| `nl` | Néerlandais |
| `ja` | Japonais |
| `zh` | Chinois |
| `ru` | Russe |

La langue cible est détectée automatiquement depuis la locale système au premier démarrage.

---

## Licence

MIT — projet open source de J. Zacharie.
