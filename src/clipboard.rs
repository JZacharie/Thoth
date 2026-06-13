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
        simulate_ctrl_c()?;
        std::thread::sleep(Duration::from_millis(100));
        let text = self.inner.get_text()?;
        Ok(text)
    }

    pub fn paste_text(&mut self, text: &str, restore: bool) -> Result<()> {
        self.inner.set_text(text)?;
        simulate_ctrl_v()?;
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
}

#[cfg(windows)]
mod platform {
    use anyhow::Result;
    use rdev::{EventType, Key, simulate};
    use std::time::Duration;

    unsafe extern "system" {
        fn GetAsyncKeyState(vkey: i32) -> i16;
    }

    pub fn wait_for_modifiers_release() {
        let start = std::time::Instant::now();
        // 0x10 = Shift, 0x11 = Ctrl, 0x12 = Alt, 0x5B = LWin, 0x5C = RWin
        while unsafe {
            (GetAsyncKeyState(0x10) as u16 & 0x8000) != 0
                || (GetAsyncKeyState(0x11) as u16 & 0x8000) != 0
                || (GetAsyncKeyState(0x12) as u16 & 0x8000) != 0
                || (GetAsyncKeyState(0x5B) as u16 & 0x8000) != 0
                || (GetAsyncKeyState(0x5C) as u16 & 0x8000) != 0
        } {
            if start.elapsed() > Duration::from_millis(500) {
                break;
            }
            std::thread::sleep(Duration::from_millis(10));
        }
    }

    pub fn simulate_ctrl_c() -> Result<()> {
        wait_for_modifiers_release();
        simulate(&EventType::KeyPress(Key::ControlLeft))?;
        simulate(&EventType::KeyPress(Key::KeyC))?;
        std::thread::sleep(Duration::from_millis(10));
        simulate(&EventType::KeyRelease(Key::KeyC))?;
        simulate(&EventType::KeyRelease(Key::ControlLeft))?;
        Ok(())
    }

    pub fn simulate_ctrl_v() -> Result<()> {
        wait_for_modifiers_release();
        simulate(&EventType::KeyPress(Key::ControlLeft))?;
        simulate(&EventType::KeyPress(Key::KeyV))?;
        std::thread::sleep(Duration::from_millis(10));
        simulate(&EventType::KeyRelease(Key::KeyV))?;
        simulate(&EventType::KeyRelease(Key::ControlLeft))?;
        Ok(())
    }
}

#[cfg(not(windows))]
mod platform {
    use anyhow::Result;

    pub fn wait_for_modifiers_release() {}

    pub fn simulate_ctrl_c() -> Result<()> {
        tracing::warn!("simulate_ctrl_c: not supported on this platform");
        Ok(())
    }

    pub fn simulate_ctrl_v() -> Result<()> {
        tracing::warn!("simulate_ctrl_v: not supported on this platform");
        Ok(())
    }
}

use platform::*;
