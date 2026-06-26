#[cfg(any(windows, target_os = "macos", target_os = "linux"))]
mod tray_impl {
    use anyhow::Result;
    use std::path::PathBuf;
    use std::sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    };
    use tokio::sync::oneshot;
    use tray_icon::{
        Icon, TrayIcon, TrayIconBuilder,
        menu::{CheckMenuItem, Menu, MenuEvent, MenuItem, PredefinedMenuItem},
    };

    use crate::auto_start;
    use crate::metrics::UsageMetrics;

    struct MenuStrings {
        status_enabled: &'static str,
        status_disabled: &'static str,
        toggle_disable: &'static str,
        toggle_enable: &'static str,
        auto_start: &'static str,
        config: &'static str,
        stats: &'static str,
        reset_stats: &'static str,
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
                auto_start: "Démarrer au démarrage",
                config: "Configuration",
                stats: "Statistiques",
                reset_stats: "Réinitialiser les stats",
                logs: "Journaux",
                quit: "Quitter",
                tooltip: "Thoth — Traducteur instantané",
            },
            _ => MenuStrings {
                status_enabled: "Thoth — Enabled",
                status_disabled: "Thoth — Disabled",
                toggle_disable: "Disable",
                toggle_enable: "Enable",
                auto_start: "Start on boot",
                config: "Configuration",
                stats: "Statistics",
                reset_stats: "Reset statistics",
                logs: "Logs",
                quit: "Quit",
                tooltip: "Thoth — Instant translator",
            },
        }
    }

    struct MenuItems {
        status_item: MenuItem,
        toggle_item: MenuItem,
        auto_start_item: CheckMenuItem,
        config_item: MenuItem,
        stats_item: MenuItem,
        reset_stats_item: MenuItem,
        logs_item: MenuItem,
        quit_item: MenuItem,
    }

    fn load_icons() -> Result<(Icon, Icon)> {
        let png_bytes = include_bytes!("../resources/thoth.png");
        let decoded = image::load_from_memory_with_format(png_bytes, image::ImageFormat::Png)?;
        let rgba_img = decoded.to_rgba8();
        let (width, height) = rgba_img.dimensions();
        let raw = rgba_img.clone().into_raw();
        let color_icon = Icon::from_rgba(raw, width, height)?;

        let grayscale_img = image::DynamicImage::ImageRgba8(rgba_img).grayscale();
        let grayscale_rgba = grayscale_img.to_rgba8();
        let grayscale_icon = Icon::from_rgba(grayscale_rgba.into_raw(), width, height)?;

        Ok((color_icon, grayscale_icon))
    }

    fn build_tray(
        enabled: &AtomicBool,
        color_icon: &Icon,
        grayscale_icon: &Icon,
    ) -> Result<(TrayIcon, MenuItems, MenuStrings)> {
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

        let initial_enabled = enabled.load(Ordering::Relaxed);
        let mut builder = TrayIconBuilder::new()
            .with_tooltip(s.tooltip)
            .with_menu(Box::new(menu));

        builder = builder.with_icon(if initial_enabled {
            color_icon.clone()
        } else {
            grayscale_icon.clone()
        });

        let tray = builder.build()?;

        Ok((
            tray,
            MenuItems {
                status_item,
                toggle_item,
                auto_start_item,
                config_item,
                stats_item,
                reset_stats_item,
                logs_item,
                quit_item,
            },
            s,
        ))
    }

    fn open_log(log_path: &std::path::Path) {
        #[cfg(windows)]
        let _ = std::process::Command::new("powershell")
            .args([
                "-NoProfile",
                "-Command",
                &format!(
                    "Start-Process powershell -ArgumentList '-NoExit', '-Command', \
                     'Get-Content \"{}\" -Wait -Tail 50'",
                    log_path.to_string_lossy()
                ),
            ])
            .spawn();
        #[cfg(target_os = "macos")]
        let _ = std::process::Command::new("Console").arg(log_path).spawn();
        #[cfg(target_os = "linux")]
        let _ = std::process::Command::new("xdg-open").arg(log_path).spawn();
    }

    #[allow(clippy::too_many_arguments)]
    fn handle_menu_event(
        event_id: &tray_icon::menu::MenuId,
        items: &MenuItems,
        tray: &TrayIcon,
        enabled: &AtomicBool,
        color_icon: &Icon,
        grayscale_icon: &Icon,
        shutdown_tx: &mut Option<oneshot::Sender<()>>,
        log_path: &std::path::Path,
        s: &MenuStrings,
    ) -> bool {
        let quit_id = items.quit_item.id();
        if *event_id == quit_id {
            tracing::info!("tray: quit requested");
            if let Some(tx) = shutdown_tx.take() {
                let _ = tx.send(());
            }
            return true;
        }

        let toggle_id = items.toggle_item.id();
        if *event_id == toggle_id {
            let new_state = !enabled.load(Ordering::Relaxed);
            enabled.store(new_state, Ordering::Relaxed);
            items.toggle_item.set_text(if new_state {
                s.toggle_disable
            } else {
                s.toggle_enable
            });
            items.status_item.set_text(if new_state {
                s.status_enabled
            } else {
                s.status_disabled
            });
            let _ = tray.set_icon(Some(if new_state {
                color_icon.clone()
            } else {
                grayscale_icon.clone()
            }));
            return false;
        }

        let auto_start_id = items.auto_start_item.id();
        if *event_id == auto_start_id {
            let new_state = !auto_start::is_enabled();
            if new_state {
                let _ = auto_start::enable();
            } else {
                let _ = auto_start::disable();
            }
            items.auto_start_item.set_checked(new_state);
            return false;
        }

        let config_id = items.config_item.id();
        if *event_id == config_id {
            if let Ok(exe_path) = std::env::current_exe() {
                let _ = std::process::Command::new(exe_path).arg("--config").spawn();
            }
            return false;
        }

        let stats_id = items.stats_item.id();
        if *event_id == stats_id {
            if let Ok(exe_path) = std::env::current_exe() {
                let _ = std::process::Command::new(exe_path).arg("--stats").spawn();
            }
            return false;
        }

        let reset_id = items.reset_stats_item.id();
        if *event_id == reset_id {
            UsageMetrics::default().save();
            tracing::info!("stats reset");
            return false;
        }

        let logs_id = items.logs_item.id();
        if *event_id == logs_id {
            if log_path.exists() {
                open_log(log_path);
            } else {
                tracing::warn!("log file not found: {}", log_path.display());
            }
            return false;
        }

        false
    }

    #[cfg(windows)]
    pub fn start(
        shutdown_tx: oneshot::Sender<()>,
        enabled: Arc<AtomicBool>,
        log_path: PathBuf,
        _config_path: PathBuf,
    ) -> Result<()> {
        let (color_icon, grayscale_icon) = load_icons()?;
        let (tray, items, s) = build_tray(&enabled, &color_icon, &grayscale_icon)?;
        let mut shutdown_tx = Some(shutdown_tx);

        unsafe {
            let mut msg = std::mem::zeroed::<windows_sys::Win32::UI::WindowsAndMessaging::MSG>();
            loop {
                let result = windows_sys::Win32::UI::WindowsAndMessaging::GetMessageW(
                    &mut msg,
                    std::ptr::null_mut(),
                    0,
                    0,
                );
                if result == 0 || result == -1 {
                    break;
                }
                windows_sys::Win32::UI::WindowsAndMessaging::TranslateMessage(&msg);
                windows_sys::Win32::UI::WindowsAndMessaging::DispatchMessageW(&msg);

                while let Ok(event) = MenuEvent::receiver().try_recv() {
                    if handle_menu_event(
                        &event.id,
                        &items,
                        &tray,
                        &enabled,
                        &color_icon,
                        &grayscale_icon,
                        &mut shutdown_tx,
                        &log_path,
                        &s,
                    ) {
                        windows_sys::Win32::UI::WindowsAndMessaging::PostQuitMessage(0);
                    }
                }
            }
        }

        Ok(())
    }

    #[cfg(target_os = "macos")]
    pub fn start(
        shutdown_tx: oneshot::Sender<()>,
        enabled: Arc<AtomicBool>,
        log_path: PathBuf,
        _config_path: PathBuf,
    ) -> Result<()> {
        let (color_icon, grayscale_icon) = load_icons()?;
        let (tray, items, s) = build_tray(&enabled, &color_icon, &grayscale_icon)?;
        let mut shutdown_tx = Some(shutdown_tx);

        loop {
            if let Ok(event) =
                MenuEvent::receiver().recv_timeout(std::time::Duration::from_millis(200))
            {
                if handle_menu_event(
                    &event.id,
                    &items,
                    &tray,
                    &enabled,
                    &color_icon,
                    &grayscale_icon,
                    &mut shutdown_tx,
                    &log_path,
                    &s,
                ) {
                    break;
                }
            }
        }

        Ok(())
    }

    #[cfg(target_os = "linux")]
    pub fn start(
        shutdown_tx: oneshot::Sender<()>,
        enabled: Arc<AtomicBool>,
        log_path: PathBuf,
        _config_path: PathBuf,
    ) -> Result<()> {
        gtk::init().map_err(|e| anyhow::anyhow!("gtk init failed: {e}"))?;
        let (color_icon, grayscale_icon) = load_icons()?;
        let (tray, items, s) = build_tray(&enabled, &color_icon, &grayscale_icon)?;
        let mut shutdown_tx = Some(shutdown_tx);

        loop {
            while gtk::events_pending() {
                gtk::main_iteration();
            }

            if let Ok(event) =
                MenuEvent::receiver().recv_timeout(std::time::Duration::from_millis(200))
                && handle_menu_event(
                    &event.id,
                    &items,
                    &tray,
                    &enabled,
                    &color_icon,
                    &grayscale_icon,
                    &mut shutdown_tx,
                    &log_path,
                    &s,
                )
            {
                break;
            }
        }

        Ok(())
    }
}

#[cfg(any(windows, target_os = "macos", target_os = "linux"))]
pub use tray_impl::start;

#[cfg(not(any(windows, target_os = "macos", target_os = "linux")))]
pub fn start(
    _shutdown_tx: tokio::sync::oneshot::Sender<()>,
    _enabled: std::sync::Arc<std::sync::atomic::AtomicBool>,
    _log_path: std::path::PathBuf,
    _config_path: std::path::PathBuf,
) -> anyhow::Result<()> {
    tracing::warn!("system tray not supported on this platform");
    Ok(())
}
