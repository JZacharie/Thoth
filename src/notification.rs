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

    /// Detect the D-Bus session bus address, which may be missing under sudo.
    fn detect_dbus_address() -> Option<String> {
        if let Ok(addr) = std::env::var("DBUS_SESSION_BUS_ADDRESS")
            && !addr.is_empty()
        {
            return Some(addr);
        }

        let uid = unsafe { libc::getuid() };

        // When running under sudo, try the original user's session bus
        if let Ok(sudo_uid_str) = std::env::var("SUDO_UID")
            && let Ok(sudo_uid) = sudo_uid_str.parse::<u32>()
        {
            let path = format!("/run/user/{}/bus", sudo_uid);
            if std::path::Path::new(&path).exists() {
                return Some(format!("unix:path={}", path));
            }
            // Some distros use display manager-managed paths
            let alt = format!("/run/user/{}/wayland-0", sudo_uid);
            if std::path::Path::new(&alt).exists() {
                return Some(format!("unix:path={}", path));
            }
        }

        // Fallback: try the current process UID's bus socket
        if uid != 0 {
            let path = format!("/run/user/{}/bus", uid);
            if std::path::Path::new(&path).exists() {
                return Some(format!("unix:path={}", path));
            }
        }

        None
    }

    fn fallback_notify(summary: &str, body: &str) {
        let mut cmd = Command::new("notify-send");
        cmd.args([summary, body]);
        // Ensure D-Bus address is set for sudo contexts
        if let Some(addr) = detect_dbus_address() {
            cmd.env("DBUS_SESSION_BUS_ADDRESS", &addr);
        }
        if cmd.output().is_ok() {
            return;
        }
        let mut cmd2 = Command::new("dunstify");
        cmd2.args([summary, body]);
        if let Some(addr) = detect_dbus_address() {
            cmd2.env("DBUS_SESSION_BUS_ADDRESS", &addr);
        }
        if cmd2.output().is_ok() {
            return;
        }
        tracing::debug!("no notification daemon found (install dunst, mako, or notify-send)");
    }

    fn try_notify(summary: &str, body: &str, timeout: notify_rust::Timeout) {
        // If running under sudo and DBUS_SESSION_BUS_ADDRESS is missing, set it
        // so notify-rust can connect to the user's session bus.
        if std::env::var("DBUS_SESSION_BUS_ADDRESS").is_err()
            && let Some(addr) = detect_dbus_address()
        {
            // SAFETY: called once before any threads use this variable,
            // and only when the env var is not already set.
            unsafe { std::env::set_var("DBUS_SESSION_BUS_ADDRESS", &addr) };
        }

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
