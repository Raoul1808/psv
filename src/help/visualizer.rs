use crate::help::HelpSection;

pub struct VisualizerSection;

impl HelpSection for VisualizerSection {
    fn section_name(&self) -> &str {
        "Visualizer"
    }

    fn section_ui(&mut self, ui: &mut eframe::egui::Ui) {
        ui.label("psv is first and foremost a push_swap visualizer.");
        ui.add_space(10.);
        ui.horizontal_wrapped(|ui| {
            ui.label("When first starting psv, you are asked to provide the path to your push_swap executable. If this path ever changes, remember to update it by selecting");
            let _ = ui.button("File");
            ui.label("->");
            let _ = ui.button("Select push_swap executable");
        });
        ui.add_space(10.);
        ui.label("Next, select your number preset:");
        ui.horizontal_wrapped(|ui| {
            ui.add_space(10.);
            ui.label("-");
            ui.strong("Random Normalized:");
            ui.label("generate a series of numbers from 0 to N-1, shuffled");
        });
        ui.horizontal_wrapped(|ui| {
            ui.add_space(10.);
            ui.label("-");
            ui.strong("Random Ranged:");
            ui.label("generate N random numbers within the specified range");
        });
        ui.horizontal_wrapped(|ui| {
            ui.add_space(10.);
            ui.label("-");
            ui.strong("User Input:");
            ui.label("manually enter a series of numbers. Useful if you want to test a single case repeatedly");
        });
        ui.add_space(10.);
        ui.label("Next, select the sorting strategy you want to use.");
        ui.label("The sorting strategy will be given to your push_swap program as an argument.");
        ui.label("If no sorting strategy is selected, no argument will be added. This is useful for push_swap programs made before 2025 that do not implement this feature.");
        ui.add_space(10.);
        ui.horizontal_wrapped(|ui| {
            ui.label("Finally, click");
            let _ = ui.button("Visualize");
            ui.label("to run push_swap. Execution is done in parallel, meaning you can do something else in the UI in the meantime, if you want to.");
        });
        ui.horizontal_wrapped(|ui| {
            ui.label("While push_swap is running, the");
            let _ = ui.button("Visualize");
            ui.label("button turns into a");
            let _ = ui.button("Kill");
            ui.label("button, which can be used to abort the execution.");
        });
        ui.label("If push_swap execution fails, an error will be logged in the console and will display on the left panel.");
        ui.add_space(10.);
        ui.label("Once the program finishes executing, instructions are loaded into memory and a center view will display.");
    }
}
