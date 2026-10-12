use std::path::PathBuf;

use eframe::egui::{self, Widget};

pub struct Loader {
    push_swap_path: Option<PathBuf>,
}

impl Loader {
    pub fn new() -> Self {
        Self {
            push_swap_path: None,
        }
    }

    pub fn set_path(&mut self, path: PathBuf) {
        self.push_swap_path = Some(path);
    }

    pub fn ui(&mut self, ui: &mut egui::Ui) {
        let path_str = self
            .push_swap_path
            .as_ref()
            .map(|p| p.display().to_string())
            .unwrap_or("None".into());
        egui::Label::new(format!("Executable path: {}", path_str))
            .truncate()
            .ui(ui);
    }
}
