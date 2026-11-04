use simple_logger::SimpleLogger;

use crate::{app::App, config::Config};

mod app;
mod benchmark;
mod cancellation;
mod common;
mod config;
mod exec;
mod gradient;
mod help;
mod sim;
mod visualizer;

fn main() -> eframe::Result {
    SimpleLogger::new()
        .with_level(log::LevelFilter::Warn)
        .with_module_level("psv", log::LevelFilter::Info)
        .init()
        .expect("failed to setup logger");

    let native_options = eframe::NativeOptions::default();

    let config = Config::load_or_default();
    eframe::run_native(
        "push_swap visualizer",
        native_options,
        Box::new(|cc| Ok(Box::new(App::new(cc, config)))),
    )
}
