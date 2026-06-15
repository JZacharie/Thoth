use anyhow::Result;
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex};
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
    Reformulate,
    ScreenshotAnalysis,
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

pub fn start(
    tx: mpsc::Sender<HotkeyAction>,
    pattern: Arc<Mutex<HotkeyPattern>>,
    enabled: Arc<AtomicBool>,
) -> Result<()> {
    #[cfg(windows)]
    {
        platform_win::start(tx, pattern, enabled)
    }
    #[cfg(target_os = "macos")]
    {
        platform_macos::start(tx, enabled)
    }
    #[cfg(not(any(windows, target_os = "macos")))]
    {
        let _ = (tx, pattern, enabled);
        tracing::warn!("global hotkeys not supported on this platform");
        Ok(())
    }
}

#[cfg(windows)]
mod platform_win {
    use anyhow::Result;
    use std::sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    };
    use tokio::sync::mpsc;
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        DispatchMessageW, GetMessageW, MSG, TranslateMessage, WM_HOTKEY,
    };

    use super::{HotkeyAction, HotkeyKey, HotkeyPattern, Modifier};

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
        tx: mpsc::Sender<HotkeyAction>,
        pattern: Arc<Mutex<HotkeyPattern>>,
        enabled: Arc<AtomicBool>,
    ) -> Result<()> {
        let pat_default = pattern.lock().unwrap().clone();
        let hotkeys: [(i32, u32, u32); 5] = [
            (
                1,
                get_win32_modifiers(&pat_default.modifiers) | 0x4000,
                get_win32_vk(&pat_default.key),
            ),
            (
                2,
                get_win32_modifiers(&[Modifier::Ctrl, Modifier::Shift, Modifier::Win]) | 0x4000,
                get_win32_vk(&HotkeyKey::Comma),
            ),
            (
                3,
                get_win32_modifiers(&[Modifier::Ctrl, Modifier::Shift, Modifier::Win]) | 0x4000,
                get_win32_vk(&HotkeyKey::Colon),
            ),
            (
                4,
                get_win32_modifiers(&[Modifier::Ctrl, Modifier::Shift, Modifier::Win]) | 0x4000,
                get_win32_vk(&HotkeyKey::Letter('r')),
            ),
            (
                5,
                get_win32_modifiers(&[Modifier::Ctrl, Modifier::Shift, Modifier::Win]) | 0x4000,
                get_win32_vk(&HotkeyKey::Letter('p')),
            ),
        ];

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

            for &(id, fs, vk) in &hotkeys {
                unsafe {
                    if RegisterHotKey(std::ptr::null_mut(), id, fs, vk) == 0 {
                        tracing::error!("RegisterHotKey (id={id}) failed: {}", GetLastError());
                    }
                }
            }
            tracing::info!("RegisterHotKey: all hotkeys registered");

            unsafe {
                let mut msg = std::mem::zeroed::<MSG>();
                while GetMessageW(&mut msg, std::ptr::null_mut(), 0, 0) != 0 {
                    if msg.message == WM_HOTKEY && enabled.load(Ordering::Relaxed) {
                        let action = match msg.wParam as i32 {
                            1 => HotkeyAction::TranslateDefault,
                            2 => HotkeyAction::TranslateEnglish,
                            3 => HotkeyAction::ExecuteInstruction,
                            4 => HotkeyAction::Reformulate,
                            5 => HotkeyAction::ScreenshotAnalysis,
                            _ => continue,
                        };
                        if tx.try_send(action).is_err() {
                            tracing::warn!("hotkey channel full, dropping event");
                        }
                    }
                    TranslateMessage(&msg);
                    DispatchMessageW(&msg);
                }

                for &(id, _, _) in &hotkeys {
                    UnregisterHotKey(std::ptr::null_mut(), id);
                }
            }
        });

        Ok(())
    }
}

#[cfg(target_os = "macos")]
mod platform_macos {
    use anyhow::Result;
    use rdev::{EventType, Key, listen};
    use std::collections::HashSet;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::{Arc, Mutex};
    use tokio::sync::mpsc;

    use super::HotkeyAction;

