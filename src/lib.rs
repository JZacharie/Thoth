pub mod auto_start;
pub mod clipboard;
pub mod config;
pub mod dialog;
pub mod groq_client;
pub mod gui;
pub mod hotkey;
pub mod metrics;
pub mod mqtt;
pub mod notification;
pub mod orchestrator;
pub mod s3_storage;
pub mod screenshot;
pub mod secure_storage;
pub mod tray;
pub mod vision;

pub use config::{BehaviorConfig, Config, GroqConfig, validate_language};
pub use groq_client::{GroqClient, is_sensitive};
pub use hotkey::{HotkeyKey, HotkeyPattern, Modifier};

pub type PylosConfig = GroqConfig;
pub type PylosClient = GroqClient;
pub mod pylos_client {
    pub use crate::groq_client::*;
}

use std::sync::atomic::{AtomicBool, Ordering};

static INSECURE: AtomicBool = AtomicBool::new(false);

pub fn set_insecure(val: bool) {
    INSECURE.store(val, Ordering::Relaxed);
}

pub fn is_insecure() -> bool {
    INSECURE.load(Ordering::Relaxed)
}
