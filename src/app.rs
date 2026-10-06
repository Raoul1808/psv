use std::path::PathBuf;

use eframe::egui;
use egui_file_dialog::FileDialog;

pub struct App {
    push_swap_path: Option<PathBuf>,
    file_dialog: FileDialog,
}

impl App {
    pub fn new(_cc: &eframe::CreationContext) -> Self {
        Self {
            push_swap_path: None,
            file_dialog: FileDialog::new(),
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
