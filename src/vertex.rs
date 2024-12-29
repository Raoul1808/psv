#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Vertex {
    pub position: [f32; 3],
    pub color: [f32; 3],
}

unsafe impl bytemuck::Zeroable for Vertex {}
unsafe impl bytemuck::Pod for Vertex {}

pub const VERTICES: &[Vertex] = &[
    Vertex {
        position: [0.0, 0.5, 0.0],
        color: [1.0, 0.0, 0.0],
    },
    Vertex {
        position: [-0.5, -0.5, 0.0],
        color: [0.0, 1.0, 0.0],
    },
    Vertex {
        position: [0.5, -0.5, 0.0],
        color: [0.0, 0.0, 1.0],
    },
];

pub const INDICES: &[u32] = &[0, 1, 2];

impl Vertex {
    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x3,
                },
            ],
        }
    }
}

#[derive(Debug)]
pub struct VertexIndexPair {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
}

impl VertexIndexPair {
    pub fn extend(&mut self, other: VertexIndexPair) {
        if other.vertices.is_empty() || other.indices.is_empty() {
            return;
        }
        self.vertices.extend(other.vertices);
        if !self.indices.is_empty() {
            let index_offset = self.indices[self.indices.len() - 2] + 1;
            let other_indices = other.indices.iter().map(|i| i + index_offset);
            self.indices.extend(other_indices);
        } else {
            self.indices.extend(other.indices);
        }
    }
}
