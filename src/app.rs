use std::sync::Arc;

use egui_wgpu::ScreenDescriptor;
use pollster::FutureExt;
use winit::{
    application::ApplicationHandler,
    event::{KeyEvent, WindowEvent},
    event_loop::ActiveEventLoop,
    keyboard::Key,
    window::{Window, WindowId},
};

use crate::{egui_tools::EguiRenderer, gpu::WgpuContext};

#[derive(Default)]
pub struct App<'a> {
    window: Option<Arc<Window>>,
    wgpu_ctx: Option<WgpuContext<'a>>,
    egui_renderer: Option<EguiRenderer>,
}

impl App<'_> {
    fn handle_input(&mut self, _event: &KeyEvent) {}

    fn handle_redraw(&mut self) {
        let egui_renderer = self.egui_renderer.as_mut().expect("no egui renderer");
        let wgpu_ctx = self.wgpu_ctx.as_mut().expect("no wgpu context");
        let window = self.window.as_ref().expect("no window");

        wgpu_ctx.begin_render_pass();

        let screen_descriptor = ScreenDescriptor {
            size_in_pixels: [
                wgpu_ctx.surface_config.width,
                wgpu_ctx.surface_config.height,
            ],
            pixels_per_point: window.scale_factor() as f32,
        };

        // TODO: Rename this (this is NOT *A* render pass)
        let render_pass = wgpu_ctx.render_pass.as_mut().expect("not rendering");
        {
            egui_renderer.begin_frame(window);

            egui::Window::new("egui integration test")
                .resizable(true)
                .vscroll(true)
                .show(egui_renderer.context(), |ui| {
                    ui.label("Hello, world!");
                });

            egui_renderer.end_frame_and_draw(
                &wgpu_ctx.device,
                &wgpu_ctx.queue,
                &mut render_pass.encoder,
                window,
                &render_pass.surface_view,
                screen_descriptor,
            );
        }

        wgpu_ctx.submit_render_passes();
    }
}

impl ApplicationHandler for App<'_> {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        if self.window.is_none() {
            let attr = Window::default_attributes().with_title("push_swap visualizer");
            let window = Arc::new(event_loop.create_window(attr).expect("no window"));
            let wgpu_ctx = WgpuContext::new(window.clone()).block_on();
            let egui_renderer = EguiRenderer::new(
                &wgpu_ctx.device,
                wgpu_ctx.surface_config.format,
                None,
                1,
                &window,
            );
            self.window = Some(window.clone());
            self.wgpu_ctx = Some(wgpu_ctx);
            self.egui_renderer = Some(egui_renderer);
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        self.egui_renderer
            .as_mut()
            .expect("no egui renderer")
            .handle_input(self.window.as_ref().unwrap(), &event);
        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::KeyboardInput { event, .. } => {
                if event.state.is_pressed() {
                    if let Key::Named(winit::keyboard::NamedKey::Escape) =
                        event.logical_key.as_ref()
                    {
                        event_loop.exit();
                    }
                }
                self.handle_input(&event);
            }
            WindowEvent::Resized(new_size) => {
                if let Some(wgpu_ctx) = self.wgpu_ctx.as_mut() {
                    wgpu_ctx.resize(new_size.into());
                }
            }
            WindowEvent::RedrawRequested => {
                self.handle_redraw();
            }
            _ => {}
        }
    }
}
