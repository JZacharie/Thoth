#[cfg(windows)]
mod platform {
    use anyhow::Result;
    use std::path::Path;
    use winreg::RegKey;
    use winreg::enums::*;

    const RUN_KEY: &str = r"Software\Microsoft\Windows\CurrentVersion\Run";
    const APP_NAME: &str = "Thoth";

    pub fn enable() -> Result<()> {
        let exe_path = std::env::current_exe()?;
        if !exe_path.exists() {
            anyhow::bail!("binary not found at: {}", exe_path.display());
        }
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        let (key, _) = hkcu.create_subkey(RUN_KEY)?;
        key.set_value(APP_NAME, &exe_path.to_string_lossy().to_string())?;
        tracing::info!("auto-start enabled: {}", exe_path.display());
        Ok(())
    }

    pub fn disable() -> Result<()> {
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        let key = hkcu.open_subkey_with_flags(RUN_KEY, KEY_SET_VALUE)?;
        key.delete_value(APP_NAME)?;
        tracing::info!("auto-start disabled");
        Ok(())
    }

    pub fn is_enabled() -> bool {
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        let key = match hkcu.open_subkey_with_flags(RUN_KEY, KEY_READ) {
            Ok(k) => k,
            Err(_) => return false,
        };
        match key.get_value::<String, _>(APP_NAME) {
            Ok(path) => {
                let exe = std::env::current_exe().ok();
                match exe {
                    Some(current) => Path::new(&path) == current,
                    None => false,
                }
            }
            Err(_) => false,
        }
    }
}

#[cfg(not(windows))]
#[allow(dead_code)]
mod platform {
    pub fn enable() -> anyhow::Result<()> {
        tracing::warn!("auto-start not supported on this platform");
        Ok(())
    }
    pub fn disable() -> anyhow::Result<()> {
        tracing::warn!("auto-start not supported on this platform");
        Ok(())
    }
    pub fn is_enabled() -> bool {
        false
    }
}

#[cfg(windows)]
pub use platform::*;
