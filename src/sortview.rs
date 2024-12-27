use std::cmp::Ordering;

use cgmath::{ortho, Matrix4, SquareMatrix};
use egui::Widget;

use crate::vertex::{Vertex, VertexIndexPair};

pub struct SortView {
    tris_data: Option<VertexIndexPair>,
    projection: Matrix4<f32>,
    num_range: u32,
}

impl SortView {
    pub fn new() -> Self {
        Self {
            tris_data: None,
            projection: Matrix4::identity(),
            num_range: 10,
        }
    }

    pub fn get_tris_data(&mut self) -> Option<VertexIndexPair> {
        self.tris_data.take()
    }

    pub fn get_projection_matrix(&self) -> Matrix4<f32> {
        self.projection
    }

    pub fn generate(&mut self) {
        let mut vertices = vec![];
        let mut indices = vec![];
        let mut next_index = 0;
        for i in 0..self.num_range {
            let i = i as f32;
            let half_range = self.num_range as f32 / 2.;
            let color = match i.partial_cmp(&half_range) {
                Some(Ordering::Equal) => [1.0, 1.0, 0.0],
                Some(Ordering::Less) => {
                    let half = self.num_range as f32 / 2.;
                    [1.0, i / half, 0.0]
                }
                Some(Ordering::Greater) | None => {
                    let half = self.num_range as f32 / 2.;
                    [1. - (i - half) / half, 1.0, 0.0]
                }
            };
            vertices.push(Vertex {
                position: [0.0, i, 0.0],
                color,
            });
            vertices.push(Vertex {
                position: [i + 1.0, i, 0.0],
                color,
            });
            vertices.push(Vertex {
                position: [i + 1.0, i + 1.0, 0.0],
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
        self.projection = ortho(
            0.,
            self.num_range as f32,
            self.num_range as f32,
            0.,
            -1.,
            1.,
        );
        self.tris_data = Some(VertexIndexPair { vertices, indices });
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
                    self.generate();
                }
            });
    }
}
