# Thoth — Instant LLM-Powered Text Manipulation

## Overview

Thoth is a cross-platform desktop system-tray application written in Rust that provides instant LLM-powered text manipulation via global hotkeys. Users select text in any application, press a hotkey, and the selected text is sent to an LLM gateway for translation, reformulation, or custom processing — then automatically pasted back.

## Problem Statement

Users frequently switch between applications to copy text, open a browser, paste into ChatGPT/Google Translate, copy the result, and paste it back. This context-switching breaks workflow and wastes time. Thoth eliminates these steps by bringing LLM capabilities directly into any application via a single keystroke.

## Target Audience

- Knowledge workers who frequently translate or reformulate text
- Developers, writers, and multilingual professionals
- Power users who want keyboard-driven AI assistance
- Anyone using LLMs regularly for text tasks

## Key Features

### 5 Configurable Global Hotkeys

| Action | Default Hotkey |
|---|---|
| Translate to configured language | Ctrl+Shift+Win+N |
| Translate to English | Ctrl+Shift+Win+, |
| Custom prompt GUI overlay | Ctrl+Shift+Win+: |
| Reformulate/rewrite text | Ctrl+Shift+Win+R |
| Screenshot analysis | Ctrl+Shift+Win+P |

All hotkeys are user-configurable. Supported modifiers: Win, Ctrl, Alt, Shift + any letter, number, F1-F24, or punctuation key.

### Automatic Clipboard Workflow

1. User selects text in any application
2. Presses hotkey
3. Thoth simulates Ctrl+C to capture the selection
4. Sends text to LLM via HTTPS
5. Receives response and writes it to clipboard
6. Simulates Ctrl+V to paste the result
7. Restores original clipboard content

### 10 Target Languages

French, English, Spanish, German, Italian, Portuguese, Dutch, Japanese, Chinese, Russian

### Screenshot Analysis

Captures the active window, uploads to S3/MinIO, sends to Gemini Vision for analysis, and pastes the result. Supports configurable system prompt for custom analysis instructions.

### Native GUI (eframe/egui)

Three GUI modes launched via CLI flags or tray menu:
- Prompt GUI: custom instruction overlay with history (saved in Windows registry)
- Config Editor: edit all settings in-app (endpoint, model, hotkey, MQTT, S3, Vision)
- Statistics Dashboard: translation count, errors, latency, per-model usage

## Architecture

Thoth runs as a background system-tray process with no visible window. It consists of:

- **Hotkey Listener**: Windows uses RegisterHotKey API; macOS uses rdev listener; Linux has a stub
- **Orchestrator**: Main event loop that dispatches hotkeys and coordinates the pipeline
- **Clipboard Manager**: Reads/writes clipboard via arboard crate; simulates Ctrl+C/V via rdev
- **Pylos Client**: HTTP client (reqwest) for communicating with LLM gateway
- **Sensitive Data Filter**: Blocks requests containing API keys, JWTs, credit cards, private keys, database URIs, Slack tokens
- **Config Storage**: Windows uses DPAPI encryption (CryptProtectData) in registry; macOS uses Keychain; Linux uses plain TOML file
- **Metrics**: Anonymous usage statistics stored locally as JSON
- **Notifications**: Native OS toast notifications (notify-rust)
- **System Tray**: Tray icon with menu for quick actions (Windows + macOS)

## Security & Privacy

- **Encrypted config at rest**: DPAPI on Windows, Keychain on macOS
- **HTTPS enforced**: All non-localhost endpoints auto-upgraded to HTTPS
- **Authenticode verification**: WinVerifyTrust validates binary signature at startup on release builds
- **Sensitive data detection**: Blocks requests containing 20+ patterns (API keys, secrets, PII)
- **Redacted logging**: Never logs original or translated text — only lengths and content hashes
- **Clipboard preservation**: Original content restored after each operation
- **Open source**: MIT license, fully auditable codebase

## Technology Stack

- **Language**: Rust (edition 2024, minimum MSRV 1.88.0)
- **Async runtime**: Tokio (full features)
- **GUI**: eframe/egui v0.29 (native, no web views, OpenGL via glow)
- **System tray**: tray-icon v0.24
- **Global hotkeys**: RegisterHotKey (Windows), rdev (macOS)
- **Clipboard**: arboard, rdev (keyboard simulation)
- **HTTP**: reqwest v0.13
- **Encryption**: Windows CryptProtectData (DPAPI), keyring (macOS/Linux)
- **Image capture**: xcap (active window screenshots)
- **S3**: rust-s3 v0.37 (MinIO-compatible)
- **MQTT**: rumqttc v0.25 (EMQX broker)
- **Installer**: WiX v4 (Windows MSI)

## Platform Support

| Platform | Support Level |
|---|---|
| Windows 10/11 (x86_64) | Full — hotkeys, tray, DPAPI, Authenticode, MSI installer |
| macOS (aarch64) | Full — hotkeys, tray, Keychain, LaunchAgent, .app bundle |
| Linux (x86_64, X11) | Partial — auto-start, config file, CLI operations |

## CI/CD Pipeline

- GitHub Actions workflow triggered on pushes to main and tags v*
- Jobs: lint (actionlint + gitleaks), check (fmt + clippy + tests + deny), MSRV check, builds for Windows/macOS/Linux, MSI packaging, and GitHub Release
- Windows binaries are self-signed during CI (temporary certificate)
- Code signing provided by SignPath Foundation (planned for production)

## Distribution

Pre-built binaries available via GitHub Releases:
- Windows: .exe + .zip (includes certificate), .msi installer
- macOS: .app bundle in .zip
- Linux: binary in .tar.gz

## Repository

- GitHub: https://github.com/JZacharie/Thoth
- License: MIT
- Author: JZacharie
- Written in Rust, approximately 17 source modules

## Use Cases

1. **Quick translation**: Select text in any app → press hotkey → text is replaced by translation
2. **Writing assistance**: Select text → reformulate hotkey → improve clarity and tone
3. **Custom automation**: Select text → custom prompt GUI → enter instruction → result pasted automatically
4. **Visual QA**: Screenshot analysis hotkey → capture a question on screen → get the answer pasted directly
5. **Email/chat response**: Highlight a message → translate or reformulate → paste reply
