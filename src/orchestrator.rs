use std::time::Instant;

use anyhow::Result;
use tokio::sync::mpsc;

use crate::clipboard::ClipboardManager;
use crate::config::Config;
use crate::metrics::UsageMetrics;
use crate::notification;
use crate::pylos_client::{PylosClient, is_sensitive};

pub struct Orchestrator {
    hotkey_rx: mpsc::Receiver<()>,
    clipboard: ClipboardManager,
    pylos: PylosClient,
    metrics: UsageMetrics,
}

impl Orchestrator {
    pub fn new(hotkey_rx: mpsc::Receiver<()>, config: Config) -> Result<Self> {
        let clipboard = ClipboardManager::new()?;
        let target_language = config.behavior.validated_language().to_string();
        let pylos = PylosClient::new(config.pylos.clone(), target_language);
        let metrics = UsageMetrics::load();
        Ok(Self {
            hotkey_rx,
            clipboard,
            pylos,
            metrics,
        })
    }

    pub async fn run(&mut self) {
        loop {
            if self.hotkey_rx.recv().await.is_none() {
                tracing::info!("hotkey channel closed, shutting down");
                break;
            }

            tracing::debug!("orchestrator: processing hotkey event");
            let start = Instant::now();

            let original_text = match self.clipboard.copy_selected_text() {
                Ok(t) => t,
                Err(e) => {
                    tracing::error!("clipboard copy failed: {e}");
                    self.metrics.record_error();
                    self.metrics.save();
                    notification::notify_error("Impossible de copier le texte");
                    continue;
                }
            };

            if original_text.is_empty() {
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

            let translated = match self.pylos.translate(&original_text).await {
                Ok(t) => t,
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
            };

            if let Err(e) = self.clipboard.paste_text(&translated) {
                tracing::error!("clipboard paste failed: {e}");
                self.metrics.record_error();
                self.metrics.save();
                notification::notify_error("Impossible de coller le texte");
                continue;
            }

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
