#![windows_subsystem = "windows"]

use std::path::PathBuf;
use std::sync::{Arc, Mutex, atomic::AtomicBool};

use thoth::config::Config;
use thoth::hotkey::HotkeyPattern;
use thoth::orchestrator::Orchestrator;
use tracing_subscriber::EnvFilter;

#[cfg(windows)]
fn show_crash_dialog(message: &str, log_path: &std::path::Path) {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;

    let title: Vec<u16> = OsStr::new("Thoth — Erreur critique\0")
        .encode_wide()
        .collect();
    let body_str = format!(
        "{}\n\nUn fichier de log a été enregistré à : {}\n\nVoulez-vous ouvrir le fichier de log ?",
        message,
        log_path.display()
    );
    let body: Vec<u16> = OsStr::new(&format!("{}\0", body_str))
        .encode_wide()
        .collect();

    unsafe {
        let result = windows_sys::Win32::UI::WindowsAndMessaging::MessageBoxW(
            std::ptr::null_mut(),
            body.as_ptr(),
            title.as_ptr(),
            windows_sys::Win32::UI::WindowsAndMessaging::MB_YESNO
                | windows_sys::Win32::UI::WindowsAndMessaging::MB_ICONERROR,
        );
        if result == windows_sys::Win32::UI::WindowsAndMessaging::IDYES {
            let _ = std::process::Command::new("notepad.exe")
                .arg(log_path)
                .spawn();
        }
    }
}

#[tokio::main]
async fn main() {
    let result = main_inner().await;
    if let Err(ref e) = result {
        tracing::error!("Fatal error: {:?}", e);
        let config = Config::load().unwrap_or_default();
        let log_file = if let Some(ref path_str) = config.behavior.log_path {
            PathBuf::from(path_str)
        } else {
            let exe_dir = std::env::current_exe()
                .ok()
                .and_then(|p| p.parent().map(PathBuf::from))
                .unwrap_or_else(|| PathBuf::from("."));
            exe_dir.join("thoth.log")
        };
        #[cfg(windows)]
        show_crash_dialog(&format!("Erreur fatale : {}", e), &log_file);
        std::process::exit(1);
    }
}

async fn main_inner() -> anyhow::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let is_insecure = args.iter().any(|arg| arg == "--insecure");
    thoth::set_insecure(is_insecure);
    let is_gui = args
        .iter()
        .any(|arg| arg == "--prompt" || arg == "--config");

    #[cfg(windows)]
    {
        if !cfg!(debug_assertions) {
            #[allow(clippy::collapsible_if)]
            if let Err(e) = verify_self_signature() {
                tracing::error!(
                    "Executable signature verification failed: {e}. Terminating for security."
                );
                thoth::notification::notify_error(&format!(
                    "Erreur de sécurité : signature invalide. ({})",
                    e
                ));
                std::process::exit(1);
            }
        }
    }

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

    let log_file_for_panic = log_file.clone();
    std::panic::set_hook(Box::new(move |info| {
        let payload = if let Some(s) = info.payload().downcast_ref::<&str>() {
            *s
        } else if let Some(s) = info.payload().downcast_ref::<String>() {
            s.as_str()
        } else {
            "Unknown panic payload"
        };
        let location = info
            .location()
            .map(|l| format!("{}:{}:{}", l.file(), l.line(), l.column()))
            .unwrap_or_else(|| "unknown location".to_string());
        let msg = format!("Panic occurred at {}: {}", location, payload);
        tracing::error!("{}", msg);

        #[cfg(windows)]
        show_crash_dialog(&msg, &log_file_for_panic);
    }));

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

#[cfg(windows)]
#[allow(
    non_snake_case,
    clippy::upper_case_acronyms,
    clippy::manual_c_str_literals
)]
fn verify_self_signature() -> anyhow::Result<()> {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;
    use std::ptr;

    #[repr(C)]
    struct GUID {
        data1: u32,
        data2: u16,
        data3: u16,
        data4: [u8; 8],
    }

    #[repr(C)]
    struct WINTRUST_FILE_INFO {
        cbStruct: u32,
        pcwszFilePath: *const u16,
        hFile: *mut std::ffi::c_void,
        pgKnownSubject: *const GUID,
    }

    #[repr(C)]
    struct WINTRUST_DATA {
        cbStruct: u32,
        pPolicyCallbackData: *mut std::ffi::c_void,
        pSIPClientData: *mut std::ffi::c_void,
        dwUIChoice: u32,
        fdwRevocationChecks: u32,
        unionChoice: u32,
        file_info: *const WINTRUST_FILE_INFO,
        dwStateAction: u32,
        hWVTStateData: *mut std::ffi::c_void,
        pwszURLReference: *mut u16,
        dwProvFlags: u32,
        dwUIContext: u32,
        pSignatureSettings: *mut std::ffi::c_void,
    }

    // WINTRUST_ACTION_GENERIC_VERIFY_V2
    let action_id = GUID {
        data1: 0x00aac56b,
        data2: 0xcd44,
        data3: 0x11d0,
        data4: [0x8c, 0xeb, 0x00, 0xc0, 0x4f, 0xc2, 0x95, 0xee],
    };

    let exe_path = std::env::current_exe()?;
    let mut path_wide: Vec<u16> = OsStr::new(&exe_path).encode_wide().collect();
    path_wide.push(0);

    let file_info = WINTRUST_FILE_INFO {
        cbStruct: std::mem::size_of::<WINTRUST_FILE_INFO>() as u32,
        pcwszFilePath: path_wide.as_ptr(),
        hFile: ptr::null_mut(),
        pgKnownSubject: ptr::null(),
    };

    let wintrust_data = WINTRUST_DATA {
        cbStruct: std::mem::size_of::<WINTRUST_DATA>() as u32,
        pPolicyCallbackData: ptr::null_mut(),
        pSIPClientData: ptr::null_mut(),
        dwUIChoice: 2,          // WTD_UI_NONE = 2
        fdwRevocationChecks: 0, // WTD_REVOKE_NONE
        unionChoice: 1,         // WTD_CHOICE_FILE = 1
        file_info: &file_info,
        dwStateAction: 0,
        hWVTStateData: ptr::null_mut(),
        pwszURLReference: ptr::null_mut(),
        dwProvFlags: 0x00000040, // WTD_REVOCATION_CHECK_NONE = 0x00000040
        dwUIContext: 0,
        pSignatureSettings: ptr::null_mut(),
    };

    unsafe {
        let wintrust =
            windows_sys::Win32::System::LibraryLoader::LoadLibraryA(b"wintrust.dll\0".as_ptr());
        if wintrust.is_null() {
            return Err(anyhow::anyhow!("Failed to load wintrust.dll"));
        }
        let win_verify_trust_addr = windows_sys::Win32::System::LibraryLoader::GetProcAddress(
            wintrust,
            b"WinVerifyTrust\0".as_ptr(),
        );
        if win_verify_trust_addr.is_none() {
            return Err(anyhow::anyhow!(
                "Failed to find WinVerifyTrust in wintrust.dll"
            ));
        }
        let win_verify_trust: unsafe extern "system" fn(
            hwnd: *mut std::ffi::c_void,
            pgActionID: *const GUID,
            pWintrustData: *const WINTRUST_DATA,
        ) -> i32 = std::mem::transmute(win_verify_trust_addr);

        let result = win_verify_trust(ptr::null_mut(), &action_id, &wintrust_data);
        if result != 0 {
            return Err(anyhow::anyhow!(
                "Signature verification failed: WinVerifyTrust returned {:x}",
                result
            ));
        }
    }
    Ok(())
}
