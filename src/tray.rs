#[cfg(windows)]
mod platform {
    use crate::auto_start;
    use crate::metrics::UsageMetrics;
    use anyhow::Result;
    use std::path::PathBuf;
    use std::sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    };
    use tokio::sync::oneshot;
    use tray_icon::{
        Icon, TrayIconBuilder,
        menu::{CheckMenuItem, Menu, MenuEvent, MenuItem, PredefinedMenuItem},
    };
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        DispatchMessageW, GetMessageW, MSG, PostQuitMessage, TranslateMessage,
    };

    struct MenuStrings {
        status_enabled: &'static str,
        status_disabled: &'static str,
        toggle_disable: &'static str,
        toggle_enable: &'static str,
        auto_start: &'static str,
        config: &'static str,
        stats: &'static str,
        reset_stats: &'static str,
        stats_translations: &'static str,
        stats_errors: &'static str,
        stats_latency: &'static str,
        stats_models: &'static str,
        logs: &'static str,
        quit: &'static str,
        tooltip: &'static str,
    }

    fn menu_strings() -> MenuStrings {
        match crate::config::system_language().as_str() {
            "fr" => MenuStrings {
                status_enabled: "Thoth — Actif",
                status_disabled: "Thoth — Désactivé",
                toggle_disable: "Désactiver",
                toggle_enable: "Activer",
                auto_start: "Démarrer avec Windows",
                config: "Configuration",
                stats: "Statistiques",
                reset_stats: "Réinitialiser les stats",
                stats_translations: "Traductions",
                stats_errors: "Erreurs",
                stats_latency: "Latence moy.",
                stats_models: "Modèles",
                logs: "Journaux",
                quit: "Quitter",
                tooltip: "Thoth — Traducteur instantané",
            },
            _ => MenuStrings {
                status_enabled: "Thoth — Enabled",
                status_disabled: "Thoth — Disabled",
                toggle_disable: "Disable",
                toggle_enable: "Enable",
                auto_start: "Start with Windows",
                config: "Configuration",
                stats: "Statistics",
                reset_stats: "Reset statistics",
                stats_translations: "Translations",
                stats_errors: "Errors",
                stats_latency: "Avg. latency",
                stats_models: "Models",
                logs: "Logs",
                quit: "Quit",
                tooltip: "Thoth — Instant translator",
            },
        }
    }

    pub fn start(
        shutdown_tx: oneshot::Sender<()>,
        enabled: Arc<AtomicBool>,
        log_path: PathBuf,
        config_path: PathBuf,
    ) -> Result<()> {
        let s = menu_strings();
        let menu = Menu::new();

        let status_item = MenuItem::new(s.status_enabled, true, None);
        let toggle_item = MenuItem::new(s.toggle_disable, true, None);
        let auto_start_item =
            CheckMenuItem::new(s.auto_start, true, auto_start::is_enabled(), None);
        let config_item = MenuItem::new(s.config, true, None);
        let stats_item = MenuItem::new(s.stats, true, None);
        let reset_stats_item = MenuItem::new(s.reset_stats, true, None);
        let logs_item = MenuItem::new(s.logs, true, None);
        let quit_item = MenuItem::new(s.quit, true, None);

        menu.append(&status_item)?;
        menu.append(&PredefinedMenuItem::separator())?;
        menu.append(&toggle_item)?;
        menu.append(&auto_start_item)?;
        menu.append(&PredefinedMenuItem::separator())?;
        menu.append(&config_item)?;
        menu.append(&PredefinedMenuItem::separator())?;
        menu.append(&stats_item)?;
        menu.append(&reset_stats_item)?;
        menu.append(&PredefinedMenuItem::separator())?;
        menu.append(&logs_item)?;
        menu.append(&PredefinedMenuItem::separator())?;
        menu.append(&quit_item)?;

        // Load thoth.png from embedded resources
        let png_bytes = include_bytes!("../resources/thoth.png");
        let decoded = image::load_from_memory_with_format(png_bytes, image::ImageFormat::Png)?;
        let rgba_img = decoded.to_rgba8();
        let (width, height) = rgba_img.dimensions();
        let color_icon = Icon::from_rgba(rgba_img.clone().into_raw(), width, height)?;

        // Génère la version noir et blanc (grayscale) de l'image
        let grayscale_decoded = image::DynamicImage::ImageRgba8(rgba_img).grayscale();
        let grayscale_rgba = grayscale_decoded.to_rgba8();
        let grayscale_icon = Icon::from_rgba(grayscale_rgba.into_raw(), width, height)?;

        let initial_enabled = enabled.load(Ordering::Relaxed);

        let mut builder = TrayIconBuilder::new()
            .with_tooltip(s.tooltip)
            .with_menu(Box::new(menu));

        if initial_enabled {
            builder = builder.with_icon(color_icon.clone());
        } else {
            builder = builder.with_icon(grayscale_icon.clone());
        }
        let tray = builder.build()?;

        let mut shutdown_tx = Some(shutdown_tx);

        unsafe {
            let mut msg = std::mem::zeroed::<MSG>();
            loop {
                let result = GetMessageW(&mut msg, std::ptr::null_mut(), 0, 0);
                if result == 0 || result == -1 {
                    break;
                }
                TranslateMessage(&msg);
                DispatchMessageW(&msg);

                while let Ok(event) = MenuEvent::receiver().try_recv() {
                    if event.id == quit_item.id() {
                        tracing::info!("tray: quit requested");
                        if let Some(tx) = shutdown_tx.take() {
                            let _ = tx.send(());
                        }
                        PostQuitMessage(0);
                    } else if event.id == toggle_item.id() {
                        let new_state = !enabled.load(Ordering::Relaxed);
                        enabled.store(new_state, Ordering::Relaxed);
                        let s = menu_strings();
                        toggle_item.set_text(if new_state {
                            s.toggle_disable
                        } else {
                            s.toggle_enable
                        });
                        status_item.set_text(if new_state {
                            s.status_enabled
                        } else {
                            s.status_disabled
                        });
                        if new_state {
                            let _ = tray.set_icon(Some(color_icon.clone()));
                        } else {
                            let _ = tray.set_icon(Some(grayscale_icon.clone()));
                        }
                    } else if event.id == auto_start_item.id() {
                        let new_state = !auto_start::is_enabled();
                        if new_state {
                            let _ = auto_start::enable();
                        } else {
                            let _ = auto_start::disable();
                        }
                        auto_start_item.set_checked(new_state);
                    } else if event.id == config_item.id() {
                        if config_path.exists() {
                            let _ = std::process::Command::new("cmd")
                                .args(["/c", "start", "", &config_path.to_string_lossy()])
                                .spawn();
                        }
                    } else if event.id == stats_item.id() {
                        let s = menu_strings();
                        let metrics = UsageMetrics::load();
                        let models = metrics
                            .model_usage
                            .iter()
                            .map(|(m, c)| format!("{m}: {c}"))
                            .collect::<Vec<_>>()
                            .join(", ");
                        let msg = format!(
                            "{}: {}\n{}: {}\n{}: {:.0} ms\n{}: {}",
                            s.stats_translations,
                            metrics.total_translations,
                            s.stats_errors,
                            metrics.total_errors,
                            s.stats_latency,
                            metrics.avg_latency_ms(),
                            s.stats_models,
                            models,
                        );
                        tracing::info!("Stats:\n{}", msg);
                    } else if event.id == reset_stats_item.id() {
                        UsageMetrics::default().save();
                        tracing::info!("stats reset");
                    } else if event.id == logs_item.id() {
                        if log_path.exists() {
                            let _ = std::process::Command::new("powershell")
                                .args([
                                    "-NoProfile",
                                    "-Command",
                                    &format!(
                                        "Start-Process powershell -ArgumentList '-NoExit', '-Command', 'Get-Content \\\"{}\\\" -Wait -Tail 50'",
                                        log_path.to_string_lossy()
                                    ),
                                ])
                                .spawn();
                        } else {
                            tracing::warn!("log file not found: {}", log_path.display());
                        }
                    }
                }
            }
        }

        Ok(())
    }
}

#[cfg(not(windows))]
mod platform {
    use anyhow::Result;
    use std::path::PathBuf;
    use std::sync::{Arc, atomic::AtomicBool};
    use tokio::sync::oneshot;

    pub fn start(
        _shutdown_tx: oneshot::Sender<()>,
        _enabled: Arc<AtomicBool>,
        _log_path: PathBuf,
        _config_path: PathBuf,
    ) -> Result<()> {
        tracing::warn!("system tray not supported on this platform");
        Ok(())
    }
}

pub use platform::*;
