# Thoth — Project Context for BMAD

## Project Overview

Thoth is a lightweight, high-performance system application written in Rust for Windows. It acts as an instant translation, grammar correction, and text optimization tool.

When the user triggers the global shortcut `Win + N`:
1. Thoth simulates a copy operation (`Ctrl + C`) to copy the currently selected text.
2. It retrieves the text from the Windows clipboard.
3. It sends the text to the local **Pylos** gateway (`POST /v1/chat/completions`) using the `gemma4:12b` (or `gemini4:12b`) model.
4. Using a strict system prompt, it ensures that the LLM only outputs the translated/corrected text (no preamble, no conversational comments).
5. It writes the result back to the Windows clipboard.
6. It simulates a paste operation (`Ctrl + V`) to replace the user's original selection.

## Tech Stack

| Layer | Technology |
|---|---|
| Language | Rust (2021 edition) |
| OS | Windows 10/11 |
| Async runtime | Tokio |
| Clipboard API | arboard |
| Global Hook | rdev / inputbot |
| HTTP Client | reqwest |

## Twelve-Factor App Compliance

Thoth adheres to the [12-factor app](https://12factor.net/) methodology wherever applicable for a desktop system utility:

### I. Codebase
One codebase tracked in Git, many deployments. Every Windows machine running Thoth uses the same GitHub release artifact. Branches and tags (`v*`) manage the release workflow via CI/CD.

### II. Dependencies
All Rust dependencies are explicitly declared in `Cargo.toml` with pinned minor versions. `cargo build --release` produces a statically linked binary with zero external runtime dependencies. `cargo audit` and `cargo outdated` run in CI to verify integrity and freshness.

### III. Configuration
All environment-specific values (Pylos endpoint URL, model name, system prompt, timeout) are extracted into a `Config` struct loaded at startup — no hardcoded paths or secrets in code. Secrets (API keys) are inherited from environment variables. Future iterations will support a config file in `%APPDATA%/thoth/`.

### IV. Backing Services
Pylos (the LLM gateway) is treated as an attached resource — the endpoint URL is configurable, and Thoth fails gracefully (logs the error, no crash) if Pylos is unreachable. No distinction between local and remote Pylos; config change is sufficient to switch.

### V. Build, Release, Run
- **Build**: `cargo build --release` in CI produces `thoth.exe`.
- **Release**: GitHub Release attaches the binary + SHA256 checksum on every `v*` tag.
- **Run**: The user downloads the release artifact and executes it. No build step on the target machine.

### VI. Processes
Thoth runs as a single, stateless, long-lived background process. It maintains no in-memory state between hotkey invocations — each `Win + N` cycle is independent. The system tray icon allows the user to quit cleanly.

### VII. Port Binding
N/A — Thoth is a desktop GUI-less application, not a network service. It connects *outbound* to Pylos (port 3000), but does not bind to any port itself.

### VIII. Concurrency
Thoth scales vertically within a single process via Tokio's async runtime. The hotkey listener, clipboard operations, and HTTP client all share the same Tokio reactor — no thread-per-request overhead. The process model is not horizontally scalable (single-user desktop app).

### IX. Disposability
- **Fast startup**: Binary loads and registers the global hotkey in < 500ms. No database migrations, no network waits.
- **Graceful shutdown**: The system tray "Quitter" triggers a clean shutdown via Tokio's `oneshot` channel. All resources (clipboard, keyboard hook) are released.
- **Crash resilience**: Errors in the hotkey cycle are logged and swallowed — the orchestrator continues listening. No crash can leave the system in an inconsistent state.

### X. Dev/Prod Parity
- **Same binary**: The same `cargo build --release` artifact runs in dev and prod.
- **Same backing service**: Pylos is always local (localhost:3000) in both environments.
- **Same Rust toolchain**: CI uses `dtolnay/rust-toolchain@stable`, matching the developer's local setup.
- CI runs `cargo test` and `cargo clippy` on the same `windows-latest` runner, eliminating OS divergence.

### XI. Logs
Thoth treats logs as event streams via the `tracing` crate. Logs are written to stdout (captured by the OS) and optionally to a file in `%APPDATA%/thoth/logs/` with automatic rotation. No log parsing logic is built into the application — external tools (or the user) consume the raw stream. CI fails on `tracing::error!` events when configured.

### XII. Admin Processes
Admin tasks (config validation, hotkey test, connectivity check) are designed as one-off CLI flags:
- `thoth --check-config` — validate the configuration file.
- `thoth --test-pylos` — ping Pylos and report connectivity.
- `thoth --version` — print the current version.
These run in the same codebase and environment as the main process.

## Development Conventions

- Run all tests: `cargo test`
- Format: `cargo fmt`
- Lint: `cargo clippy`
- Build: `cargo build --release`

## Performance Budget

| Métrique | Cible |
|---|---|
| Taille du binaire | < 5 MB |
| Latence cycle complet (hotkey → collage) | < 2 s |
| Mémoire résidente | < 50 MB |
| Startup time (hotkey enregistré) | < 500 ms |

## QA Checklist (Validation Manuelle Pré-Release)

Avant chaque release, valider Thoth sur les applications suivantes :

- [ ] Chrome / Edge — champ de texte libre, zone de saisie web
- [ ] Notepad — texte brut
- [ ] VS Code — éditeur de code
- [ ] Microsoft Teams — chat
- [ ] Microsoft Word — document formaté
- [ ] Outlook — composition d'email
- [ ] Terminal (cmd, PowerShell) — ligne de commande

Cas de test :
- [ ] Texte court (1 mot) → traduction instantanée
- [ ] Texte long (500+ mots) → pas de timeout
- [ ] Pas de texte sélectionné → pas d'action
- [ ] Pylos hors ligne → notification d'erreur, pas de crash
- [ ] Presse-papier restauré après opération
- [ ] Hot-reload config → nouveau modèle utilisé sans redémarrage
- [ ] Désactiver/Réactiver depuis la system tray
- [ ] Quitter proprement depuis le menu tray

## Debugging & Crash Reporting

- Logs : `%APPDATA%/thoth/logs/` (rotation automatique)
- Crash reports : `%APPDATA%/thoth/crash-reports/` (généré par hook panic)
- Backtrace structurée via `tracing-error`
- Rapport de crash = timestamp + version + backtrace + dernier événement tracing
