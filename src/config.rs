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
            endpoint: "https://pylos-dev.p.zacharie.org".into(),
            model: "gemini4:e2b".into(),
            fallback_model: Some("gemma4:12b".into()),
            timeout_secs: 30,
            secret: "your_secret_key_here".into(),
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
    pub fn load() -> anyhow::Result<Self> {
        #[cfg(windows)]
        {
            if let Ok(Some(content)) = win_secure::load_from_registry() {
                #[allow(clippy::collapsible_if)]
                if let Ok(config) = toml::from_str::<Config>(&content) {
                    return Ok(config);
                }
            }
        }

        // Migration ou fallback sur fichier plat
        let config_path = Self::path();
        if config_path.exists() {
            let content = std::fs::read_to_string(&config_path)?;
            let config: Config = toml::from_str(&content)?;
            #[cfg(windows)]
            {
                let _ = win_secure::save_to_registry(&content);
                let _ = std::fs::remove_file(&config_path); // Supprime le fichier plat non sécurisé
            }
            Ok(config)
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
        assert_eq!(cfg.endpoint, "https://pylos-dev.p.zacharie.org");
        assert_eq!(cfg.model, "gemini4:e2b");
        assert_eq!(cfg.fallback_model, Some("gemma4:12b".into()));
        assert_eq!(cfg.timeout_secs, 30);
        assert_eq!(cfg.secret, "your_secret_key_here");
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
        assert_eq!(cfg.pylos.model, "gemini4:e2b");
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
