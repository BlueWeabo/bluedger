#[cfg(target_arch = "wasm32")]
use std::sync::mpsc::{Receiver, Sender, channel};

#[cfg(target_arch = "wasm32")]
use std::ops::RangeInclusive;

#[cfg(target_arch = "wasm32")]
use egui::{Id, Rangef, Vec2};

#[cfg(target_arch = "wasm32")]
use serde::{Deserialize, Serialize};

#[cfg(target_arch = "wasm32")]
use crate::{FileObject, Funds, YearlyGraphPoints};

#[cfg(target_arch = "wasm32")]
use egui_plot::{GridMark, Legend, Line, Plot, PlotPoint, PlotPoints, HoverPosition};

#[cfg(target_arch = "wasm32")]
use chrono::{DateTime, Datelike, Utc};

#[cfg(target_arch = "wasm32")]
#[derive(Deserialize, Serialize)]
#[serde(default)]
pub struct TemplateApp {
    last_loaded_file_contents: String,
    year: String,
    month: String,
    assets: Funds,
    last_month_expenses: Funds,
    current_expenses: Funds,
    expenses_plot: YearlyGraphPoints,
    income_plot: YearlyGraphPoints,
    assets_plot: YearlyGraphPoints,
    liabilities_plot: YearlyGraphPoints,
    equity_plot: YearlyGraphPoints,
    #[serde(skip)]
    channel_file: (Sender<String>, Receiver<String>),
    #[serde(skip)]
    channel_assets: (Sender<Funds>, Receiver<Funds>),
    #[serde(skip)]
    channel_last_expenses: (Sender<Funds>, Receiver<Funds>),
    #[serde(skip)]
    channel_current_expenses: (Sender<Funds>, Receiver<Funds>),
    #[serde(skip)]
    channel_expenses_plot: (Sender<YearlyGraphPoints>, Receiver<YearlyGraphPoints>),
    #[serde(skip)]
    channel_income_plot: (Sender<YearlyGraphPoints>, Receiver<YearlyGraphPoints>),
    #[serde(skip)]
    channel_assets_plot: (Sender<YearlyGraphPoints>, Receiver<YearlyGraphPoints>),
    #[serde(skip)]
    channel_liabilities_plot: (Sender<YearlyGraphPoints>, Receiver<YearlyGraphPoints>),
    #[serde(skip)]
    channel_equity_plot: (Sender<YearlyGraphPoints>, Receiver<YearlyGraphPoints>),
    #[serde(skip)]
    expenses_plot_points: Vec<PlotPoint>,
    #[serde(skip)]
    income_plot_points: Vec<PlotPoint>,
    #[serde(skip)]
    assets_plot_points: Vec<PlotPoint>,
    #[serde(skip)]
    liabilities_plot_ploints: Vec<PlotPoint>,
    #[serde(skip)]
    equity_plot_points: Vec<PlotPoint>,
    #[serde(skip)]
    update: bool,
    #[serde(skip)]
    update_plot: bool,
}

#[cfg(target_arch = "wasm32")]
impl Default for TemplateApp {
    fn default() -> Self {
        Self {
            last_loaded_file_contents: "test".to_owned(),
            year: "".to_owned(),
            month: "".to_owned(),
            channel_file: channel(),
            assets: Funds {
                amounts: Vec::new(),
                currencies: Vec::new(),
                size: 0,
            },
            last_month_expenses: Funds {
                amounts: Vec::new(),
                currencies: Vec::new(),
                size: 0,
            },
            current_expenses: Funds {
                amounts: Vec::new(),
                currencies: Vec::new(),
                size: 0,
            },
            expenses_plot: YearlyGraphPoints { points: Vec::new() },
            income_plot: YearlyGraphPoints { points: Vec::new() },
            assets_plot: YearlyGraphPoints { points: Vec::new() },
            liabilities_plot: YearlyGraphPoints { points: Vec::new() },
            equity_plot: YearlyGraphPoints { points: Vec::new() },
            channel_assets: channel(),
            channel_last_expenses: channel(),
            channel_current_expenses: channel(),
            channel_expenses_plot: channel(),
            channel_income_plot: channel(),
            channel_assets_plot: channel(),
            channel_liabilities_plot: channel(),
            channel_equity_plot: channel(),
            update: false,
            update_plot: true,
            expenses_plot_points: Vec::new(),
            income_plot_points: Vec::new(),
            assets_plot_points: Vec::new(),
            liabilities_plot_ploints: Vec::new(),
            equity_plot_points: Vec::new(),
        }
    }
}

