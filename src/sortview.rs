use std::cmp::Ordering;

use cgmath::{ortho, Matrix4, SquareMatrix};
use egui::Widget;
use rand::{seq::SliceRandom, thread_rng};

use crate::vertex::{Vertex, VertexIndexPair};

pub struct SortView {
    projection: Matrix4<f32>,
    stack_a: Vec<u32>,
    stack_b: Vec<u32>,
    num_range: u32,
    regenerate_render_data: bool,
}

impl SortView {
    pub fn new() -> Self {
        Self {
            projection: Matrix4::identity(),
            stack_a: vec![],
            stack_b: vec![],
            num_range: 10,
            regenerate_render_data: false,
        }
    }

    pub fn get_tris_data(&mut self) -> Option<VertexIndexPair> {
        if self.regenerate_render_data {
            self.regenerate_render_data = false;
            let mut data = self.generate_tris_data(self.stack_a.as_slice());
            data.extend(self.generate_tris_data(self.stack_b.as_slice()));
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

    fn generate_tris_data(&self, stack: &[u32]) -> VertexIndexPair {
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
            vertices.push(Vertex {
                position: [0.0, i, 0.0],
                color,
            });
            vertices.push(Vertex {
                position: [num + 1.0, i, 0.0],
                color,
            });
            vertices.push(Vertex {
                position: [num + 1.0, i + 1.0, 0.0],
                color,
            });
            vertices.push(Vertex {
                position: [0.0, i + 1.0, 0.0],
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
        egui::Window::new("push_swap visualizer")
            .resizable(true)
            .movable(true)
            .collapsible(true)
            .show(ui, |ui| {
                egui::DragValue::new(&mut self.num_range)
                    .range(10..=1000)
                    .ui(ui);
                if ui.button("Generate").clicked() {
                    self.stack_a = (0..self.num_range).collect();
                    self.update_projection();
                    self.regenerate_render_data = true;
                }
                if ui.button("Shuffle").clicked() {
                    self.stack_a.shuffle(&mut thread_rng());
                    self.regenerate_render_data = true;
                }
            });
    }
}
