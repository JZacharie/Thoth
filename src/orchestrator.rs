use std::time::Instant;

use anyhow::Result;
use tokio::sync::mpsc;

use crate::clipboard::ClipboardManager;
use crate::config::Config;
use crate::hotkey::HotkeyAction;
use crate::metrics::UsageMetrics;
use crate::mqtt::MqttPublisher;
use crate::notification;
use crate::pylos_client::{PylosClient, is_sensitive};
use crate::s3_storage::S3Storage;

pub struct Orchestrator {
    hotkey_rx: mpsc::Receiver<HotkeyAction>,
    clipboard: ClipboardManager,
    pylos: PylosClient,
    metrics: UsageMetrics,
    restore_clipboard: bool,
    default_target_language: String,
    config: Config,
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
            config,
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

    async fn handle_screenshot_analysis(&mut self) -> Result<String> {
        let start = Instant::now();
        notification::notify_screenshot_analysis();

        let (png_data, window_title) = crate::screenshot::capture_active_window()?;

        let s3_url = if let Ok(Some(s3)) = S3Storage::new(&self.config.s3) {
            let session_id = uuid::Uuid::new_v4();
            let timestamp = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();
            let key = format!("screenshot_{}_{}.png", session_id, timestamp);
            match s3.upload_png(&png_data, &key).await {
                Ok(url) => {
                    tracing::info!("screenshot uploaded to S3: {url}");
                    Some(url)
                }
                Err(e) => {
                    tracing::warn!("S3 upload failed (continuing without): {e}");
                    None
                }
            }
        } else {
            None
        };

        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(
                self.config.pylos.timeout_secs,
            ))
            .build()?;

        let mut endpoint = self.config.pylos.endpoint.clone();
        while endpoint.ends_with('/') {
            endpoint.pop();
        }

        let vision = crate::vision::VisionAnalyzer::new(
            client,
            endpoint,
            self.config.pylos.secret.clone(),
            self.config.vision.clone(),
        );

        let answer = match vision.analyze_screenshot(&png_data).await {
            Ok(text) => {
                let trimmed = text.trim().to_string();
                if trimmed.is_empty() || trimmed.len() < 3 {
                    tracing::info!("vision returned short/empty result, falling back to text");
                    self.text_fallback().await?
                } else {
                    tracing::info!("vision analysis successful: '{}'", trimmed);
                    trimmed
                }
            }
            Err(e) => {
                tracing::warn!("vision analysis failed: {e}, falling back to text");
                self.text_fallback().await?
            }
        };

        let latency = start.elapsed().as_millis() as u64;

        let log_entry = serde_json::json!({
            "timestamp": chrono_or_fallback(),
            "window_title": window_title,
            "s3_url": s3_url,
            "question_detected": true,
            "answer_proposed": answer,
            "latency_ms": latency,
        });

        tracing::info!(
            "screenshot analysis log: {}",
            serde_json::to_string(&log_entry)?
        );

        if let Ok(mqtt) = MqttPublisher::new(&self.config.mqtt).await {
            if let Err(e) = mqtt.publish_json(&log_entry).await {
                tracing::warn!("MQTT publish failed: {e}");
            }
        } else {
            tracing::warn!("MQTT not configured, skipping publish");
        }

        Ok(answer)
    }

    async fn text_fallback(&mut self) -> Result<String> {
        tracing::info!("text fallback: selecting all text via Ctrl+A / Cmd+A");
        let original_text = self.clipboard.copy_selected_text()?;
        if original_text.is_empty() {
            anyhow::bail!("no text found in fallback mode");
        }
        tracing::info!("text fallback: captured {} chars", original_text.len());
        let prompt = format!("{}\n\n{}", self.config.vision.system_prompt, original_text);
        self.pylos.execute_instruction(&prompt).await
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

            if action == HotkeyAction::ScreenshotAnalysis {
                tracing::info!("orchestrator: starting screenshot analysis");
                match self.handle_screenshot_analysis().await {
                    Ok(answer) => {
                        if let Err(e) = self.clipboard.paste_text(&answer, self.restore_clipboard) {
                            tracing::error!("screenshot paste failed: {e}");
                            notification::notify_error("Impossible de coller la réponse");
                        } else {
                            notification::notify_success();
                        }
                    }
                    Err(e) => {
                        tracing::error!("screenshot analysis failed: {e}");
                        notification::notify_error(&format!("Analyse d'écran échouée : {e}"));
                    }
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
                HotkeyAction::ExecuteInstruction | HotkeyAction::ScreenshotAnalysis => {
                    unreachable!()
                }
                HotkeyAction::Reformulate => {
                    tracing::info!(
                        "orchestrator: reformulating text (len: {}, hash: {:x})",
                        original_text.len(),
                        {
                            use std::collections::hash_map::DefaultHasher;
                            use std::hash::{Hash, Hasher};
                            let mut s = DefaultHasher::new();
                            original_text.hash(&mut s);
                            s.finish()
                        }
                    );

                    match self.pylos.reformulate(&original_text).await {
                        Ok(t) => {
                            tracing::info!(
                                "orchestrator: reformulation successful (len: {}, hash: {:x})",
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
                            tracing::error!("pylos request failed on reformulate: {e}");
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

pub fn chrono_or_fallback() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    let secs = now.as_secs();
    let millis = now.subsec_millis();
    let days = secs / 86400;
    let time = secs % 86400;
    let hours = time / 3600;
    let minutes = (time % 3600) / 60;
    let seconds = time % 60;
    format!("{days}d {hours:02}:{minutes:02}:{seconds:02}.{millis:03}")
}
