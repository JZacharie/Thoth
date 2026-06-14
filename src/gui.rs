use eframe::egui;

use crate::clipboard::ClipboardManager;
use crate::config::Config;
use crate::pylos_client::PylosClient;

use crate::metrics::UsageMetrics;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GuiMode {
    Prompt,
    Config,
    Stats,
}

pub struct ThothGuiApp {
    mode: GuiMode,
    config: Config,

    // Prompt Mode states
    prompt_input: String,
    history: Vec<String>,
    selected_history_index: Option<usize>,
    status_msg: String,
    is_loading: bool,
    original_text: String,
    active_window: *mut std::ffi::c_void,

    // Config Mode states
    endpoint: String,
    model: String,
    fallback_model: String,
    timeout_secs: u64,
    secret: String,
    target_language: String,
    restore_clipboard: bool,
    show_notifications: bool,
    debounce_ms: u64,
    hotkey: String,
    #[allow(dead_code)]
    log_path: String,
}

impl ThothGuiApp {
    pub fn new(mode: GuiMode, config: Config) -> Self {
        let history = load_history();

        let original_text = if mode == GuiMode::Prompt {
            ClipboardManager::new()
                .and_then(|mut cm| cm.copy_selected_text())
                .unwrap_or_default()
        } else {
            String::new()
        };

        #[cfg(windows)]
        let active_window =
            unsafe { windows_sys::Win32::UI::WindowsAndMessaging::GetForegroundWindow() };
        #[cfg(not(windows))]
        let active_window = std::ptr::null_mut();

        Self {
            mode,
            endpoint: config.pylos.endpoint.clone(),
            model: config.pylos.model.clone(),
            fallback_model: config.pylos.fallback_model.clone().unwrap_or_default(),
            timeout_secs: config.pylos.timeout_secs,
            secret: config.pylos.secret.clone(),
            target_language: config.behavior.target_language.clone(),
            restore_clipboard: config.behavior.restore_clipboard,
            show_notifications: config.behavior.show_notifications,
            debounce_ms: config.behavior.debounce_ms,
            hotkey: config.behavior.hotkey.clone(),
            log_path: config.behavior.log_path.clone().unwrap_or_default(),
            config,
            prompt_input: String::new(),
            history,
            selected_history_index: None,
            status_msg: String::new(),
            is_loading: false,
            original_text,
            active_window,
        }
    }

    fn run_instruction(&mut self, _ctx: &egui::Context, instruction: String) {
        if self.original_text.is_empty() {
            self.status_msg = "Aucun texte sélectionné à analyser".to_string();
            return;
        }

        self.is_loading = true;
        self.status_msg = "Analyse et exécution en cours...".to_string();

        // Add to history
        if !instruction.trim().is_empty() {
            self.history.retain(|h| h != &instruction);
            self.history.insert(0, instruction.clone());
            if self.history.len() > 20 {
                self.history.truncate(20);
            }
            save_history(&self.history);
        }

        let pylos = PylosClient::new(
            self.config.pylos.clone(),
            self.config.behavior.target_language.clone(),
        );
        let original = self.original_text.clone();
        let active_window_addr = self.active_window as usize;
        let restore_clipboard = self.config.behavior.restore_clipboard;

        tokio::spawn(async move {
            // We build the custom instruction prompt combining the user instruction and the selected text
            let prompt = format!(
                "Instruction : {}\n\nTexte à traiter :\n{}",
                instruction, original
            );

            match pylos.execute_instruction(&prompt).await {
                Ok(result) => {
                    if let Ok(mut cm) = ClipboardManager::new() {
                        #[cfg(windows)]
                        unsafe {
                            let active_window = active_window_addr as *mut std::ffi::c_void;
                            if !active_window.is_null() {
                                windows_sys::Win32::UI::WindowsAndMessaging::SetForegroundWindow(
                                    active_window,
                                );
                                std::thread::sleep(std::time::Duration::from_millis(150));
                            }
                        }
                        let _ = cm.paste_text(&result, restore_clipboard);
                    }
                    std::process::exit(0);
                }
                Err(e) => {
                    tracing::error!("Failed to execute instruction: {e}");
                    // We can't update UI directly from thread easily without a channel or shared state,
                    // but since this exits or fails, we can just print it
                    std::process::exit(1);
                }
            }
        });
    }
}

