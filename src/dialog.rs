use anyhow::Result;
use std::sync::{Arc, Mutex};

#[cfg(windows)]
pub fn show_prompt_dialog() -> Result<String> {
    let result = Arc::new(Mutex::new(None));
    let result_clone = result.clone();

    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_inner_size(eframe::egui::vec2(300.0, 120.0))
            .with_resizable(false),
        ..Default::default()
    };

    struct App {
        input: String,
        result: Arc<Mutex<Option<String>>>,
    }

    impl eframe::App for App {
        fn update(&mut self, ctx: &eframe::egui::Context, _frame: &mut eframe::Frame) {
            eframe::egui::CentralPanel::default().show(ctx, |ui| {
                ui.label("Entrez votre instruction personnalisée :");
                let resp = ui.text_edit_singleline(&mut self.input);
                resp.request_focus();

                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    if ui.button("OK").clicked()
                        || ui.input(|i| i.key_pressed(eframe::egui::Key::Enter))
                    {
                        let mut res = self.result.lock().unwrap();
                        *res = Some(self.input.clone());
                        ctx.send_viewport_cmd(eframe::egui::ViewportCommand::Close);
                    }
                    if ui.button("Annuler").clicked() {
                        ctx.send_viewport_cmd(eframe::egui::ViewportCommand::Close);
                    }
                });
            });
        }
    }

    let app = App {
        input: String::new(),
        result: result_clone,
    };

    eframe::run_native(
        "Thoth — Instruction",
        options,
        Box::new(|_cc| Ok(Box::new(app))),
    )
    .map_err(|e| anyhow::anyhow!("Eframe error: {:?}", e))?;

    let res = result.lock().unwrap().clone();
    res.ok_or_else(|| anyhow::anyhow!("cancelled"))
}

#[cfg(not(windows))]
pub fn show_prompt_dialog() -> Result<String> {
    tracing::warn!("input dialog not supported on this platform");
    anyhow::bail!("input dialog not supported on this platform")
}