    fn modifier_from_key(key: &Key) -> Option<&'static str> {
        match key {
            Key::ControlLeft | Key::ControlRight => Some("ctrl"),
            Key::Alt | Key::AltGr => Some("alt"),
            Key::ShiftLeft | Key::ShiftRight => Some("shift"),
            #[cfg(target_os = "macos")]
            Key::MetaLeft | Key::MetaRight => Some("cmd"),
            #[cfg(not(target_os = "macos"))]
            Key::MetaLeft | Key::MetaRight | Key::SuperLeft | Key::SuperRight => Some("win"),
            _ => None,
        }
    }

    fn key_to_str(key: &Key) -> Option<String> {
        match key {
            Key::KeyA => Some("a"),
            Key::KeyB => Some("b"),
            Key::KeyC => Some("c"),
            Key::KeyD => Some("d"),
            Key::KeyE => Some("e"),
            Key::KeyF => Some("f"),
            Key::KeyG => Some("g"),
            Key::KeyH => Some("h"),
            Key::KeyI => Some("i"),
            Key::KeyJ => Some("j"),
            Key::KeyK => Some("k"),
            Key::KeyL => Some("l"),
            Key::KeyM => Some("m"),
            Key::KeyN => Some("n"),
            Key::KeyO => Some("o"),
            Key::KeyP => Some("p"),
            Key::KeyQ => Some("q"),
            Key::KeyR => Some("r"),
            Key::KeyS => Some("s"),
            Key::KeyT => Some("t"),
            Key::KeyU => Some("u"),
            Key::KeyV => Some("v"),
            Key::KeyW => Some("w"),
            Key::KeyX => Some("x"),
            Key::KeyY => Some("y"),
            Key::KeyZ => Some("z"),
            Key::Num0 => Some("0"),
            Key::Num1 => Some("1"),
            Key::Num2 => Some("2"),
            Key::Num3 => Some("3"),
            Key::Num4 => Some("4"),
            Key::Num5 => Some("5"),
            Key::Num6 => Some("6"),
            Key::Num7 => Some("7"),
            Key::Num8 => Some("8"),
            Key::Num9 => Some("9"),
            Key::Comma => Some(","),
            Key::Semicolon => Some(";"),
            _ => None,
        }
    }

    pub fn start(tx: mpsc::Sender<HotkeyAction>, enabled: Arc<AtomicBool>) -> Result<()> {
        let pressed: Arc<Mutex<HashSet<String>>> = Arc::new(Mutex::new(HashSet::new()));
        let tx_clone = tx.clone();
        let enabled_clone = enabled.clone();

        std::thread::spawn(move || {
            if let Err(e) = listen(move |event| {
                if !enabled_clone.load(Ordering::Relaxed) {
                    return;
                }

                match event.event_type {
                    EventType::KeyPress(key) => {
                        if let Some(mod_name) = modifier_from_key(&key) {
                            pressed.lock().unwrap().insert(mod_name.to_string());
                        }
                        if let Some(key_name) = key_to_str(&key) {
                            pressed.lock().unwrap().insert(key_name);
                        }
                    }
                    EventType::KeyRelease(key) => {
                        if let Some(mod_name) = modifier_from_key(&key) {
                            pressed.lock().unwrap().remove(mod_name);
                        }
                        if let Some(key_name) = key_to_str(&key) {
                            let mut p = pressed.lock().unwrap();
                            p.remove(&key_name);

                            // Check for Ctrl+Shift+Cmd/Win+<key> combinations
                            let has_ctrl = p.contains("ctrl");
                            let has_shift = p.contains("shift");
                            let has_cmd = p.contains("cmd") || p.contains("win");

                            if has_ctrl && has_shift && has_cmd {
                                let action = match key_name.as_str() {
                                    "n" => Some(HotkeyAction::TranslateDefault),
                                    "," => Some(HotkeyAction::TranslateEnglish),
                                    ";" => Some(HotkeyAction::ExecuteInstruction),
                                    "r" => Some(HotkeyAction::Reformulate),
                                    "p" => Some(HotkeyAction::ScreenshotAnalysis),
                                    _ => None,
                                };
                                if let Some(action) = action {
                                    if tx_clone.try_send(action).is_err() {
                                        tracing::warn!("hotkey channel full, dropping event");
                                    }
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }) {
                tracing::error!("rdev listen failed: {e}");
            }
        });

        Ok(())
    }
}
