use anyhow::Result;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use tokio::sync::mpsc;

#[derive(Debug, Clone, PartialEq)]
pub enum Modifier {
    Win,
    Ctrl,
    Alt,
    Shift,
}

#[derive(Debug, Clone, PartialEq)]
pub enum HotkeyAction {
    TranslateDefault,
    TranslateEnglish,
    ExecuteInstruction,
    Reformulate,
    ScreenshotAnalysis,
    Custom(String),
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

#[derive(Debug, Clone)]
pub struct HotkeyConfig {
    pub translate_system: HotkeyPattern,
    pub translate_english: HotkeyPattern,
    pub execute_instruction: HotkeyPattern,
    pub custom_instructions: Vec<(HotkeyPattern, String)>,
}

impl HotkeyConfig {
    pub fn from_config(config: &crate::config::Config) -> Self {
        let translate_system = HotkeyPattern::parse(&config.behavior.hotkey_translate_system)
            .unwrap_or_else(|_| HotkeyPattern::parse("Ctrl+Shift+Win+,").unwrap());
        let translate_english = HotkeyPattern::parse(&config.behavior.hotkey_translate_english)
            .unwrap_or_else(|_| HotkeyPattern::parse("Ctrl+Shift+Win+;").unwrap());
        let execute_instruction = HotkeyPattern::parse(&config.behavior.hotkey)
            .unwrap_or_else(|_| HotkeyPattern::parse("Ctrl+Shift+Win+:").unwrap());

        let mut custom_instructions = Vec::new();
        for custom in &config.behavior.custom_instructions {
            if let Ok(pat) = HotkeyPattern::parse(&custom.hotkey) {
                custom_instructions.push((pat, custom.instruction.clone()));
            }
        }

        Self {
            translate_system,
            translate_english,
            execute_instruction,
            custom_instructions,
        }
    }
}

#[allow(dead_code)]
fn match_pattern(pressed: &std::collections::HashSet<String>, pattern: &HotkeyPattern) -> bool {
    let mut expected_mods = std::collections::HashSet::new();
    for m in &pattern.modifiers {
        match m {
            Modifier::Ctrl => {
                expected_mods.insert("ctrl".to_string());
            }
            Modifier::Shift => {
                expected_mods.insert("shift".to_string());
            }
            Modifier::Alt => {
                expected_mods.insert("alt".to_string());
            }
            Modifier::Win => {
                expected_mods.insert("win".to_string());
            }
        }
    }

    let mut pressed_mods = std::collections::HashSet::new();
    for k in pressed {
        match k.as_str() {
            "ctrl" => {
                pressed_mods.insert("ctrl".to_string());
            }
            "shift" => {
                pressed_mods.insert("shift".to_string());
            }
            "alt" => {
                pressed_mods.insert("alt".to_string());
            }
            "win" | "cmd" => {
                pressed_mods.insert("win".to_string());
            }
            _ => {}
        }
    }

    if expected_mods != pressed_mods {
        return false;
    }

    let key_str = match &pattern.key {
        HotkeyKey::Letter(ch) => ch.to_string(),
        HotkeyKey::Number(n) => n.to_string(),
        HotkeyKey::Space => "space".to_string(),
        HotkeyKey::Comma => ",".to_string(),
        HotkeyKey::Semicolon => ";".to_string(),
        HotkeyKey::Colon => ":".to_string(),
        HotkeyKey::F(n) => format!("f{}", n),
    };

    pressed.contains(&key_str)
}

pub fn start(
    tx: mpsc::Sender<HotkeyAction>,
    config: &crate::config::Config,
    enabled: Arc<AtomicBool>,
) -> Result<()> {
    let hotkey_config = HotkeyConfig::from_config(config);
    #[cfg(windows)]
    {
        platform_win::start(tx, hotkey_config, enabled)
    }
    #[cfg(target_os = "macos")]
    {
        platform_macos::start(tx, hotkey_config, enabled)
    }
    #[cfg(not(any(windows, target_os = "macos", target_os = "linux")))]
    {
        let _ = (tx, hotkey_config, enabled);
        tracing::warn!("global hotkeys not supported on this platform");
        Ok(())
    }
    #[cfg(target_os = "linux")]
    {
        platform_linux::start(tx, hotkey_config, enabled)
    }
}

#[cfg(windows)]
mod platform_win {
    use anyhow::Result;
    use std::sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    };
    use tokio::sync::mpsc;
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        DispatchMessageW, GetMessageW, MSG, TranslateMessage, WM_HOTKEY,
    };

    use super::{HotkeyAction, HotkeyConfig, HotkeyKey, Modifier};

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
        hotkey_config: HotkeyConfig,
        enabled: Arc<AtomicBool>,
    ) -> Result<()> {
        let mut hotkeys = vec![
            (
                1,
                get_win32_modifiers(&hotkey_config.translate_system.modifiers) | 0x4000,
                get_win32_vk(&hotkey_config.translate_system.key),
            ),
            (
                2,
                get_win32_modifiers(&hotkey_config.translate_english.modifiers) | 0x4000,
                get_win32_vk(&hotkey_config.translate_english.key),
            ),
            (
                3,
                get_win32_modifiers(&hotkey_config.execute_instruction.modifiers) | 0x4000,
                get_win32_vk(&hotkey_config.execute_instruction.key),
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

        for (i, (pat, _)) in hotkey_config.custom_instructions.iter().enumerate() {
            hotkeys.push((
                100 + i as i32,
                get_win32_modifiers(&pat.modifiers) | 0x4000,
                get_win32_vk(&pat.key),
            ));
        }

        let hotkey_config_clone = hotkey_config.clone();

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
                            id if id >= 100 => {
                                let idx = (id - 100) as usize;
                                if idx < hotkey_config_clone.custom_instructions.len() {
                                    HotkeyAction::Custom(
                                        hotkey_config_clone.custom_instructions[idx].1.clone(),
                                    )
                                } else {
                                    continue;
                                }
                            }
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

    use super::{HotkeyAction, HotkeyConfig, HotkeyKey, HotkeyPattern, Modifier, match_pattern};

    fn modifier_from_key(key: &Key) -> Option<&'static str> {
        match key {
            Key::ControlLeft | Key::ControlRight => Some("ctrl"),
            Key::Alt | Key::AltGr => Some("alt"),
            Key::ShiftLeft | Key::ShiftRight => Some("shift"),
            Key::MetaLeft | Key::MetaRight => Some("cmd"),
            _ => None,
        }
    }

    fn key_to_str(key: &Key) -> Option<&'static str> {
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
            Key::SemiColon => Some(";"),
            _ => None,
        }
    }

    pub fn start(
        tx: mpsc::Sender<HotkeyAction>,
        hotkey_config: HotkeyConfig,
        enabled: Arc<AtomicBool>,
    ) -> Result<()> {
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
                            pressed.lock().unwrap().insert(key_name.to_string());
                        } else if let Some(ref name) = event.name {
                            pressed.lock().unwrap().insert(name.to_lowercase());
                        }
                    }
                    EventType::KeyRelease(key) => {
                        if let Some(mod_name) = modifier_from_key(&key) {
                            pressed.lock().unwrap().remove(mod_name);
                        }
                        let mut key_name_to_remove = None;
                        if let Some(key_name) = key_to_str(&key) {
                            key_name_to_remove = Some(key_name.to_string());
                        } else if let Some(ref name) = event.name {
                            key_name_to_remove = Some(name.to_lowercase());
                        }

                        if let Some(kname) = key_name_to_remove {
                            let mut p = pressed.lock().unwrap();
                            p.remove(&kname);
                            let mut action = None;
                            if match_pattern(&p, &hotkey_config.translate_system) {
                                action = Some(HotkeyAction::TranslateDefault);
                            } else if match_pattern(&p, &hotkey_config.translate_english) {
                                action = Some(HotkeyAction::TranslateEnglish);
                            } else if match_pattern(&p, &hotkey_config.execute_instruction) {
                                action = Some(HotkeyAction::ExecuteInstruction);
                            } else if match_pattern(
                                &p,
                                &HotkeyPattern {
                                    modifiers: vec![Modifier::Ctrl, Modifier::Shift, Modifier::Win],
                                    key: HotkeyKey::Letter('r'),
                                },
                            ) {
                                action = Some(HotkeyAction::Reformulate);
                            } else if match_pattern(
                                &p,
                                &HotkeyPattern {
                                    modifiers: vec![Modifier::Ctrl, Modifier::Shift, Modifier::Win],
                                    key: HotkeyKey::Letter('p'),
                                },
                            ) {
                                action = Some(HotkeyAction::ScreenshotAnalysis);
                            } else {
                                for (pat, inst) in &hotkey_config.custom_instructions {
                                    if match_pattern(&p, pat) {
                                        action = Some(HotkeyAction::Custom(inst.clone()));
                                        break;
                                    }
                                }
                            }
                            if let Some(action) = action
                                && tx_clone.try_send(action).is_err()
                            {
                                tracing::warn!("hotkey channel full, dropping event");
                            }
                        }
                    }
                    _ => {}
                }
            }) {
                tracing::error!("rdev listen failed: {e:?}");
            }
        });
        Ok(())
    }
}

