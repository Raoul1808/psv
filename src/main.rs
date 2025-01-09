use std::{env::args, process::exit};

use app::App;
use bench::benchmark;
use winit::{
    error::EventLoopError,
    event_loop::{ControlFlow, EventLoop},
};

mod app;
mod bench;
mod egui_tools;
mod gpu;
mod gradient;
mod gui;
mod sim;
mod sortview;
mod vertex;

fn main() -> Result<(), EventLoopError> {
    let args: Vec<_> = args().collect();
    if args.len() > 1 && ["b", "bench", "benchmark"].contains(&args[1].as_str()) {
        benchmark();
        exit(0);
    }

    let event_loop = EventLoop::new().expect("failed to create event loop");
    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = App::default();
    event_loop.run_app(&mut app)
}
