use app::App;
use winit::{
    error::EventLoopError,
    event_loop::{ControlFlow, EventLoop},
};

mod app;
mod egui_tools;
mod gpu;
mod vertex;

fn main() -> Result<(), EventLoopError> {
    let event_loop = EventLoop::new().expect("failed to create event loop");
    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = App::default();
    event_loop.run_app(&mut app)
}
