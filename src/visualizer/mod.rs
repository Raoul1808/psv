use std::{
    path::PathBuf,
    time::{Duration, Instant},
};

use eframe::egui;

use crate::{
    config::{ColorProfile, Config, VisualizerSideViewMode},
    sim::Sim,
    visualizer::{
        controls::Controls, details::Details, gpu::GpuVisualizer, loader::Loader, visuals::Visuals,
    },
};

mod controls;
mod details;
mod exec;
mod gpu;
mod loader;
mod visuals;

pub use controls::PlayState;

pub struct Visualizer {
    gpu_view: GpuVisualizer,
    controls: Controls,
    loader: Loader,
    visuals: Visuals,
    details: Details,
    sim: Sim,
    need_redraw: bool,
    last_instant: Instant,
    duration_accumulated: Duration,
    side_view_mode: VisualizerSideViewMode,
}

impl Visualizer {
    pub fn new<'a>(cc: &'a eframe::CreationContext<'a>, config: &Config) -> Self {
        let gpu_view = GpuVisualizer::new(cc);
        let sim = Sim::default();
        Self {
            gpu_view,
            sim,
            controls: Controls::new(),
            loader: Loader::new(),
            visuals: Visuals::new(),
            details: Details::new(),
            need_redraw: true,
            last_instant: Instant::now(),
            duration_accumulated: Duration::default(),
            side_view_mode: config.preferred_side_panel,
        }
    }

    pub fn update_path(&mut self, path: PathBuf) {
        self.loader.set_path(path);
    }

    // PERF: This could definitely be optimized
    fn play_sim(&mut self, ui: &mut egui::Ui) {
        match self.controls.play_state() {
            PlayState::PlayingForward => {
                let exec_interval = self.controls.playback_speed();
                let current_instant = Instant::now();
                let catching_duration = current_instant.duration_since(self.last_instant);
                while self.duration_accumulated <= catching_duration {
                    self.sim.step();
                    self.need_redraw = true;
                    self.duration_accumulated += exec_interval;
                    self.details.mark_follow_this_frame();
                }
                self.duration_accumulated -= catching_duration;
                if self.sim.program_counter() == self.sim.instructions().iter().len() {
                    self.controls.stop();
                }
                ui.request_repaint();
            }
            PlayState::PlayingBackward => {
                let exec_interval = self.controls.playback_speed();
                let current_instant = Instant::now();
                let catching_duration = current_instant.duration_since(self.last_instant);
                while self.duration_accumulated <= catching_duration {
                    self.sim.undo();
                    self.need_redraw = true;
                    self.duration_accumulated += exec_interval;
                    self.details.mark_follow_this_frame();
                }
                self.duration_accumulated -= catching_duration;
                if self.sim.program_counter() == 0 {
                    self.controls.stop();
                }
                ui.request_repaint();
            }
            _ => {
                self.duration_accumulated = Duration::ZERO;
            }
        }
        self.last_instant = Instant::now();
    }

    fn load_sim_if_instructions_loaded(&mut self, color_profile: &ColorProfile) {
        if self.loader.instructions_ready() {
            let instructions = self.loader.take_instructions().unwrap();
            let numbers = self.loader.numbers();
            match Sim::new(numbers, instructions) {
                Ok(s) => {
                    self.sim = s;
                    self.gpu_view.update(
                        &self.sim,
                        &color_profile.colors,
                        color_profile.clear_color,
                    );
                }
                Err(e) => {
                    log::error!("failed to create sim: {e}");
                }
            }
        }
    }

    pub fn should_lock_app_mode(&self) -> bool {
        self.loader.is_working()
    }

    pub fn ui(&mut self, ui: &mut egui::Ui, config: &mut Config) {
        self.load_sim_if_instructions_loaded(config.get_current_profile());

        egui::Panel::bottom("visualizer_progress")
            .resizable(false)
            .show(ui, |ui| {
                self.controls.ui(ui, &mut self.sim);
                if self.controls.need_redraw() {
                    self.need_redraw = true;
                    self.details.mark_follow_this_frame();
                }
            });
        egui::Panel::left("visualizer_load_options")
            .resizable(false)
            .min_size(250.)
            .show(ui, |ui| {
                self.loader.ui(ui);
            });
        match &self.side_view_mode {
            VisualizerSideViewMode::Visuals => {
                egui::Panel::right("visualizer_visuals")
                    .resizable(false)
                    .show(ui, |ui| {
                        self.visuals.ui(ui, config);
                        self.need_redraw |= self.visuals.need_redraw();
                    });
            }
            VisualizerSideViewMode::Details => {
                egui::Panel::right("visualizer_details")
                    .resizable(false)
                    .show(ui, |ui| {
                        self.details.ui(ui, &self.sim);
                        if self.details.can_skip() {
                            let skip = self.details.skip_to().unwrap_or_default();
                            self.sim.skip_to(skip);
                            self.controls.stop();
                            self.need_redraw = true;
                        }
                    });
            }
            _ => {}
        }
        self.play_sim(ui);
        egui::CentralPanel::default().show(ui, |ui| {
            if self.need_redraw {
                self.sim.make_contiguous();
                let prof = config.get_current_profile();
                self.gpu_view
                    .update(&self.sim, &prof.colors, prof.clear_color);
            }
            self.gpu_view.paint(ui);
        });
        self.need_redraw = false;
    }

    pub fn menu_ui(&mut self, ui: &mut egui::Ui) {
        ui.menu_button("Visualizer", |ui| {
            if ui.button("Reset").clicked() {
                self.loader.clear();
                self.gpu_view.clear();
                self.controls.clear();
                self.sim = Sim::default();
            }
            ui.separator();
            let mut view = None;
            if ui
                .button(format!(
                    "Visuals {}",
                    if self.side_view_mode == VisualizerSideViewMode::Visuals {
                        "✔️"
                    } else {
                        ""
                    }
                ))
                .clicked()
            {
                view = if self.side_view_mode == VisualizerSideViewMode::Visuals {
                    Some(VisualizerSideViewMode::None)
                } else {
                    Some(VisualizerSideViewMode::Visuals)
                };
            }
            if ui
                .button(format!(
                    "Details {}",
                    if self.side_view_mode == VisualizerSideViewMode::Details {
                        "✔️"
                    } else {
                        ""
                    }
                ))
                .clicked()
            {
                view = if self.side_view_mode == VisualizerSideViewMode::Details {
                    Some(VisualizerSideViewMode::None)
                } else {
                    Some(VisualizerSideViewMode::Details)
                };
            }
            if let Some(view) = view {
                self.side_view_mode = view;
            }
        });
    }
}
