#[derive(Debug, thiserror::Error)]
pub enum ThothError {
    #[error("Hotkey error: {0}")]
    Hotkey(String),

    #[error("Clipboard error: {0}")]
    Clipboard(String),

    #[error("Pylos error: {0}")]
    Pylos(String),

    #[error("Config error: {0}")]
    Config(String),

    #[error("Notification error: {0}")]
    Notification(String),

    #[error("Tray error: {0}")]
    Tray(String),

    #[error("Sensitive data detected — request blocked")]
    SensitiveData,
}
