use serde::{Deserialize, Serialize};

const SUPPORTED_LANGUAGES: &[&str] = &["fr", "en", "es", "de", "it", "pt", "nl", "ja", "zh", "ru"];

pub fn validate_language(code: &str) -> bool {
    SUPPORTED_LANGUAGES.contains(&code)
}

pub fn system_language() -> String {
    sys_locale::get_locale()
        .as_deref()
        .and_then(|l| l.get(..2))
        .filter(|l| SUPPORTED_LANGUAGES.contains(l))
        .unwrap_or("en")
        .to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PylosConfig {
    pub endpoint: String,
    pub model: String,
    pub fallback_model: Option<String>,
    pub timeout_secs: u64,
    pub secret: String,
}

impl Default for PylosConfig {
    fn default() -> Self {
        Self {
            endpoint: "http://localhost:11434".into(),
            model: "gemma4:12b".into(),
            fallback_model: Some("gemini4:12b".into()),
            timeout_secs: 30,
            secret: String::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BehaviorConfig {
    pub target_language: String,
    pub restore_clipboard: bool,
    pub show_notifications: bool,
    pub debounce_ms: u64,
    pub hotkey: String,
    #[serde(default)]
    pub log_path: Option<String>,
}

impl Default for BehaviorConfig {
    fn default() -> Self {
        Self {
            target_language: system_language(),
            restore_clipboard: true,
            show_notifications: true,
            debounce_ms: 500,
            hotkey: "Ctrl+Shift+Win+N".into(),
            log_path: None,
        }
    }
}

impl BehaviorConfig {
    pub fn validated_language(&self) -> &str {
        if validate_language(&self.target_language) {
            &self.target_language
        } else {
            tracing::warn!(
                "unsupported language '{}', falling back to 'en'",
                self.target_language
            );
            "en"
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Config {
    pub pylos: PylosConfig,
    pub behavior: BehaviorConfig,
}

impl Config {
    pub fn load() -> anyhow::Result<Self> {
        let config_path = Self::path();
        if config_path.exists() {
            let content = std::fs::read_to_string(&config_path)?;
            Ok(toml::from_str(&content)?)
        } else {
            let config = Config::default();
            config.save()?;
            Ok(config)
        }
    }

    pub fn save(&self) -> anyhow::Result<()> {
        let path = Self::path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let content = toml::to_string_pretty(self)?;
        std::fs::write(&path, content)?;
        Ok(())
    }

    pub fn path() -> std::path::PathBuf {
        let base = std::env::var("APPDATA")
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|_| std::path::PathBuf::from("."));
        base.join("thoth").join("config.toml")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pylos_default() {
        let cfg = PylosConfig::default();
        assert_eq!(cfg.endpoint, "http://localhost:11434");
        assert_eq!(cfg.model, "gemma4:12b");
        assert_eq!(cfg.fallback_model, Some("gemini4:12b".into()));
        assert_eq!(cfg.timeout_secs, 30);
    }

    #[test]
    fn test_behavior_default() {
        let cfg = BehaviorConfig::default();
        let sys = system_language();
        assert_eq!(cfg.target_language, sys);
        assert!(cfg.show_notifications);
        assert!(cfg.restore_clipboard);
        assert_eq!(cfg.debounce_ms, 500);
        assert_eq!(cfg.hotkey, "Ctrl+Shift+Win+N");
        assert_eq!(cfg.log_path, None);
    }

    #[test]
    fn test_config_default() {
        let cfg = Config::default();
        let sys = system_language();
        assert_eq!(cfg.pylos.model, "gemma4:12b");
        assert_eq!(cfg.behavior.target_language, sys);
    }

    #[test]
    fn test_config_path_construction() {
        let path = Config::path();
        assert!(path.to_string_lossy().contains("thoth"));
        assert!(path.to_string_lossy().ends_with("config.toml"));
    }

    #[test]
    fn test_validate_language() {
        assert!(validate_language("fr"));
        assert!(validate_language("en"));
        assert!(validate_language("ja"));
        assert!(!validate_language("zz"));
        assert!(!validate_language(""));
    }

    #[test]
    fn test_validated_language_valid() {
        let cfg = BehaviorConfig {
            target_language: "en".into(),
            ..Default::default()
        };
        assert_eq!(cfg.validated_language(), "en");
    }

    #[test]
    fn test_validated_language_invalid() {
        let cfg = BehaviorConfig {
            target_language: "zz".into(),
            ..Default::default()
        };
        assert_eq!(cfg.validated_language(), "en");
    }

    #[test]
    fn test_system_language() {
        let lang = system_language();
        assert!(
            validate_language(&lang),
            "system_language() returned '{lang}' which is not supported"
        );
    }
}