#[cfg(target_arch = "wasm32")]
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

#[cfg(target_arch = "wasm32")]
impl eframe::App for TemplateApp {
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let text_editor_id: Id = Id::new("text_editor");

        if let Ok(text) = self.channel_file.1.try_recv() {
            self.last_loaded_file_contents = text;
        }

        if let Ok(fund) = self.channel_assets.1.try_recv() {
            self.assets = fund;
        }

        if let Ok(fund) = self.channel_last_expenses.1.try_recv() {
            self.last_month_expenses = fund;
        }

        if let Ok(fund) = self.channel_current_expenses.1.try_recv() {
            self.current_expenses = fund;
        }

        if let Ok(plot) = self.channel_expenses_plot.1.try_recv() {
            self.expenses_plot = plot;
            self.update_plot = true;
        }

        if let Ok(plot) = self.channel_income_plot.1.try_recv() {
            self.income_plot = plot;
            self.update_plot = true;
        }

        if let Ok(plot) = self.channel_assets_plot.1.try_recv() {
            self.assets_plot = plot;
            self.update_plot = true;
        }

        if let Ok(plot) = self.channel_liabilities_plot.1.try_recv() {
            self.liabilities_plot = plot;
            self.update_plot = true;
        }

        if let Ok(plot) = self.channel_equity_plot.1.try_recv() {
            self.equity_plot = plot;
            self.update_plot = true;
        }

        if self.update {
            let ctx = ui.ctx().clone();
            let sender_assets = self.channel_assets.0.clone();
            ehttp::fetch(ehttp::Request::get("/api/assets"), move |response| {
                let assets = response.unwrap().json::<Funds>().unwrap();
                let _ = sender_assets.send(assets);
                ctx.request_repaint();
            });
            let ctx = ui.ctx().clone();
            let sender_last_expenses = self.channel_last_expenses.0.clone();
            ehttp::fetch(
                ehttp::Request::get("/api/last_month_expenses"),
                move |response| {
                    let last_month_expenses = response.unwrap().json::<Funds>().unwrap();
                    let _ = sender_last_expenses.send(last_month_expenses);
                    ctx.request_repaint();
                },
            );
            let ctx = ui.ctx().clone();
            let sender_current_expenses = self.channel_current_expenses.0.clone();
            ehttp::fetch(
                ehttp::Request::get("/api/current_expenses"),
                move |response| {
                    let current_expenses = response.unwrap().json::<Funds>().unwrap();
                    let _ = sender_current_expenses.send(current_expenses);
                    ctx.request_repaint();
                },
            );
            let ctx = ui.ctx().clone();
            let sender_expenses_plot = self.channel_expenses_plot.0.clone();
            ehttp::fetch(ehttp::Request::get("/api/expenses_plot"), move |response| {
                let expenses_plot = response.unwrap().json::<YearlyGraphPoints>().unwrap();
                let _ = sender_expenses_plot.send(expenses_plot);
                ctx.request_repaint();
            });
            let ctx = ui.ctx().clone();
            let sender_income_plot = self.channel_income_plot.0.clone();
            ehttp::fetch(ehttp::Request::get("/api/income_plot"), move |response| {
                let income_plot = response.unwrap().json::<YearlyGraphPoints>().unwrap();
                let _ = sender_income_plot.send(income_plot);
                ctx.request_repaint();
            });
            let ctx = ui.ctx().clone();
            let sender_assets_plot = self.channel_assets_plot.0.clone();
            ehttp::fetch(ehttp::Request::get("/api/assets_plot"), move |response| {
                let assets_plot = response.unwrap().json::<YearlyGraphPoints>().unwrap();
                let _ = sender_assets_plot.send(assets_plot);
                ctx.request_repaint();
            });
            let ctx = ui.ctx().clone();
            let sender_liabilities_plot = self.channel_liabilities_plot.0.clone();
            ehttp::fetch(
                ehttp::Request::get("/api/liabilities_plot"),
                move |response| {
                    let liabilities_plot = response.unwrap().json::<YearlyGraphPoints>().unwrap();
                    let _ = sender_liabilities_plot.send(liabilities_plot);
                    ctx.request_repaint();
                },
            );
            let ctx = ui.ctx().clone();
            let sender_equity_plot = self.channel_equity_plot.0.clone();
            ehttp::fetch(ehttp::Request::get("/api/equity_plot"), move |response| {
                let equity_plot = response.unwrap().json::<YearlyGraphPoints>().unwrap();
                let _ = sender_equity_plot.send(equity_plot);
                ctx.request_repaint();
            });
            self.update = false;
        }