impl eframe::App for ThothGuiApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Apply styling for a modern, sleek premium dark look
        let mut visuals = egui::Visuals::dark();
        visuals.window_rounding = 8.0.into();
        visuals.widgets.active.rounding = 4.0.into();
        visuals.widgets.hovered.rounding = 4.0.into();
        visuals.widgets.inactive.rounding = 4.0.into();
        ctx.set_visuals(visuals);

        egui::TopBottomPanel::top("top_bar").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("Menu", |ui| {
                    if ui.button("Consigne (Prompt)").clicked() {
                        self.mode = GuiMode::Prompt;
                        self.status_msg.clear();
                        ui.close_menu();
                    }
                    if ui.button("Configuration").clicked() {
                        self.mode = GuiMode::Config;
                        self.status_msg.clear();
                        ui.close_menu();
                    }
                    if ui.button("Statistiques").clicked() {
                        self.mode = GuiMode::Stats;
                        self.status_msg.clear();
                        ui.close_menu();
                    }
                    ui.separator();
                    if ui.button("Quitter").clicked() {
                        std::process::exit(0);
                    }
                });
            });
        });

        match self.mode {
            GuiMode::Prompt => {
                egui::CentralPanel::default().show(ctx, |ui| {
                    ui.vertical_centered(|ui| {
                        ui.heading("Thoth — Assistant IA");
                    });
                    ui.add_space(8.0);

                    // Check for key navigation in history
                    let history_len = self.history.len();
                    if ui.input(|i| i.key_pressed(egui::Key::ArrowDown)) {
                        self.selected_history_index = match self.selected_history_index {
                            None => {
                                if history_len > 0 {
                                    Some(0)
                                } else {
                                    None
                                }
                            }
                            Some(idx) => Some((idx + 1).min(history_len - 1)),
                        };
                        if let Some(idx) = self.selected_history_index {
                            self.prompt_input = self.history[idx].clone();
                        }
                    }
                    if ui.input(|i| i.key_pressed(egui::Key::ArrowUp)) {
                        self.selected_history_index = match self.selected_history_index {
                            None => {
                                if history_len > 0 {
                                    Some(history_len - 1)
                                } else {
                                    None
                                }
                            }
                            Some(0) => None,
                            Some(idx) => Some(idx - 1),
                        };
                        if let Some(idx) = self.selected_history_index {
                            self.prompt_input = self.history[idx].clone();
                        } else {
                            self.prompt_input.clear();
                        }
                    }

                    // Input Text Box
                    let response = ui.add(
                        egui::TextEdit::singleline(&mut self.prompt_input)
                            .hint_text(
                                "Saisissez votre consigne (ex: résumer, expliquer, répondre...)",
                            )
                            .desired_width(ui.available_width()),
                    );

                    // Auto-focus the input box on start
                    response.request_focus();

                    // If user presses Enter
                    if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                        let val = self.prompt_input.clone();
                        if !val.trim().is_empty() {
                            self.run_instruction(ctx, val);
                        }
                    }

                    ui.add_space(8.0);

                    // History quick selection
                    let mut clicked_item = None;
                    if !self.history.is_empty() {
                        ui.label("Historique des consignes :");
                        egui::ScrollArea::vertical()
                            .max_height(120.0)
                            .show(ui, |ui| {
                                for (idx, hist_item) in self.history.iter().enumerate() {
                                    let is_selected = self.selected_history_index == Some(idx);
                                    if ui.selectable_label(is_selected, hist_item).clicked() {
                                        clicked_item = Some(hist_item.clone());
                                    }
                                }
                            });
                    }
                    if let Some(hist_item) = clicked_item {
                        self.run_instruction(ctx, hist_item);
                    }

                    if !self.status_msg.is_empty() {
                        ui.add_space(8.0);
                        ui.horizontal(|ui| {
                            if self.is_loading {
                                ui.spinner();
                            }
                            ui.label(&self.status_msg);
                        });
                    }

                    ui.add_space(8.0);
                    ui.horizontal(|ui| {
                        if ui.button("Lancer").clicked() {
                            let val = self.prompt_input.clone();
                            if !val.trim().is_empty() {
                                self.run_instruction(ctx, val);
                            }
                        }
                        if ui.button("Annuler").clicked() {
                            std::process::exit(0);
                        }
                    });
                });
            }
            GuiMode::Config => {
                egui::CentralPanel::default().show(ctx, |ui| {
                    ui.vertical_centered(|ui| {
                        ui.heading("Configuration de Thoth");
                    });
                    ui.add_space(10.0);

                    egui::ScrollArea::vertical().show(ui, |ui| {
                        ui.group(|ui| {
                            ui.label("Paramètres Ollama / Pylos");
                            ui.horizontal(|ui| {
                                ui.label("Endpoint :");
                                ui.text_edit_singleline(&mut self.endpoint);
                            });
                            ui.horizontal(|ui| {
                                ui.label("Modèle :");
                                ui.text_edit_singleline(&mut self.model);
                            });
                            ui.horizontal(|ui| {
                                ui.label("Modèle de secours :");
                                ui.text_edit_singleline(&mut self.fallback_model);
                            });
                            ui.horizontal(|ui| {
                                ui.label("Timeout (sec) :");
                                ui.add(egui::DragValue::new(&mut self.timeout_secs).range(1..=300));
                            });
                            ui.horizontal(|ui| {
                                ui.label("Clé Secrète :");
                                ui.text_edit_singleline(&mut self.secret);
                            });
                        });

                        ui.add_space(8.0);

                        ui.group(|ui| {
                            ui.label("Paramètres comportementaux");
                            ui.horizontal(|ui| {
                                ui.label("Langue cible :");
                                ui.text_edit_singleline(&mut self.target_language);
                            });
                            ui.checkbox(&mut self.restore_clipboard, "Restaurer le presse-papiers après collage");
                            ui.checkbox(&mut self.show_notifications, "Afficher les notifications système");
                            ui.horizontal(|ui| {
                                ui.label("Anti-rebond (ms) :");
                                ui.add(egui::DragValue::new(&mut self.debounce_ms).range(50..=2000));
                            });
                            ui.horizontal(|ui| {
                                ui.label("Raccourci clavier :");
                                ui.text_edit_singleline(&mut self.hotkey);
                            });
                        });
                    });

                    ui.add_space(10.0);
                    ui.horizontal(|ui| {
                        if ui.button("Enregistrer").clicked() {
                            self.config.pylos.endpoint = self.endpoint.clone();
                            self.config.pylos.model = self.model.clone();
                            self.config.pylos.fallback_model = if self.fallback_model.trim().is_empty() {
                                None
                            } else {
                                Some(self.fallback_model.clone())
                            };
                            self.config.pylos.timeout_secs = self.timeout_secs;
                            self.config.pylos.secret = self.secret.clone();

                            self.config.behavior.target_language = self.target_language.clone();
                            self.config.behavior.restore_clipboard = self.restore_clipboard;
                            self.config.behavior.show_notifications = self.show_notifications;
                            self.config.behavior.debounce_ms = self.debounce_ms;
                            self.config.behavior.hotkey = self.hotkey.clone();

                            if let Err(e) = self.config.save() {
                                self.status_msg = format!("Erreur d'enregistrement : {e}");
                            } else {
                                self.status_msg = "Configuration enregistrée avec succès. Veuillez redémarrer Thoth pour appliquer.".to_string();
                                std::process::exit(0);
                            }
                        }
                        if ui.button("Fermer").clicked() {
                            std::process::exit(0);
                        }
                    });

                    if !self.status_msg.is_empty() {
                        ui.add_space(8.0);
                        ui.label(&self.status_msg);
                    }
                });
            }
            GuiMode::Stats => {
                egui::CentralPanel::default().show(ctx, |ui| {
                    ui.vertical_centered(|ui| {
                        ui.heading("Statistiques d'utilisation");
                    });
                    ui.add_space(10.0);

                    let metrics = UsageMetrics::load();
                    egui::Grid::new("stats_grid")
                        .num_columns(2)
                        .spacing([40.0, 8.0])
                        .striped(true)
                        .show(ui, |ui| {
                            ui.label("Traductions réussies :");
                            ui.label(metrics.total_translations.to_string());
                            ui.end_row();

                            ui.label("Erreurs rencontrées :");
                            ui.label(metrics.total_errors.to_string());
                            ui.end_row();

                            ui.label("Volume de texte traité :");
                            ui.label(format!("{} octets", metrics.total_bytes_processed));
                            ui.end_row();

                            ui.label("Latence moyenne :");
                            ui.label(format!("{:.0} ms", metrics.avg_latency_ms()));
                            ui.end_row();
                        });

                    if !metrics.model_usage.is_empty() {
                        ui.add_space(15.0);
                        ui.label("Utilisation par modèle :");
                        ui.group(|ui| {
                            for (model, count) in &metrics.model_usage {
                                ui.label(format!("{}: {} fois", model, count));
                            }
                        });
                    }

                    ui.add_space(20.0);
                    ui.horizontal(|ui| {
                        if ui.button("Réinitialiser les stats").clicked() {
                            UsageMetrics::default().save();
                        }
                        if ui.button("Fermer").clicked() {
                            self.mode = GuiMode::Prompt;
                        }
                    });
                });
            }
        }
    }
}

fn load_history() -> Vec<String> {
    #[cfg(windows)]
    {
        use winreg::RegKey;
        use winreg::enums::HKEY_CURRENT_USER;
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        if let Ok(val) = hkcu
            .open_subkey("Software\\Thoth")
            .and_then(|key| key.get_value::<String, _>("History"))
        {
            return val
                .split('\n')
                .map(|s| s.to_string())
                .filter(|s| !s.is_empty())
                .collect();
        }
    }
    vec![
        "résumer".to_string(),
        "expliquer".to_string(),
        "répondre au mail".to_string(),
        "corriger l'orthographe".to_string(),
    ]
}

fn save_history(history: &[String]) {
    #[cfg(windows)]
    {
        use winreg::RegKey;
        use winreg::enums::HKEY_CURRENT_USER;
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        if let Ok((key, _)) = hkcu.create_subkey("Software\\Thoth") {
            let val = history.join("\n");
            let _ = key.set_value("History", &val);
        }
    }
}
