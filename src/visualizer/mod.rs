use std::time::{Duration, Instant};

use eframe::egui;

use crate::{
    sim::{PushSwapInstruction, Sim},
    visualizer::{
        controls::{Controls, PlayState},
        gpu::GpuVisualizer,
    },
};

mod controls;
mod gpu;

pub struct Visualizer {
    gpu_view: GpuVisualizer,
    controls: Controls,
    sim: Sim,
    need_redraw: bool,
    last_instant: Instant,
    exec_interval: Duration,
    duration_accumulated: Duration,
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
            last_instant: Instant::now(),
            exec_interval: Duration::from_secs_f64(1. / 60.),
            duration_accumulated: Duration::default(),
        }
    }

    // PERF: This could definitely be optimized
    pub fn play_sim(&mut self, ui: &mut egui::Ui) {
        match self.controls.play_state() {
            PlayState::PlayingForward => {
                let current_instant = Instant::now();
                let catching_duration = current_instant.duration_since(self.last_instant);
                while self.duration_accumulated <= catching_duration {
                    self.sim.step();
                    self.need_redraw = true;
                    self.duration_accumulated += self.exec_interval;
                }
                self.duration_accumulated -= catching_duration;
                ui.request_repaint();
            }
            PlayState::PlayingBackward => {
                let current_instant = Instant::now();
                let catching_duration = current_instant.duration_since(self.last_instant);
                while self.duration_accumulated <= catching_duration {
                    self.sim.undo();
                    self.need_redraw = true;
                    self.duration_accumulated += self.exec_interval;
                }
                self.duration_accumulated -= catching_duration;
                ui.request_repaint();
            }
            _ => {
                self.duration_accumulated = Duration::ZERO;
            }
        }
        self.last_instant = Instant::now();
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
        self.play_sim(ui);
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
