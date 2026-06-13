#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::path::PathBuf;
use std::sync::{Arc, Mutex, atomic::AtomicBool};

use thoth::config::Config;
use thoth::hotkey::HotkeyPattern;
use thoth::orchestrator::Orchestrator;
use tracing_subscriber::EnvFilter;

fn log_path() -> PathBuf {
    let base = std::env::var("APPDATA")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("."));
    base.join("thoth").join("thoth.log")
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let log_file = log_path();
    if let Some(parent) = log_file.parent() {
        std::fs::create_dir_all(parent).ok();
    }

    let file_appender =
        tracing_appender::rolling::never(log_file.parent().unwrap(), log_file.file_name().unwrap());
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .with_writer(non_blocking)
        .init();

    let config = Config::load()?;
    tracing::info!("Thoth v{} starting", env!("CARGO_PKG_VERSION"));

    let mut config = config;
    if config.pylos.secret.is_empty() {
        config.pylos.secret = uuid_v4();
        config.save()?;
        tracing::info!("generated new X-Thoth-Secret");
    }

    let enabled = Arc::new(AtomicBool::new(true));
    let hotkey_pattern = Arc::new(Mutex::new(
        HotkeyPattern::parse(&config.behavior.hotkey)
            .unwrap_or_else(|_| HotkeyPattern::default_win_n()),
    ));

    let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel::<()>();

    let config_path = Config::path();
    let tray_enabled = enabled.clone();
    let _tray = std::thread::spawn(move || {
        if let Err(e) = thoth::tray::start(shutdown_tx, tray_enabled, log_file, config_path) {
            tracing::error!("tray error: {e}");
        }
    });

    let (hotkey_tx, hotkey_rx) = tokio::sync::mpsc::channel::<()>(16);
    thoth::hotkey::start(hotkey_tx, hotkey_pattern, enabled)?;
    tracing::info!("hotkey listener started ({})", config.behavior.hotkey);

    let mut orchestrator = Orchestrator::new(hotkey_rx, config)?;

    tokio::select! {
        _ = orchestrator.run() => {}
        _ = async { shutdown_rx.await.ok() } => {
            tracing::info!("shutdown signal received");
        }
    }

    tracing::info!("Thoth shutting down");
    Ok(())
}

fn uuid_v4() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    let seed = now.as_nanos() as u64;
    format!("thoth-{seed:016x}")
}
