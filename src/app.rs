use std::fmt::Display;

use eframe::egui::{self, Widget};
use egui_alignments::center_horizontal;

use crate::{config::Config, visualizer::Visualizer};

#[derive(Debug, Clone, Copy, PartialEq)]
enum AppMode {
    Visualize,
    Benchmark,
}

impl Display for AppMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AppMode::Visualize => write!(f, "Visualize"),
            AppMode::Benchmark => write!(f, "Benchmark"),
        }
    }
}

pub struct App {
    app_mode: AppMode,
    visualizer: Visualizer,
    config: Config,
}

impl App {
    pub fn new(cc: &eframe::CreationContext, config: Config) -> Self {
        let mut visualizer = Visualizer::new(cc);
        if let Some(path) = config.push_swap_path.clone() {
            visualizer.update_path(path);
        }
        Self {
            config,
            app_mode: AppMode::Visualize,
            visualizer,
        }
    }

    fn select_executable(&mut self) {
        let file = rfd::FileDialog::new().pick_file();
        if let Some(file) = file {
            self.visualizer.update_path(file.clone());
            self.config.push_swap_path = Some(file);
            let res = self.config.save();
            if let Err(e) = res {
                eprintln!("error while saving config: {e}");
            }
        }
    }

    fn first_time_ui(&mut self, ui: &mut egui::Ui) {
        egui::Modal::new("missing_path".into()).show(ui, |ui| {
            ui.horizontal(|ui| {
                center_horizontal(ui, |ui| ui.heading("Setup"));
            });
            ui.separator();
            ui.label("This program requires a push_swap executable to run!");
            ui.label("Hint: you can change this later with File -> Select push_swap executable");
            ui.add_space(7.);
            center_horizontal(ui, |ui| {
                if egui::Button::new("Select push_swap executable")
                    .min_size([ui.available_width(), 0.0].into())
                    .ui(ui)
                    .clicked()
                {
                    self.select_executable();
                }
            });
        });
    }

    fn app_ui(&mut self, ui: &mut egui::Ui) {
        egui::Panel::top("menu_bar").show(ui, |ui| {
            egui::MenuBar::new().ui(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Select push_swap executable").clicked() {
                        self.select_executable();
                    }
                    ui.separator();
                    if ui.button("Quit").clicked() {
                        ui.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                });
                match self.app_mode {
                    AppMode::Visualize => {
                        self.visualizer.menu_ui(ui);
                    }
                    AppMode::Benchmark => {}
                }
                ui.menu_button("Help", |ui| {
                    ui.add_enabled(false, egui::Button::new("TODO: add button"));
                    ui.separator();
                    if ui.button("About psv").clicked() {
                        println!("About modal!!");
                    }
                });
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
                    egui::ComboBox::new("app_mode", "App Mode:")
                        .selected_text(self.app_mode.to_string())
                        .show_ui(ui, |ui| {
                            ui.selectable_value(
                                &mut self.app_mode,
                                AppMode::Visualize,
                                AppMode::Visualize.to_string(),
                            );
                            ui.selectable_value(
                                &mut self.app_mode,
                                AppMode::Benchmark,
                                AppMode::Benchmark.to_string(),
                            );
                        });
                });
            });
        });

        match self.app_mode {
            AppMode::Visualize => {
                self.visualizer.ui(ui);
            }
            AppMode::Benchmark => {
                ui.label("todo: implement benchmarking ui");
            }
        }
    }
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut eframe::egui::Ui, _frame: &mut eframe::Frame) {
        let path_set = self.config.push_swap_path.is_some();
        if path_set {
            self.app_ui(ui);
        } else {
            self.first_time_ui(ui);
        }
    }
}
