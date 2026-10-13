use crate::{app::App, config::Config};

mod app;
mod cancellation;
mod config;
mod sim;
mod visualizer;

fn main() -> eframe::Result {
    let native_options = eframe::NativeOptions::default();

    let config = Config::load_or_default().expect("failed to load config");
    eframe::run_native(
        "push_swap visualizer",
        native_options,
        Box::new(|cc| Ok(Box::new(App::new(cc, config)))),
    )
}
