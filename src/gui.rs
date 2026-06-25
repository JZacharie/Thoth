use eframe::egui;
use eframe::egui::{Color32, CornerRadius, FontId, Stroke, Vec2};

use crate::clipboard::ClipboardManager;
use crate::config::Config;
use crate::metrics::UsageMetrics;
use crate::pylos_client::PylosClient;

// ── Palette ────────────────────────────────────────────────────────────────────
// Based on the README screenshots: deep navy background, slightly lighter cards,
// accent blue button, coloured stat values.
const BG: Color32 = Color32::from_rgb(13, 17, 23); // #0d1117 window body
const BG_CARD: Color32 = Color32::from_rgb(22, 27, 34); // #161b22 cards / input
const BG_FIELD: Color32 = Color32::from_rgb(28, 35, 51); // #1c2333 text fields
const BG_HOVER: Color32 = Color32::from_rgb(33, 41, 58); // history row hover

const ACCENT_BLUE: Color32 = Color32::from_rgb(31, 149, 255); // Execute button
const ACCENT_BLUE_HOV: Color32 = Color32::from_rgb(66, 169, 255);
const ACCENT_GREEN: Color32 = Color32::from_rgb(63, 185, 80); // Save button
const ACCENT_GREEN_HOV: Color32 = Color32::from_rgb(88, 205, 105);

const TEXT_WHITE: Color32 = Color32::from_rgb(230, 237, 243);
const TEXT_MUTED: Color32 = Color32::from_rgb(120, 137, 162); // labels, placeholders
const TEXT_HISTORY: Color32 = Color32::from_rgb(160, 174, 196); // italic history items

const BORDER: Color32 = Color32::from_rgb(48, 58, 80);
const DIVIDER: Color32 = Color32::from_rgb(38, 46, 64); // thin separators

// Stat colours
const C_TEAL: Color32 = Color32::from_rgb(45, 212, 191); // Translations
const C_RED: Color32 = Color32::from_rgb(240, 82, 82); // Errors
const C_YELLOW: Color32 = Color32::from_rgb(250, 176, 5); // Latency
const C_PURPLE: Color32 = Color32::from_rgb(168, 114, 242); // Volume

// Bar colours for model usage
const BAR_BLUE: Color32 = Color32::from_rgb(88, 130, 213);
const BAR_GREEN: Color32 = Color32::from_rgb(63, 185, 80);

const PAD: i8 = 20;
const ROUNDING: u8 = 8;

// ── Mode ──────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GuiMode {
    Prompt,
    Config,
    Stats,
}

// ── App state ─────────────────────────────────────────────────────────────────

pub struct ThothGuiApp {
    mode: GuiMode,
    config: Config,

    // Prompt
    prompt_input: String,
    history: Vec<String>,
    selected_history_index: Option<usize>,
    status_msg: String,
    status_ok: bool,
    is_loading: bool,
    original_text: String,
    #[allow(dead_code)]
    active_window: *mut std::ffi::c_void,

    // Config
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

    // MQTT config
    mqtt_broker: String,
    mqtt_username: String,
    mqtt_password: String,
    mqtt_topic: String,
    mqtt_port: u16,
    mqtt_use_tls: bool,

    // S3 config
    s3_endpoint: String,
    s3_bucket: String,
    #[allow(dead_code)]
    s3_access_key: String,
    s3_secret_key: String,

