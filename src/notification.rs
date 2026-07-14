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

#[cfg(target_os = "linux")]
mod platform {
    use notify_rust::Notification;
    use std::process::Command;

    fn fallback_notify(summary: &str, body: &str) {
        if Command::new("notify-send")
            .args([summary, body])
            .output()
            .is_ok()
        {
            return;
        }
        if Command::new("dunstify")
            .args([summary, body])
            .output()
            .is_ok()
        {
            return;
        }
        tracing::debug!("no notification daemon found (install dunst, mako, or notify-send)");
    }

    fn try_notify(summary: &str, body: &str, timeout: notify_rust::Timeout) {
        if let Err(e) = Notification::new()
            .summary(summary)
            .body(body)
            .timeout(timeout)
            .show()
        {
            tracing::warn!("notification via notify-rust failed: {e}, trying fallback");
            fallback_notify(summary, body);
        }
    }

    pub fn notify_success() {
        try_notify(
            "Thoth",
            "✓ Texte traduit avec succès",
            notify_rust::Timeout::Milliseconds(2000),
        );
    }

    pub fn notify_error(context: &str) {
        try_notify(
            "Thoth",
            &format!("✗ {context}"),
            notify_rust::Timeout::Milliseconds(4000),
        );
    }

    pub fn notify_warning(context: &str) {
        try_notify(
            "Thoth",
            &format!("⚠ {context}"),
            notify_rust::Timeout::Milliseconds(4000),
        );
    }

    pub fn notify_screenshot_analysis() {
        try_notify(
            "Thoth",
            "📷 Analyse d'écran en cours…",
            notify_rust::Timeout::Milliseconds(2000),
        );
    }
}

#[cfg(target_os = "macos")]
mod platform {
    use notify_rust::Notification;

    pub fn notify_success() {
        if let Err(e) = Notification::new()
            .summary("Thoth")
            .body("✓ Texte traduit avec succès")
            .timeout(notify_rust::Timeout::Milliseconds(2000))
            .show()
        {
            tracing::warn!("notification failed: {e}");
        }
    }

    pub fn notify_error(context: &str) {
        if let Err(e) = Notification::new()
            .summary("Thoth")
            .body(&format!("✗ {context}"))
            .timeout(notify_rust::Timeout::Milliseconds(4000))
            .show()
        {
            tracing::warn!("notification failed: {e}");
        }
    }

    pub fn notify_warning(context: &str) {
        if let Err(e) = Notification::new()
            .summary("Thoth")
            .body(&format!("⚠ {context}"))
            .timeout(notify_rust::Timeout::Milliseconds(4000))
            .show()
        {
            tracing::warn!("notification failed: {e}");
        }
    }

    pub fn notify_screenshot_analysis() {
        if let Err(e) = Notification::new()
            .summary("Thoth")
            .body("📷 Analyse d'écran en cours…")
            .timeout(notify_rust::Timeout::Milliseconds(2000))
            .show()
        {
            tracing::warn!("notification failed: {e}");
        }
    }
}

#[cfg(not(any(windows, target_os = "macos", target_os = "linux")))]
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
