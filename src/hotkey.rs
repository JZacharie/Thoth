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

#[derive(Debug, Clone, PartialEq)]
pub enum HotkeyKey {
    Letter(char),
    Number(u8),
    Space,
    F(u8),
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
            anyhow::bail!("invalid hotkey format: '{s}' — expected e.g. Win+N");
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
    use rdev::{Event, EventType, Key, listen};
    use std::sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    };
    use tokio::sync::mpsc;

    use super::{HotkeyKey, HotkeyPattern, Modifier};

    fn key_to_letter(key: &Key) -> Option<char> {
        match key {
            Key::KeyA => Some('a'),
            Key::KeyB => Some('b'),
            Key::KeyC => Some('c'),
            Key::KeyD => Some('d'),
            Key::KeyE => Some('e'),
            Key::KeyF => Some('f'),
            Key::KeyG => Some('g'),
            Key::KeyH => Some('h'),
            Key::KeyI => Some('i'),
            Key::KeyJ => Some('j'),
            Key::KeyK => Some('k'),
            Key::KeyL => Some('l'),
            Key::KeyM => Some('m'),
            Key::KeyN => Some('n'),
            Key::KeyO => Some('o'),
            Key::KeyP => Some('p'),
            Key::KeyQ => Some('q'),
            Key::KeyR => Some('r'),
            Key::KeyS => Some('s'),
            Key::KeyT => Some('t'),
            Key::KeyU => Some('u'),
            Key::KeyV => Some('v'),
            Key::KeyW => Some('w'),
            Key::KeyX => Some('x'),
            Key::KeyY => Some('y'),
            Key::KeyZ => Some('z'),
            _ => None,
        }
    }

    fn key_to_digit(key: &Key) -> Option<u8> {
        match key {
            Key::Num0 => Some(0),
            Key::Num1 => Some(1),
            Key::Num2 => Some(2),
            Key::Num3 => Some(3),
            Key::Num4 => Some(4),
            Key::Num5 => Some(5),
            Key::Num6 => Some(6),
            Key::Num7 => Some(7),
            Key::Num8 => Some(8),
            Key::Num9 => Some(9),
            _ => None,
        }
    }

    fn key_to_f(key: &Key) -> Option<u8> {
        match key {
            Key::F1 => Some(1),
            Key::F2 => Some(2),
            Key::F3 => Some(3),
            Key::F4 => Some(4),
            Key::F5 => Some(5),
            Key::F6 => Some(6),
            Key::F7 => Some(7),
            Key::F8 => Some(8),
            Key::F9 => Some(9),
            Key::F10 => Some(10),
            Key::F11 => Some(11),
            Key::F12 => Some(12),
            _ => None,
        }
    }

    pub fn start(
        tx: mpsc::Sender<()>,
        pattern: Arc<Mutex<HotkeyPattern>>,
        enabled: Arc<AtomicBool>,
    ) -> Result<()> {
        std::thread::spawn(move || {
            let mut ctrl_pressed = false;
            let mut alt_pressed = false;
            let mut shift_pressed = false;
            let mut meta_pressed = false;

            let callback = move |event: Event| {
                if !enabled.load(Ordering::Relaxed) {
                    return;
                }

                match event.event_type {
                    EventType::KeyPress(Key::ControlLeft)
                    | EventType::KeyPress(Key::ControlRight) => {
                        ctrl_pressed = true;
                    }
                    EventType::KeyRelease(Key::ControlLeft)
                    | EventType::KeyRelease(Key::ControlRight) => {
                        ctrl_pressed = false;
                    }
                    EventType::KeyPress(Key::Alt) => {
                        alt_pressed = true;
                    }
                    EventType::KeyRelease(Key::Alt) => {
                        alt_pressed = false;
                    }
                    EventType::KeyPress(Key::ShiftLeft) | EventType::KeyPress(Key::ShiftRight) => {
                        shift_pressed = true;
                    }
                    EventType::KeyRelease(Key::ShiftLeft)
                    | EventType::KeyRelease(Key::ShiftRight) => {
                        shift_pressed = false;
                    }
                    EventType::KeyPress(Key::MetaLeft) | EventType::KeyPress(Key::MetaRight) => {
                        meta_pressed = true;
                    }
                    EventType::KeyRelease(Key::MetaLeft)
                    | EventType::KeyRelease(Key::MetaRight) => {
                        meta_pressed = false;
                    }
                    EventType::KeyPress(key) => {
                        let pat = pattern.lock().unwrap();
                        let mods_ok = pat.modifiers.iter().all(|m| match m {
                            Modifier::Win => meta_pressed,
                            Modifier::Ctrl => ctrl_pressed,
                            Modifier::Alt => alt_pressed,
                            Modifier::Shift => shift_pressed,
                        });
                        if !mods_ok {
                            return;
                        }
                        let key_ok = match &pat.key {
                            HotkeyKey::Letter(ch) => {
                                key_to_letter(&key) == Some(*ch)
                            }
                            HotkeyKey::Number(n) => key_to_digit(&key) == Some(*n),
                            HotkeyKey::Space => matches!(key, Key::Space),
                            HotkeyKey::F(n) => key_to_f(&key) == Some(*n),
                        };
                        if key_ok {
                            tracing::debug!("hotkey triggered");
                            if tx.try_send(()).is_err() {
                                tracing::warn!("hotkey channel full, dropping event");
                            }
                        }
                    }
                    _ => {}
                }
            };

            if let Err(err) = listen(callback) {
                tracing::error!("hotkey listener error: {err:?}");
            }
        });

        Ok(())
    }
}

#[cfg(not(windows))]
mod platform {
    use super::HotkeyPattern;
    use anyhow::Result;
    use std::sync::{Arc, Mutex, atomic::AtomicBool};
    use tokio::sync::mpsc;

    pub fn start(
        _tx: mpsc::Sender<()>,
        _pattern: Arc<Mutex<HotkeyPattern>>,
        _enabled: Arc<AtomicBool>,
    ) -> Result<()> {
        tracing::warn!("global hotkey not supported on this platform");
        Ok(())
    }
}

pub fn start(
    tx: mpsc::Sender<()>,
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