    // Vision config
    vision_model: String,
    vision_hotkey: String,
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
            mqtt_broker: config.mqtt.broker.clone(),
            mqtt_username: config.mqtt.username.clone(),
            mqtt_password: config.mqtt.password.clone(),
            mqtt_topic: config.mqtt.topic.clone(),
            mqtt_port: config.mqtt.port,
            mqtt_use_tls: config.mqtt.use_tls,
            s3_endpoint: config.s3.endpoint.clone(),
            s3_bucket: config.s3.bucket.clone(),
            s3_access_key: config.s3.access_key.clone(),
            s3_secret_key: config.s3.secret_key.clone(),
            vision_model: config.vision.model.clone(),
            vision_hotkey: config.vision.hotkey.clone(),
            config,
            prompt_input: String::new(),
            history,
            selected_history_index: None,
            status_msg: String::new(),
            status_ok: true,
            is_loading: false,
            original_text,
            active_window,
        }
    }

    fn run_instruction(&mut self, _ctx: &egui::Context, instruction: String) {
        if self.original_text.is_empty() {
            self.status_msg = "Aucun texte sélectionné.".to_string();
            self.status_ok = false;
            return;
        }
        self.is_loading = true;
        self.status_msg = "Analyse en cours…".to_string();
        self.status_ok = true;

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
        #[cfg(windows)]
        let active_window_addr = self.active_window as usize;
        let restore_clipboard = self.config.behavior.restore_clipboard;

        tokio::spawn(async move {
            let prompt = format!(
                "Instruction : {}\n\nTexte à traiter :\n{}",
                instruction, original
            );
            match pylos.execute_instruction(&prompt).await {
                Ok(result) => {
                    if let Ok(mut cm) = ClipboardManager::new() {
                        #[cfg(windows)]
                        unsafe {
                            let aw = active_window_addr as *mut std::ffi::c_void;
                            if !aw.is_null() {
                                windows_sys::Win32::UI::WindowsAndMessaging::SetForegroundWindow(
                                    aw,
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
                    std::process::exit(1);
                }
            }
        });
    }
}

// ── Theme ─────────────────────────────────────────────────────────────────────

fn apply_theme(ctx: &egui::Context) {
    let mut vis = egui::Visuals::dark();
    vis.panel_fill = BG;
    vis.window_fill = BG;
    vis.extreme_bg_color = BG_FIELD;
    vis.faint_bg_color = BG_CARD;
    vis.code_bg_color = BG_FIELD;
    vis.window_corner_radius = CornerRadius::same(10);
    vis.window_stroke = Stroke::new(1.0_f32, BORDER);

    let r = CornerRadius::same(ROUNDING);
    for w in [
        &mut vis.widgets.noninteractive,
        &mut vis.widgets.inactive,
        &mut vis.widgets.hovered,
        &mut vis.widgets.active,
        &mut vis.widgets.open,
    ] {
        w.corner_radius = r;
    }
    vis.widgets.noninteractive.fg_stroke = Stroke::new(1.0_f32, TEXT_WHITE);
    vis.widgets.inactive.fg_stroke = Stroke::new(1.0_f32, TEXT_WHITE);
    vis.widgets.hovered.fg_stroke = Stroke::new(1.0_f32, TEXT_WHITE);
    vis.widgets.active.fg_stroke = Stroke::new(1.0_f32, TEXT_WHITE);

    vis.widgets.inactive.weak_bg_fill = BG_FIELD;
    vis.widgets.hovered.weak_bg_fill = BG_HOVER;
    vis.widgets.active.weak_bg_fill = BG_HOVER;
    vis.widgets.inactive.bg_fill = BG_FIELD;

    vis.widgets.inactive.bg_stroke = Stroke::new(1.0_f32, BORDER);
    vis.widgets.hovered.bg_stroke = Stroke::new(1.5_f32, ACCENT_BLUE);
    vis.widgets.active.bg_stroke = Stroke::new(1.5_f32, ACCENT_BLUE);

    vis.selection.bg_fill = Color32::from_rgba_premultiplied(31, 149, 255, 55);
    vis.selection.stroke = Stroke::new(1.0_f32, ACCENT_BLUE);

    ctx.set_visuals(vis);

    let mut style = (*ctx.global_style()).clone();
    style
        .text_styles
        .insert(egui::TextStyle::Body, FontId::proportional(14.0));
    style
        .text_styles
        .insert(egui::TextStyle::Button, FontId::proportional(14.0));
    style
        .text_styles
        .insert(egui::TextStyle::Heading, FontId::proportional(16.0));
    style
        .text_styles
        .insert(egui::TextStyle::Small, FontId::proportional(12.5));
    style.spacing.item_spacing = Vec2::new(8.0, 8.0);
    style.spacing.button_padding = Vec2::new(14.0, 8.0);
    style.spacing.window_margin = egui::Margin::same(0i8);
    ctx.set_global_style(style);
}

// ── App ───────────────────────────────────────────────────────────────────────

impl eframe::App for ThothGuiApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        apply_theme(ui.ctx());
        match self.mode {
            GuiMode::Prompt => self.draw_prompt(ui),
            GuiMode::Config => self.draw_config(ui),
            GuiMode::Stats => self.draw_stats(ui),
        }
    }
}

// ── Prompt panel ─────────────────────────────────────────────────────────────

impl ThothGuiApp {
    fn draw_prompt(&mut self, ui: &mut egui::Ui) {
        egui::CentralPanel::default()
            .frame(
                egui::Frame::NONE
                    .fill(BG)
                    .inner_margin(egui::Margin::same(PAD)),
            )
            .show_inside(ui, |ui| {
                // ── Textarea ──────────────────────────────────────────────
                let input_frame = egui::Frame::NONE
                    .fill(BG_CARD)
                    .stroke(Stroke::new(1.0_f32, BORDER))
                    .corner_radius(CornerRadius::same(ROUNDING))
                    .inner_margin(egui::Margin::same(12i8));

                input_frame.show(ui, |ui| {
                    let hint = egui::RichText::new(
                        "Enter your instruction… (e.g., summarize, translate, fix grammar)",
                    )
                    .color(TEXT_MUTED);

                    let te = egui::TextEdit::multiline(&mut self.prompt_input)
                        .hint_text(hint)
                        .frame(egui::Frame::NONE)
                        .desired_width(ui.available_width())
                        .desired_rows(5)
                        .text_color(TEXT_WHITE)
                        .font(FontId::proportional(14.0));

                    ui.add(te).request_focus();
                });

                // ── Keyboard navigation ───────────────────────────────────
                let hlen = self.history.len();
                if ui.input(|i| i.key_pressed(egui::Key::ArrowDown)) {
                    self.selected_history_index = match self.selected_history_index {
                        None if hlen > 0 => Some(0),
                        Some(i) => Some((i + 1).min(hlen - 1)),
                        _ => None,
                    };
                    if let Some(i) = self.selected_history_index {
                        self.prompt_input = self.history[i].clone();
                    }
                }
                if ui.input(|i| i.key_pressed(egui::Key::ArrowUp)) {
                    self.selected_history_index = match self.selected_history_index {
                        None if hlen > 0 => Some(hlen - 1),
                        Some(0) => {
                            self.prompt_input.clear();
                            None
                        }
                        Some(i) => Some(i - 1),
                        _ => None,
                    };
                    if let Some(i) = self.selected_history_index {
                        self.prompt_input = self.history[i].clone();
                    }
                }

                // Enter (without Shift) to execute
                if ui.input(|i| i.key_pressed(egui::Key::Enter) && !i.modifiers.shift) {
                    let val = self.prompt_input.clone();
                    if !val.trim().is_empty() {
                        self.run_instruction(ui.ctx(), val);
                    }
                }

                ui.add_space(16.0);

                // ── Recent History ────────────────────────────────────────
                if !self.history.is_empty() {
                    ui.label(
                        egui::RichText::new("Recent History")
                            .color(TEXT_WHITE)
                            .strong()
                            .size(14.0),
                    );
                    ui.add_space(6.0);

                    let mut clicked: Option<String> = None;
                    egui::ScrollArea::vertical()
                        .max_height(160.0)
                        .show(ui, |ui| {
                            for (idx, item) in self.history.iter().enumerate() {
                                let is_sel = self.selected_history_index == Some(idx);
                                let w = ui.available_width();
                                let (rect, resp) = ui
                                    .allocate_exact_size(Vec2::new(w, 28.0), egui::Sense::click());

                                if resp.hovered() || is_sel {
                                    ui.painter()
                                        .rect_filled(rect, CornerRadius::same(4), BG_HOVER);
                                }

                                // Italic history item
                                ui.painter().text(
                                    rect.left_center() + Vec2::new(8.0, 0.0),
                                    egui::Align2::LEFT_CENTER,
                                    item,
                                    FontId::new(13.5, egui::FontFamily::Proportional),
                                    if is_sel { TEXT_WHITE } else { TEXT_HISTORY },
                                );

                                if resp.clicked() {
                                    clicked = Some(item.clone());
                                }

                                // Thin divider between items
                                if idx + 1 < self.history.len() {
                                    let (dr, _) = ui.allocate_exact_size(
                                        Vec2::new(w, 1.0),
                                        egui::Sense::hover(),
                                    );
                                    ui.painter().line_segment(
                                        [dr.left_center(), dr.right_center()],
                                        Stroke::new(1.0_f32, DIVIDER),
                                    );
                                }
                            }
                        });

                    if let Some(item) = clicked {
                        self.run_instruction(ui.ctx(), item);
                    }
                }

                // ── Status ────────────────────────────────────────────────
                if !self.status_msg.is_empty() {
                    ui.add_space(6.0);
                    ui.horizontal(|ui| {
                        if self.is_loading {
                            ui.spinner();
                        }
                        ui.label(
                            egui::RichText::new(&self.status_msg)
                                .color(if self.status_ok { C_TEAL } else { C_RED })
                                .size(12.5),
                        );
                    });
                }

                // ── Execute button — bottom right ────────────────────────
                ui.add_space(10.0);
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if blue_button(ui, "Execute", 110.0, 36.0) {
                        let val = self.prompt_input.clone();
                        if !val.trim().is_empty() {
                            self.run_instruction(ui.ctx(), val);
                        }
                    }
                });
            });
    }

    // ── Config panel ──────────────────────────────────────────────────────────

    fn draw_config(&mut self, ui: &mut egui::Ui) {
        egui::CentralPanel::default()
            .frame(
                egui::Frame::NONE
                    .fill(BG)
                    .inner_margin(egui::Margin::same(PAD)),
            )
            .show_inside(ui, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    // Each field: label above, full-width input below
                    cfg_field(ui, "Endpoint", |ui| {
                        ui.add(
                            egui::TextEdit::singleline(&mut self.endpoint)
                                .desired_width(f32::INFINITY)
                                .text_color(TEXT_WHITE),
                        );
                    });
                    cfg_field(ui, "Model", |ui| {
                        ui.add(
                            egui::TextEdit::singleline(&mut self.model)
                                .desired_width(f32::INFINITY)
                                .text_color(TEXT_WHITE),
                        );
                    });
                    cfg_field(ui, "Fallback Model", |ui| {
                        ui.add(
                            egui::TextEdit::singleline(&mut self.fallback_model)
                                .desired_width(f32::INFINITY)
                                .text_color(TEXT_WHITE),
                        );
                    });
                    cfg_field(ui, "Target Language", |ui| {
                        ui.add(
                            egui::TextEdit::singleline(&mut self.target_language)
                                .desired_width(f32::INFINITY)
                                .text_color(TEXT_WHITE),
                        );
                    });
                    cfg_field(ui, "Hotkey", |ui| {
                        ui.add(
                            egui::TextEdit::singleline(&mut self.hotkey)
                                .desired_width(f32::INFINITY)
                                .text_color(TEXT_WHITE),
                        );
                    });
                    cfg_field(ui, "Timeout (s)", |ui| {
                        ui.add(
                            egui::DragValue::new(&mut self.timeout_secs)
                                .range(1..=300)
                                .suffix(" s"),
                        );
                    });

                    ui.add_space(4.0);

                    // Checkboxes
                    ui.horizontal(|ui| {
                        ui.checkbox(&mut self.restore_clipboard, "");
                        ui.label(
                            egui::RichText::new("Restore clipboard after operation")
                                .color(TEXT_WHITE)
                                .size(14.0),
                        );
                    });
                    ui.horizontal(|ui| {
                        ui.checkbox(&mut self.show_notifications, "");
                        ui.label(
                            egui::RichText::new("Show notifications")
                                .color(TEXT_WHITE)
                                .size(14.0),
                        );
                    });

                    ui.add_space(4.0);

                    if !self.status_msg.is_empty() {
                        ui.label(
                            egui::RichText::new(&self.status_msg)
                                .color(if self.status_ok { C_TEAL } else { C_RED })
                                .size(12.5),
                        );
                        ui.add_space(4.0);
                    }

                    // ── MQTT Configuration ────────────────────────────────
                    ui.add_space(12.0);
                    ui.label(
                        egui::RichText::new("MQTT Configuration")
                            .color(TEXT_WHITE)
                            .strong()
                            .size(14.0),
                    );
                    ui.add_space(4.0);
                    cfg_field(ui, "MQTT Broker", |ui| {
                        ui.add(
                            egui::TextEdit::singleline(&mut self.mqtt_broker)
                                .desired_width(f32::INFINITY)
                                .text_color(TEXT_WHITE),
                        );
                    });
                    cfg_field(ui, "MQTT Username", |ui| {
                        ui.add(
                            egui::TextEdit::singleline(&mut self.mqtt_username)
                                .desired_width(f32::INFINITY)
                                .text_color(TEXT_WHITE),
                        );
                    });
                    cfg_field(ui, "MQTT Password", |ui| {
                        ui.add(
                            egui::TextEdit::singleline(&mut self.mqtt_password)
                                .password(true)
                                .desired_width(f32::INFINITY)
                                .text_color(TEXT_WHITE),
                        );
                    });
                    cfg_field(ui, "MQTT Topic", |ui| {
                        ui.add(
                            egui::TextEdit::singleline(&mut self.mqtt_topic)
                                .desired_width(f32::INFINITY)
                                .text_color(TEXT_WHITE),
                        );
                    });
                    cfg_field(ui, "MQTT Port", |ui| {
                        ui.add(egui::DragValue::new(&mut self.mqtt_port).range(1..=65535));
                    });
                    ui.horizontal(|ui| {
                        ui.checkbox(&mut self.mqtt_use_tls, "");
                        ui.label(egui::RichText::new("Use TLS").color(TEXT_WHITE).size(14.0));
                    });

                    // ── S3 Configuration ──────────────────────────────────
                    ui.add_space(12.0);
                    ui.label(
                        egui::RichText::new("S3 / MinIO Configuration")
                            .color(TEXT_WHITE)
                            .strong()
                            .size(14.0),
                    );
                    ui.add_space(4.0);
                    cfg_field(ui, "S3 Endpoint", |ui| {
                        ui.add(
                            egui::TextEdit::singleline(&mut self.s3_endpoint)
                                .desired_width(f32::INFINITY)
                                .text_color(TEXT_WHITE),
                        );
                    });
                    cfg_field(ui, "S3 Bucket", |ui| {
                        ui.add(
                            egui::TextEdit::singleline(&mut self.s3_bucket)
                                .desired_width(f32::INFINITY)
                                .text_color(TEXT_WHITE),
                        );
                    });
                    cfg_field(ui, "S3 Secret Key", |ui| {
                        ui.add(
                            egui::TextEdit::singleline(&mut self.s3_secret_key)
                                .password(true)
                                .desired_width(f32::INFINITY)
                                .text_color(TEXT_WHITE),
                        );
                    });

                    // ── Vision Configuration ──────────────────────────────
                    ui.add_space(12.0);
                    ui.label(
                        egui::RichText::new("Vision / Screenshot Analysis")
                            .color(TEXT_WHITE)
                            .strong()
                            .size(14.0),
                    );
                    ui.add_space(4.0);
                    cfg_field(ui, "Vision Model", |ui| {
                        ui.add(
                            egui::TextEdit::singleline(&mut self.vision_model)
                                .desired_width(f32::INFINITY)
                                .text_color(TEXT_WHITE),
                        );
                    });
                    cfg_field(ui, "Screenshot Hotkey", |ui| {
                        ui.add(
                            egui::TextEdit::singleline(&mut self.vision_hotkey)
                                .desired_width(f32::INFINITY)
                                .text_color(TEXT_WHITE),
                        );
                    });
                });

                // Buttons — bottom right: Cancel (gray) | Save (green)
                ui.add_space(8.0);
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if green_button(ui, "Save", 100.0, 36.0) {
                        self.save_config();
                    }
                    ui.add_space(8.0);
                    if gray_button(ui, "Cancel", 100.0, 36.0) {
                        std::process::exit(0);
                    }
                });
            });
    }

    fn save_config(&mut self) {
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

        self.config.mqtt.broker = self.mqtt_broker.clone();
        self.config.mqtt.username = self.mqtt_username.clone();
        self.config.mqtt.password = self.mqtt_password.clone();
        self.config.mqtt.topic = self.mqtt_topic.clone();
        self.config.mqtt.port = self.mqtt_port;
        self.config.mqtt.use_tls = self.mqtt_use_tls;

        self.config.s3.endpoint = self.s3_endpoint.clone();
        self.config.s3.bucket = self.s3_bucket.clone();
        self.config.s3.secret_key = self.s3_secret_key.clone();

        self.config.vision.model = self.vision_model.clone();
        self.config.vision.hotkey = self.vision_hotkey.clone();

        if let Err(e) = self.config.save() {
            self.status_msg = format!("Erreur : {e}");
            self.status_ok = false;
        } else {
            std::process::exit(0);
        }
    }

    // ── Stats panel ───────────────────────────────────────────────────────────

    fn draw_stats(&mut self, ui: &mut egui::Ui) {
        egui::CentralPanel::default()
            .frame(
                egui::Frame::NONE
                    .fill(BG)
                    .inner_margin(egui::Margin::same(PAD)),
            )
            .show_inside(ui, |ui| {
                let metrics = UsageMetrics::load();
                let avail_w = ui.available_width();
                let card_w = (avail_w - 12.0) / 2.0;
                let card_h = 100.0;

                // Row 1
                ui.horizontal(|ui| {
                    stat_card(
                        ui,
                        "Translations",
                        &format_num(metrics.total_translations),
                        C_TEAL,
                        card_w,
                        card_h,
                    );
                    ui.add_space(12.0);
                    stat_card(
                        ui,
                        "Errors",
                        &metrics.total_errors.to_string(),
                        C_RED,
                        card_w,
                        card_h,
                    );
                });
                ui.add_space(12.0);
                // Row 2
                ui.horizontal(|ui| {
                    stat_card(
                        ui,
                        "Avg Latency",
                        &format!("{:.1}s", metrics.avg_latency_ms() / 1000.0),
                        C_YELLOW,
                        card_w,
                        card_h,
                    );
                    ui.add_space(12.0);
                    let mb = metrics.total_bytes_processed as f64 / 1_048_576.0;
                    stat_card(
                        ui,
                        "Volume",
                        &format!("{mb:.1} MB"),
                        C_PURPLE,
                        card_w,
                        card_h,
                    );
                });

                // Model usage bars
                if !metrics.model_usage.is_empty() {
                    ui.add_space(20.0);
                    ui.label(
                        egui::RichText::new("Model Usage")
                            .color(TEXT_WHITE)
                            .strong()
                            .size(14.0),
                    );
                    ui.add_space(8.0);

                    let total: u64 = metrics.model_usage.values().sum();
                    let bar_colors = [BAR_BLUE, BAR_GREEN, ACCENT_BLUE, C_PURPLE, C_YELLOW];
                    for (i, (model, count)) in metrics.model_usage.iter().enumerate() {
                        let pct = if total > 0 {
                            *count as f32 / total as f32
                        } else {
                            0.0
                        };
                        let bar_color = bar_colors[i % bar_colors.len()];
                        model_usage_bar(ui, model, *count, pct, bar_color);
                        ui.add_space(6.0);
                    }
                }

                // Reset button — bottom right
                ui.add_space(16.0);
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if gray_button(ui, "Reset", 90.0, 32.0) {
                        UsageMetrics::default().save();
                    }
                });
            });
    }
}

