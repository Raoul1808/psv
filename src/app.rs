use std::{fmt::Display, path::PathBuf};

use eframe::egui;
use egui_file_dialog::FileDialog;

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
    push_swap_path: Option<PathBuf>,
    file_dialog: FileDialog,
    app_mode: AppMode,
}

impl App {
    pub fn new(_cc: &eframe::CreationContext) -> Self {
        Self {
            push_swap_path: None,
            file_dialog: FileDialog::new(),
            app_mode: AppMode::Visualize,
        }
    }
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut eframe::egui::Ui, _frame: &mut eframe::Frame) {
        egui::Panel::top("menu_bar").show(ui, |ui| {
            egui::MenuBar::new().ui(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Select push_swap executable").clicked() {
                        self.file_dialog.pick_file();
                    }
                    ui.separator();
                    if ui.button("Quit").clicked() {
                        ui.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                });
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

        egui::CentralPanel::default().show(ui, |ui| {
            ui.label(format!(
                "push_swap executable path: {}",
                self.push_swap_path
                    .as_ref()
                    .map(|p| p.display().to_string())
                    .unwrap_or("None".into())
            ));
            if ui.button("Select").clicked() {
                self.file_dialog.pick_file();
            }
        });

        self.file_dialog.update(ui);
        if let Some(path) = self.file_dialog.take_picked() {
            self.push_swap_path = Some(path.to_path_buf());
        }
    }
}
