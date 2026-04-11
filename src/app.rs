use std::ptr::null;

use egui::{Id, Rangef};

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct TemplateApp {
    // Example stuff:
    label: String,
    last_loaded_file_contents: String,

    //##[serde(skip)] // This how you opt-out of serialization of a field
    value: f32,
}

impl Default for TemplateApp {
    fn default() -> Self {
        Self {
            // Example stuff:
            label: "Hello World!".to_owned(),
            last_loaded_file_contents: "test".to_owned(),
            value: 2.7,
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
        let TEXT_EDITOR_ID : Id = Id::new("text_editor");

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
            ui.add_sized(ui.available_size(), egui::TextEdit::multiline(&mut self.last_loaded_file_contents).id(TEXT_EDITOR_ID));
        });

        egui::CentralPanel::default().show_inside(ui, |ui| {

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
