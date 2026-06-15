#[cfg(windows)]
mod platform {
    use notify_rust::Notification;

    pub fn notify_success() {
        if let Err(e) = Notification::new()
            .summary("Thoth")
            .body("✓ Texte traduit avec succès")
            .appname("Thoth")
            .timeout(2000)
            .show()
        {
            tracing::warn!("notification failed: {e}");
        }
    }

    pub fn notify_error(context: &str) {
        if let Err(e) = Notification::new()
            .summary("Thoth")
            .body(&format!("✗ {context}"))
            .appname("Thoth")
            .timeout(4000)
            .show()
        {
            tracing::warn!("notification failed: {e}");
        }
    }

    pub fn notify_warning(context: &str) {
        if let Err(e) = Notification::new()
            .summary("Thoth")
            .body(&format!("⚠ {context}"))
            .appname("Thoth")
            .timeout(4000)
            .show()
        {
            tracing::warn!("notification failed: {e}");
        }
    }

    pub fn notify_screenshot_analysis() {
        if let Err(e) = Notification::new()
            .summary("Thoth")
            .body("📷 Analyse d'écran en cours…")
            .appname("Thoth")
            .timeout(2000)
            .show()
        {
            tracing::warn!("notification failed: {e}");
        }
    }
}

#[cfg(not(windows))]
mod platform {
    pub fn notify_success() {
        tracing::info!("[notification] success (unsupported on this platform)");
    }

    pub fn notify_error(context: &str) {
        tracing::error!("[notification] {context}");
    }

    pub fn notify_warning(context: &str) {
        tracing::warn!("[notification] {context}");
    }

    pub fn notify_screenshot_analysis() {
        tracing::info!("[notification] screenshot analysis started");
    }
}

pub use platform::*;