        if self.update_plot {
            self.expenses_plot_points = self
                .expenses_plot
                .points
                .iter()
                .map(|p| PlotPoint::new(p[0], p[1]))
                .collect();
            self.income_plot_points = self
                .income_plot
                .points
                .iter()
                .map(|p| PlotPoint::new(p[0], p[1]))
                .collect();
            self.assets_plot_points = self
                .assets_plot
                .points
                .iter()
                .map(|p| PlotPoint::new(p[0], p[1]))
                .collect();
            self.liabilities_plot_ploints = self
                .liabilities_plot
                .points
                .iter()
                .map(|p| PlotPoint::new(p[0], p[1]))
                .collect();
            self.equity_plot_points = self
                .equity_plot
                .points
                .iter()
                .map(|p| PlotPoint::new(p[0], p[1]))
                .collect();
            self.update_plot = false;
        }

        egui::Panel::top("header").show(ui, |ui| {
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

        egui::Panel::bottom("footer").show(ui, |ui| {
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

        let window_width = ui.ctx().input(|i| i.content_rect()).width();
        egui::Panel::left("text_editor_panel")
            .size_range(Rangef::new(window_width * 0.2, window_width * 0.5))
            .show(ui, |ui| {
                ui.vertical(|ui| {
                    ui.horizontal(|ui| {
                        let ui_available_size = ui.available_size();
                        let available_size =
                            Vec2::new(ui_available_size.x / 3.0, ui_available_size.y);
                        ui.vertical(|ui| {
                            ui.add_sized(available_size, egui::Label::new("Year"));
                            ui.add_sized(
                                available_size,
                                egui::TextEdit::singleline(&mut self.year),
                            );
                        });
                        ui.vertical(|ui| {
                            ui.add_sized(available_size, egui::Label::new("Month"));
                            ui.add_sized(
                                available_size,
                                egui::TextEdit::singleline(&mut self.month),
                            );
                        });
                        let ui_available_size = ui.available_size();
                        let available_size =
                            Vec2::new(ui_available_size.x, ui_available_size.y / 2.0);
                        ui.vertical(|ui| {
                            if ui
                                .add_sized(available_size, egui::Button::new("Load File"))
                                .clicked()
                            {
                                self.update = true;
                                let ctx = ui.ctx().clone();
                                let sender = self.channel_file.0.clone();
                                ehttp::fetch(
                                    ehttp::Request::post_json(
                                        "/api/get",
                                        &FileObject {
                                            year: self.year.parse().unwrap(),
                                            month: self.month.parse().unwrap(),
                                            contents: "".to_owned(),
                                        },
                                    )
                                    .unwrap(),
                                    move |response| {
                                        let file = response.unwrap().json::<FileObject>().unwrap();
                                        let _ = sender.send(file.contents);
                                        ctx.request_repaint();
                                    },
                                );
                            }
                            if ui
                                .add_sized(available_size, egui::Button::new("Save File"))
                                .clicked()
                            {
                                self.update = true;
                                let ctx = ui.ctx().clone();
                                ehttp::fetch(
                                    ehttp::Request::post_json(
                                        "/api/update",
                                        &FileObject {
                                            year: self.year.parse().unwrap(),
                                            month: self.month.parse().unwrap(),
                                            contents: self.last_loaded_file_contents.clone(),
                                        },
                                    )
                                    .unwrap(),
                                    move |_| {
                                        ctx.request_repaint();
                                    },
                                );
                            }
                        });
                    });
                    ui.add_space(10.0);
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        ui.add_sized(
                            ui.available_size(),
                            egui::TextEdit::multiline(&mut self.last_loaded_file_contents)
                                .code_editor()
                                .id(text_editor_id),
                        );
                    });
                });
            });
        egui::Panel::right("right_toolbar_panel")
            .resizable(false)
            .show(ui, |_ui| {});
        egui::Panel::top("inner_top_panel").show(ui, |ui| {
            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    let ui_size = ui.available_size();
                    let avail_size = Vec2::new(ui_size.x / 3.0, ui_size.y);
                    ui.vertical(|ui| {
                        ui.add_sized(avail_size, egui::Label::new("Current Assets"));
                        if self.assets.size > 0 {
                            for i in 0..self.assets.size {
                                ui.horizontal(|ui| {
                                    ui.label(
                                        self.assets.amounts.get(i as usize).unwrap().to_string()
                                    );
                                    ui.label(self.assets.currencies.get(i as usize).unwrap());
                                });
                            }
                        }
                    });
                    ui.vertical(|ui| {
                        ui.add_sized(avail_size, egui::Label::new("Last Month Expenses"));
                        if self.last_month_expenses.size > 0 {
                            for i in 0..self.last_month_expenses.size {
                                ui.horizontal(|ui| {
                                    ui.label(
                                        self.last_month_expenses
                                            .amounts
                                            .get(i as usize)
                                            .unwrap()
                                            .to_string(),
                                    );
                                    ui.label(
                                        self.last_month_expenses
                                            .currencies
                                            .get(i as usize)
                                            .unwrap(),
                                    );
                                });
                            }
                        }
                    });
                    ui.vertical(|ui| {
                        ui.add_sized(avail_size, egui::Label::new("Current Expenses"));
                        if self.current_expenses.size > 0 {
                            for i in 0..self.current_expenses.size {
                                ui.horizontal(|ui| {
                                    ui.label(
                                        self.current_expenses
                                            .amounts
                                            .get(i as usize)
                                            .unwrap()
                                            .to_string(),
                                    );
                                    ui.label(
                                        self.current_expenses.currencies.get(i as usize).unwrap(),
                                    );
                                });
                            }
                        }
                    });
                });
            });
            egui::CentralPanel::default().show(ui, |ui| {
                ui.vertical(|ui| {
                    Plot::new("Income-Expenses")
                        .legend(Legend::default())
                        .allow_scroll(false)
                        .allow_zoom(false)
                        .allow_drag(false)
                        .allow_axis_zoom_drag(false)
                        .default_x_bounds(0.0, 12.3)
                        .label_formatter(|pos| match pos {
                            HoverPosition::NearDataPoint {
                                plot_name,
                                position,
                                index,
                            } => {
                                Some(format!("{}: {:.2} EUR", plot_name, position.y))
                            }
                            _ => None,
                        })
                        .x_axis_formatter(special_x_axis_formatter_previous_months)
                        .show(ui, |plot_ui| {
                            plot_ui.line(
                                Line::new(
                                    "expenses",
                                    PlotPoints::Borrowed(&self.expenses_plot_points),
                                )
                                .color(egui::Color32::from_rgb(200, 100, 100)),
                            );
                            plot_ui.line(
                                Line::new("income", PlotPoints::Borrowed(&self.income_plot_points))
                                    .color(egui::Color32::from_rgb(100, 200, 100)),
                            );
                            plot_ui.line(
                                Line::new("assets", PlotPoints::Borrowed(&self.assets_plot_points))
                                    .color(egui::Color32::from_rgb(100, 100, 200)),
                            );
                            plot_ui.line(
                                Line::new(
                                    "liabilities",
                                    PlotPoints::Borrowed(&self.liabilities_plot_ploints),
                                )
                                .color(egui::Color32::from_rgb(200, 100, 200)),
                            );
                            plot_ui.line(
                                Line::new("equity", PlotPoints::Borrowed(&self.equity_plot_points))
                                    .color(egui::Color32::from_rgb(100, 200, 200)),
                            );
                        });
                    ui.horizontal(|ui| {
                        ui.label("");
                    })
                })
            });
        });
    }
}

#[cfg(target_arch = "wasm32")]
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

#[cfg(target_arch = "wasm32")]
fn special_x_axis_formatter_previous_months(
    grid: GridMark,
    _value: &RangeInclusive<f64>,
) -> String {
    let date: DateTime<Utc> = Utc::now();
    let month = date.month() as f64;
    let year = date.year();
    let month_to_convert = if month - 12.0 + grid.value <= 0.0 {
        month + grid.value
    } else {
        month - 12.0 + grid.value
    };
    format!(
        "{} {}",
        match month_to_convert {
            1.0 => "Jan".to_string(),
            2.0 => "Feb".to_string(),
            3.0 => "Mar".to_string(),
            4.0 => "Apr".to_string(),
            5.0 => "May".to_string(),
            6.0 => "Jun".to_string(),
            7.0 => "Jul".to_string(),
            8.0 => "Aug".to_string(),
            9.0 => "Sep".to_string(),
            10.0 => "Oct".to_string(),
            11.0 => "Nov".to_string(),
            12.0 => "Dec".to_string(),
            _ => "Error".to_string(),
        },
        year
    )
}
