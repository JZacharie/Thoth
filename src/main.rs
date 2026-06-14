#![windows_subsystem = "windows"]

use std::path::PathBuf;
use std::sync::{Arc, Mutex, atomic::AtomicBool};

use thoth::config::Config;
use thoth::hotkey::HotkeyPattern;
use thoth::orchestrator::Orchestrator;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let is_gui = args
        .iter()
        .any(|arg| arg == "--prompt" || arg == "--config");

    if !is_gui {
        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            let current_pid = std::process::id();
            let _ = std::process::Command::new("taskkill")
                .args([
                    "/F",
                    "/FI",
                    &format!("PID ne {}", current_pid),
                    "/IM",
                    "thoth.exe",
                ])
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .creation_flags(0x08000000) // CREATE_NO_WINDOW
                .status();
            // Give a brief moment for the OS to release the hotkey and file handles
            std::thread::sleep(std::time::Duration::from_millis(200));
        }
    }

    let config = Config::load().unwrap_or_default();

    if is_gui {
        let mode = if args.iter().any(|arg| arg == "--config") {
            thoth::gui::GuiMode::Config
        } else {
            thoth::gui::GuiMode::Prompt
        };

        let mut options = eframe::NativeOptions::default();
        let mut viewport = eframe::egui::ViewportBuilder::default()
            .with_inner_size(if mode == thoth::gui::GuiMode::Config {
                eframe::egui::vec2(450.0, 500.0)
            } else {
                eframe::egui::vec2(450.0, 300.0)
            })
            .with_resizable(true);

        if mode == thoth::gui::GuiMode::Prompt {
            viewport = viewport.with_always_on_top();
        }
        options.viewport = viewport;

        eframe::run_native(
            "Thoth",
            options,
            Box::new(move |_cc| Ok(Box::new(thoth::gui::ThothGuiApp::new(mode, config)))),
        )
        .map_err(|e| anyhow::anyhow!("Failed to run eframe: {:?}", e))?;
        return Ok(());
    }

    let log_file = if let Some(ref path_str) = config.behavior.log_path {
        PathBuf::from(path_str)
    } else {
        let exe_dir = std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(PathBuf::from))
            .unwrap_or_else(|| PathBuf::from("."));
        exe_dir.join("thoth.log")
    };

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

    let (hotkey_tx, hotkey_rx) = tokio::sync::mpsc::channel::<thoth::hotkey::HotkeyAction>(16);
    thoth::hotkey::start(hotkey_tx, hotkey_pattern, enabled)?;
    tracing::info!("hotkey listener started ({})", config.behavior.hotkey);

    let mut orchestrator = Orchestrator::new(hotkey_rx, config.clone())?;

    tracing::info!(
        "Testing connection to Ollama/Pylos endpoint at {}...",
        orchestrator.endpoint()
    );
    match orchestrator.test_connection().await {
        Ok(_) => {
            tracing::info!("Connection to Ollama/Pylos endpoint is OK");

            // Effectue un test de traduction au démarrage
            let test_model = config.pylos.model.clone();
            tracing::info!("Testing translation with model '{}'...", test_model);
            match orchestrator.test_translate("Hello world").await {
                Ok(translated) => {
                    tracing::info!(
                        "Translation test successful: 'Hello world' -> '{}'",
                        translated.trim()
                    );
                }
                Err(e) => {
                    tracing::error!("Translation test failed with model '{}': {}", test_model, e);
                }
            }
        }
        Err(e) => {
            tracing::error!("Connection test failed: {e}");
            thoth::notification::notify_error(&format!(
                "Impossible de se connecter à Ollama/Pylos ({}) : {}",
                orchestrator.endpoint(),
                e
            ));
        }
    }

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
