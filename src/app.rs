use std::sync::mpsc::{channel, Receiver, Sender};

use egui::{Id, Rangef, Vec2};
use rfd::{AsyncFileDialog};

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct TemplateApp {
    last_loaded_file_contents: String,
    year: String,
    month: String,
    #[serde(skip)]
    text_channel: (Sender<String>, Receiver<String>),
}

impl Default for TemplateApp {
    fn default() -> Self {
        Self {
            last_loaded_file_contents: "test".to_owned(),
            year: "".to_owned(),
            month: "".to_owned(),
            text_channel: channel(),
        }
    }
}

impl TemplateApp {
    // Called on startup/open
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        if let Some(storage) = cc.storage {
            eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default()
        } else {
            Default::default()
        }
    }
}

impl eframe::App for TemplateApp {
    /// Called by the framework to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        // Put your widgets into a `SidePanel`, `TopBottomPanel`, `CentralPanel`, `Window` or `Area`.
        // For inspiration and more examples, go to https://emilk.github.io/egui
        let text_editor_id : Id = Id::new("text_editor");
        if let Ok(text) = self.text_channel.1.try_recv() {
            self.last_loaded_file_contents = text;
        }

        egui::Panel::top("header").show_inside(ui, |ui|{
            egui::MenuBar::new().ui(ui, |ui| {
                // NOTE: no File->Quit on web pages!
                let is_web = cfg!(target_arch = "wasm32");
                if !is_web {
                    ui.menu_button("File", |ui| {
                        if ui.button("Quit").clicked() {
                            ui.send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                    });
                    ui.add_space(16.0);
                }

                egui::widgets::global_theme_preference_buttons(ui);
            });
            ui.vertical_centered(|ui| {
                ui.heading("bluedger");
            });
        });

        egui::Panel::bottom("footer").show_inside(ui, |ui| {
            ui.vertical_centered(|ui| {
                egui::warn_if_debug_build(ui);
            });
            ui.horizontal(|ui| {
                powered_by_egui_and_eframe(ui);
                ui.add(egui::github_link_file!(
                    "https://github.com/BlueWeabo/bluedger/blob/main/",
                    "Source code."
                ));
            });
        });

        let window_width = ui.ctx().input(|i| {i.content_rect()}).width();
        egui::Panel::left("text_editor_panel").size_range(Rangef::new(window_width * 0.2, window_width * 0.5)).show_inside(ui, |ui|{
            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    let ui_available_size = ui.available_size();
                    let available_size = Vec2::new(ui_available_size.x / 3.0, ui_available_size.y);
                    ui.vertical(|ui| {
                        ui.label("Year");
                        ui.add_sized(available_size, egui::TextEdit::singleline(&mut self.year));
                    });
                    ui.vertical(|ui| {
                        ui.label("Month");
                        ui.add_sized(available_size, egui::TextEdit::singleline(&mut self.month));
                    });
                    let ui_available_size = ui.available_size();
                    let available_size = Vec2::new(ui_available_size.x, ui_available_size.y / 2.0);
                    ui.vertical(|ui| {
                        if ui.add_sized(available_size, egui::Button::new("Load File")).clicked() {
                            let sender = self.text_channel.0.clone();
                            let task = AsyncFileDialog::new().pick_file();
                            let ctx = ui.ctx().clone();
                            execute(async move {
                                let file = task.await;
                                if let Some(file) = file {
                                    let text = file.read().await;
                                    let _ = sender.send(String::from_utf8_lossy(&text).to_string());
                                    ctx.request_repaint();
                                }
                            });
                        }
                        if ui.add_sized(available_size, egui::Button::new("Save File")).clicked() {
                            // save the file
                        }
                    });
                });
                ui.add_space(10.0);
                egui::ScrollArea::vertical().show(ui, |ui| {
                    ui.add_sized(ui.available_size(), egui::TextEdit::multiline(&mut self.last_loaded_file_contents).code_editor().id(text_editor_id));
                });
            });
        });

        egui::CentralPanel::default().show_inside(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label("NOOO");
            });
        });
    }
}

fn powered_by_egui_and_eframe(ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 0.0;
        ui.label("Powered by ");
        ui.hyperlink_to("egui", "https://github.com/emilk/egui");
        ui.label(" and ");
        ui.hyperlink_to(
            "eframe",
            "https://github.com/emilk/egui/tree/master/crates/eframe",
        );
        ui.label(".");
    });
}

#[cfg(not(target_arch = "wasm32"))]
fn execute<F: Future<Output = ()> + Send + 'static>(f: F) {
    // this is stupid... use any executor of your choice instead

    use smol::future::block_on;
    std::thread::spawn(move || block_on(f));
}

#[cfg(target_arch = "wasm32")]
fn execute<F: Future<Output = ()> + 'static>(f: F) {
    wasm_bindgen_futures::spawn_local(f);
}
