use bytemuck::offset_of;
use eframe::{
    egui, egui_wgpu,
    wgpu::{self, VertexAttribute, util::DeviceExt},
};

use crate::{gradient::Gradient, sim::Sim};

#[repr(C)]
#[derive(Debug, Default, Clone, Copy)]
pub struct Vertex {
    pub position: [f32; 3],
    pub color: [f32; 4],
}

impl Vertex {
    pub fn array_stride() -> wgpu::BufferAddress {
        size_of::<Vertex>() as _
    }

    pub fn step_mode() -> wgpu::VertexStepMode {
        wgpu::VertexStepMode::Vertex
    }

    pub fn vertex_attributes() -> [VertexAttribute; 2] {
        [
            wgpu::VertexAttribute {
                offset: offset_of!(Vertex, position) as wgpu::BufferAddress,
                shader_location: 0,
                format: wgpu::VertexFormat::Float32x3,
            },
            wgpu::VertexAttribute {
                offset: offset_of!(Vertex, color) as wgpu::BufferAddress,
                shader_location: 1,
                format: wgpu::VertexFormat::Float32x4,
            },
        ]
    }
}

unsafe impl bytemuck::Zeroable for Vertex {}
unsafe impl bytemuck::Pod for Vertex {}

pub const VERTICES: &[Vertex] = &[
    Vertex {
        position: [-2., -2., -2.],
        color: [0., 0., 0., 1.],
    },
    Vertex {
        position: [-2., -3., -2.],
        color: [0., 0., 0., 1.],
    },
    Vertex {
        position: [-3., -3., -2.],
        color: [0., 0., 0., 1.],
    },
];
pub const INDICES: &[u32] = &[0, 1, 2];

struct VisualizerRenderResources {
    pipeline: wgpu::RenderPipeline,
    bind_group: wgpu::BindGroup,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    num_indices: u32,
    uniform_buffer: wgpu::Buffer,
}

impl VisualizerRenderResources {
    fn prepare(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        vertices: Option<&[Vertex]>,
        indices: Option<&[u32]>,
        mat: [[f32; 4]; 4],
    ) {
        if let Some(vertices) = vertices {
            self.vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("visualizer vertices"),
                contents: bytemuck::cast_slice(vertices),
                usage: wgpu::BufferUsages::VERTEX,
            });
        }
        if let Some(indices) = indices {
            self.index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("visualizer indices"),
                contents: bytemuck::cast_slice(indices),
                usage: wgpu::BufferUsages::INDEX,
            });
            self.num_indices = indices.len() as u32;
        }
        queue.write_buffer(
            &self.uniform_buffer,
            0,
            bytemuck::cast_slice(mat.as_slice()),
        );
    }

    fn paint(&self, render_pass: &mut wgpu::RenderPass<'_>) {
        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, &self.bind_group, &[]);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
        render_pass.draw_indexed(0..self.num_indices, 0, 0..1);
    }
}

struct VisualizerCallback {
    vertices: Option<Vec<Vertex>>,
    indices: Option<Vec<u32>>,
    mat: [[f32; 4]; 4],
}

impl egui_wgpu::CallbackTrait for VisualizerCallback {
    fn prepare(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        _screen_descriptor: &egui_wgpu::ScreenDescriptor,
        _egui_encoder: &mut wgpu::CommandEncoder,
        callback_resources: &mut egui_wgpu::CallbackResources,
    ) -> Vec<wgpu::CommandBuffer> {
        let resources: &mut VisualizerRenderResources = callback_resources.get_mut().unwrap();
        resources.prepare(
            device,
            queue,
            self.vertices.as_deref(),
            self.indices.as_deref(),
            self.mat,
        );
        Vec::new()
    }

    fn paint(
        &self,
        _info: eframe::egui::PaintCallbackInfo,
        render_pass: &mut wgpu::RenderPass<'static>,
        callback_resources: &egui_wgpu::CallbackResources,
    ) {
        let resources: &VisualizerRenderResources = callback_resources.get().unwrap();
        resources.paint(render_pass);
    }
}

pub struct GpuVisualizer {
    mat: [[f32; 4]; 4],
    vertices: Option<Vec<Vertex>>,
    indices: Option<Vec<u32>>,
    rendering: bool,
    render_once: bool,
}

