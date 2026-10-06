use eframe::egui;

pub struct App {}

impl App {
    pub fn new(_cc: &eframe::CreationContext) -> Self {
        Self {}
    }
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut eframe::egui::Ui, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ui, |ui| {
            ui.heading("hell word");
        });
    }
}
