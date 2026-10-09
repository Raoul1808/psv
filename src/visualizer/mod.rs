use eframe::egui;

use crate::{
    sim::{PushSwapInstruction, Sim},
    visualizer::gpu::GpuVisualizer,
};

mod gpu;

pub struct Visualizer {
    gpu_view: GpuVisualizer,
    sim: Sim,
}

impl Visualizer {
    pub fn new<'a>(cc: &'a eframe::CreationContext<'a>) -> Self {
        let gpu_view = GpuVisualizer::new(cc);
        let nums: Vec<_> = (0..100).collect();
        let mut sim = Sim::new(nums.as_slice(), [PushSwapInstruction::PushB].repeat(40)).unwrap();
        while sim.step() {}
        sim.make_contiguous();
        Self { gpu_view, sim }
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
            self.gpu_view.update(&self.sim);
            self.gpu_view.paint(ui);
        });
    }
}
