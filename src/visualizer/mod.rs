use eframe::egui;

use crate::{
    sim::{PushSwapInstruction, Sim},
    visualizer::{controls::Controls, gpu::GpuVisualizer},
};

mod controls;
mod gpu;

pub struct Visualizer {
    gpu_view: GpuVisualizer,
    controls: Controls,
    sim: Sim,
    need_redraw: bool,
}

impl Visualizer {
    pub fn new<'a>(cc: &'a eframe::CreationContext<'a>) -> Self {
        let gpu_view = GpuVisualizer::new(cc);
        let nums: Vec<_> = (0..100).collect();
        let sim = Sim::new(nums.as_slice(), [PushSwapInstruction::PushB].repeat(99)).unwrap();
        Self {
            gpu_view,
            sim,
            controls: Controls::new(),
            need_redraw: true,
        }
    }

    pub fn ui(&mut self, ui: &mut egui::Ui) {
        egui::Panel::bottom("visualizer_progress")
            .resizable(false)
            .show(ui, |ui| {
                self.controls.ui(ui, &mut self.sim);
                self.need_redraw |= self.controls.need_redraw();
            });
        egui::Panel::left("visualizer_load_options")
            .resizable(false)
            .show(ui, |ui| {
                ui.label("load options");
            });
        egui::Panel::right("visualizer_details")
            .resizable(false)
            .show(ui, |ui| {
                ui.label("details");
            });
        egui::CentralPanel::default().show(ui, |ui| {
            if self.need_redraw {
                self.sim.make_contiguous();
                self.gpu_view.update(&self.sim);
            }
            self.gpu_view.paint(ui);
        });
        self.need_redraw = false;
    }
}
