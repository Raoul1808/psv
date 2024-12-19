use std::sync::Arc;

use pollster::FutureExt;
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::ActiveEventLoop,
    keyboard::Key,
    window::{Window, WindowId},
};

use crate::gpu::WgpuContext;

#[derive(Default)]
pub struct App<'a> {
    window: Option<Arc<Window>>,
    wgpu_ctx: Option<WgpuContext<'a>>,
}

impl ApplicationHandler for App<'_> {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        if self.window.is_none() {
            let attr = Window::default_attributes().with_title("push_swap visualizer");
            let window = Arc::new(
                event_loop
                    .create_window(attr)
                    .expect("failed to create window"),
            );
            let wgpu_ctx = WgpuContext::new(window.clone()).block_on();
            self.window = Some(window.clone());
            self.wgpu_ctx = Some(wgpu_ctx);
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
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
            }
            WindowEvent::Resized(new_size) => {
                if let Some(wgpu_ctx) = self.wgpu_ctx.as_mut() {
                    wgpu_ctx.resize(new_size.into());
                }
            }
            WindowEvent::RedrawRequested => {
                if let Some(wgpu_ctx) = self.wgpu_ctx.as_mut() {
                    wgpu_ctx.draw();
                }
            }
            _ => {}
        }
    }
}
