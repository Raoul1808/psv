use std::{
    path::PathBuf,
    time::{Duration, Instant},
};

use eframe::egui;

use crate::{
    sim::Sim,
    visualizer::{
        controls::{Controls, PlayState},
        gpu::GpuVisualizer,
        loader::Loader,
    },
};

mod controls;
mod exec;
mod gpu;
mod loader;

pub struct Visualizer {
    gpu_view: GpuVisualizer,
    controls: Controls,
    loader: Loader,
    sim: Sim,
    need_redraw: bool,
    last_instant: Instant,
    exec_interval: Duration,
    duration_accumulated: Duration,
}

impl Visualizer {
    pub fn new<'a>(cc: &'a eframe::CreationContext<'a>) -> Self {
        let gpu_view = GpuVisualizer::new(cc);
        let sim = Sim::new(&[], vec![]).unwrap();
        Self {
            gpu_view,
            sim,
            controls: Controls::new(),
            loader: Loader::new(),
            need_redraw: true,
            last_instant: Instant::now(),
            exec_interval: Duration::from_secs_f64(1. / 60.),
            duration_accumulated: Duration::default(),
        }
    }

    pub fn update_path(&mut self, path: PathBuf) {
        self.loader.set_path(path);
    }

    // PERF: This could definitely be optimized
    fn play_sim(&mut self, ui: &mut egui::Ui) {
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

    fn load_sim_if_instructions_loaded(&mut self) {
        if self.loader.instructions_ready() {
            let instructions = self.loader.take_instructions().unwrap();
            let numbers = self.loader.numbers();
            match Sim::new(numbers, instructions) {
                Ok(s) => {
                    self.sim = s;
                    self.gpu_view.update(&self.sim);
                }
                Err(e) => {
                    eprintln!("{e}");
                }
            }
        }
    }

    pub fn ui(&mut self, ui: &mut egui::Ui) {
        self.load_sim_if_instructions_loaded();

        egui::Panel::bottom("visualizer_progress")
            .resizable(false)
            .show(ui, |ui| {
                self.controls.ui(ui, &mut self.sim);
                self.need_redraw |= self.controls.need_redraw();
            });
        egui::Panel::left("visualizer_load_options")
            .resizable(false)
            .min_size(250.)
            .show(ui, |ui| {
                self.loader.ui(ui);
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
