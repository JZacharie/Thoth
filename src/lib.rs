pub mod auto_start;
pub mod clipboard;
pub mod config;
pub mod dialog;
pub mod gui;
pub mod hotkey;
pub mod metrics;
pub mod notification;
pub mod orchestrator;
pub mod pylos_client;
pub mod tray;

pub use config::{BehaviorConfig, Config, PylosConfig, validate_language};
pub use hotkey::{HotkeyKey, HotkeyPattern, Modifier};
pub use pylos_client::{PylosClient, is_sensitive};

use std::sync::atomic::{AtomicBool, Ordering};

static INSECURE: AtomicBool = AtomicBool::new(false);

pub fn set_insecure(val: bool) {
    INSECURE.store(val, Ordering::Relaxed);
}

pub fn is_insecure() -> bool {
    INSECURE.load(Ordering::Relaxed)
}
