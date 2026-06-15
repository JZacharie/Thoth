# Changelog

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