// ── Buttons ───────────────────────────────────────────────────────────────────

fn blue_button(ui: &mut egui::Ui, label: &str, w: f32, h: f32) -> bool {
    let (rect, resp) = ui.allocate_exact_size(Vec2::new(w, h), egui::Sense::click());
    let color = if resp.hovered() {
        ACCENT_BLUE_HOV
    } else {
        ACCENT_BLUE
    };
    ui.painter().rect_filled(rect, CornerRadius::same(6), color);
    ui.painter().text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        label,
        FontId::proportional(14.0),
        Color32::WHITE,
    );
    resp.clicked()
}

fn green_button(ui: &mut egui::Ui, label: &str, w: f32, h: f32) -> bool {
    let (rect, resp) = ui.allocate_exact_size(Vec2::new(w, h), egui::Sense::click());
    let color = if resp.hovered() {
        ACCENT_GREEN_HOV
    } else {
        ACCENT_GREEN
    };
    ui.painter().rect_filled(rect, CornerRadius::same(6), color);
    ui.painter().text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        label,
        FontId::proportional(14.0),
        Color32::WHITE,
    );
    resp.clicked()
}

fn gray_button(ui: &mut egui::Ui, label: &str, w: f32, h: f32) -> bool {
    let (rect, resp) = ui.allocate_exact_size(Vec2::new(w, h), egui::Sense::click());
    let bg = if resp.hovered() {
        Color32::from_rgb(58, 65, 84)
    } else {
        Color32::from_rgb(45, 52, 68)
    };
    ui.painter().rect_filled(rect, CornerRadius::same(6), bg);
    ui.painter().rect_stroke(
        rect,
        CornerRadius::same(6),
        Stroke::new(1.0_f32, BORDER),
        egui::StrokeKind::Middle,
    );
    ui.painter().text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        label,
        FontId::proportional(14.0),
        TEXT_WHITE,
    );
    resp.clicked()
}

