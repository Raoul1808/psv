use crate::help::HelpSection;

pub struct AppModesSection;

impl HelpSection for AppModesSection {
    fn section_name(&self) -> &str {
        "App modes"
    }

    fn section_ui(&mut self, ui: &mut eframe::egui::Ui) {
        ui.label("psv operates in multiple modes. You can select between Visualizer mode and Benchmarking mode in the top right corner.");
        ui.add_space(10.);
        ui.label("Note: App mode cannot be changed if push_swap is running in either mode.");
    }
}
