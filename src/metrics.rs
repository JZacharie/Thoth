use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UsageMetrics {
    pub total_translations: u64,
    pub total_errors: u64,
    pub total_bytes_processed: u64,
    pub total_latency_ms: u64,
    pub model_usage: HashMap<String, u64>,
}

#[allow(dead_code)]
impl UsageMetrics {
    pub fn record_success(&mut self, bytes: u64, latency_ms: u64, model: &str) {
        self.total_translations += 1;
        self.total_bytes_processed += bytes;
        self.total_latency_ms += latency_ms;
        *self.model_usage.entry(model.into()).or_insert(0) += 1;
    }

    pub fn record_error(&mut self) {
        self.total_errors += 1;
    }

    pub fn avg_latency_ms(&self) -> f64 {
        if self.total_translations == 0 {
            0.0
        } else {
            self.total_latency_ms as f64 / self.total_translations as f64
        }
    }

    pub fn load() -> Self {
        let path = Self::path();
        if path.exists() {
            std::fs::read_to_string(&path)
                .ok()
                .and_then(|s| serde_json::from_str(&s).ok())
                .unwrap_or_default()
        } else {
            Self::default()
        }
    }

    pub fn save(&self) {
        let path = Self::path();
        if let Some(Err(e)) = path.parent().map(std::fs::create_dir_all) {
            tracing::warn!("failed to create metrics directory: {e}");
            return;
        }
        match serde_json::to_string_pretty(self) {
            Ok(content) => {
                if let Err(e) = std::fs::write(&path, content) {
                    tracing::warn!("failed to save metrics: {e}");
                }
            }
            Err(e) => tracing::warn!("failed to serialize metrics: {e}"),
        }
    }

    fn path() -> std::path::PathBuf {
        directories::ProjectDirs::from("org", "Thoth", "Thoth")
            .map(|d| d.data_dir().to_path_buf())
            .unwrap_or_else(|| {
                std::env::var("APPDATA")
                    .map(std::path::PathBuf::from)
                    .unwrap_or_else(|_| std::path::PathBuf::from("."))
                    .join("thoth")
            })
            .join("metrics.json")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_metrics_empty() {
        let m = UsageMetrics::default();
        assert_eq!(m.total_translations, 0);
        assert_eq!(m.total_errors, 0);
        assert_eq!(m.avg_latency_ms(), 0.0);
    }

    #[test]
    fn test_record_success() {
        let mut m = UsageMetrics::default();
        m.record_success(100, 500, "gemma4:12b");
        assert_eq!(m.total_translations, 1);
        assert_eq!(m.total_bytes_processed, 100);
        assert_eq!(m.total_latency_ms, 500);
        assert_eq!(*m.model_usage.get("gemma4:12b").unwrap(), 1);
    }

    #[test]
    fn test_record_error() {
        let mut m = UsageMetrics::default();
        m.record_error();
        assert_eq!(m.total_errors, 1);
    }

    #[test]
    fn test_multiple_records() {
        let mut m = UsageMetrics::default();
        m.record_success(50, 200, "gemma4:12b");
        m.record_success(150, 600, "gemini4:12b");
        m.record_error();

        assert_eq!(m.total_translations, 2);
        assert_eq!(m.total_errors, 1);
        assert_eq!(m.total_bytes_processed, 200);
        assert_eq!(m.total_latency_ms, 800);
        assert_eq!(m.avg_latency_ms(), 400.0);
        assert_eq!(*m.model_usage.get("gemma4:12b").unwrap(), 1);
        assert_eq!(*m.model_usage.get("gemini4:12b").unwrap(), 1);
    }

    #[test]
    fn test_avg_latency_no_division_by_zero() {
        let m = UsageMetrics::default();
        assert_eq!(m.avg_latency_ms(), 0.0);
    }

    #[test]
    fn test_model_usage_tracking() {
        let mut m = UsageMetrics::default();
        m.record_success(10, 100, "gemma4:12b");
        m.record_success(20, 200, "gemma4:12b");
        m.record_success(30, 300, "gemini4:12b");

        assert_eq!(*m.model_usage.get("gemma4:12b").unwrap(), 2);
        assert_eq!(*m.model_usage.get("gemini4:12b").unwrap(), 1);
    }
}
