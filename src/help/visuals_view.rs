use crate::help::HelpSection;

pub struct VisualsViewSection;

impl HelpSection for VisualsViewSection {
    fn section_name(&self) -> &str {
        "Visuals view"
    }

    fn section_ui(&mut self, ui: &mut eframe::egui::Ui) {
        ui.horizontal_wrapped(|ui| {
            ui.label("In the");
            let _ = ui.button("View");
            ui.label("menu, you can toggle on or off a side Visuals menu.");
        });
        ui.add_space(10.);
        ui.label(
            "This menu allows you to configure the visualizer's colors to your heart's content.",
        );
        ui.label("You can create color profiles with each a background color and a gradient for the numbers.");
        ui.label("The selected profile will be remembered in subsequent runs.");
        ui.add_space(10.);
        ui.label("Remember to save your changes to your color profile!");
    }
}
