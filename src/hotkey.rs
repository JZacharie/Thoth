use anyhow::Result;
use std::sync::{Arc, Mutex, atomic::AtomicBool};
use tokio::sync::mpsc;

#[derive(Debug, Clone, PartialEq)]
pub enum Modifier {
    Win,
    Ctrl,
    Alt,
    Shift,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum HotkeyAction {
    TranslateDefault,
    TranslateEnglish,
    ExecuteInstruction,
}

#[derive(Debug, Clone, PartialEq)]
pub enum HotkeyKey {
    Letter(char),
    Number(u8),
    Space,
    F(u8),
    Comma,
    Semicolon,
    Colon,
}

#[derive(Debug, Clone, PartialEq)]
pub struct HotkeyPattern {
    pub modifiers: Vec<Modifier>,
    pub key: HotkeyKey,
}

impl HotkeyPattern {
    pub fn parse(s: &str) -> Result<Self> {
        let parts: Vec<&str> = s.split('+').collect();
        if parts.len() < 2 {
            anyhow::bail!("invalid hotkey format: '{s}' — expected e.g. Ctrl+Win+N");
        }
        let mut modifiers = Vec::new();
        for part in parts.iter().take(parts.len() - 1) {
            let m = match part.trim().to_lowercase().as_str() {
                "win" => Modifier::Win,
                "ctrl" => Modifier::Ctrl,
                "alt" => Modifier::Alt,
                "shift" => Modifier::Shift,
                other => anyhow::bail!("unknown modifier: '{other}'"),
            };
            modifiers.push(m);
        }
        let key_str = parts.last().unwrap().trim();
        let key = if let Some(ch) = key_str.chars().next() {
            if key_str.len() == 1 && ch.is_ascii_alphabetic() {
                HotkeyKey::Letter(ch.to_ascii_lowercase())
            } else if let Ok(n) = key_str.parse::<u8>() {
                HotkeyKey::Number(n)
            } else {
                match key_str.to_lowercase().as_str() {
                    "space" => HotkeyKey::Space,
                    "comma" | "," => HotkeyKey::Comma,
                    "semicolon" | ";" => HotkeyKey::Semicolon,
                    "colon" | ":" => HotkeyKey::Colon,
                    s if s.starts_with('f') && s[1..].parse::<u8>().is_ok() => {
                        HotkeyKey::F(s[1..].parse().unwrap())
                    }
                    _ => anyhow::bail!("unknown key: '{key_str}'"),
                }
            }
        } else {
            anyhow::bail!("empty key in hotkey");
        };
        Ok(Self { modifiers, key })
    }

    pub fn default_win_n() -> Self {
        Self {
            modifiers: vec![Modifier::Win],
            key: HotkeyKey::Letter('n'),
        }
    }
}

#[cfg(windows)]
mod platform {
    use anyhow::Result;
    use std::sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    };
    use tokio::sync::mpsc;
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        DispatchMessageW, GetMessageW, MSG, TranslateMessage, WM_HOTKEY,
    };

    use super::{HotkeyKey, HotkeyPattern, Modifier};

    fn get_win32_modifiers(modifiers: &[Modifier]) -> u32 {
        let mut m = 0;
        for modifier in modifiers {
            match modifier {
                Modifier::Alt => m |= 0x0001,
                Modifier::Ctrl => m |= 0x0002,
                Modifier::Shift => m |= 0x0004,
                Modifier::Win => m |= 0x0008,
            }
        }
        m
    }

    fn get_win32_vk(key: &HotkeyKey) -> u32 {
        match key {
            HotkeyKey::Letter(ch) => ch.to_ascii_uppercase() as u32,
            HotkeyKey::Number(n) => (*n as u32) + 0x30,
            HotkeyKey::Space => 0x20,
            HotkeyKey::F(n) => 0x70 + (*n as u32 - 1),
            HotkeyKey::Comma => 0xBC,
            HotkeyKey::Semicolon => 0xBA,
            HotkeyKey::Colon => 0xBF,
        }
    }

    pub fn start(
        tx: mpsc::Sender<super::HotkeyAction>,
        pattern: Arc<Mutex<HotkeyPattern>>,
        enabled: Arc<AtomicBool>,
    ) -> Result<()> {
        let pat_default = pattern.lock().unwrap().clone();
        let fs_default = get_win32_modifiers(&pat_default.modifiers);
        let vk_default = get_win32_vk(&pat_default.key);

        let pat_english = HotkeyPattern::parse("Ctrl+Shift+Win+,").unwrap();
        let fs_english = get_win32_modifiers(&pat_english.modifiers);
        let vk_english = get_win32_vk(&pat_english.key);

        let pat_instruction = HotkeyPattern::parse("Ctrl+Shift+Win+:").unwrap();
        let fs_instruction = get_win32_modifiers(&pat_instruction.modifiers);
        let vk_instruction = get_win32_vk(&pat_instruction.key);

        std::thread::spawn(move || {
            unsafe extern "system" {
                fn RegisterHotKey(
                    hwnd: *mut std::ffi::c_void,
                    id: i32,
                    fs_modifiers: u32,
                    vk: u32,
                ) -> i32;
                fn UnregisterHotKey(hwnd: *mut std::ffi::c_void, id: i32) -> i32;
                fn GetLastError() -> u32;
            }

            unsafe {
                // MOD_NOREPEAT = 0x4000
                if RegisterHotKey(std::ptr::null_mut(), 1, fs_default | 0x4000, vk_default) == 0 {
                    let err = GetLastError();
                    tracing::error!("RegisterHotKey (Default) failed with error code: {err}");
                    return;
                }
                if RegisterHotKey(std::ptr::null_mut(), 2, fs_english | 0x4000, vk_english) == 0 {
                    let err = GetLastError();
                    tracing::error!("RegisterHotKey (English) failed with error code: {err}");
                    UnregisterHotKey(std::ptr::null_mut(), 1);
                    return;
                }
                if RegisterHotKey(
                    std::ptr::null_mut(),
                    3,
                    fs_instruction | 0x4000,
                    vk_instruction,
                ) == 0
                {
                    let err = GetLastError();
                    tracing::error!("RegisterHotKey (Instruction) failed with error code: {err}");
                    UnregisterHotKey(std::ptr::null_mut(), 1);
                    UnregisterHotKey(std::ptr::null_mut(), 2);
                    return;
                }
                tracing::info!("RegisterHotKey: all three global hotkeys registered successfully");

                let mut msg = std::mem::zeroed::<MSG>();
                while GetMessageW(&mut msg, std::ptr::null_mut(), 0, 0) != 0 {
                    if msg.message == WM_HOTKEY && enabled.load(Ordering::Relaxed) {
                        let action = match msg.wParam as i32 {
                            1 => super::HotkeyAction::TranslateDefault,
                            2 => super::HotkeyAction::TranslateEnglish,
                            3 => super::HotkeyAction::ExecuteInstruction,
                            _ => continue,
                        };
                        tracing::info!("RegisterHotKey: hotkey triggered for action {:?}", action);
                        if tx.try_send(action).is_err() {
                            tracing::warn!("hotkey channel full, dropping event");
                        }
                    }
                    TranslateMessage(&msg);
                    DispatchMessageW(&msg);
                }

                UnregisterHotKey(std::ptr::null_mut(), 1);
                UnregisterHotKey(std::ptr::null_mut(), 2);
                UnregisterHotKey(std::ptr::null_mut(), 3);
            }
        });

        Ok(())
    }
}