impl GpuVisualizer {
    pub fn new<'a>(cc: &eframe::CreationContext<'a>) -> Self {
        let wgpu_render_state = cc.wgpu_render_state.as_ref().expect("no render state??");

        let device = &wgpu_render_state.device;

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("visualizer"),
            source: wgpu::ShaderSource::Wgsl(include_str!("./shader.wgsl").into()),
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("visualizer"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("visualizer"),
            bind_group_layouts: &[Some(&bind_group_layout)],
            immediate_size: 0,
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("visualizer"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: None,
                buffers: &[Some(wgpu::VertexBufferLayout {
                    array_stride: Vertex::array_stride(),
                    step_mode: Vertex::step_mode(),
                    attributes: Vertex::vertex_attributes().as_slice(),
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: wgpu_render_state.target_format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::all(),
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        });

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("visualizer vertices"),
            contents: bytemuck::cast_slice(VERTICES),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("visualizer indices"),
            contents: bytemuck::cast_slice(INDICES),
            usage: wgpu::BufferUsages::INDEX,
        });
        let num_indices = INDICES.len() as u32;

        let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("visualizer"),
            contents: bytemuck::cast_slice(&[0.0_f32; 16]),
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::UNIFORM,
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("visualizer"),
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
        });

        wgpu_render_state
            .renderer
            .write()
            .callback_resources
            .insert(VisualizerRenderResources {
                pipeline,
                bind_group,
                vertex_buffer,
                index_buffer,
                num_indices,
                uniform_buffer,
            });
        Self {
            mat: ortho(0., 1., 1., 0., -1., 1.),
            vertices: None,
            indices: None,
            rendering: false,
            render_once: true,
        }
    }

    pub fn update(&mut self, sim: &Sim, color: &Gradient, back_color: [f32; 4]) {
        let (stack_a, stack_b) = sim.as_slices();
        let len = stack_a.len() + stack_b.len();
        self.mat = ortho(0., len as f32 * 2., len as f32, 0., -1., 1.);

        let mut vertices = vec![];
        let mut indices = vec![];

        {
            let x = len as f32 * 2.;
            let y = len as f32;
            let z = -0.5;
            vertices.push(Vertex {
                position: [0.0, 0.0, z],
                color: back_color,
            });
            vertices.push(Vertex {
                position: [x, 0.0, z],
                color: back_color,
            });
            vertices.push(Vertex {
                position: [x, y, z],
                color: back_color,
            });
            vertices.push(Vertex {
                position: [0.0, y, z],
                color: back_color,
            });
            indices.extend_from_slice(&[0, 1, 2, 2, 3, 0]);
        }

        let mut next_index = 4;
        for (i, num) in stack_a.iter().enumerate() {
            let i = i as f32;
            let num = num.normalized as f32;
            let t = num / (len as f32 - 1.0);
            let color = color.color_at(t);
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
        for (i, num) in stack_b.iter().enumerate() {
            let len = len as f32;
            let i = i as f32;
            let num = num.normalized as f32;
            let t = num / (len - 1.);
            let color = color.color_at(t);
            vertices.push(Vertex {
                position: [len, i, 0.0],
                color,
            });
            vertices.push(Vertex {
                position: [len + num + 1.0, i, 0.0],
                color,
            });
            vertices.push(Vertex {
                position: [len + num + 1.0, i + 1.0, 0.0],
                color,
            });
            vertices.push(Vertex {
                position: [len, i + 1.0, 0.0],
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
        self.vertices = Some(vertices);
        self.indices = Some(indices);
        self.rendering = true;
    }

    pub fn clear(&mut self) {
        self.vertices = Some(VERTICES.to_vec());
        self.indices = Some(INDICES.to_vec());
        self.rendering = false;
        self.render_once = true;
    }

    pub fn paint(&mut self, ui: &mut egui::Ui) {
        if self.rendering {
            ui.request_repaint();
        }
        if self.render_once {
            self.render_once = false;
            ui.request_repaint();
        }
        let size = ui.available_size();
        let (rect, _response) = ui.allocate_exact_size(size, egui::Sense::drag());

        ui.painter().add(egui_wgpu::Callback::new_paint_callback(
            rect,
            VisualizerCallback {
                mat: self.mat,
                vertices: self.vertices.take(),
                indices: self.indices.take(),
            },
        ));
    }
}

fn ortho(left: f32, right: f32, bottom: f32, top: f32, near: f32, far: f32) -> [[f32; 4]; 4] {
    let tx = -(right + left) / (right - left);
    let ty = -(top + bottom) / (top - bottom);
    let tz = -(far + near) / (far - near);
    [
        [2. / (right - left), 0., 0., 0.],
        [0., 2. / (top - bottom), 0., 0.],
        [0., 0., -2. / (far - near), 0.],
        [tx, ty, tz, 1.],
    ]
}
