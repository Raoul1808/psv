use std::cmp::Ordering;

use cgmath::{ortho, Matrix4, SquareMatrix};
use egui::Widget;
use rand::{seq::SliceRandom, thread_rng};

use crate::{
    sim::PushSwapSim,
    vertex::{Vertex, VertexIndexPair},
};

pub struct SortView {
    projection: Matrix4<f32>,
    num_range: u32,
    regenerate_render_data: bool,
    sim: PushSwapSim,
    instructions_raw: String,
}

impl SortView {
    pub fn new() -> Self {
        Self {
            projection: Matrix4::identity(),
            num_range: 10,
            regenerate_render_data: false,
            sim: Default::default(),
            instructions_raw: String::new(),
        }
    }

    pub fn get_tris_data(&mut self) -> Option<VertexIndexPair> {
        if self.regenerate_render_data {
            self.regenerate_render_data = false;
            self.sim.make_contiguous();
            let stack_a = self.sim.stack_a();
            let stack_b = self.sim.stack_b();
            let mut data = self.generate_tris_data(stack_a, false);
            data.extend(self.generate_tris_data(stack_b, true));
            Some(data)
        } else {
            None
        }
    }

    pub fn get_projection_matrix(&self) -> Matrix4<f32> {
        self.projection
    }

    fn update_projection(&mut self) {
        self.projection = ortho(
            0.,
            self.num_range as f32 * 2.,
            self.num_range as f32,
            0.,
            -1.,
            1.,
        );
    }

    fn generate_tris_data(&self, stack: &[u32], offset: bool) -> VertexIndexPair {
        let mut vertices = vec![];
        let mut indices = vec![];
        let mut next_index = 0;
        for (i, num) in stack.iter().enumerate() {
            let i = i as f32;
            let num = (*num) as f32;
            let half_range = self.num_range as f32 / 2.;
            let color = match num.partial_cmp(&half_range) {
                Some(Ordering::Equal) => [1.0, 1.0, 0.0],
                Some(Ordering::Less) => {
                    let half = self.num_range as f32 / 2.;
                    [1.0, num / half, 0.0]
                }
                Some(Ordering::Greater) | None => {
                    let half = self.num_range as f32 / 2.;
                    [1. - (num - half) / half, 1.0, 0.0]
                }
            };
            let o = if offset { self.num_range as f32 } else { 0. };
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
        egui::Window::new("Visualization Loader")
            .resizable(true)
            .movable(true)
            .collapsible(true)
            .show(ui, |ui| {
                egui::DragValue::new(&mut self.num_range)
                    .range(10..=1000)
                    .ui(ui);
                ui.add(egui::TextEdit::multiline(&mut self.instructions_raw));
                if ui.button("Load Sorted").clicked() {
                    let numbers: Vec<_> = (0..self.num_range).collect();
                    let res = self.sim.load(&numbers, &self.instructions_raw);
                    if let Err(index) = res {
                        eprintln!("Loading error: instruction {} is not valid a valid push_swap instruction", index);
                    }
                    self.update_projection();
                    self.regenerate_render_data = true;
                }
                if ui.button("Load Random").clicked() {
                    let mut numbers: Vec<_> = (0..self.num_range).collect();
                    numbers.shuffle(&mut thread_rng());
                    let res = self.sim.load(&numbers, &self.instructions_raw);
                    if let Err(index) = res {
                        eprintln!("Loading error: instruction {} is not valid a valid push_swap instruction", index);
                    }
                    self.update_projection();
                    self.regenerate_render_data = true;
                }
            });
        if self.sim.ui(ui) {
            self.regenerate_render_data = true;
        }
    }
}
