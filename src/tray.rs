#[cfg(windows)]
mod platform {
    use crate::auto_start;
    use crate::metrics::UsageMetrics;
    use anyhow::Result;
    use std::sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    };
    use tokio::sync::oneshot;
    use tray_icon::{
        TrayIconBuilder,
        menu::{Menu, MenuEvent, MenuItem},
    };

    pub fn start(shutdown_tx: oneshot::Sender<()>, enabled: Arc<AtomicBool>) -> Result<()> {
        let menu = Menu::new();

        let status_item = MenuItem::new("Thoth — Actif", true, None);
        let toggle_item = MenuItem::new("Désactiver", false, None);
        let auto_start_item = MenuItem::new(
            "Démarrer avec Windows",
            false,
            Some(auto_start::is_enabled()),
        );
        let stats_item = MenuItem::new("Statistiques", false, None);
        let reset_stats_item = MenuItem::new("Réinitialiser les stats", false, None);
        let quit_item = MenuItem::new("Quitter", false, None);

        menu.append(&status_item)?;
        menu.append_separator()?;
        menu.append(&toggle_item)?;
        menu.append(&auto_start_item)?;
        menu.append_separator()?;
        menu.append(&stats_item)?;
        menu.append(&reset_stats_item)?;
        menu.append_separator()?;
        menu.append(&quit_item)?;

        let _tray = TrayIconBuilder::new()
            .with_tooltip("Thoth — Traducteur instantané")
            .with_menu(Box::new(menu))
            .build()?;

        let mut shutdown_tx = Some(shutdown_tx);
        let mut auto_start_enabled = auto_start::is_enabled();

        loop {
            if let Some(event) = MenuEvent::receiver().recv() {
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
                    auto_start_enabled = !auto_start_enabled;
                    if auto_start_enabled {
                        let _ = auto_start::enable();
                    } else {
                        let _ = auto_start::disable();
                    }
                    auto_start_item.set_checked(auto_start_enabled);
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
                }
            }
        }

        Ok(())
    }
}

#[cfg(not(windows))]
mod platform {
    use anyhow::Result;
    use std::sync::{Arc, atomic::AtomicBool};
    use tokio::sync::oneshot;

    pub fn start(_shutdown_tx: oneshot::Sender<()>, _enabled: Arc<AtomicBool>) -> Result<()> {
        tracing::warn!("system tray not supported on this platform");
        Ok(())
    }
}

pub use platform::*;