#[cfg(target_os = "linux")]
mod platform_linux {
    use anyhow::Result;
    use rdev::{EventType, Key, listen};
    use std::collections::HashSet;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::{Arc, Mutex};
    use tokio::sync::mpsc;

    use super::{HotkeyAction, HotkeyConfig, HotkeyKey, HotkeyPattern, Modifier, match_pattern};

    fn modifier_from_key(key: &Key) -> Option<&'static str> {
        match key {
            Key::ControlLeft | Key::ControlRight => Some("ctrl"),
            Key::Alt | Key::AltGr => Some("alt"),
            Key::ShiftLeft | Key::ShiftRight => Some("shift"),
            Key::MetaLeft | Key::MetaRight | Key::Unknown(133) | Key::Unknown(134) => Some("win"),
            _ => None,
        }
    }

    fn key_to_str(key: &Key) -> Option<&'static str> {
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
            Key::SemiColon => Some(";"),
            _ => None,
        }
    }

    pub fn start(
        tx: mpsc::Sender<HotkeyAction>,
        hotkey_config: HotkeyConfig,
        enabled: Arc<AtomicBool>,
    ) -> Result<()> {
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
                            pressed.lock().unwrap().insert(key_name.to_string());
                        } else if let Some(ref name) = event.name {
                            pressed.lock().unwrap().insert(name.to_lowercase());
                        }
                    }
                    EventType::KeyRelease(key) => {
                        if let Some(mod_name) = modifier_from_key(&key) {
                            pressed.lock().unwrap().remove(mod_name);
                        }
                        let mut key_name_to_remove = None;
                        if let Some(key_name) = key_to_str(&key) {
                            key_name_to_remove = Some(key_name.to_string());
                        } else if let Some(ref name) = event.name {
                            key_name_to_remove = Some(name.to_lowercase());
                        }

                        if let Some(kname) = key_name_to_remove {
                            let mut p = pressed.lock().unwrap();
                            p.remove(&kname);
                            let mut action = None;
                            if match_pattern(&p, &hotkey_config.translate_system) {
                                action = Some(HotkeyAction::TranslateDefault);
                            } else if match_pattern(&p, &hotkey_config.translate_english) {
                                action = Some(HotkeyAction::TranslateEnglish);
                            } else if match_pattern(&p, &hotkey_config.execute_instruction) {
                                action = Some(HotkeyAction::ExecuteInstruction);
                            } else if match_pattern(
                                &p,
                                &HotkeyPattern {
                                    modifiers: vec![Modifier::Ctrl, Modifier::Shift, Modifier::Win],
                                    key: HotkeyKey::Letter('r'),
                                },
                            ) {
                                action = Some(HotkeyAction::Reformulate);
                            } else if match_pattern(
                                &p,
                                &HotkeyPattern {
                                    modifiers: vec![Modifier::Ctrl, Modifier::Shift, Modifier::Win],
                                    key: HotkeyKey::Letter('p'),
                                },
                            ) {
                                action = Some(HotkeyAction::ScreenshotAnalysis);
                            } else {
                                for (pat, inst) in &hotkey_config.custom_instructions {
                                    if match_pattern(&p, pat) {
                                        action = Some(HotkeyAction::Custom(inst.clone()));
                                        break;
                                    }
                                }
                            }
                            if let Some(action) = action
                                && tx_clone.try_send(action).is_err()
                            {
                                tracing::warn!("hotkey channel full, dropping event");
                            }
                        }
                    }
                    _ => {}
                }
            }) {
                tracing::error!("rdev listen failed: {e:?}");
            }
        });
        Ok(())
    }
}
