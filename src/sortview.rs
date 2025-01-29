use std::time::{Duration, Instant};

use cgmath::{Matrix4, SquareMatrix};

use crate::{
    gui::{LoadingOptions, PlaybackControls, VisualOptions},
    sim::PushSwapSim,
    vertex::{Vertex, VertexIndexPair},
};

pub struct SortView {
    projection: Matrix4<f32>,
    regenerate_render_data: bool,
    show_visual: bool,
    visual: VisualOptions,
    show_load: bool,
    load: LoadingOptions,
    show_playback: bool,
    playback: PlaybackControls,
    sim: PushSwapSim,
    playing_sim: bool,
    last_instant: Instant,
    exec_interval: Duration,
    duration_accumulated: Duration,
    scale_factor: f32,
}

impl SortView {
    pub fn new() -> Self {
        Self {
            projection: Matrix4::identity(),
            regenerate_render_data: false,
            show_visual: false,
            visual: VisualOptions::default(),
            show_load: true,
            load: LoadingOptions::default(),
            show_playback: false,
            playback: PlaybackControls::default(),
            sim: Default::default(),
            playing_sim: false,
            last_instant: Instant::now(),
            exec_interval: Duration::from_secs_f64(1. / 60.),
            duration_accumulated: Duration::ZERO,
            scale_factor: 1.0,
        }
    }

    pub fn scale_factor(&self) -> f32 {
        self.scale_factor
    }

    pub fn get_tris_data(&mut self) -> Option<VertexIndexPair> {
        if self.regenerate_render_data {
            self.regenerate_render_data = false;
            self.sim.make_contiguous();
            let stack_a = self.sim.stack_a();
            let stack_b = self.sim.stack_b();
            let num_range = stack_a.len() as u32 + stack_b.len() as u32;
            let mut data = self.generate_tris_data(num_range, stack_a, false);
            data.extend(self.generate_tris_data(num_range, stack_b, true));
            Some(data)
        } else {
            None
        }
    }

    pub fn get_projection_matrix(&self) -> Matrix4<f32> {
        self.projection
    }

    pub fn clear_color(&self) -> [f32; 3] {
        self.visual.clear_color()
    }

    pub fn keyboard_input(&mut self, event: &winit::event::KeyEvent) {
        use winit::keyboard::{KeyCode, PhysicalKey::Code};
        if !event.state.is_pressed() {
            return;
        }
        match event.physical_key {
            Code(KeyCode::Space) => {
                self.playing_sim = if self.sim.program_counter() == self.sim.instructions().len() {
                    self.sim.skip_to(0);
                    true
                } else {
                    !self.playing_sim
                };
            }
            Code(KeyCode::ArrowLeft) => {
                if !self.playing_sim {
                    self.regenerate_render_data = self.sim.undo();
                }
            }
            Code(KeyCode::ArrowRight) => {
                if !self.playing_sim {
                    self.regenerate_render_data = self.sim.step();
                }
            }
            _ => {}
        }
    }

    fn generate_tris_data(&self, num_range: u32, stack: &[u32], offset: bool) -> VertexIndexPair {
        let mut vertices = vec![];
        let mut indices = vec![];
        let mut next_index = 0;
        for (i, num) in stack.iter().enumerate() {
            let i = i as f32;
            let num = (*num) as f32;
            let o = if offset { num_range as f32 } else { 0. };
            let t = num / num_range as f32;
            let color = self.visual.color_at(t);
            vertices.push(Vertex {
                position: [0.0 + o, i, 0.0],
                color,
            });
            vertices.push(Vertex {
                position: [num + 1.0 + o, i, 0.0],
                color,
            });
            vertices.push(Vertex {
                position: [num + 1.0 + o, i + 1.0, 0.0],
                color,
            });
            vertices.push(Vertex {
                position: [0.0 + o, i + 1.0, 0.0],
                color,
            });
            indices.extend_from_slice(&[
                next_index,
                next_index + 1,
                next_index + 2,
                next_index + 2,
                next_index + 3,
                next_index,
            ]);
            next_index += 4;
        }
        VertexIndexPair { vertices, indices }
    }

    pub fn egui_menu(&mut self, ui: &egui::Context) {
        ui.style_mut(|ui| {
            ui.visuals.window_fill =
                egui::Color32::from_rgba_unmultiplied(0x1b, 0x1b, 0x1b, self.visual.opacity());
        });
        egui::Window::new("push_swap visualizer")
            .resizable(true)
            .movable(true)
            .collapsible(true)
            .show(ui, |ui| {
                ui.checkbox(&mut self.show_visual, "Show Visual Options Window").on_hover_text("Shows a floating window with visual-related options such as changing used colors and changing the transparency of the GUI windows.");
                ui.checkbox(&mut self.show_load, "Show Loading Options Window").on_hover_text("Shows a floating window to generate a sequence of numbers and load push_swap instructions.");
                ui.checkbox(&mut self.show_playback, "Show Playback Controls").on_hover_text("Shows a floating window with playback controls and a table of running instructions.");
                ui.small(format!("Running psv v{}", env!("CARGO_PKG_VERSION")));
            });
        self.visual
            .ui(ui, &mut self.show_visual, &mut self.scale_factor);
        self.load.ui(
            ui,
            &mut self.show_load,
            &mut self.sim,
            &mut self.regenerate_render_data,
            &mut self.projection,
            &mut self.playing_sim,
            &mut self.show_playback,
        );
        let mut temp_stop = false;
        self.playback.ui(
            ui,
            &mut self.show_playback,
            &mut self.sim,
            &mut self.playing_sim,
            &mut temp_stop,
            &mut self.exec_interval,
            &mut self.regenerate_render_data,
        );
        if self.playing_sim && !temp_stop {
            let current_instant = Instant::now();
            let catching_duration = current_instant.duration_since(self.last_instant);
            while self.duration_accumulated <= catching_duration {
                self.sim.step();
                self.regenerate_render_data = true;
                self.duration_accumulated += self.exec_interval;
            }
            self.duration_accumulated -= catching_duration;
        } else {
            self.duration_accumulated = Duration::ZERO;
        }
        self.last_instant = Instant::now();
    }
}
