use std::{borrow::Cow, sync::Arc};

use wgpu::util::DeviceExt;
use winit::window::Window;

use crate::vertex::{Vertex, VERTICES};

pub struct WgpuRenderPass {
    pub surface_texture: wgpu::SurfaceTexture,
    pub surface_view: wgpu::TextureView,
    pub encoder: wgpu::CommandEncoder,
}

pub struct WgpuContext<'a> {
    pub instance: wgpu::Instance,
    pub surface: wgpu::Surface<'a>,
    pub surface_config: wgpu::SurfaceConfiguration,
    pub adapter: wgpu::Adapter,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub render_pipeline: wgpu::RenderPipeline,
    pub vertex_buffer: wgpu::Buffer,
    pub num_vertices: u32,
    pub clear_color: wgpu::Color,
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
            .request_device(
                &wgpu::DeviceDescriptor {
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::default(),
                    ..Default::default()
                },
                None,
            )
            .await
            .expect("no device found");

        let size = window.inner_size();
        let width = size.width.max(1);
        let height = size.height.max(1);
        let surface_config = surface
            .get_default_config(&adapter, width, height)
            .expect("no surface config");
        surface.configure(&device, &surface_config);

        let shader_str = include_str!("shader.wgsl");
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(shader_str)),
            label: None,
        });
        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: None,
            layout: None,
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
                targets: &[Some(surface_config.format.into())],
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

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(VERTICES),
            usage: wgpu::BufferUsages::VERTEX,
        });
        let num_vertices = VERTICES.len() as u32;

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
            num_vertices,
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

    pub fn update_vertex_buffer(&mut self, vertices: &[Vertex]) {
        self.vertex_buffer.unmap();
        self.vertex_buffer = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: None,
                contents: bytemuck::cast_slice(vertices),
                usage: wgpu::BufferUsages::VERTEX,
            });
        self.num_vertices = vertices.len() as u32;
    }

    pub fn update_clear_color(&mut self, color: impl Into<wgpu::Color>) {
        self.clear_color = color.into();
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
            rpass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            rpass.draw(0..self.num_vertices, 0..1);
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
