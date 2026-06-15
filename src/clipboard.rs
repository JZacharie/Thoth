use std::time::Duration;

use anyhow::Result;
use arboard::Clipboard;

pub struct ClipboardManager {
    inner: Clipboard,
    previous: Option<String>,
}

impl ClipboardManager {
    pub fn new() -> Result<Self> {
        let inner = Clipboard::new()?;
        Ok(Self {
            inner,
            previous: None,
        })
    }

    pub fn copy_selected_text(&mut self) -> Result<String> {
        self.previous = self.inner.get_text().ok();
        platform::simulate_copy()?;
        std::thread::sleep(Duration::from_millis(100));
        let text = self.inner.get_text()?;
        Ok(text)
    }

    pub fn paste_text(&mut self, text: &str, restore: bool) -> Result<()> {
        self.inner.set_text(text)?;
        platform::simulate_paste()?;
        if restore {
            std::thread::sleep(Duration::from_millis(250));
            self.restore()?;
        }
        Ok(())
    }

    pub fn restore(&mut self) -> Result<()> {
        if let Some(prev) = self.previous.take() {
            self.inner.set_text(prev)?;
        } else {
            let _ = self.inner.clear();
        }
        Ok(())
    }

    pub fn simulate_select_all(&mut self) -> Result<()> {
        platform::simulate_select_all()
    }
}

#[cfg(any(windows, target_os = "macos"))]
mod platform {
    use anyhow::Result;
    use rdev::{EventType, Key, simulate};
    use std::time::Duration;

    fn modifier_key() -> Key {
        #[cfg(target_os = "macos")]
        {
            Key::MetaLeft
        }
        #[cfg(windows)]
        {
            Key::ControlLeft
        }
    }

    fn wait_for_modifiers_release() {
        let start = std::time::Instant::now();
        #[cfg(windows)]
        {
            unsafe extern "system" {
                fn GetAsyncKeyState(vkey: i32) -> i16;
            }
            unsafe {
                while (GetAsyncKeyState(0x10) as u16 & 0x8000) != 0
                    || (GetAsyncKeyState(0x11) as u16 & 0x8000) != 0
                    || (GetAsyncKeyState(0x12) as u16 & 0x8000) != 0
                    || (GetAsyncKeyState(0x5B) as u16 & 0x8000) != 0
                    || (GetAsyncKeyState(0x5C) as u16 & 0x8000) != 0
                {
                    if start.elapsed() > Duration::from_millis(500) {
                        break;
                    }
                    std::thread::sleep(Duration::from_millis(10));
                }
            }
        }
        #[cfg(not(windows))]
        {
            std::thread::sleep(Duration::from_millis(50));
        }
    }

    fn press(key: Key) -> Result<()> {
        simulate(&EventType::KeyPress(key))?;
        std::thread::sleep(Duration::from_millis(10));
        simulate(&EventType::KeyRelease(key))?;
        Ok(())
    }

    fn press_with_modifier(key: Key) -> Result<()> {
        wait_for_modifiers_release();
        simulate(&EventType::KeyPress(modifier_key()))?;
        std::thread::sleep(Duration::from_millis(10));
        press(key)?;
        simulate(&EventType::KeyRelease(modifier_key()))?;
        Ok(())
    }

    pub fn simulate_copy() -> Result<()> {
        press_with_modifier(Key::KeyC)
    }

    pub fn simulate_paste() -> Result<()> {
        press_with_modifier(Key::KeyV)
    }

    pub fn simulate_select_all() -> Result<()> {
        press_with_modifier(Key::KeyA)
    }
}

#[cfg(not(any(windows, target_os = "macos")))]
mod platform {
    use anyhow::Result;

    pub fn simulate_copy() -> Result<()> {
        tracing::warn!("keyboard simulation not supported on Linux");
        Ok(())
    }

    pub fn simulate_paste() -> Result<()> {
        tracing::warn!("keyboard simulation not supported on Linux");
        Ok(())
    }

    pub fn simulate_select_all() -> Result<()> {
        tracing::warn!("keyboard simulation not supported on Linux");
        Ok(())
    }
}
