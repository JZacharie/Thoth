use std::time::Instant;

use anyhow::Result;
use tokio::sync::mpsc;

use crate::clipboard::ClipboardManager;
use crate::config::Config;
use crate::hotkey::HotkeyAction;
use crate::metrics::UsageMetrics;
use crate::notification;
use crate::pylos_client::{PylosClient, is_sensitive};

pub struct Orchestrator {
    hotkey_rx: mpsc::Receiver<HotkeyAction>,
    clipboard: ClipboardManager,
    pylos: PylosClient,
    metrics: UsageMetrics,
    restore_clipboard: bool,
    default_target_language: String,
}

impl Orchestrator {
    pub fn new(hotkey_rx: mpsc::Receiver<HotkeyAction>, config: Config) -> Result<Self> {
        let clipboard = ClipboardManager::new()?;
        let target_language = config.behavior.validated_language().to_string();
        let pylos = PylosClient::new(config.pylos.clone(), target_language.clone());
        let metrics = UsageMetrics::load();
        Ok(Self {
            hotkey_rx,
            clipboard,
            pylos,
            metrics,
            restore_clipboard: config.behavior.restore_clipboard,
            default_target_language: target_language,
        })
    }

    pub async fn test_connection(&self) -> Result<()> {
        self.pylos.test_connection().await
    }

    pub async fn test_translate(&self, text: &str) -> Result<String> {
        self.pylos.translate(text).await
    }

    pub fn endpoint(&self) -> &str {
        self.pylos.endpoint()
    }

    pub async fn run(&mut self) {
        loop {
            let action = match self.hotkey_rx.recv().await {
                Some(a) => a,
                None => {
                    tracing::info!("hotkey channel closed, shutting down");
                    break;
                }
            };

            tracing::info!("orchestrator: hotkey event received: {:?}", action);

            if action == HotkeyAction::ExecuteInstruction {
                tracing::info!("orchestrator: spawning prompt GUI");
                if let Ok(exe_path) = std::env::current_exe() {
                    let _ = std::process::Command::new(exe_path).arg("--prompt").spawn();
                }
                continue;
            }

            #[cfg(windows)]
            let active_window =
                unsafe { windows_sys::Win32::UI::WindowsAndMessaging::GetForegroundWindow() };

            let start = Instant::now();

            let original_text = match self.clipboard.copy_selected_text() {
                Ok(t) => {
                    tracing::info!(
                        "orchestrator: successfully captured clipboard text (len: {})",
                        t.len()
                    );
                    t
                }
                Err(e) => {
                    tracing::error!("clipboard copy failed: {e}");
                    self.metrics.record_error();
                    self.metrics.save();
                    notification::notify_error("Impossible de copier le texte");
                    continue;
                }
            };

            if original_text.is_empty() {
                tracing::warn!("orchestrator: copied text is empty, skipping translation");
                if let Err(e) = self.clipboard.restore() {
                    tracing::error!("clipboard restore failed after empty text: {e}");
                }
                continue;
            }

            if is_sensitive(&original_text) {
                tracing::warn!("sensitive data detected, blocking request");
                if let Err(e) = self.clipboard.restore() {
                    tracing::error!("clipboard restore failed: {e}");
                }
                self.metrics.record_error();
                self.metrics.save();
                notification::notify_warning("Texte sensible détecté — envoi bloqué");
                continue;
            }

            let translated = match action {
                HotkeyAction::ExecuteInstruction => unreachable!(),
                _ => {
                    let target_lang = match action {
                        HotkeyAction::TranslateDefault => &self.default_target_language,
                        HotkeyAction::TranslateEnglish => "en",
                        _ => unreachable!(),
                    };

                    tracing::info!(
                        "orchestrator: translating text to {} (len: {}, hash: {:x})",
                        target_lang,
                        original_text.len(),
                        {
                            use std::collections::hash_map::DefaultHasher;
                            use std::hash::{Hash, Hasher};
                            let mut s = DefaultHasher::new();
                            original_text.hash(&mut s);
                            s.finish()
                        }
                    );

                    match self.pylos.translate_to(&original_text, target_lang).await {
                        Ok(t) => {
                            tracing::info!(
                                "orchestrator: translation successful (len: {}, hash: {:x})",
                                t.len(),
                                {
                                    use std::collections::hash_map::DefaultHasher;
                                    use std::hash::{Hash, Hasher};
                                    let mut s = DefaultHasher::new();
                                    t.hash(&mut s);
                                    s.finish()
                                }
                            );
                            t
                        }
                        Err(e) => {
                            tracing::error!("pylos request failed: {e}");
                            self.metrics.record_error();
                            self.metrics.save();
                            notification::notify_error(
                                "Pylos introuvable — vérifiez qu'il est en cours d'exécution",
                            );
                            if let Err(e) = self.clipboard.restore() {
                                tracing::error!("clipboard restore failed: {e}");
                            }
                            continue;
                        }
                    }
                }
            };

            #[cfg(windows)]
            unsafe {
                if !active_window.is_null() {
                    tracing::info!("orchestrator: restoring focus to original window");
                    windows_sys::Win32::UI::WindowsAndMessaging::SetForegroundWindow(active_window);
                    std::thread::sleep(std::time::Duration::from_millis(100));
                }
            }

            if let Err(e) = self
                .clipboard
                .paste_text(&translated, self.restore_clipboard)
            {
                tracing::error!("clipboard paste failed: {e}");
                self.metrics.record_error();
                self.metrics.save();
                notification::notify_error("Impossible de coller le texte");
                continue;
            }

            tracing::info!("orchestrator: text pasted successfully");

            let latency = start.elapsed().as_millis() as u64;
            self.metrics.record_success(
                original_text.len() as u64,
                latency,
                &self.pylos.model_name(),
            );
            self.metrics.save();
            notification::notify_success();
        }
    }
}
