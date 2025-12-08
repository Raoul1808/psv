use std::{borrow::Cow, sync::Arc};

use cgmath::SquareMatrix;
use wgpu::util::DeviceExt;
use winit::window::Window;

use crate::vertex::{INDICES, VERTICES, Vertex, VertexIndexPair};

pub struct WgpuRenderPass {
    pub surface_texture: wgpu::SurfaceTexture,
    pub surface_view: wgpu::TextureView,
    pub encoder: wgpu::CommandEncoder,
}

#[allow(dead_code)]
pub struct WgpuContext<'a> {
    instance: wgpu::Instance,
    surface: wgpu::Surface<'a>,
    pub surface_config: wgpu::SurfaceConfiguration,
    adapter: wgpu::Adapter,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    render_pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    camera_uniform: wgpu::Buffer,
    camera_bind_group: wgpu::BindGroup,
    num_indices: u32,
    clear_color: wgpu::Color,
    pub render_pass: Option<WgpuRenderPass>,
}

impl<'a> WgpuContext<'a> {
    pub async fn new(window: Arc<Window>) -> WgpuContext<'a> {
        let instance = wgpu::Instance::default();
        let surface = instance
            .create_surface(window.clone())
            .expect("no window surface");
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions::default())
            .await
            .expect("no graphics adapter found");
        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
                ..Default::default()
            })
            .await
            .expect("no device found");

        let size = window.inner_size();
        let width = size.width.max(1);
        let height = size.height.max(1);
        let swapchain_capabilities = surface.get_capabilities(&adapter);
        let swapchain_format = swapchain_capabilities
            .formats
            .iter()
            .find(|d| **d == wgpu::TextureFormat::Bgra8UnormSrgb)
            .expect("failed to select proper surface texture format");
        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: *swapchain_format,
            width,
            height,
            present_mode: wgpu::PresentMode::AutoVsync,
            desired_maximum_frame_latency: 0,
            alpha_mode: swapchain_capabilities.alpha_modes[0],
            view_formats: vec![],
        };
        surface.configure(&device, &surface_config);

        let shader_str = include_str!("shader.wgsl");
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(shader_str)),
            label: None,
        });

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(VERTICES),
            usage: wgpu::BufferUsages::VERTEX,
        });
        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(INDICES),
            usage: wgpu::BufferUsages::INDEX,
        });
        let num_indices = INDICES.len() as u32;

        let camera_data: [[f32; 4]; 4] = cgmath::Matrix4::<f32>::identity().into();
        let camera_uniform = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&[camera_data]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let camera_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: None,
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

        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &camera_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_uniform.as_entire_binding(),
            }],
            label: Some("camera binding group"),
        });

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[&camera_bind_group_layout],
                push_constant_ranges: &[],
            });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[Vertex::desc()],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                compilation_options: Default::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format: surface_config.format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        let clear_color = wgpu::Color {
            r: 0.1,
            g: 0.2,
            b: 0.3,
            a: 1.0,
        };

        Self {
            instance,
            surface,
            adapter,
            device,
            queue,
            surface_config,
            render_pipeline,
            vertex_buffer,
            index_buffer,
            camera_uniform,
            camera_bind_group,
            num_indices,
            clear_color,
            render_pass: None,
        }
    }

    pub fn surface_size(&self) -> (u32, u32) {
        let width = self.surface_config.width;
        let height = self.surface_config.height;
        (width, height)
    }

    pub fn resize(&mut self, new_size: (u32, u32)) {
        let (width, height) = new_size;
        self.surface_config.width = width.max(1);
        self.surface_config.height = height.max(1);
        self.surface.configure(&self.device, &self.surface_config);
    }

    pub fn update_buffers(&mut self, pair: VertexIndexPair) {
        let VertexIndexPair { vertices, indices } = pair;
        let (vertices, indices) = if vertices.is_empty() || indices.is_empty() {
            (VERTICES, INDICES)
        } else {
            (vertices.as_slice(), indices.as_slice())
        };
        self.vertex_buffer = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: None,
                contents: bytemuck::cast_slice(vertices),
                usage: wgpu::BufferUsages::VERTEX,
            });
        self.index_buffer = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: None,
                contents: bytemuck::cast_slice(indices),
                usage: wgpu::BufferUsages::INDEX,
            });
        self.num_indices = indices.len() as u32;
    }

    pub fn update_projection_matrix(&mut self, matrix: [[f32; 4]; 4]) {
        self.queue
            .write_buffer(&self.camera_uniform, 0, bytemuck::cast_slice(&[matrix]));
    }

    pub fn update_clear_color(&mut self, color: [f32; 3]) {
        self.clear_color = wgpu::Color {
            r: color[0] as f64,
            g: color[1] as f64,
            b: color[2] as f64,
            a: 1.0,
        };
    }

    pub fn begin_render_pass(&mut self) {
        if self.render_pass.is_some() {
            panic!("begin_render_pass called twice!");
        }

        let surface_texture = self
            .surface
            .get_current_texture()
            .expect("no current texture");
        let surface_view = surface_texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        {
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &surface_view,
                    resolve_target: None,
                    depth_slice: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(self.clear_color),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            rpass.set_pipeline(&self.render_pipeline);
            rpass.set_bind_group(0, &self.camera_bind_group, &[]);
            rpass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            rpass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
            rpass.draw_indexed(0..self.num_indices, 0, 0..1);
        }
        let render_pass = WgpuRenderPass {
            surface_texture,
            surface_view,
            encoder,
        };
        self.render_pass = Some(render_pass);
    }

    pub fn submit_render_passes(&mut self) {
        if self.render_pass.is_none() {
            panic!("submit_render_passes called before begin_render_pass!");
        }
        let WgpuRenderPass {
            surface_texture,
            encoder,
            ..
        } = self.render_pass.take().unwrap();
        self.queue.submit(Some(encoder.finish()));
        surface_texture.present();
    }
}
