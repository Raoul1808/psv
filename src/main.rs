use crate::{app::App, config::Config};

mod app;
mod benchmark;
mod cancellation;
mod common;
mod config;
mod exec;
mod gradient;
mod sim;
mod visualizer;

fn main() -> eframe::Result {
    let native_options = eframe::NativeOptions::default();

    let config = Config::load_or_default();
    eframe::run_native(
        "push_swap visualizer",
        native_options,
        Box::new(|cc| Ok(Box::new(App::new(cc, config)))),
    )
}