// ── Config field widget ───────────────────────────────────────────────────────

fn cfg_field(ui: &mut egui::Ui, label: &str, content: impl FnOnce(&mut egui::Ui)) {
    ui.label(egui::RichText::new(label).color(TEXT_WHITE).size(14.0));
    ui.add_space(2.0);
    content(ui);
    ui.add_space(8.0);
}

// ── Stat card widget ──────────────────────────────────────────────────────────

fn stat_card(ui: &mut egui::Ui, label: &str, value: &str, value_color: Color32, w: f32, h: f32) {
    let (rect, _) = ui.allocate_exact_size(Vec2::new(w, h), egui::Sense::hover());
    ui.painter()
        .rect_filled(rect, CornerRadius::same(ROUNDING), BG_CARD);
    ui.painter().rect_stroke(
        rect,
        CornerRadius::same(ROUNDING),
        Stroke::new(1.0_f32, BORDER),
        egui::StrokeKind::Middle,
    );

    // Label at top-centre
    ui.painter().text(
        rect.center_top() + Vec2::new(0.0, 18.0),
        egui::Align2::CENTER_CENTER,
        label,
        FontId::proportional(14.0),
        TEXT_WHITE,
    );
    // Value large
    ui.painter().text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        value,
        FontId::proportional(28.0),
        value_color,
    );
}

