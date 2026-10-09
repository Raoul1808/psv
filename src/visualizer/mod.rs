use eframe::egui;

use crate::visualizer::gpu::GpuVisualizer;

mod gpu;

pub struct Visualizer {
    gpu_view: GpuVisualizer,
}

impl Visualizer {
    pub fn new<'a>(cc: &'a eframe::CreationContext<'a>) -> Self {
        let gpu_view = GpuVisualizer::new(cc);
        Self { gpu_view }
    }

    pub fn ui(&mut self, ui: &mut egui::Ui) {
        egui::CentralPanel::default().show(ui, |ui| {
            egui::Panel::bottom("visualizer_progress").show(ui, |ui| {
                ui.label("progress bar");
            });
            egui::Panel::left("visualizer_load_options").show(ui, |ui| {
                ui.label("load options");
            });
            egui::Panel::right("visualizer_details").show(ui, |ui| {
                ui.label("details");
            });
            self.gpu_view.paint(ui);
        });
    }
}
