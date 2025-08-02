use std::sync::Arc;

use egui_wgpu::ScreenDescriptor;
use pollster::FutureExt;
use winit::{
    application::ApplicationHandler,
    dpi::PhysicalSize,
    event::{KeyEvent, WindowEvent},
    event_loop::ActiveEventLoop,
    keyboard::Key,
    window::{Window, WindowId},
};

use crate::{egui_tools::EguiRenderer, gpu::WgpuContext, sortview::SortView};

#[derive(Default)]
pub struct App<'a> {
    sort_view: Option<SortView>,
    egui_renderer: Option<EguiRenderer>,
    wgpu_ctx: Option<WgpuContext<'a>>,
    window: Option<Arc<Window>>,
}

impl App<'_> {
    fn handle_input(&mut self, event: &KeyEvent) {
        let sort_view = self.sort_view.as_mut().expect("no sort view");
        sort_view.keyboard_input(event);
    }

    fn handle_redraw(&mut self) {
        let egui_renderer = self.egui_renderer.as_mut().expect("no egui renderer");
        let wgpu_ctx = self.wgpu_ctx.as_mut().expect("no wgpu context");
        let window = self.window.as_ref().expect("no window");
        let sort_view = self.sort_view.as_mut().expect("no sort view");

        wgpu_ctx.begin_render_pass();

        let (surface_width, surface_height) = wgpu_ctx.surface_size();

        let screen_descriptor = ScreenDescriptor {
            size_in_pixels: [surface_width, surface_height],
            pixels_per_point: window.scale_factor() as f32 * sort_view.scale_factor(),
        };

        if let Some(data) = sort_view.get_tris_data() {
            wgpu_ctx.update_buffers(data);
        }
        let projection = sort_view.get_projection_matrix();
        wgpu_ctx.update_projection_matrix(projection.into());

        // TODO: Rename this (this is NOT *A* render pass)
        let render_pass = wgpu_ctx.render_pass.as_mut().expect("not rendering");
        {
            egui_renderer.begin_frame(window);

            sort_view.egui_menu(egui_renderer.context());

            egui_renderer.end_frame_and_draw(
                &wgpu_ctx.device,
                &wgpu_ctx.queue,
                &mut render_pass.encoder,
                window,
                &render_pass.surface_view,
                screen_descriptor,
            );
        }

        wgpu_ctx.update_clear_color(sort_view.clear_color());
        wgpu_ctx.submit_render_passes();
    }
}

impl ApplicationHandler for App<'_> {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        if self.window.is_none() {
            let attr = Window::default_attributes()
                .with_title("push_swap visualizer")
                .with_inner_size(PhysicalSize::new(1280, 720));
            let window = Arc::new(event_loop.create_window(attr).expect("no window"));
            let wgpu_ctx = WgpuContext::new(window.clone()).block_on();
            let egui_renderer = EguiRenderer::new(
                &wgpu_ctx.device,
                wgpu_ctx.surface_config.format,
                None,
                1,
                &window,
            );
            let sort_view = SortView::new();
            self.window = Some(window.clone());
            self.wgpu_ctx = Some(wgpu_ctx);
            self.egui_renderer = Some(egui_renderer);
            self.sort_view = Some(sort_view);
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        let egui_renderer = self.egui_renderer.as_mut().expect("no egui renderer");
        let window = self.window.as_ref().expect("no window");
        egui_renderer.handle_input(window, &event);
        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::KeyboardInput { event, .. } => {
                if !egui_renderer.wants_keyboard_input() {
                    if event.state.is_pressed() {
                        if let Key::Named(winit::keyboard::NamedKey::Escape) =
                            event.logical_key.as_ref()
                        {
                            event_loop.exit();
                        }
                    }
                    self.handle_input(&event);
                }
            }
            WindowEvent::Resized(new_size) => {
                if let Some(wgpu_ctx) = self.wgpu_ctx.as_mut() {
                    wgpu_ctx.resize(new_size.into());
                }
            }
            WindowEvent::RedrawRequested => {
                self.handle_redraw();
                self.window.as_ref().unwrap().request_redraw();
            }
            _ => {}
        }
    }
}
