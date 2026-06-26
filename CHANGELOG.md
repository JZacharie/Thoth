# Changelog

## [1.3.1] - 2026-06-26

### Added
- **Linux native support** — native Linux support for global hotkeys, notifications, and tray integration

### Fixed
- **Linux compilation** — fixed missing system library `glib-2.0` on Linux builds by adding `pkg-config`, `libglib2.0-dev`, `libgtk-3-dev` and `libayatana-appindicator3-dev` to the workflow dependencies.

## [1.2.1] - 2026-06-25

### Added
- **find-skills script** — added helper script for skills discovery
- **Checksums & auto-release** — added sha256 checksums generation and auto-release triggers in workflow

### Changed
- **UI & egui improvements** — migrated rendering to use `Ui` instead of `Context`, allowed Ubuntu font license, and updated egui API calls (CornerRadius, Frame::NONE, global_style)
- **CI / CD optimizations** — optimized Windows CI build config, standardized cache keys, and refactored binary signing to avoid network dependencies
- **MSRV** — updated minimum supported Rust version to 1.92, upgraded eframe to 0.34, and optimized CI signing process

### Fixed
- **macOS .app packaging** — archive .app bundle to preserve directory structure before artifact upload
- **Windows cert store issues** — fixed windows-sys types for cert store (`HCERTSTORE`, `CERT_CONTEXT`) and moved imports to module level
- **Logger initialization** — initialize logger before signature check and auto-install dev cert to TrustedPublisher
- **Formatting** — sorted imports and improved `init_logger` signature formatting

## [1.2.0] - 2026-06-24

### Added
- **Temporary Code Signing & MSI** — added temporary code signing for Windows binaries and MSI installers
- **Privacy & security policy** — added `PRIVACY.md` and permitted insecure builds to bypass signature verification
- **Dependency bumps** — bumped `time`, `rumqttc`, `image`, and `rfd` crates

### Changed
- **macOS auth** — enabled apple-native keyring support to fix macOS authentication
- **CI upgrades** — bumped `actions/checkout` to version 7

### Fixed
- **Warnings & errors** — resolved pattern unused warnings, fixed gitleaks conditional check when no license secret is present

## [1.1.0] - 2026-06-15

### Added
- **Screenshot & Vision Analysis** — capture active window with `xcap`, analyze with Gemini Vision via Pylos, upload screenshot to MinIO S3, publish results to EMQX MQTT
- **Cross-platform hotkeys** — `RegisterHotKey` (Windows), `rdev` listener (macOS), stub (Linux)
- **Cross-platform tray icon** — abstracted `tray_impl` module supporting Windows and macOS (Linux stub)
- **Auto-start** — Windows Registry, macOS LaunchAgent plist, Linux `.desktop` file
- **Secure storage via keyring** — macOS Keychain, Linux Secret Service
- **Cross-platform CLI prompts** — `rfd` native dialogs replacing `MessageBoxW`
- **Cross-platform file paths** — `directories` crate replacing manual path computation
- **Integration tests** — 9 new tests covering config, MQTT, S3, vision, and orchestrator
- **CI/CD multiplatform pipeline** — parallel builds for Windows (x86_64), macOS (aarch64), Linux (x86_64) with artifact upload and GitHub Release
- **MSI release signing** — WiX MSI installer with version from git tag
- **`.cargo/audit.toml`** — advisory ignore list for cargo-deny
- **`.typos.toml`** — French dictionary entries

### Changed
- `MessageBoxW` replaced by `rfd` for cross-platform crash dialogs
- `Config::path()` and `UsageMetrics::path()` use `directories` crate
- Module structure: new `src/screenshot.rs`, `src/vision.rs`, `src/s3_storage.rs`, `src/mqtt.rs`, `src/auto_start.rs`, `src/secure_storage.rs`
- Tray icon refactored into conditional `tray_impl` module (Windows + macOS)
- `gitleaks-action` updated to v3, `upload-artifact` to v7, `download-artifact` to v8
- `chrono` removed in favor of `time` crate + `chrono_or_fallback()` helper

### Fixed
- `clippy::too-many-arguments` on `handle_menu_event` — allow annotation added
- `clippy::op-ref` on event_id comparisons
- `.gitleaks.toml` — `config-path` input removed (unsupported by gitleaks-action)
- Project compiles with zero warnings on all targets (`-D warnings`)
- `cargo fmt`, `cargo deny`, `cargo audit` all pass
