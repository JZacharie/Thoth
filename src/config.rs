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
    #[serde(default)]
    pub secret: String,
}

impl Default for PylosConfig {
    fn default() -> Self {
        Self {
            endpoint: "https://api.groq.com/openai".into(),
            model: "llama-3.1-8b-instant".into(),
            fallback_model: Some("llama-3.3-70b-versatile".into()),
            timeout_secs: 120,
            secret: std::env::var("THOTH_PYLOS_SECRET").unwrap_or_default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CustomInstructionHotkey {
    pub hotkey: String,
    pub instruction: String,
}

fn default_hotkey_translate_system() -> String {
    "Ctrl+Shift+Win+,".to_string()
}

fn default_hotkey_translate_english() -> String {
    "Ctrl+Shift+Win+;".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BehaviorConfig {
    pub target_language: String,
    pub restore_clipboard: bool,
    pub show_notifications: bool,
    pub debounce_ms: u64,
    pub hotkey: String,
    #[serde(default = "default_hotkey_translate_system")]
    pub hotkey_translate_system: String,
    #[serde(default = "default_hotkey_translate_english")]
    pub hotkey_translate_english: String,
    #[serde(default)]
    pub custom_instructions: Vec<CustomInstructionHotkey>,
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
            hotkey: "Ctrl+Shift+Win+:".into(),
            hotkey_translate_system: default_hotkey_translate_system(),
            hotkey_translate_english: default_hotkey_translate_english(),
            custom_instructions: Vec::new(),
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MqttConfig {
    pub broker: String,
    pub username: String,
    pub password: String,
    pub topic: String,
    pub port: u16,
    pub use_tls: bool,
}

impl Default for MqttConfig {
    fn default() -> Self {
        Self {
            broker: "mqtt-emqx.p.zacharie.org".into(),
            username: "joseph".into(),
            password: std::env::var("MQTT_PASSWORD").unwrap_or_default(),
            topic: "thoth/answers".into(),
            port: 8883,
            use_tls: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct S3Config {
    pub endpoint: String,
    pub bucket: String,
    pub access_key: String,
    pub secret_key: String,
    pub region: String,
}

impl Default for S3Config {
    fn default() -> Self {
        Self {
            endpoint: "https://minio-170-api.zacharie.org".into(),
            bucket: "thoth-screenshots".into(),
            access_key: "joseph".into(),
            secret_key: std::env::var("MINIO_SECRET_KEY").unwrap_or_default(),
            region: "auto".into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisionConfig {
    pub model: String,
    pub hotkey: String,
    pub system_prompt: String,
}

impl Default for VisionConfig {
    fn default() -> Self {
        Self {
            model: "gemini-3.5-flash".into(),
            hotkey: "Ctrl+Shift+Win+P".into(),
            system_prompt: "Analyse cette image de fenêtre. Identifie les questions posées. Pour chaque question, trouve la réponse correcte. Si les choix de réponse comportent un préfixe (comme une lettre A, B, C... ou un numéro 1, 2, 3...), renvoie UNIQUEMENT la lettre ou le numéro correspondant à la réponse correcte. Sinon, renvoie la réponse sous la forme la plus concise possible. Ne fournis aucune phrase d'introduction ni explication.".into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Config {
    pub pylos: PylosConfig,
    pub behavior: BehaviorConfig,
    pub mqtt: MqttConfig,
    pub s3: S3Config,
    pub vision: VisionConfig,
}

#[cfg(windows)]
mod win_secure {
    use anyhow::Result;
    use std::ptr;
    use winreg::RegKey;
    use winreg::enums::*;

    #[repr(C)]
    #[allow(non_snake_case)]
    struct DATA_BLOB {
        cbData: u32,
        pbData: *mut u8,
    }

    unsafe extern "system" {
        fn CryptProtectData(
            pDataIn: *const DATA_BLOB,
            szDataDescr: *const u16,
            pOptionalEntropy: *const DATA_BLOB,
            pvReserved: *mut std::ffi::c_void,
            pPromptStruct: *mut std::ffi::c_void,
            dwFlags: u32,
            pDataOut: *mut DATA_BLOB,
        ) -> i32;

        fn CryptUnprotectData(
            pDataIn: *const DATA_BLOB,
            szDataDescr: *mut u16,
            pOptionalEntropy: *const DATA_BLOB,
            pvReserved: *mut std::ffi::c_void,
            pPromptStruct: *mut std::ffi::c_void,
            dwFlags: u32,
            pDataOut: *mut DATA_BLOB,
        ) -> i32;

        fn LocalFree(hMem: *mut std::ffi::c_void) -> *mut std::ffi::c_void;
    }

    fn encrypt_bytes(data: &[u8]) -> Result<Vec<u8>> {
        let input = DATA_BLOB {
            cbData: data.len() as u32,
            pbData: data.as_ptr() as *mut u8,
        };
        let mut output = DATA_BLOB {
            cbData: 0,
            pbData: ptr::null_mut(),
        };
        unsafe {
            if CryptProtectData(
                &input,
                ptr::null(),
                ptr::null(),
                ptr::null_mut(),
                ptr::null_mut(),
                0,
                &mut output,
            ) == 0
            {
                return Err(anyhow::anyhow!("CryptProtectData failed"));
            }
            let result = std::slice::from_raw_parts(output.pbData, output.cbData as usize).to_vec();
            LocalFree(output.pbData as *mut std::ffi::c_void);
            Ok(result)
        }
    }

    fn decrypt_bytes(data: &[u8]) -> Result<Vec<u8>> {
        let input = DATA_BLOB {
            cbData: data.len() as u32,
            pbData: data.as_ptr() as *mut u8,
        };
        let mut output = DATA_BLOB {
            cbData: 0,
            pbData: ptr::null_mut(),
        };
        unsafe {
            if CryptUnprotectData(
                &input,
                ptr::null_mut(),
                ptr::null(),
                ptr::null_mut(),
                ptr::null_mut(),
                0,
                &mut output,
            ) == 0
            {
                return Err(anyhow::anyhow!("CryptUnprotectData failed"));
            }
            let result = std::slice::from_raw_parts(output.pbData, output.cbData as usize).to_vec();
            LocalFree(output.pbData as *mut std::ffi::c_void);
            Ok(result)
        }
    }

    pub fn save_to_registry(content: &str) -> Result<()> {
        let encrypted = encrypt_bytes(content.as_bytes())?;
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        let (key, _) = hkcu.create_subkey("Software\\Thoth")?;
        key.set_raw_value(
            "Config",
            &winreg::RegValue {
                vtype: REG_BINARY,
                bytes: encrypted.into(),
            },
        )?;
        Ok(())
    }

    pub fn load_from_registry() -> Result<Option<String>> {
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        let key = match hkcu.open_subkey("Software\\Thoth") {
            Ok(k) => k,
            Err(_) => return Ok(None),
        };
        let value = match key.get_raw_value("Config") {
            Ok(v) => v,
            Err(_) => return Ok(None),
        };
        let decrypted = decrypt_bytes(&value.bytes)?;
        Ok(Some(String::from_utf8(decrypted)?))
    }
}

impl Config {
    fn resolve_secrets(mut config: Config) -> Config {
        if config.pylos.secret.is_empty() {
            config.pylos.secret = std::env::var("THOTH_PYLOS_SECRET").unwrap_or_default();
        }
        if config.mqtt.password.is_empty() {
            config.mqtt.password = std::env::var("MQTT_PASSWORD").unwrap_or_default();
        }
        if config.s3.secret_key.is_empty() {
            config.s3.secret_key = std::env::var("MINIO_SECRET_KEY").unwrap_or_default();
        }
        config
    }

    pub fn load() -> anyhow::Result<Self> {
        #[cfg(windows)]
        {
            if let Ok(Some(content)) = win_secure::load_from_registry() {
                #[allow(clippy::collapsible_if)]
                if let Ok(config) = toml::from_str::<Config>(&content) {
                    return Ok(Self::resolve_secrets(config));
                }
            }
        }

        let config_path = Self::path();
        if config_path.exists() {
            let content = std::fs::read_to_string(&config_path)?;
            let config: Config = toml::from_str(&content)?;
            #[cfg(windows)]
            {
                let _ = win_secure::save_to_registry(&content);
                let _ = std::fs::remove_file(&config_path);
            }
            Ok(Self::resolve_secrets(config))
        } else {
            let config = Config::default();
            config.save()?;
            Ok(config)
        }
    }

    pub fn save(&self) -> anyhow::Result<()> {
        let content = toml::to_string_pretty(self)?;
        #[cfg(windows)]
        {
            win_secure::save_to_registry(&content)?;
            Ok(())
        }
        #[cfg(not(windows))]
        {
            let path = Self::path();
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::write(&path, content)?;
            Ok(())
        }
    }

    pub fn path() -> std::path::PathBuf {
        directories::ProjectDirs::from("org", "Thoth", "Thoth")
            .map(|d| d.config_dir().to_path_buf())
            .unwrap_or_else(|| {
                std::env::var("APPDATA")
                    .map(std::path::PathBuf::from)
                    .unwrap_or_else(|_| std::path::PathBuf::from("."))
                    .join("thoth")
            })
            .join("config.toml")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pylos_default() {
        let cfg = PylosConfig::default();
        assert_eq!(cfg.endpoint, "https://api.groq.com/openai");
        assert_eq!(cfg.model, "llama-3.1-8b-instant");
        assert_eq!(cfg.fallback_model, Some("llama-3.3-70b-versatile".into()));
        assert_eq!(cfg.timeout_secs, 120);
        let env_val = std::env::var("THOTH_PYLOS_SECRET").unwrap_or_default();
        assert_eq!(cfg.secret, env_val);
    }

    #[test]
    fn test_behavior_default() {
        let cfg = BehaviorConfig::default();
        let sys = system_language();
        assert_eq!(cfg.target_language, sys);
        assert!(cfg.show_notifications);
        assert!(cfg.restore_clipboard);
        assert_eq!(cfg.debounce_ms, 500);
        assert_eq!(cfg.hotkey, "Ctrl+Shift+Win+:");
        assert_eq!(cfg.log_path, None);
    }

    #[test]
    fn test_config_default() {
        let cfg = Config::default();
        let sys = system_language();
        assert_eq!(cfg.pylos.model, "llama-3.1-8b-instant");
        assert_eq!(cfg.behavior.target_language, sys);
    }

    #[test]
    fn test_config_path_construction() {
        let path = Config::path();
        let lossy = path.to_string_lossy().to_lowercase();
        assert!(
            lossy.contains("thoth"),
            "path '{}' should contain 'thoth'",
            path.display()
        );
        assert!(
            path.to_string_lossy().ends_with("config.toml"),
            "path should end with config.toml"
        );
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
