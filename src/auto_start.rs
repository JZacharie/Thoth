use anyhow::Result;
use std::path::PathBuf;

#[cfg(windows)]
mod platform {
    use super::*;
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
                    Some(current) => std::path::Path::new(&path) == current,
                    None => false,
                }
            }
            Err(_) => false,
        }
    }
}

#[cfg(target_os = "macos")]
mod platform {
    use super::*;

    fn plist_path() -> PathBuf {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".into());
        PathBuf::from(home).join("Library/LaunchAgents/org.thoth.Thoth.plist")
    }

    pub fn enable() -> Result<()> {
        let exe_path = std::env::current_exe()?;
        let plist = plist_path();
        if let Some(parent) = plist.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let content = format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>org.thoth.Thoth</string>
    <key>ProgramArguments</key>
    <array>
        <string>{}</string>
    </array>
    <key>RunAtLoad</key>
    <true/>
    <key>KeepAlive</key>
    <false/>
</dict>
</plist>
"#,
            exe_path.display()
        );

        std::fs::write(&plist, content)?;
        tracing::info!("auto-start enabled via plist: {}", plist.display());
        Ok(())
    }

    pub fn disable() -> Result<()> {
        let plist = plist_path();
        if plist.exists() {
            std::fs::remove_file(&plist)?;
            tracing::info!("auto-start disabled");
        }
        Ok(())
    }

    pub fn is_enabled() -> bool {
        plist_path().exists()
    }
}

#[cfg(target_os = "linux")]
mod platform {
    use super::*;

    fn desktop_path() -> PathBuf {
        let config = std::env::var("XDG_CONFIG_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|_| {
                let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".into());
                PathBuf::from(home).join(".config")
            });
        config.join("autostart/thoth.desktop")
    }

    pub fn enable() -> Result<()> {
        let exe_path = std::env::current_exe()?;
        let desktop = desktop_path();
        if let Some(parent) = desktop.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let content = format!(
            "[Desktop Entry]\n\
             Type=Application\n\
             Name=Thoth\n\
             Exec={}\n\
             Terminal=false\n\
             NoDisplay=true\n",
            exe_path.display()
        );

        std::fs::write(&desktop, content)?;
        tracing::info!("auto-start enabled via .desktop: {}", desktop.display());
        Ok(())
    }

    pub fn disable() -> Result<()> {
        let desktop = desktop_path();
        if desktop.exists() {
            std::fs::remove_file(&desktop)?;
            tracing::info!("auto-start disabled");
        }
        Ok(())
    }

    pub fn is_enabled() -> bool {
        desktop_path().exists()
    }
}

#[cfg(not(any(windows, target_os = "macos", target_os = "linux")))]
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

pub use platform::*;
