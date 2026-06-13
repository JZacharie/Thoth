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

    pub fn start(
        shutdown_tx: oneshot::Sender<()>,
        enabled: Arc<AtomicBool>,
        log_path: PathBuf,
    ) -> Result<()> {
        let menu = Menu::new();

        let status_item = MenuItem::new("Thoth — Actif", true, None);
        let toggle_item = MenuItem::new("Désactiver", true, None);
        let auto_start_item = CheckMenuItem::new(
            "Démarrer avec Windows",
            true,
            auto_start::is_enabled(),
            None,
        );
        let stats_item = MenuItem::new("Statistiques", true, None);
        let reset_stats_item = MenuItem::new("Réinitialiser les stats", true, None);
        let logs_item = MenuItem::new("Journaux", true, None);
        let quit_item = MenuItem::new("Quitter", true, None);

        menu.append(&status_item)?;
        menu.append(&PredefinedMenuItem::separator())?;
        menu.append(&toggle_item)?;
        menu.append(&auto_start_item)?;
        menu.append(&PredefinedMenuItem::separator())?;
        menu.append(&stats_item)?;
        menu.append(&reset_stats_item)?;
        menu.append(&PredefinedMenuItem::separator())?;
        menu.append(&logs_item)?;
        menu.append(&PredefinedMenuItem::separator())?;
        menu.append(&quit_item)?;

        let icon = Icon::from_resource(1, Some((32, 32))).ok();

        let mut builder = TrayIconBuilder::new()
            .with_tooltip("Thoth — Traducteur instantané")
            .with_menu(Box::new(menu));
        if let Some(ico) = icon {
            builder = builder.with_icon(ico);
        }
        let _tray = builder.build()?;

        let mut shutdown_tx = Some(shutdown_tx);

        loop {
            match MenuEvent::receiver().recv() {
                Ok(event) => {
                    if event.id == quit_item.id() {
                        tracing::info!("tray: quit requested");
                        if let Some(tx) = shutdown_tx.take() {
                            let _ = tx.send(());
                        }
                        break;
                    } else if event.id == toggle_item.id() {
                        let new_state = !enabled.load(Ordering::Relaxed);
                        enabled.store(new_state, Ordering::Relaxed);
                        let label = if new_state { "Désactiver" } else { "Activer" };
                        toggle_item.set_text(label);
                        status_item.set_text(if new_state {
                            "Thoth — Actif"
                        } else {
                            "Thoth — Désactivé"
                        });
                    } else if event.id == auto_start_item.id() {
                        let new_state = !auto_start::is_enabled();
                        if new_state {
                            let _ = auto_start::enable();
                        } else {
                            let _ = auto_start::disable();
                        }
                        auto_start_item.set_checked(new_state);
                    } else if event.id == stats_item.id() {
                        let metrics = UsageMetrics::load();
                        let msg = format!(
                            "Traductions : {}\nErreurs : {}\nLatence moy. : {:.0} ms\nModèles : {}",
                            metrics.total_translations,
                            metrics.total_errors,
                            metrics.avg_latency_ms(),
                            metrics
                                .model_usage
                                .iter()
                                .map(|(m, c)| format!("{m}: {c}"))
                                .collect::<Vec<_>>()
                                .join(", "),
                        );
                        tracing::info!("Stats:\n{}", msg);
                    } else if event.id == reset_stats_item.id() {
                        UsageMetrics::default().save();
                        tracing::info!("stats reset");
                    } else if event.id == logs_item.id() {
                        if log_path.exists() {
                            let _ = std::process::Command::new("cmd")
                                .args(["/c", "start", "", &log_path.to_string_lossy()])
                                .spawn();
                        } else {
                            tracing::warn!("log file not found: {}", log_path.display());
                        }
                    }
                }
                Err(e) => {
                    tracing::error!("menu event error: {e:?}");
                    break;
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
    ) -> Result<()> {
        tracing::warn!("system tray not supported on this platform");
        Ok(())
    }
}

pub use platform::*;
