use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::ActiveEventLoop,
    keyboard::Key,
    window::{Window, WindowId},
};

pub struct AppState {
    #[allow(dead_code)]
    window: Window,
}

#[derive(Default)]
pub struct App {
    app_state: Option<AppState>,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let window = event_loop
            .create_window(Window::default_attributes())
            .expect("failed to create window");
        let state = AppState { window };
        self.app_state = Some(state);
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
            _ => {}
        }
    }
}