#[cfg(not(windows))]
mod platform {
    use super::{HotkeyAction, HotkeyPattern};
    use anyhow::Result;
    use std::sync::{Arc, Mutex, atomic::AtomicBool};
    use tokio::sync::mpsc;

    pub fn start(
        _tx: mpsc::Sender<HotkeyAction>,
        _pattern: Arc<Mutex<HotkeyPattern>>,
        _enabled: Arc<AtomicBool>,
    ) -> Result<()> {
        tracing::warn!("global hotkey not supported on this platform");
        Ok(())
    }
}

pub fn start(
    tx: mpsc::Sender<HotkeyAction>,
    pattern: Arc<Mutex<HotkeyPattern>>,
    enabled: Arc<AtomicBool>,
) -> Result<()> {
    platform::start(tx, pattern, enabled)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_win_n() {
        let h = HotkeyPattern::parse("Win+N").unwrap();
        assert_eq!(h.modifiers, vec![Modifier::Win]);
        assert_eq!(h.key, HotkeyKey::Letter('n'));
    }

    #[test]
    fn test_parse_ctrl_shift_t() {
        let h = HotkeyPattern::parse("Ctrl+Shift+T").unwrap();
        assert_eq!(h.modifiers, vec![Modifier::Ctrl, Modifier::Shift]);
        assert_eq!(h.key, HotkeyKey::Letter('t'));
    }

    #[test]
    fn test_parse_alt_space() {
        let h = HotkeyPattern::parse("Alt+Space").unwrap();
        assert_eq!(h.modifiers, vec![Modifier::Alt]);
        assert_eq!(h.key, HotkeyKey::Space);
    }

    #[test]
    fn test_parse_f_keys() {
        let h = HotkeyPattern::parse("Ctrl+F5").unwrap();
        assert_eq!(h.modifiers, vec![Modifier::Ctrl]);
        assert_eq!(h.key, HotkeyKey::F(5));
    }

    #[test]
    fn test_parse_number() {
        let h = HotkeyPattern::parse("Win+1").unwrap();
        assert_eq!(h.modifiers, vec![Modifier::Win]);
        assert_eq!(h.key, HotkeyKey::Number(1));
    }

    #[test]
    fn test_parse_invalid_empty() {
        assert!(HotkeyPattern::parse("").is_err());
    }

    #[test]
    fn test_parse_invalid_no_modifier() {
        assert!(HotkeyPattern::parse("N").is_err());
    }

    #[test]
    fn test_parse_invalid_modifier() {
        assert!(HotkeyPattern::parse("Super+N").is_err());
    }

    #[test]
    fn test_default_win_n() {
        let h = HotkeyPattern::default_win_n();
        assert_eq!(h.modifiers, vec![Modifier::Win]);
        assert_eq!(h.key, HotkeyKey::Letter('n'));
    }

    #[test]
    fn test_case_insensitive() {
        let h = HotkeyPattern::parse("win+n").unwrap();
        assert_eq!(h.modifiers, vec![Modifier::Win]);
        assert_eq!(h.key, HotkeyKey::Letter('n'));
    }

    #[test]
    fn test_parse_ctrl_alt_shift() {
        let h = HotkeyPattern::parse("Ctrl+Alt+Shift+F1").unwrap();
        assert_eq!(
            h.modifiers,
            vec![Modifier::Ctrl, Modifier::Alt, Modifier::Shift]
        );
        assert_eq!(h.key, HotkeyKey::F(1));
    }
}