// ── Model usage bar widget ────────────────────────────────────────────────────

fn model_usage_bar(ui: &mut egui::Ui, model: &str, count: u64, pct: f32, color: Color32) {
    let avail_w = ui.available_width();
    let label_w = 110.0;
    let pct_w = 42.0;
    let bar_area = avail_w - label_w - pct_w - 16.0;
    let bar_h = 16.0;

    ui.horizontal(|ui| {
        // Model name
        ui.add_sized(
            Vec2::new(label_w, bar_h),
            egui::Label::new(egui::RichText::new(model).color(TEXT_MUTED).size(13.0)),
        );

        // Bar background + fill
        let (rect, _) = ui.allocate_exact_size(Vec2::new(bar_area, bar_h), egui::Sense::hover());
        ui.painter()
            .rect_filled(rect, CornerRadius::same(4), BG_FIELD);
        let fill_w = (bar_area * pct).max(0.0);
        if fill_w > 0.0 {
            let fill = egui::Rect::from_min_size(rect.min, Vec2::new(fill_w, bar_h));
            ui.painter().rect_filled(fill, CornerRadius::same(4), color);
        }

        // Percentage label
        ui.label(
            egui::RichText::new(format!("{}% ({}×)", (pct * 100.0) as u32, count))
                .color(TEXT_MUTED)
                .size(12.5),
        );
    });
}

// ── Number formatting ─────────────────────────────────────────────────────────

fn format_num(n: u64) -> String {
    let s = n.to_string();
    let mut result = String::new();
    for (i, c) in s.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            result.push(',');
        }
        result.push(c);
    }
    result.chars().rev().collect()
}

// ── Persistence ───────────────────────────────────────────────────────────────

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
    #[cfg(not(windows))]
    let _ = history;
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
